//! The sink a conversion writes its auxiliary output files to.
//!
//! A conversion returns the HTML document as a string, but some options ask it
//! to emit *companion* files alongside that HTML — today just the stylesheet
//! copied under the `copycss` attribute (see the [`copycss`](crate::copycss)
//! module). The library computes each such file's contents and its path
//! *relative to the output directory*, but it does not own that directory: the
//! caller decides where the primary HTML lands, so the caller must decide where
//! the companion files land too.
//!
//! [`AssetWriter`] is the seam. A caller that wants `copycss` to take effect
//! hands one of the `_with_writer` entry points ([`convert_with_writer`],
//! [`convert_file_with_writer`]) an `AssetWriter`; the converter calls
//! [`AssetWriter::write_asset`] once per companion file, with a relative path
//! and the bytes to write. [`DirAssetWriter`] is the ready-made implementation
//! that writes to a directory tree on disk — what the `adoc` CLI uses, rooted
//! at the output file's directory.
//!
//! [`convert_with_writer`]: crate::convert_with_writer
//! [`convert_file_with_writer`]: crate::convert_file_with_writer

use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

/// A sink for the companion files a conversion produces alongside the HTML it
/// returns.
///
/// The converter calls [`write_asset`](Self::write_asset) once for each file,
/// passing a path *relative to the output directory* and the bytes to write.
/// The caller controls where the output directory is, and therefore where the
/// files ultimately land; see [`DirAssetWriter`] for the on-disk case.
///
/// Only the `copycss` stylesheet copy uses this today, so at most one asset is
/// written per conversion.
pub trait AssetWriter {
    /// Writes `content` to `path`, interpreted relative to the caller's output
    /// directory.
    ///
    /// `path` is always relative and never escapes the output directory (a
    /// [`DirAssetWriter`] additionally clamps it), so an implementation may
    /// join it under a root of its choosing. It may contain more than one
    /// component — the stylesheet copied under `copycss` mirrors its
    /// `stylesdir` web path, for example `css/theme.css`.
    ///
    /// # Errors
    ///
    /// Returns any [`io::Error`] the write encounters; the `_with_writer` entry
    /// points propagate it.
    fn write_asset(&mut self, path: &Path, content: &[u8]) -> io::Result<()>;
}

/// An [`AssetWriter`] that writes companion files to a directory tree rooted at
/// a caller-chosen output directory, creating intermediate directories as
/// needed.
///
/// This is the implementation the `adoc` CLI installs, rooted at the directory
/// that holds the output HTML, so a copied stylesheet lands next to it (or in
/// the `stylesdir` subdirectory the HTML links to).
///
/// The relative path each asset is written under is *clamped* to the root: its
/// root/prefix components are dropped and its `..` components resolved so the
/// write can never escape the output directory, matching the confinement the
/// jailed safe modes apply to reads.
#[derive(Clone, Debug)]
pub struct DirAssetWriter {
    /// The output directory every asset path is resolved under.
    root: PathBuf,
}

impl DirAssetWriter {
    /// Creates a writer that roots every asset at `root`, the output directory.
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }
}

impl AssetWriter for DirAssetWriter {
    fn write_asset(&mut self, path: &Path, content: &[u8]) -> io::Result<()> {
        // Fold the relative path onto the root a component at a time, keeping
        // only the normal segments: a root or drive prefix is dropped and a
        // `..` pops the previous segment (clamping at the root), so the result
        // can never climb above the output directory.
        let mut dest = self.root.clone();
        let mut depth = 0usize;
        for component in path.components() {
            match component {
                Component::Normal(part) => {
                    dest.push(part);
                    depth += 1;
                }
                Component::ParentDir if depth > 0 => {
                    dest.pop();
                    depth -= 1;
                }
                // A leading `..`, a root, a drive prefix, or a `.` contributes
                // nothing: the write stays anchored at the root.
                _ => {}
            }
        }

        // Create the destination's parent, unless it is empty (a bare file name
        // rooted at an empty root writes into the current directory, which needs
        // no `create_dir_all`).
        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(dest, content)
    }
}

/// An in-memory [`AssetWriter`] that records the `(path, content)` pairs it is
/// handed, so the copycss and entry-point tests can assert on what would be
/// written without touching the filesystem.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct RecordingAssetWriter {
    pub(crate) written: Vec<(PathBuf, Vec<u8>)>,
}

#[cfg(test)]
impl AssetWriter for RecordingAssetWriter {
    fn write_asset(&mut self, path: &Path, content: &[u8]) -> io::Result<()> {
        self.written.push((path.to_path_buf(), content.to_vec()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{AssetWriter, DirAssetWriter};

    // A multi-component relative path is written under the root, creating the
    // intermediate directory.
    #[test]
    fn writes_nested_asset_under_the_root() {
        let root = std::env::temp_dir().join(format!("adoc-asset-nested-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut writer = DirAssetWriter::new(&root);

        writer
            .write_asset(Path::new("css/theme.css"), b"body {}")
            .expect("write");

        let dest = root.join("css").join("theme.css");
        assert_eq!(std::fs::read(&dest).unwrap(), b"body {}");
        let _ = std::fs::remove_dir_all(&root);
    }

    // A path that tries to climb above the root with `..` (or an absolute root)
    // is clamped back inside it, so a write can never escape the output
    // directory.
    #[test]
    fn clamps_escaping_paths_to_the_root() {
        let root = std::env::temp_dir().join(format!("adoc-asset-clamp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut writer = DirAssetWriter::new(&root);

        writer
            .write_asset(Path::new("../../escape.css"), b"x")
            .expect("write");

        assert_eq!(std::fs::read(root.join("escape.css")).unwrap(), b"x");
        let _ = std::fs::remove_dir_all(&root);
    }
}
