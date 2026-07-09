use crate::{convert, convert_file_with, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/api/pages/set-safe-mode.adoc");

// Asciidoctor's "Set the Safe Mode Using the API" page, tracked from the
// library crate. Asciidoctor's `:safe` API option is this crate's
// `Options::safe_mode`, and the default (`SECURE`) is the same. The observable
// effect of the safe mode in this renderer is whether the default stylesheet is
// linked (`SECURE`) or embedded (a lower mode), so that is what the tests below
// key off.
//
// The Ruby-specific ways to express the option value — a string, a symbol, or
// an integer — have no counterpart in this crate's typed `SafeMode` enum, so
// those spans are non-normative.

// The default safe mode is `SECURE`, and the safe mode is changeable from the
// API. This crate's `Options::safe_mode` is the `:safe` option; leaving it
// unset defaults to `SafeMode::Secure` (which links the stylesheet), and
// setting it to a lower mode embeds it.
#[test]
fn the_api_default_is_secure_and_the_safe_mode_is_settable() {
    verifies!(
        r#"
= Set the Safe Mode Using the API
:navtitle: Set Safe Mode

When using Asciidoctor via the API, the default xref:ROOT:safe-modes.adoc[safe mode] is `SECURE`.
You can change the safe mode using the `:safe` API option.

"#
    );

    // The default (no `safe_mode`) is `Secure`: the stylesheet is linked.
    assert!(convert("= Doc\n\nBody.").contains("./asciidoctor.css"));

    // Changing the safe mode changes that: a lower mode embeds instead.
    let server = convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Server),
    );
    assert!(server.contains("<style>"));
}

// The safe mode is accepted by every conversion entry point. Asciidoctor's
// `:safe` option is honored by all entrypoints; here, `Options::safe_mode` is
// honored by both `convert_with` and `convert_file_with`.
#[test]
fn safe_mode_is_honored_by_every_entry_point() {
    verifies!(
        r#"
== Set :safe option

The safe mode can be controlled from the API using the `:safe` option.
The `:safe` option is accepted by all xref:index.adoc#entrypoints[entrypoint methods] (e.g., `Asciidoctor#convert_file`).

"#
    );

    let opts = Options::new().safe_mode(SafeMode::Server);

    // `convert_with` honors the safe mode (embeds under `Server`).
    assert!(convert_with("= Doc\n\nBody.", &opts).contains("<style>"));

    // `convert_file_with` honors it too, on the same source read from a file.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-set-safe-mode-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, "= Doc\n\nBody.").expect("write temp input");
    let from_file = convert_file_with(&path, &opts).expect("convert_file_with reads and renders");
    let _ = std::fs::remove_file(&path);
    assert!(from_file.contains("<style>"));
}

// The three Ruby representations of the `:safe` value — a string, a symbol, and
// an integer — are Ruby API details. This crate takes a typed `SafeMode` enum
// value (`SafeMode::Server` here) instead, so these spans are non-normative.
non_normative!(
    r#"
The `:safe` option accepts the safe mode as a string:

[,ruby]
----
Asciidoctor.convert_file 'doc.adoc', safe: 'server'
----

as a symbol (preferred):

[,ruby]
----
Asciidoctor.convert_file 'doc.adoc', safe: :server
----

as an integer:

[,ruby]
----
Asciidoctor.convert_file 'doc.adoc', safe: 10
----

You can also set the xref:cli:set-safe-mode.adoc[safe mode from the CLI] and xref:ROOT:reference-safe-mode.adoc[enable or disable content based on the current safe mode].
"#
);
