//! Coverage of Asciidoctor's own Ruby test suite, vendored verbatim under
//! `ref/asciidoctor/test/`, from the CLI's point of view.
//!
//! Asciidoctor is `adoc`'s compatibility oracle, so its command-line test suite
//! is the most direct statement of the behavior `adoc` must match. Each module
//! here tracks one `*_test.rb` file, reproducing it line for line: every Ruby
//! `test` block we port becomes a `#[test]` whose `verifies!` block reproduces
//! those lines and re-expresses the Ruby assertions against `adoc`'s own
//! pipeline (`Cli` parsing plus `run`/`run_with_input`). Ruby tests for
//! behavior `adoc` does not have — the Ruby CLI internals, flags with no
//! counterpart, other backends — are tracked as `non_normative!`.
//!
//! The `asciidoc-html5` library crate ports the library-facing Ruby suites
//! under its own `asciidoctor_rb` tree; this crate ports the CLI-facing ones,
//! starting with `invoker_test.rb`.

mod invoker_test;
