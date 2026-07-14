use crate::{convert_file_with, tests::sdd::*, Options, SafeMode};

// These tests assert the standalone document shell (its stylesheet linking), so
// they render the string entry points in standalone mode explicitly, which
// default to embedded, body-only output. `convert_file_with` is standalone by
// default and is used directly.
fn convert(source: &str) -> String {
    crate::convert_with(source, &Options::new().standalone(true))
}

fn convert_with(source: &str, options: &Options) -> String {
    crate::convert_with(source, &options.clone().standalone(true))
}

track_file!("docs/modules/api/pages/set-safe-mode.adoc");

// This crate's "Set the Safe Mode Using the API" page. It documents that
// `Options::safe_mode` chooses the mode, that it applies to every `_with` entry
// point, and that leaving it unset keeps the `secure` default. The Rust
// snippets are verified here; the surrounding prose is non-normative.

non_normative!(
    r#"
= Set the Safe Mode Using the API
:navtitle: Set Safe Mode
:description: How to choose the safe mode when converting with the asciidoc_html5 API.

When you convert with the `asciidoc_html5` API, the default
xref:ROOT:safe-modes.adoc[safe mode] is `secure`. Change it with
`Options::safe_mode`, then convert with `convert_with` or `convert_file_with`.

[NOTE]
====
The prose on this page is non-normative documentation. The API invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// `Options::safe_mode` chooses the mode and applies to every `_with` entry
// point.
#[test]
fn safe_mode_applies_to_every_with_entry_point() {
    verifies!(
        r#"
== Set the safe mode

`Options::safe_mode` takes a `SafeMode` value -- `SafeMode::Unsafe`,
`SafeMode::Safe`, `SafeMode::Server`, or `SafeMode::Secure` -- and applies to
every `_with` entry point:

[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let opts = Options::new().safe_mode(SafeMode::Server);
let html = convert_with("= Doc\n\nBody.", &opts);
assert!(html.contains("<style>"));
----

"#
    );

    let opts = Options::new().safe_mode(SafeMode::Server);
    let html = convert_with("= Doc\n\nBody.", &opts);
    assert!(html.contains("<style>"));

    // The same safe mode applies through `convert_file_with`.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-set-safe-mode-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, "= Doc\n\nBody.").expect("write temp input");
    let from_file = convert_file_with(&path, &opts).expect("convert_file_with reads and renders");
    let _ = std::fs::remove_file(&path);
    assert!(from_file.contains("<style>"));
}

// Leaving the safe mode unset keeps the `secure` default, which links the
// stylesheet.
#[test]
fn the_default_is_secure() {
    verifies!(
        r##"
Leaving the safe mode unset keeps the default, `secure`, which links the default
stylesheet instead of embedding it:

[,rust]
----
let html = asciidoc_html5::convert("= Doc\n\nBody.");
assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
----

"##
    );

    let html = convert("= Doc\n\nBody.");
    assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
}

non_normative!(
    r#"
You can also set the xref:cli:set-safe-mode.adoc[safe mode from the CLI].
"#
);
