//! Tracks the Ruby test fixtures under `ref/asciidoctor/test/fixtures/`.
//!
//! These are support scripts the Asciidoctor test suite loads as test data;
//! none states a rule about rendered HTML output. Each is tracked verbatim as
//! `non_normative!` so it is honestly accounted for as a spec surface rather
//! than counted as uncovered (an untracked `.rb` under `test/` reports every
//! line as `0`). Module names substitute `_` for the fixtures' hyphenated file
//! names; each `track_file!` keeps the real path.

mod configure_stdin;
mod include_asciidoctor;
mod tagged_class;
mod tagged_class_enclosed;
mod undef_dir_home;
