// No-op macros mirroring `asciidoc-parser`'s spec-coverage markers. They expand
// to nothing at compile time; the workspace's `sdd` tool reads them textually
// from each crate's `src/tests` tree to build a Codecov-compatible spec
// coverage report. See `sdd/README.md`.

// Use the track_file marker to declare which spec (or documentation) file the
// surrounding test module tracks.
macro_rules! track_file( ($($tt:tt)*) => {} );
pub(crate) use track_file;

// Use the non_normative marker to enclose lines of the tracked file that are
// non-normative: prose that describes rather than specifies, and so carries no
// rule for a test to verify.
macro_rules! non_normative( ($($tt:tt)*) => {} );
pub(crate) use non_normative;
