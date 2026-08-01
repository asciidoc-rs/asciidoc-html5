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
//!
//! # Encoding
//!
//! Include files are read as UTF-8 by default. When an `include::` directive's
//! `encoding` attribute names a legacy single-byte encoding this handler
//! recognizes (`iso-8859-1` or `windows-1252`), the raw bytes are transcoded to
//! UTF-8 instead, and the result is reported to the parser as already
//! transcoded (via [`IncludeContent::transcoded`]) so it raises no non-UTF-8
//! include-encoding warning. An unrecognized `encoding` value is ignored and
//! the file is read as UTF-8, matching Asciidoctor, which ignores the encoding
//! attribute when its value is not a valid encoding. A non-UTF-8 file whose
//! encoding this handler cannot transcode (no `encoding` attribute, or one it
//! does not recognize) is left unresolved, since the parser's
//! [`IncludeFileHandler`] contract accepts UTF-8 content only. This diverges
//! from Asciidoctor, which raises `invalid byte sequence in UTF-8` for an
//! undeclared non-UTF-8 include; this crate's handlers cannot raise mid-parse,
//! so the include is dropped and the parser records an include-file warning
//! instead.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use asciidoc_parser::{
    attributes::Attrlist,
    parser::{IncludeContent, IncludeFileHandler, IncludeResolution},
    Parser, SafeMode,
};

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

    /// Resolves the include `target` — as written in the directive of the file
    /// named by `source` — to a filesystem path.
    ///
    /// `source` is the path of the including file (the primary document's path
    /// for a top-level include, or a nested include's own target). Its
    /// directory is the starting point for a relative `target`; when `source`
    /// is `None` the base directory is used.
    fn resolve(&self, source: Option<&str>, target: &str) -> PathBuf {
        resolve(&self.base_dir, self.safe, source, target)
    }
}

/// Whether the safe mode confines resolution to the base-directory jail.
///
/// `safe` and `server` are jailed; `unsafe` is not. (`secure` never reaches the
/// filesystem handlers — the parser turns includes into links, and drops
/// docinfo, before consulting them.)
pub(crate) fn jailed(safe: SafeMode) -> bool {
    safe >= SafeMode::Safe
}

/// Resolves `target` against `base_dir` — relative to `source`'s directory when
/// given, else to the base directory itself — honoring the safe mode's jail.
///
/// This is shared by the [`FsIncludeFileHandler`] (whose `source` is the
/// including file) and the docinfo handler (which passes `None`, resolving
/// against the base directory). See [`read_confined`] for the matching read.
pub(crate) fn resolve(
    base_dir: &Path,
    safe: SafeMode,
    source: Option<&str>,
    target: &str,
) -> PathBuf {
    let start = source.map(directory_of).unwrap_or_default();
    if jailed(safe) {
        resolve_jailed(base_dir, start, target)
    } else {
        resolve_free(base_dir, start, target)
    }
}

/// Resolves `target` without a jail: relative targets anchor at `start` (itself
/// relative to the base directory), absolute targets are taken as-is, and `..`
/// may climb anywhere.
fn resolve_free(base_dir: &Path, start: &str, target: &str) -> PathBuf {
    if is_absolute(target) {
        return normalize(&PathBuf::from(target));
    }

    // An absolute `start` is a native path (the including file's own location),
    // so it is kept verbatim — posixifying it would corrupt a Windows path such
    // as a drive prefix or a `\\?\` verbatim path. A relative `start` is a
    // `/`-separated document path, joined onto the base directory a segment at a
    // time.
    let anchor = if is_absolute(start) {
        PathBuf::from(start)
    } else {
        join_segments(base_dir, start)
    };

    normalize(&join_segments(&anchor, target))
}

/// Resolves `target` inside the jail: the result is always within the base
/// directory. A `..` that would climb above the base is dropped, and an
/// absolute `start` or `target` is treated as relative to the base directory
/// (recovered), matching Asciidoctor.
fn resolve_jailed(base_dir: &Path, start: &str, target: &str) -> PathBuf {
    let mut segments: Vec<String> = Vec::new();

    // The starting directory contributes segments only when it sits inside the
    // jail: a relative `start` is taken relative to the base directory, and an
    // absolute `start` keeps only the portion below the base directory
    // (dropping it entirely if it lies outside).
    if is_absolute(start) {
        if let Some(rel) = strip_base_prefix(base_dir, start) {
            fold_into(&mut segments, &rel);
        }
    } else {
        fold_into(&mut segments, start);
    }

    // An absolute target is recovered to the jail root: it replaces any starting
    // segments and is reinterpreted relative to the base directory.
    if is_absolute(target) {
        segments.clear();
        fold_into(&mut segments, strip_root(target));
    } else {
        fold_into(&mut segments, target);
    }

    let mut path = base_dir.to_path_buf();
    for segment in &segments {
        path.push(segment);
    }
    path
}

/// The outcome of a [`read_confined`] call: the file's UTF-8 content, or the
/// reason it could not be read.
///
/// The two failure reasons mirror Asciidoctor's distinction between a missing
/// include file (`include file not found`) and one that is present but cannot
/// be read (`include file not readable`), and map onto the matching
/// [`IncludeResolution`] variants.
pub(crate) enum ReadOutcome {
    /// The file was read; its UTF-8 content is carried here.
    Read(String),

    /// No readable file exists at the path: it is missing, is not a regular
    /// file (e.g. a directory), lies outside the jail, or holds non-UTF-8
    /// content this handler cannot decode.
    NotFound,

    /// A regular file exists at the path but could not be read, typically
    /// because of a permission or other IO error.
    NotReadable,
}

/// The outcome of a [`read_confined_bytes`] call: the file's raw bytes, or the
/// reason it could not be read.
///
/// Mirrors [`ReadOutcome`] but carries undecoded bytes for the transcoding path
/// that honors a non-UTF-8 `encoding` attribute. Because there is no UTF-8
/// decoding step, no content is rejected as non-UTF-8 – the caller transcodes
/// the bytes. The failure reasons map onto the same [`IncludeResolution`]
/// variants.
pub(crate) enum ReadBytesOutcome {
    /// The file was read; its raw bytes are carried here.
    Read(Vec<u8>),

    /// No readable file exists at the path: it is missing, is not a regular
    /// file (e.g. a directory), or lies outside the jail.
    NotFound,

    /// A regular file exists at the path but could not be read, typically
    /// because of a permission or other IO error.
    NotReadable,
}

/// Reads the file at `path`, enforcing the safe mode's jail and classifying any
/// failure the way Asciidoctor does.
///
/// Under a jailed safe mode (`safe`/`server`) the real path is resolved
/// (following symlinks) and the read is refused when it escapes `base_dir`;
/// under `unsafe` the file is read directly. A path that does not resolve, sits
/// outside the jail, or is not a regular file yields [`ReadOutcome::NotFound`];
/// a regular file that cannot be read yields [`ReadOutcome::NotReadable`].
/// Shared by the include and docinfo handlers.
pub(crate) fn read_confined(base_dir: &Path, safe: SafeMode, path: &Path) -> ReadOutcome {
    match confined_target(base_dir, safe, path) {
        Some(real) => read_file(&real),
        None => ReadOutcome::NotFound,
    }
}

/// The raw-byte counterpart to [`read_confined`], used by the transcoding path
/// that honors a non-UTF-8 `encoding` attribute.
///
/// Enforces the same jail and classifies failures the same way, but returns the
/// file's undecoded bytes so the caller can transcode them rather than
/// rejecting anything that is not valid UTF-8.
pub(crate) fn read_confined_bytes(
    base_dir: &Path,
    safe: SafeMode,
    path: &Path,
) -> ReadBytesOutcome {
    match confined_target(base_dir, safe, path) {
        Some(real) => read_file_bytes(&real),
        None => ReadBytesOutcome::NotFound,
    }
}

/// Resolves `path` to the concrete file a read should target under `safe`, or
/// `None` when the jail forbids it.
///
/// Under a jailed safe mode (`safe`/`server`) the real path is canonicalized
/// (following symlinks) and must remain within `base_dir`; a path that does not
/// resolve or escapes the jail yields `None`, which callers report as a
/// "not found" outcome. Under `unsafe` there is no jail and `path` is used
/// as-is. Shared by the UTF-8 ([`read_confined`]) and raw-byte
/// ([`read_confined_bytes`]) reads.
fn confined_target(base_dir: &Path, safe: SafeMode, path: &Path) -> Option<PathBuf> {
    if !jailed(safe) {
        return Some(path.to_path_buf());
    }

    // Lexical resolution keeps the *path* inside the base directory, but a
    // symlink under the base directory could still point outside it. Resolve the
    // real path (following symlinks) and refuse the read when it escapes the base
    // directory. This is stricter than Asciidoctor, whose jail is purely lexical
    // (see the module docs); canonicalization also fails for a path the recovery
    // relocated to somewhere that does not exist. Either way the resource is
    // unavailable, which reads as "not found".
    let real = path.canonicalize().ok()?;
    real.starts_with(base_dir).then_some(real)
}

/// Reads `path` as UTF-8 text, classifying failure the way Asciidoctor does: a
/// path that is not a regular file is [`ReadOutcome::NotFound`] (Asciidoctor
/// gates on `File.file?` before reading), while a regular file whose read fails
/// is [`ReadOutcome::NotReadable`]. Non-UTF-8 content is treated as not found:
/// this is the UTF-8 read path, taken when no recognized `encoding` attribute
/// asks for transcoding, so undecodable bytes cannot be honored.
fn read_file(path: &Path) -> ReadOutcome {
    if !path.is_file() {
        return ReadOutcome::NotFound;
    }

    match fs::read_to_string(path) {
        Ok(content) => ReadOutcome::Read(content),

        // Both of these read as "not found": non-UTF-8 content (this handler
        // only deals in UTF-8 and does not transcode, matching the parser's
        // contract), and a file that vanished between the `is_file` check and
        // the read — a race with concurrent file generation or cleanup — which
        // surfaces as `NotFound` and must not be mistaken for an unreadable file.
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::InvalidData | io::ErrorKind::NotFound
            ) =>
        {
            ReadOutcome::NotFound
        }

        // Any other IO failure on a regular file (typically a permission error)
        // is a genuine "not readable".
        Err(_) => ReadOutcome::NotReadable,
    }
}

/// Reads `path` as raw bytes for the transcoding path, mirroring
/// [`read_file`]'s classification but without a decoding step, so nothing is
/// rejected for invalid UTF-8. A path that is not a regular file is
/// [`ReadBytesOutcome::NotFound`] (Asciidoctor's `File.file?` gate); a regular
/// file whose read fails is [`ReadBytesOutcome::NotReadable`]. The caller
/// transcodes the returned bytes per the requested `encoding`.
fn read_file_bytes(path: &Path) -> ReadBytesOutcome {
    if !path.is_file() {
        return ReadBytesOutcome::NotFound;
    }

    // The `is_file` gate already reports a missing or non-regular target as "not
    // found", so any failure reading a path that just passed it is a genuine read
    // failure – typically a permission error, or the rare race where the file is
    // removed in between – and reads as "not readable".
    match fs::read(path) {
        Ok(bytes) => ReadBytesOutcome::Read(bytes),
        Err(_) => ReadBytesOutcome::NotReadable,
    }
}

/// A legacy single-byte encoding this handler can transcode to UTF-8 when an
/// `include::` directive's `encoding` attribute names it.
///
/// Both encodings agree with Unicode on the ASCII range (0x00..=0x7F) and on
/// the high Latin-1 range (0xA0..=0xFF); they differ only in 0x80..=0x9F, where
/// ISO-8859-1 carries the C1 control characters (each byte mapping to the
/// scalar of the same value) and windows-1252 carries printable characters
/// instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LegacyEncoding {
    /// ISO-8859-1 (Latin-1): every byte maps to the Unicode scalar of the same
    /// value (U+0000..=U+00FF).
    Iso8859_1,

    /// windows-1252: Latin-1 with printable characters in place of the
    /// 0x80..=0x9F C1 controls.
    Windows1252,
}

impl LegacyEncoding {
    /// Maps an `encoding` attribute label to the encoding this handler
    /// transcodes from, or `None` for UTF-8 or a label it does not recognize
    /// (both of which are read as UTF-8). Matching is case-insensitive.
    fn from_label(label: &str) -> Option<Self> {
        match label.to_ascii_lowercase().as_str() {
            "iso-8859-1" | "iso8859-1" | "latin1" | "latin-1" => Some(Self::Iso8859_1),
            "windows-1252" | "cp1252" | "cp-1252" => Some(Self::Windows1252),
            _ => None,
        }
    }

    /// Decodes `bytes` to an owned UTF-8 string. Every byte yields exactly one
    /// character, so decoding always succeeds.
    fn decode(self, bytes: &[u8]) -> String {
        bytes.iter().map(|&byte| self.decode_byte(byte)).collect()
    }

    /// Maps a single byte to its character in this encoding.
    fn decode_byte(self, byte: u8) -> char {
        match self {
            Self::Iso8859_1 => byte as char,

            Self::Windows1252 => match byte {
                0x80..=0x9f => WINDOWS_1252_C1[(byte - 0x80) as usize],
                other => other as char,
            },
        }
    }
}

/// The 0x80..=0x9F rows of windows-1252, where it departs from ISO-8859-1. Each
/// entry is the character for byte `0x80 + index`. The five positions WHATWG
/// leaves undefined map to the C1 control of the same value, so every byte
/// still decodes to exactly one character.
const WINDOWS_1252_C1: [char; 32] = [
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', // 0x80..=0x83
    '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}', // 0x84..=0x87
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', // 0x88..=0x8B
    '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}', // 0x8C..=0x8F
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', // 0x90..=0x93
    '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', // 0x94..=0x97
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', // 0x98..=0x9B
    '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}', // 0x9C..=0x9F
];

/// Returns the [`LegacyEncoding`] the directive's `encoding` attribute asks
/// this handler to transcode from, or `None` when the file should be read as
/// UTF-8 (no `encoding` attribute, an explicit UTF-8 label, or an unrecognized
/// label, which Asciidoctor ignores).
fn legacy_encoding(attrlist: &Attrlist) -> Option<LegacyEncoding> {
    LegacyEncoding::from_label(attrlist.named_attribute("encoding")?.value())
}

impl IncludeFileHandler for FsIncludeFileHandler {
    fn resolve_target<'src>(
        &self,
        source: Option<&str>,
        target: &str,
        attrlist: &Attrlist<'src>,
        _parser: &Parser,
    ) -> IncludeResolution {
        let path = self.resolve(source, target);

        // A non-UTF-8 `encoding` this handler recognizes (e.g. `iso-8859-1`) is
        // honored by reading the raw bytes and transcoding them to UTF-8; the
        // content is returned via `IncludeContent::transcoded` so the parser
        // suppresses its non-UTF-8 include-encoding warning. Every other case –
        // no `encoding`, an explicit `utf-8`, or an unrecognized label (which
        // Asciidoctor ignores) – falls back to reading the file as UTF-8 and
        // returning it via `IncludeContent::new`, letting the parser emit that
        // warning itself if a non-UTF-8 encoding was requested. In both paths a
        // missing/non-regular file maps to `NotFound` and an unreadable one to
        // `NotReadable`, so the parser can distinguish Asciidoctor's `include
        // file not found` from `include file not readable`.
        match legacy_encoding(attrlist) {
            Some(encoding) => match read_confined_bytes(&self.base_dir, self.safe, &path) {
                ReadBytesOutcome::Read(bytes) => {
                    IncludeResolution::Found(IncludeContent::transcoded(encoding.decode(&bytes)))
                }

                ReadBytesOutcome::NotFound => IncludeResolution::NotFound,

                ReadBytesOutcome::NotReadable => IncludeResolution::NotReadable,
            },

            None => match read_confined(&self.base_dir, self.safe, &path) {
                ReadOutcome::Read(content) => {
                    IncludeResolution::Found(IncludeContent::new(strip_utf8_bom(content)))
                }

                ReadOutcome::NotFound => IncludeResolution::NotFound,

                ReadOutcome::NotReadable => IncludeResolution::NotReadable,
            },
        }
    }
}

/// Removes a leading UTF-8 byte-order mark (`U+FEFF`) from `content`, matching
/// Asciidoctor, which strips a BOM from the first line of an included file so
/// it does not leak into the first parsed line. Content without a BOM is
/// returned unchanged.
fn strip_utf8_bom(mut content: String) -> String {
    if content.starts_with('\u{feff}') {
        content.drain(..'\u{feff}'.len_utf8());
    }
    content
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
        directory_of, fold_into, is_absolute, normalize, posixify, strip_root, strip_utf8_bom,
        FsIncludeFileHandler, LegacyEncoding,
    };

    const BASE: &str = "/home/user/project";

    // `from_label` recognizes the ISO-8859-1 and windows-1252 labels
    // (case-insensitively) and treats UTF-8 and unknown labels as "read as
    // UTF-8" (`None`).
    #[test]
    fn legacy_encoding_from_label_recognizes_known_labels() {
        assert_eq!(
            LegacyEncoding::from_label("iso-8859-1"),
            Some(LegacyEncoding::Iso8859_1)
        );
        assert_eq!(
            LegacyEncoding::from_label("ISO-8859-1"),
            Some(LegacyEncoding::Iso8859_1)
        );
        assert_eq!(
            LegacyEncoding::from_label("latin1"),
            Some(LegacyEncoding::Iso8859_1)
        );
        assert_eq!(
            LegacyEncoding::from_label("windows-1252"),
            Some(LegacyEncoding::Windows1252)
        );
        assert_eq!(
            LegacyEncoding::from_label("CP1252"),
            Some(LegacyEncoding::Windows1252)
        );

        // UTF-8 and an unrecognized label both read as UTF-8 (no transcoding).
        assert_eq!(LegacyEncoding::from_label("utf-8"), None);
        assert_eq!(LegacyEncoding::from_label("iso-1000-1"), None);
    }

    // ISO-8859-1 maps every byte to the Unicode scalar of the same value,
    // including the reader-test fixture's bytes.
    #[test]
    fn iso_8859_1_decodes_each_byte_to_the_same_scalar() {
        // The `iso-8859-1.txt` fixture: `O`, 0xF9 (ù), ` est l'h`, 0xF4 (ô),
        // `pital ?`.
        let bytes = [
            0x4f, 0xf9, 0x20, 0x65, 0x73, 0x74, 0x20, 0x6c, 0x27, 0x68, 0xf4, 0x70, 0x69, 0x74,
            0x61, 0x6c, 0x20, 0x3f,
        ];
        assert_eq!(
            LegacyEncoding::Iso8859_1.decode(&bytes),
            "Où est l'hôpital ?"
        );

        // Endpoints of every sub-range map to the scalar of the same value.
        assert_eq!(
            LegacyEncoding::Iso8859_1.decode(&[0x00, 0x7f, 0x80, 0xa0, 0xff]),
            "\u{00}\u{7f}\u{80}\u{a0}\u{ff}"
        );
    }

    // windows-1252 agrees with ISO-8859-1 outside 0x80..=0x9F but carries
    // printable characters within it.
    #[test]
    fn windows_1252_decodes_the_c1_range_to_printable_characters() {
        // 0x80 -> euro, 0x92 -> right single quote, 0x99 -> trademark.
        assert_eq!(
            LegacyEncoding::Windows1252.decode(&[0x80, 0x92, 0x99]),
            "€’™"
        );

        // ASCII and the high Latin-1 range decode the same as ISO-8859-1.
        assert_eq!(LegacyEncoding::Windows1252.decode(&[0x41, 0xe9]), "Aé");
    }

    #[test]
    fn strip_utf8_bom_removes_only_a_leading_bom() {
        // A leading BOM is removed; the rest of the content is untouched.
        assert_eq!(
            strip_utf8_bom("\u{feff}= Title\nbody".to_owned()),
            "= Title\nbody"
        );
        // Content without a BOM is returned unchanged.
        assert_eq!(strip_utf8_bom("= Title".to_owned()), "= Title");
        // A BOM only counts at the very start, not mid-content.
        assert_eq!(strip_utf8_bom("a\u{feff}b".to_owned()), "a\u{feff}b");
        // Empty content is fine.
        assert_eq!(strip_utf8_bom(String::new()), "");
    }

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

    // Direct coverage of `read_confined`'s outcome classification: each read
    // failure is mapped the way the include handler needs, distinguishing a
    // missing/non-file target (`NotFound`) from a present-but-unreadable one
    // (`NotReadable`), with non-UTF-8 content folded into `NotFound`.
    mod read_outcomes {
        use std::{
            fs,
            path::PathBuf,
            sync::atomic::{AtomicU64, Ordering},
        };

        use asciidoc_parser::SafeMode;

        use crate::include_handler::{
            read_confined, read_confined_bytes, ReadBytesOutcome, ReadOutcome,
        };

        /// Creates a fresh, canonicalized temp directory (unique per call) so
        /// each case gets an isolated sandbox and the paths it builds share one
        /// absolute form.
        fn scratch() -> PathBuf {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("ahtml5-read-{}-{unique}", std::process::id()));
            fs::create_dir_all(&dir).expect("create scratch dir");
            dir.canonicalize().expect("canonicalize scratch dir")
        }

        // A readable UTF-8 file is read; its content is returned verbatim.
        #[test]
        fn a_readable_file_is_read() {
            let dir = scratch();
            let file = dir.join("part.adoc");
            fs::write(&file, "Body.\n").expect("write file");

            let outcome = read_confined(&dir, SafeMode::Unsafe, &file);
            assert!(
                matches!(&outcome, ReadOutcome::Read(content) if content == "Body.\n"),
                "unexpected outcome for a readable file",
            );

            let _ = fs::remove_dir_all(&dir);
        }

        // A path with no file at it is "not found".
        #[test]
        fn a_missing_path_is_not_found() {
            let dir = scratch();
            let missing = dir.join("absent.adoc");

            assert!(matches!(
                read_confined(&dir, SafeMode::Unsafe, &missing),
                ReadOutcome::NotFound
            ));

            let _ = fs::remove_dir_all(&dir);
        }

        // A path that resolves to a directory (not a regular file) is "not
        // found", matching Asciidoctor's `File.file?` gate rather than being
        // treated as an unreadable file.
        #[test]
        fn a_directory_is_not_found() {
            let dir = scratch();
            let subdir = dir.join("nested");
            fs::create_dir_all(&subdir).expect("create nested dir");

            assert!(matches!(
                read_confined(&dir, SafeMode::Unsafe, &subdir),
                ReadOutcome::NotFound
            ));

            let _ = fs::remove_dir_all(&dir);
        }

        // A regular file whose bytes are not valid UTF-8 is reported as "not
        // found": this handler only reads UTF-8 and does not transcode.
        #[test]
        fn a_non_utf8_file_is_not_found() {
            let dir = scratch();
            let file = dir.join("latin1.adoc");
            // `0xFF` is not a valid UTF-8 lead byte, so the read fails to decode.
            fs::write(&file, [b'A', 0xff, b'B']).expect("write non-utf8 file");

            assert!(matches!(
                read_confined(&dir, SafeMode::Unsafe, &file),
                ReadOutcome::NotFound
            ));

            let _ = fs::remove_dir_all(&dir);
        }

        // The raw-byte read returns the same non-UTF-8 file's bytes verbatim
        // (rather than folding it into `NotFound` as the UTF-8 read does), so the
        // transcoding path can decode them per the `encoding` attribute.
        #[test]
        fn a_non_utf8_file_is_read_as_raw_bytes() {
            let dir = scratch();
            let file = dir.join("latin1.adoc");
            fs::write(&file, [b'A', 0xff, b'B']).expect("write non-utf8 file");

            let outcome = read_confined_bytes(&dir, SafeMode::Unsafe, &file);
            assert!(
                matches!(&outcome, ReadBytesOutcome::Read(bytes) if bytes == &[b'A', 0xff, b'B']),
                "unexpected outcome for a raw-byte read",
            );

            let _ = fs::remove_dir_all(&dir);
        }

        // A missing target is "not found" on the raw-byte read too.
        #[test]
        fn a_missing_path_is_not_found_for_raw_bytes() {
            let dir = scratch();
            let missing = dir.join("absent.adoc");

            assert!(matches!(
                read_confined_bytes(&dir, SafeMode::Unsafe, &missing),
                ReadBytesOutcome::NotFound
            ));

            let _ = fs::remove_dir_all(&dir);
        }

        // A regular file that exists but cannot be read (permission stripped) is
        // "not readable", distinct from "not found". Permission bits only bite on
        // Unix and not as root, mirroring the reader-test gate.
        #[cfg(unix)]
        #[test]
        fn an_unreadable_file_is_not_readable() {
            use std::os::unix::fs::PermissionsExt;

            let dir = scratch();
            let file = dir.join("locked.adoc");
            fs::write(&file, "secret\n").expect("write file");
            fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).expect("chmod file");

            // Root bypasses permission bits, so the file stays readable; skip.
            if fs::read_to_string(&file).is_ok() {
                let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o644));
                let _ = fs::remove_dir_all(&dir);
                return;
            }

            let outcome = read_confined(&dir, SafeMode::Unsafe, &file);

            let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o644));
            let _ = fs::remove_dir_all(&dir);

            assert!(matches!(outcome, ReadOutcome::NotReadable));
        }
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

        // A non-UTF-8 include named with a matching `encoding` attribute is
        // transcoded to UTF-8 and its content is rendered in place; without the
        // attribute the same file is undecodable and left unresolved.
        #[test]
        fn encoding_attribute_transcodes_a_non_utf8_include() {
            let main = project("encoding");
            let base = main.parent().expect("base dir");

            // `latin1.txt` holds ISO-8859-1 bytes for `Où est l'hôpital ?`
            // (0xF9 = ù, 0xF4 = ô).
            fs::write(
                base.join("latin1.txt"),
                [
                    0x4f, 0xf9, 0x20, 0x65, 0x73, 0x74, 0x20, 0x6c, 0x27, 0x68, 0xf4, 0x70, 0x69,
                    0x74, 0x61, 0x6c, 0x20, 0x3f, 0x0a,
                ],
            )
            .expect("write latin1 file");

            // With `encoding=iso-8859-1` the bytes are transcoded and rendered.
            fs::write(
                &main,
                "= Main\n\n....\ninclude::latin1.txt[encoding=iso-8859-1]\n....\n",
            )
            .expect("rewrite main");
            let html = convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();
            assert!(html.contains("Où est l'hôpital ?"), "{html}");

            // Without the attribute the same file is not valid UTF-8, so it is
            // left unresolved and the content never appears.
            fs::write(&main, "= Main\n\n....\ninclude::latin1.txt[]\n....\n")
                .expect("rewrite main");
            let html = convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();
            assert!(!html.contains("Où est l'hôpital ?"), "{html}");
        }

        // An `encoding` include whose target does not exist is left unresolved,
        // just like a UTF-8 include of a missing file: the transcoding read
        // reports "not found", so the directive is replaced with an "Unresolved
        // directive" message.
        #[test]
        fn encoding_include_of_a_missing_file_is_unresolved() {
            let main = project("encoding-missing");
            fs::write(
                &main,
                "= Main\n\n....\ninclude::no-such.txt[encoding=iso-8859-1]\n....\n",
            )
            .expect("rewrite main");

            let html = convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();
            assert!(html.contains("Unresolved directive"), "{html}");
        }

        // An `encoding` include of a present-but-unreadable file is left
        // unresolved via the "not readable" outcome (distinct from "not found"),
        // just like a UTF-8 include. Simulating an unreadable file relies on Unix
        // permission bits and does not work as root, so this is Unix-only and
        // skips under root.
        #[cfg(unix)]
        #[test]
        fn encoding_include_of_an_unreadable_file_is_unresolved() {
            use std::os::unix::fs::PermissionsExt;

            let main = project("encoding-unreadable");
            let base = main.parent().expect("base dir");

            // ISO-8859-1 bytes (`Où`), then strip permissions so the file exists
            // but cannot be read.
            let locked = base.join("locked.txt");
            fs::write(&locked, [0x4f, 0xf9, 0x0a]).expect("write locked file");
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("chmod");

            // Root bypasses permission bits, so the file stays readable; skip.
            if fs::read(&locked).is_ok() {
                let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));
                return;
            }

            fs::write(
                &main,
                "= Main\n\n....\ninclude::locked.txt[encoding=iso-8859-1]\n....\n",
            )
            .expect("rewrite main");

            let html = convert_file_with(&main, &Options::new().safe_mode(SafeMode::Safe)).unwrap();

            let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));

            assert!(html.contains("Unresolved directive"), "{html}");
            assert!(!html.contains("Où"), "{html}");
        }
    }
}
