//! Filesystem-backed resolution of [docinfo] files, anchored at a base
//! directory and confined by the [safe mode](SafeMode).
//!
//! `asciidoc-parser` decides *which* docinfo files apply (from the `docinfo`,
//! `docinfodir`, and `docinfosubs` attributes and the document name) and
//! applies attribute substitution, but delegates reading the files to a
//! [`DocinfoFileHandler`]. This module supplies one that reads from the local
//! filesystem, resolving each computed file name against the document's base
//! directory and, under the `safe` and `server` safe modes, refusing to escape
//! it — the same jail the [include handler](crate::include_handler) enforces.
//!
//! The parser drops docinfo entirely under [`SafeMode::Secure`] and above
//! (without consulting any handler), so this handler is only ever asked to
//! resolve a file under `unsafe`, `safe`, or `server`.
//!
//! [docinfo]: https://docs.asciidoctor.org/asciidoc/latest/docinfo/

use std::path::PathBuf;

use asciidoc_parser::{parser::DocinfoFileHandler, Parser, SafeMode};

use crate::include_handler::{read_confined, resolve};

/// Reads docinfo files from the filesystem, anchored at a base directory and
/// honoring the safe mode's jail (the same one [`FsIncludeFileHandler`]
/// enforces).
///
/// The parser hands each computed docinfo file name (`docinfo.html`,
/// `mydoc-docinfo-footer.html`, …) to [`resolve_docinfo`], optionally alongside
/// a `docinfodir`. This handler resolves the name against the base directory —
/// or, when `docinfodir` is set, against that subdirectory — and reads it.
/// Under [`SafeMode::Safe`] and [`SafeMode::Server`] an absolute or climbing
/// `docinfodir` is recovered back inside the base directory, so reads never
/// escape it; under [`SafeMode::Unsafe`] there is no such restriction.
///
/// [`FsIncludeFileHandler`]: crate::include_handler::FsIncludeFileHandler
/// [`resolve_docinfo`]: DocinfoFileHandler::resolve_docinfo
#[derive(Debug)]
pub(crate) struct FsDocinfoFileHandler {
    /// The base directory: the document's directory, the anchor for docinfo
    /// file names and — when jailed — the boundary reads may not cross.
    /// Expected to be absolute and canonical, matching the include handler.
    base_dir: PathBuf,

    /// The safe mode in force, which decides whether resolution is jailed.
    safe: SafeMode,
}

impl FsDocinfoFileHandler {
    /// Creates a handler anchored at `base_dir` and confined according to
    /// `safe`.
    pub(crate) fn new(base_dir: PathBuf, safe: SafeMode) -> Self {
        Self { base_dir, safe }
    }
}

impl DocinfoFileHandler for FsDocinfoFileHandler {
    fn resolve_docinfo(
        &self,
        docinfodir: Option<&str>,
        file_name: &str,
        _parser: &Parser,
    ) -> Option<String> {
        // The file name is resolved against the base directory. A `docinfodir`
        // relocates the search: a relative value is a subdirectory of the base
        // directory, an absolute one replaces it — and, when jailed, either is
        // recovered back inside the base directory. Joining the directory onto
        // the file name and resolving the pair with `source: None` reuses the
        // include handler's jail logic verbatim.
        let target = match docinfodir {
            Some(dir) => format!("{}/{file_name}", dir.trim_end_matches(['/', '\\'])),
            None => file_name.to_string(),
        };

        let path = resolve(&self.base_dir, self.safe, None, &target);

        // Asciidoctor normalizes docinfo content, dropping a single trailing
        // newline so the injected fragment sits flush against the element that
        // follows it in the output.
        read_confined(&self.base_dir, self.safe, &path)
            .map(|content| chomp_trailing_newline(&content))
    }
}

/// Removes a single trailing line ending (`\n` or `\r\n`) from `s`, if present.
fn chomp_trailing_newline(s: &str) -> String {
    s.strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(s)
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use asciidoc_parser::{parser::DocinfoFileHandler, Parser, SafeMode};

    use super::FsDocinfoFileHandler;

    /// Writes `files` (name → content) into a fresh temp directory and returns
    /// its canonical path, so the handler's jail comparisons share one absolute
    /// form with the paths it resolves.
    fn scratch(files: &[(&str, &str)]) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("adoc-docinfo-{}-{:p}", std::process::id(), files));
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

    fn handler(dir: &std::path::Path, safe: SafeMode) -> FsDocinfoFileHandler {
        FsDocinfoFileHandler::new(dir.to_path_buf(), safe)
    }

    #[test]
    fn reads_a_file_from_the_base_directory_and_chomps_the_trailing_newline() {
        let dir = scratch(&[("docinfo.html", "<meta name=\"x\">\n")]);
        let got = handler(&dir, SafeMode::Server).resolve_docinfo(
            None,
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got.as_deref(), Some("<meta name=\"x\">"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_relative_docinfodir_is_a_subdirectory_of_the_base() {
        let dir = scratch(&[("meta/docinfo.html", "IN-META")]);
        let got = handler(&dir, SafeMode::Server).resolve_docinfo(
            Some("meta"),
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got.as_deref(), Some("IN-META"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_file_resolves_to_none() {
        let dir = scratch(&[]);
        let got = handler(&dir, SafeMode::Server).resolve_docinfo(
            None,
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got, None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_jailed_climbing_docinfodir_is_clamped_to_the_base() {
        // Under `server`, a `docinfodir` that tries to climb out with `..` has
        // the climb clamped at the base directory: `../../docinfo.html` folds to
        // `docinfo.html` inside the base, so the in-base file is read.
        let dir = scratch(&[("docinfo.html", "IN-BASE")]);
        let got = handler(&dir, SafeMode::Server).resolve_docinfo(
            Some("../.."),
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got.as_deref(), Some("IN-BASE"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_jailed_absolute_docinfodir_cannot_escape_the_base() {
        // Under `server`, an absolute `docinfodir` pointing outside the base is
        // recovered relative to the base (never read as-is), so the outside file
        // is not reachable.
        let base = scratch(&[]);
        let other = scratch(&[("docinfo.html", "OUTSIDE")]);
        let got = handler(&base, SafeMode::Server).resolve_docinfo(
            Some(other.to_str().unwrap()),
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got, None);
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn an_unsafe_absolute_docinfodir_is_honored() {
        // Without a jail (`unsafe`), an absolute `docinfodir` is used as-is, so
        // a file outside the base directory is read.
        let base = scratch(&[]);
        let other = scratch(&[("docinfo.html", "OUTSIDE")]);
        let got = handler(&base, SafeMode::Unsafe).resolve_docinfo(
            Some(other.to_str().unwrap()),
            "docinfo.html",
            &Parser::default(),
        );
        assert_eq!(got.as_deref(), Some("OUTSIDE"));
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_dir_all(&other);
    }
}
