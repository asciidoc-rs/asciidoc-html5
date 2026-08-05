//! Filesystem-backed resolution of inline SVG images (`opts=inline`), anchored
//! at a base directory and confined by the [safe mode](SafeMode).
//!
//! An inline SVG image (`image:diagram.svg[opts=inline]`) embeds the SVG file's
//! *contents* directly into the output as an `<svg>` element. `asciidoc-parser`
//! parses the macro and prepares the embedded markup, but delegates reading the
//! SVG file to an [`SvgFileHandler`]. This module supplies one that reads from
//! the local filesystem, resolving the target against the document's base
//! directory (the target already carries any `imagesdir` prefix) and, under the
//! `safe` and `server` safe modes, refusing to escape it — the same jail the
//! [include handler](crate::include_handler) enforces.
//!
//! The parser only renders the `inline` (and `interactive`) SVG modes below
//! [`SafeMode::Secure`]; at `secure` an SVG image renders as an ordinary
//! `<img>` without consulting any handler, so this handler is only ever asked
//! to resolve a target under `unsafe`, `safe`, or `server`. When no handler is
//! registered, or it cannot read the file, the parser degrades the inline SVG
//! to its alt text (`<span class="alt">…</span>`), matching Ruby Asciidoctor.

use std::path::PathBuf;

use asciidoc_parser::{parser::SvgFileHandler, Parser, SafeMode};

use crate::include_handler::{read_confined, resolve, ReadOutcome};

/// Reads inline SVG images from the filesystem, anchored at a base directory
/// and honoring the safe mode's jail (the same one [`FsIncludeFileHandler`]
/// enforces).
///
/// The parser hands each resolved image target (`diagram.svg`,
/// `images/circle.svg`, …) to [`resolve_svg`]. This handler resolves the target
/// against the base directory and reads it. Under [`SafeMode::Safe`] and
/// [`SafeMode::Server`] an absolute or climbing target is recovered back inside
/// the base directory, so reads never escape it; under [`SafeMode::Unsafe`]
/// there is no such restriction.
///
/// [`FsIncludeFileHandler`]: crate::include_handler::FsIncludeFileHandler
/// [`resolve_svg`]: SvgFileHandler::resolve_svg
#[derive(Debug)]
pub(crate) struct FsSvgFileHandler {
    /// The base directory: the document's directory, the anchor for image
    /// targets and — when jailed — the boundary reads may not cross. Expected
    /// to be absolute and canonical, matching the include handler.
    base_dir: PathBuf,

    /// The safe mode in force, which decides whether resolution is jailed.
    safe: SafeMode,
}

impl FsSvgFileHandler {
    /// Creates a handler anchored at `base_dir` and confined according to
    /// `safe`.
    pub(crate) fn new(base_dir: PathBuf, safe: SafeMode) -> Self {
        Self { base_dir, safe }
    }
}

impl SvgFileHandler for FsSvgFileHandler {
    fn resolve_svg(&self, target: &str, _parser: &Parser) -> Option<String> {
        // The target is the resolved image path (already prefixed with
        // `imagesdir`), so it is resolved against the base directory with no
        // including-file `source` — the same shape the docinfo handler uses.
        // Reusing the include handler's `resolve` gives the SVG read the exact
        // jail behavior includes get: a climbing or absolute target is recovered
        // inside the base directory under `safe`/`server`, and honored as-is
        // under `unsafe`.
        let path = resolve(&self.base_dir, self.safe, None, target);

        // Inline SVG does not distinguish a missing file from an unreadable or
        // non-UTF-8 one: every failure reason collapses to `None`, and the
        // parser then falls back to the alt text.
        match read_confined(&self.base_dir, self.safe, &path) {
            ReadOutcome::Read(content) => Some(content),
            ReadOutcome::NotFound | ReadOutcome::NotReadable | ReadOutcome::NotDecodable => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use asciidoc_parser::{parser::SvgFileHandler, Parser, SafeMode};

    use super::FsSvgFileHandler;

    /// Writes `files` (name → content) into a fresh temp directory and returns
    /// its canonical path, so the handler's jail comparisons share one absolute
    /// form with the paths it resolves.
    ///
    /// The directory name is made unique with a process-wide atomic counter
    /// (the process id keeps it distinct across concurrent test binaries), so
    /// callers passing the same `files` do not collide.
    fn scratch(files: &[(&str, &str)]) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("adoc-svg-{}-{unique}", std::process::id()));
        fs::create_dir_all(&dir).expect("create scratch dir");
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create subdir");
            }
            fs::write(path, content).expect("write scratch file");
        }
        dir.canonicalize().expect("canonicalize scratch dir")
    }

    fn handler(dir: &std::path::Path, safe: SafeMode) -> FsSvgFileHandler {
        FsSvgFileHandler::new(dir.to_path_buf(), safe)
    }

    #[test]
    fn reads_an_svg_from_the_base_directory_verbatim() {
        let dir = scratch(&[("circle.svg", "<svg><circle/></svg>")]);

        let got = handler(&dir, SafeMode::Server).resolve_svg("circle.svg", &Parser::default());

        assert_eq!(got.as_deref(), Some("<svg><circle/></svg>"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reads_an_svg_under_an_imagesdir_prefix() {
        // The target the parser passes is already `imagesdir`-prefixed, so a
        // subdirectory in the target resolves under the base directory.
        let dir = scratch(&[("images/circle.svg", "<svg/>")]);

        let got =
            handler(&dir, SafeMode::Server).resolve_svg("images/circle.svg", &Parser::default());

        assert_eq!(got.as_deref(), Some("<svg/>"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_file_resolves_to_none() {
        let dir = scratch(&[]);

        let got = handler(&dir, SafeMode::Server).resolve_svg("absent.svg", &Parser::default());

        assert_eq!(got, None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_jailed_climbing_target_is_clamped_to_the_base() {
        // Under `server`, a target that tries to climb out with `..` has the
        // climb clamped at the base directory: `../../circle.svg` folds to
        // `circle.svg` inside the base, so the in-base file is read.
        let dir = scratch(&[("circle.svg", "<svg/>")]);

        let got =
            handler(&dir, SafeMode::Server).resolve_svg("../../circle.svg", &Parser::default());

        assert_eq!(got.as_deref(), Some("<svg/>"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_jailed_absolute_target_cannot_escape_the_base() {
        // Under `server`, an absolute target pointing outside the base is
        // recovered relative to the base (never read as-is), so the outside file
        // is not reachable.
        let base = scratch(&[]);
        let other = scratch(&[("secret.svg", "<svg/>")]);

        let got = handler(&base, SafeMode::Server).resolve_svg(
            other.join("secret.svg").to_str().unwrap(),
            &Parser::default(),
        );

        assert_eq!(got, None);
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn an_unsafe_absolute_target_is_honored() {
        // Without a jail (`unsafe`), an absolute target is used as-is, so a file
        // outside the base directory is read.
        let base = scratch(&[]);
        let other = scratch(&[("secret.svg", "<svg>OUTSIDE</svg>")]);

        let got = handler(&base, SafeMode::Unsafe).resolve_svg(
            other.join("secret.svg").to_str().unwrap(),
            &Parser::default(),
        );

        assert_eq!(got.as_deref(), Some("<svg>OUTSIDE</svg>"));
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(&other);
    }
}
