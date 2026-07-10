//! Filesystem-backed resolution of `include::` directives, anchored at a base
//! directory and confined by the [safe mode](SafeMode).
//!
//! `asciidoc-parser` handles the *parsing* of `include::` directives but
//! delegates the actual file lookup to an [`IncludeFileHandler`]. This module
//! supplies one that reads from the local filesystem, resolving each target
//! relative to the directory of the including file and, under the `safe` and
//! `server` safe modes, refusing to escape the base directory — the same
//! "jail" Asciidoctor enforces.
//!
//! The parser converts `include::` directives to links (without consulting any
//! handler) under [`SafeMode::Secure`] and above, so this handler is only ever
//! asked to resolve a target under `unsafe`, `safe`, or `server`.
//!
//! # Divergence from Asciidoctor: symlinks
//!
//! Asciidoctor's jail is purely *lexical* — it resolves `..` and absolute
//! targets against the base directory with string operations (Ruby's
//! `File.expand_path`) but does not follow symlinks, so a symlink placed inside
//! the base directory that points outside it can still be read under `safe` or
//! `server`. This handler is deliberately stricter: under a jailed safe mode it
//! canonicalizes the resolved path (following symlinks) and refuses any read
//! whose real location escapes the base directory. The trade-off is that a
//! symlinked include Asciidoctor would follow is instead left unresolved here.
//!
//! # Base directory
//!
//! The base directory is the anchor for relative include targets and, under a
//! jailed safe mode, the boundary reads may not cross. It is Asciidoctor's
//! `-B`/`--base-dir` (and the `base_dir` API option). When the including
//! file's own directory sits inside the base directory, targets resolve
//! relative to that directory; otherwise resolution falls back to the base
//! directory itself, matching Asciidoctor's recovery behavior.

use std::{
    fs,
    path::{Path, PathBuf},
};

use asciidoc_parser::{attributes::Attrlist, parser::IncludeFileHandler, Parser, SafeMode};

/// Resolves `include::` targets against the filesystem, anchored at a base
/// directory and honoring the safe mode's jail.
///
/// Under [`SafeMode::Safe`] and [`SafeMode::Server`] the handler is *jailed*:
/// every resolved path is clamped to the base directory, so a target that tries
/// to climb above it (with `..` or an absolute path) is recovered back inside,
/// and reads never escape. Under [`SafeMode::Unsafe`] there is no jail and
/// targets resolve freely, including to absolute paths and paths outside the
/// base directory.
#[derive(Debug)]
pub(crate) struct FsIncludeFileHandler {
    /// The base directory: the anchor for relative targets and, when jailed,
    /// the boundary reads may not cross. Expected to be absolute and
    /// canonical, so the jailed symlink-containment check (which compares a
    /// canonicalized path against it) is sound.
    base_dir: PathBuf,

    /// The safe mode in force, which decides whether resolution is jailed.
    safe: SafeMode,
}

impl FsIncludeFileHandler {
    /// Creates a handler anchored at `base_dir` and confined according to
    /// `safe`.
    pub(crate) fn new(base_dir: PathBuf, safe: SafeMode) -> Self {
        Self { base_dir, safe }
    }

    /// Whether the safe mode confines resolution to the base-directory jail.
    ///
    /// `safe` and `server` are jailed; `unsafe` is not. (`secure` never reaches
    /// this handler — the parser turns includes into links before consulting
    /// it.)
    fn jailed(&self) -> bool {
        self.safe >= SafeMode::Safe
    }

    /// Resolves the include `target` — as written in the directive of the file
    /// named by `source` — to a filesystem path.
    ///
    /// `source` is the path of the including file (the primary document's path
    /// for a top-level include, or a nested include's own target). Its
    /// directory is the starting point for a relative `target`; when `source`
    /// is `None` the base directory is used.
    fn resolve(&self, source: Option<&str>, target: &str) -> PathBuf {
        let start = source.map(directory_of).unwrap_or_default();
        if self.jailed() {
            self.resolve_jailed(start, target)
        } else {
            self.resolve_free(start, target)
        }
    }

    /// Resolves `target` without a jail: relative targets anchor at `start`
    /// (itself relative to the base directory), absolute targets are taken
    /// as-is, and `..` may climb anywhere.
    fn resolve_free(&self, start: &str, target: &str) -> PathBuf {
        if is_absolute(target) {
            return normalize(&PathBuf::from(target));
        }

        // An absolute `start` is a native path (the including file's own
        // location), so it is kept verbatim — posixifying it would corrupt a
        // Windows path such as a drive prefix or a `\\?\` verbatim path. A
        // relative `start` is a `/`-separated document path, joined onto the
        // base directory a segment at a time.
        let anchor = if is_absolute(start) {
            PathBuf::from(start)
        } else {
            join_segments(&self.base_dir, start)
        };

        normalize(&join_segments(&anchor, target))
    }

    /// Resolves `target` inside the jail: the result is always within the base
    /// directory. A `..` that would climb above the base is dropped, and an
    /// absolute `start` or `target` is treated as relative to the base
    /// directory (recovered), matching Asciidoctor.
    fn resolve_jailed(&self, start: &str, target: &str) -> PathBuf {
        let mut segments: Vec<String> = Vec::new();

        // The starting directory contributes segments only when it sits inside
        // the jail: a relative `start` is taken relative to the base directory,
        // and an absolute `start` keeps only the portion below the base
        // directory (dropping it entirely if it lies outside).
        if is_absolute(start) {
            if let Some(rel) = strip_base_prefix(&self.base_dir, start) {
                fold_into(&mut segments, &rel);
            }
        } else {
            fold_into(&mut segments, start);
        }

        // An absolute target is recovered to the jail root: it replaces any
        // starting segments and is reinterpreted relative to the base
        // directory.
        if is_absolute(target) {
            segments.clear();
            fold_into(&mut segments, strip_root(target));
        } else {
            fold_into(&mut segments, target);
        }

        let mut path = self.base_dir.clone();
        for segment in &segments {
            path.push(segment);
        }
        path
    }
}

impl IncludeFileHandler for FsIncludeFileHandler {
    fn resolve_target<'src>(
        &self,
        source: Option<&str>,
        target: &str,
        _attrlist: &Attrlist<'src>,
        _parser: &Parser,
    ) -> Option<String> {
        let path = self.resolve(source, target);

        if self.jailed() {
            // Lexical resolution keeps the *path* inside the base directory, but
            // a symlink under the base directory could still point outside it.
            // Resolve the real path (following symlinks) and refuse the read
            // when it escapes the base directory. This is stricter than
            // Asciidoctor, whose jail is purely lexical (see the module docs);
            // canonicalization also fails for a path the recovery relocated to
            // somewhere that does not exist, which likewise leaves the directive
            // unresolved.
            let real = path.canonicalize().ok()?;
            if !real.starts_with(&self.base_dir) {
                return None;
            }
            return fs::read_to_string(real).ok();
        }

        // Without a jail, a read failure (missing file, a directory, or
        // non-UTF-8 content) simply leaves the directive unresolved, which the
        // parser reports.
        fs::read_to_string(path).ok()
    }
}

/// Returns the directory portion of `path`: everything before the final path
/// separator, or the empty string when `path` has none.
///
/// Both `/` and `\` are honored as separators, since an include's `source` may
/// have been supplied on either platform.
fn directory_of(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(index) => &path[..index],
        None => "",
    }
}

/// Whether `path` is absolute: it begins with a `/` (or `\`) or with a Windows
/// drive prefix such as `C:`.
fn is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.starts_with('\\') || {
        let mut chars = path.chars();
        matches!(
            (chars.next(), chars.next()),
            (Some(letter), Some(':')) if letter.is_ascii_alphabetic()
        )
    }
}

/// Converts backslash separators to forward slashes so path handling can work
/// in terms of a single separator regardless of the platform the string came
/// from.
fn posixify(path: &str) -> String {
    path.replace('\\', "/")
}

/// Strips the leading root from `path`: a leading `/`, or a Windows drive
/// prefix like `C:/`. Used to reinterpret an absolute target relative to the
/// jail.
fn strip_root(path: &str) -> &str {
    let path = path.strip_prefix('\\').unwrap_or(path);
    if let Some(rest) = path.strip_prefix('/') {
        return rest;
    }

    // A drive prefix (`C:` or `C:/`): skip the letter, the colon, and any
    // separator that follows.
    let mut chars = path.char_indices();
    match (chars.next(), chars.next()) {
        (Some((_, letter)), Some((colon_index, ':'))) if letter.is_ascii_alphabetic() => {
            let after_colon = colon_index + 1;
            path[after_colon..]
                .strip_prefix(['/', '\\'])
                .unwrap_or(&path[after_colon..])
        }
        _ => path,
    }
}

/// Returns the portion of the absolute `path` below `base`, or `None` when
/// `path` is not within `base`. Comparison is lexical (segment by segment),
/// matching Asciidoctor's jail check.
fn strip_base_prefix(base: &Path, path: &str) -> Option<String> {
    let base = posixify(&base.to_string_lossy());
    let base = base.strip_suffix('/').unwrap_or(&base);
    let path = posixify(path);

    if path == base {
        return Some(String::new());
    }
    path.strip_prefix(&format!("{base}/"))
        .map(|rest| rest.to_string())
}

/// Folds the `/`-separated `path` onto `segments`, dropping `.` and empty
/// components and resolving each `..` by popping the previous segment. A `..`
/// with nothing to pop is discarded, clamping the result at the jail root.
fn fold_into(segments: &mut Vec<String>, path: &str) {
    for component in posixify(path).split('/') {
        match component {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other.to_string()),
        }
    }
}

/// Joins the `/`-separated relative `path` onto `base`, one component at a time
/// (so `.` and `..` are preserved for a later [`normalize`] rather than being
/// resolved against the filesystem).
fn join_segments(base: &Path, path: &str) -> PathBuf {
    let mut result = base.to_path_buf();
    for component in posixify(path).split('/') {
        if !component.is_empty() {
            result.push(component);
        }
    }
    result
}

/// Lexically normalizes `path`, resolving `.` and `..` components without
/// touching the filesystem. A `..` pops a preceding normal component; at the
/// root (or against a leading `..` in a relative path) it is preserved, so a
/// path may still climb above its start — this is only used for the unjailed
/// (`unsafe`) case, where escaping the base directory is allowed.
fn normalize(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                match normalized.components().next_back() {
                    // Pop a preceding normal component; keep climbing when there
                    // is nothing poppable (a root prefix or a leading `..`).
                    Some(Component::Normal(_)) => {
                        normalized.pop();
                    }
                    _ => normalized.push(component),
                }
            }
            other => normalized.push(other),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use asciidoc_parser::SafeMode;

    use super::{
        directory_of, fold_into, is_absolute, normalize, posixify, strip_root, FsIncludeFileHandler,
    };

    const BASE: &str = "/home/user/project";

    fn handler(safe: SafeMode) -> FsIncludeFileHandler {
        FsIncludeFileHandler::new(PathBuf::from(BASE), safe)
    }

    /// Resolves a target and renders the result with forward slashes, so the
    /// expectations below are written the same way on every platform (a
    /// `PathBuf` prints with `\` on Windows and `/` elsewhere, but the
    /// resolution logic is separator-independent).
    fn resolve(safe: SafeMode, source: Option<&str>, target: &str) -> String {
        handler(safe)
            .resolve(source, target)
            .to_string_lossy()
            .replace('\\', "/")
    }

    // With no jail (`unsafe`), a relative target anchors at the base directory
    // when there is no source file to anchor to.
    #[test]
    fn free_relative_target_anchors_at_base() {
        assert_eq!(
            resolve(SafeMode::Unsafe, None, "chapter.adoc"),
            "/home/user/project/chapter.adoc"
        );
    }

    // A relative target anchors at the directory of the including file.
    #[test]
    fn free_relative_target_anchors_at_source_directory() {
        assert_eq!(
            resolve(SafeMode::Unsafe, Some("parts/intro.adoc"), "detail.adoc"),
            "/home/user/project/parts/detail.adoc"
        );
    }

    // Under `unsafe`, a target may climb above the base directory.
    #[test]
    fn free_target_may_escape_the_base_directory() {
        assert_eq!(
            resolve(SafeMode::Unsafe, None, "../secrets.adoc"),
            "/home/user/secrets.adoc"
        );
    }

    // Under `unsafe`, an absolute target is honored as written.
    #[test]
    fn free_absolute_target_is_kept() {
        assert_eq!(
            resolve(SafeMode::Unsafe, None, "/etc/passwd"),
            "/etc/passwd"
        );
    }

    // Under a jail (`safe`), a relative target resolves within the base
    // directory.
    #[test]
    fn jailed_relative_target_resolves_within_base() {
        assert_eq!(
            resolve(SafeMode::Safe, None, "chapter.adoc"),
            "/home/user/project/chapter.adoc"
        );
    }

    // Under a jail, `..` that would climb above the base directory is dropped,
    // clamping the result inside the jail.
    #[test]
    fn jailed_target_cannot_escape_with_parent_refs() {
        assert_eq!(
            resolve(SafeMode::Safe, None, "../../../etc/passwd"),
            "/home/user/project/etc/passwd"
        );
    }

    // Under a jail, an absolute target is recovered to the jail root rather than
    // read from its literal location.
    #[test]
    fn jailed_absolute_target_is_recovered_to_the_jail() {
        assert_eq!(
            resolve(SafeMode::Server, None, "/etc/passwd"),
            "/home/user/project/etc/passwd"
        );
    }

    // Under a jail, a source directory inside the base contributes its offset.
    #[test]
    fn jailed_source_directory_inside_base_is_honored() {
        assert_eq!(
            resolve(
                SafeMode::Safe,
                Some("/home/user/project/parts/intro.adoc"),
                "detail.adoc"
            ),
            "/home/user/project/parts/detail.adoc"
        );
    }

    // Under a jail, a source directory equal to the base contributes no offset:
    // a top-level include resolves directly inside the base directory.
    #[test]
    fn jailed_source_directory_equal_to_base_has_no_offset() {
        assert_eq!(
            resolve(
                SafeMode::Safe,
                Some("/home/user/project/main.adoc"),
                "part.adoc"
            ),
            "/home/user/project/part.adoc"
        );
    }

    // Under a jail, a source directory outside the base is dropped: resolution
    // recovers to the base directory.
    #[test]
    fn jailed_source_directory_outside_base_recovers_to_base() {
        assert_eq!(
            resolve(SafeMode::Safe, Some("/etc/intro.adoc"), "detail.adoc"),
            "/home/user/project/detail.adoc"
        );
    }

    // Under `unsafe`, an absolute source directory is honored directly (the
    // target resolves alongside the including file, wherever it lives).
    #[test]
    fn free_absolute_source_directory_is_honored() {
        assert_eq!(
            resolve(SafeMode::Unsafe, Some("/var/docs/main.adoc"), "part.adoc"),
            "/var/docs/part.adoc"
        );
    }

    // A source with no directory component (a bare file name) anchors at the
    // base directory.
    #[test]
    fn source_without_a_directory_anchors_at_base() {
        assert_eq!(
            resolve(SafeMode::Unsafe, Some("main.adoc"), "part.adoc"),
            "/home/user/project/part.adoc"
        );
        assert_eq!(
            resolve(SafeMode::Safe, Some("main.adoc"), "part.adoc"),
            "/home/user/project/part.adoc"
        );
    }

    // Backslash-separated targets and a Windows drive-letter base are handled
    // the same way, so include resolution works on Windows as well as Posix.
    #[test]
    fn windows_style_paths_resolve() {
        // A backslash-separated relative target is treated like a Posix one.
        assert_eq!(
            resolve(SafeMode::Unsafe, None, r"sub\part.adoc"),
            "/home/user/project/sub/part.adoc"
        );

        // A drive-letter base with a jailed absolute target recovers into the
        // jail, dropping the drive root.
        let handler = FsIncludeFileHandler::new(PathBuf::from(r"C:\book"), SafeMode::Safe);
        assert_eq!(
            handler
                .resolve(None, r"C:\Windows\system32")
                .to_string_lossy()
                .replace('\\', "/"),
            "C:/book/Windows/system32"
        );
    }

    // `is_absolute` recognizes Posix roots, Windows backslash roots, and drive
    // prefixes, and rejects relative paths.
    #[test]
    fn is_absolute_recognizes_roots() {
        assert!(is_absolute("/etc"));
        assert!(is_absolute(r"\etc"));
        assert!(is_absolute("C:/dir"));
        assert!(is_absolute("c:rel"));
        assert!(!is_absolute("dir/file"));
        assert!(!is_absolute("file.adoc"));
        assert!(!is_absolute(""));
    }

    // `strip_root` removes a leading Posix slash, a backslash, or a drive
    // prefix (with or without a trailing separator).
    #[test]
    fn strip_root_removes_the_leading_root() {
        assert_eq!(strip_root("/x/y"), "x/y");
        assert_eq!(strip_root(r"\x\y"), r"x\y");
        assert_eq!(strip_root("C:/x/y"), "x/y");
        assert_eq!(strip_root("C:x"), "x");
        assert_eq!(strip_root("relative"), "relative");
    }

    // `directory_of` returns the portion before the final separator (either
    // kind), or the empty string when there is none.
    #[test]
    fn directory_of_returns_the_parent() {
        assert_eq!(directory_of("a/b/c.adoc"), "a/b");
        assert_eq!(directory_of(r"a\b\c.adoc"), r"a\b");
        assert_eq!(directory_of("c.adoc"), "");
        assert_eq!(directory_of(""), "");
    }

    // `posixify` converts backslash separators to forward slashes and leaves a
    // Posix path untouched.
    #[test]
    fn posixify_normalizes_separators() {
        assert_eq!(posixify(r"a\b\c"), "a/b/c");
        assert_eq!(posixify("a/b/c"), "a/b/c");
    }

    // `fold_into` drops `.` and empty components, resolves `..` by popping, and
    // clamps a `..` with nothing to pop (so it cannot climb past the start).
    #[test]
    fn fold_into_resolves_and_clamps_components() {
        fn fold(path: &str) -> Vec<String> {
            let mut segments = vec!["base".to_string()];
            fold_into(&mut segments, path);
            segments
        }

        assert_eq!(fold("a/./b"), vec!["base", "a", "b"]);
        assert_eq!(fold("a//b"), vec!["base", "a", "b"]);
        assert_eq!(fold("a/../b"), vec!["base", "b"]);
        // Two `..` pop `base` then clamp (nothing left to pop).
        assert_eq!(fold("../../x"), vec!["x"]);
    }

    // `normalize` resolves `.`/`..` lexically, popping normal components but
    // preserving a `..` that has nothing to pop (the unjailed escape case).
    #[test]
    fn normalize_folds_lexically() {
        fn norm(path: &str) -> String {
            normalize(std::path::Path::new(path))
                .to_string_lossy()
                .replace('\\', "/")
        }

        assert_eq!(norm("/a/b/../c"), "/a/c");
        // A leading `.` is dropped. (`Path::components` already elides a `.` in
        // the middle of a path, so only a leading one reaches the `CurDir`
        // arm.)
        assert_eq!(norm("./a/b"), "a/b");
        // A leading `..` in a relative path is kept (nothing to pop).
        assert_eq!(norm("../a"), "../a");
    }

    // End-to-end resolution against a real temporary project directory,
    // exercising `convert_file_with` (which anchors the base directory at the
    // primary file's directory) across safe modes.
    mod end_to_end {
        use std::{fs, path::PathBuf};

        use crate::{convert_file_with, Options, SafeMode};

        /// Creates a fresh temporary directory named for `label`, holding a
        /// `main.adoc` that includes `part.adoc`, a `part.adoc` inside the
        /// directory, and a `secret.adoc` in the *parent* directory (outside
        /// the base). Returns the path to `main.adoc`.
        fn project(label: &str) -> PathBuf {
            let root =
                std::env::temp_dir().join(format!("ahtml5-include-{label}-{}", std::process::id()));
            let base = root.join("base");
            fs::create_dir_all(&base).expect("create base dir");

            fs::write(
                base.join("main.adoc"),
                "= Main\n\nBefore.\n\ninclude::part.adoc[]\n\nAfter.\n",
            )
            .expect("write main");
            fs::write(base.join("part.adoc"), "Included from part.\n").expect("write part");
            fs::write(root.join("secret.adoc"), "Included from secret.\n").expect("write secret");

            base.join("main.adoc")
        }

        // Below `secure`, a relative include is read from disk and its content
        // is rendered in place of the directive.
        #[test]
        fn relative_include_is_resolved() {
            let main = project("relative");
            let html =
                convert_file_with(&main, &Options::new().safe_mode(SafeMode::Unsafe)).unwrap();
            assert!(html.contains("Included from part."));
            assert!(!html.contains("include::"));
        }

        // Under `unsafe` there is no jail, so an include that climbs out of the
        // base directory is read.
        #[test]
        fn unsafe_include_can_escape_the_base_directory() {
            let main = project("escape-unsafe");
            let escaping = "= Main\n\ninclude::../secret.adoc[]\n";
            fs::write(&main, escaping).expect("rewrite main");

            let html =
                convert_file_with(&main, &Options::new().safe_mode(SafeMode::Unsafe)).unwrap();
            assert!(html.contains("Included from secret."));
        }

        // Under `safe`, the same escaping include is clamped to the base
        // directory, where no such file exists, so it stays unresolved and the
        // out-of-base content never appears.
        #[test]
        fn safe_include_cannot_escape_the_base_directory() {
            let main = project("escape-safe");
            let escaping = "= Main\n\ninclude::../secret.adoc[]\n";
            fs::write(&main, escaping).expect("rewrite main");

            let html = convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();
            assert!(!html.contains("Included from secret."));
        }

        // Under `secure` (the API default), the parser converts the include to a
        // link without ever reading the file.
        #[test]
        fn secure_turns_the_include_into_a_link() {
            let main = project("secure");
            let html = convert_file_with(&main, &Options::new()).unwrap();
            assert!(!html.contains("Included from part."));
            assert!(html.contains("part.adoc"));
        }

        // A symlink inside the base directory that points outside it lives at a
        // lexically-in-jail path, so the lexical jail alone would follow it.
        // Under `unsafe` (no jail) it is read, but under `safe` the handler
        // resolves the real path and refuses to leave the base directory —
        // stricter than Asciidoctor. Symlink creation is Unix-only.
        #[cfg(unix)]
        #[test]
        fn safe_mode_does_not_follow_a_symlink_out_of_the_base() {
            let main = project("symlink");
            let base = main.parent().expect("base dir");
            let root = base.parent().expect("root dir");

            // `base/leak.adoc` -> `root/secret.adoc` (outside the base).
            std::os::unix::fs::symlink(root.join("secret.adoc"), base.join("leak.adoc"))
                .expect("create symlink");
            fs::write(&main, "= Main\n\ninclude::leak.adoc[]\n").expect("rewrite main");

            // `unsafe` has no jail, so the symlink is followed out of the base.
            let unsafe_html =
                convert_file_with(&main, &Options::new().safe_mode(SafeMode::Unsafe)).unwrap();
            assert!(unsafe_html.contains("Included from secret."));

            // `safe` resolves the real path and refuses to leave the base.
            let safe_html =
                convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();
            assert!(!safe_html.contains("Included from secret."));
        }
    }
}
