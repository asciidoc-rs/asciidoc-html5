use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/cli/pages/options.adoc");

// Asciidoctor's "CLI Options" page is a navigation stub: it carries no rule of
// its own, only a cross-reference to the option catalog in the `asciidoctor(1)`
// man page. `adoc` has no man page, and the options it supports are described
// on the task-specific pages of this crate's own `cli` documentation module
// (and listed by `adoc --help`), so the entire page is tracked as non-normative
// here.
//
// TODO (https://github.com/asciidoc-rs/asciidoc-html5/issues/94): Once `adoc`
// grows a man(1) page and its option catalog, promote any lines that become
// reproducible from non_normative! to verifies!.

non_normative!(
    r#"
= CLI Options

See xref:man1/asciidoctor.adoc#options[`asciidoctor` options].
"#
);
