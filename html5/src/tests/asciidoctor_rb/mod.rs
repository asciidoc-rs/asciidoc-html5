//! Coverage of Asciidoctor's own Ruby test suite, vendored verbatim under
//! `ref/asciidoctor/test/`.
//!
//! Asciidoctor's `html5` backend is this renderer's compatibility oracle, so
//! its test suite is a spec source the `sdd` tool measures coverage against.
//! Each module here tracks one `*_test.rb` file, reproducing it line for line:
//! every Ruby `test` block we port becomes a `#[test]` whose `verifies!` block
//! reproduces those lines and re-expresses the Ruby `assert_xpath`/`assert_css`
//! assertions against this crate's output (see `crate::tests::assert_html`).
//! Ruby tests for behavior out of scope here — other backends, or features not
//! yet rendered — are tracked as `non_normative!`.

mod attribute_list_test;
mod preamble_test;
