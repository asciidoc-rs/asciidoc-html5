use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/ROOT/pages/reference-safe-mode.adoc");

// This crate's "Safe Mode Specific Content" page. It documents the built-in
// `safe-mode-name`, `safe-mode-level`, and `safe-mode-<name>` attributes and
// shows how a document references the current safe mode and gates content on it
// with `ifdef`. Both snippets are verified against `asciidoc_html5`.
//
// The introduction, the cross-references under "Setting the safe mode", and the
// known-limitations prose describe navigation or features this renderer does
// not surface yet, so they carry no rule to verify.

non_normative!(
    r#"
= Safe Mode Specific Content
:navtitle: Safe Mode Specific Content
:description: How a document can reference the current safe mode and gate content on it using asciidoc-html5's built-in safe-mode attributes.

`asciidoc-html5` exposes the current xref:safe-modes.adoc[safe mode] to a
document through built-in attributes. You can use them to enable or disable
content based on the safe mode the processor is running under.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc and API
snippets it shows are normative: they are verified against the implementation,
so the documented behavior is guaranteed.
====

"#
);

// The current mode is reachable through `safe-mode-name` and `safe-mode-level`,
// which a document can print directly.
#[test]
fn a_document_can_reference_the_current_safe_mode() {
    verifies!(
        r#"
== Referencing the safe mode

The safe mode can be referenced through one of three document attributes:

* `safe-mode-name` -- the mode's name (for example, `unsafe` or `secure`).
* `safe-mode-level` -- the mode's numeric level (for example, `0` or `20`).
* `safe-mode-<name>` -- present only for the active mode, where `<name>` is that mode's name.

The first two carry the value of the current mode, so you can print it directly:

[,asciidoc]
----
This document was processed in {safe-mode-name} mode (level {safe-mode-level}).
----

"#
    );

    let line =
        "= Doc\n\nThis document was processed in {safe-mode-name} mode (level {safe-mode-level}).";

    // Secure is the API default.
    assert!(convert(line).contains("processed in secure mode (level 20)"));

    // A lower mode resolves to its own name and level.
    let unsafe_mode = convert_with(line, &Options::new().safe_mode(SafeMode::Unsafe));
    assert!(unsafe_mode.contains("processed in unsafe mode (level 0)"));
}

// `safe-mode-<name>` is defined only for the active mode, so `ifdef` gates
// content on it.
#[test]
fn content_can_be_gated_on_the_safe_mode() {
    verifies!(
        r#"
== Gate content on the safe mode

Because `safe-mode-<name>` is defined only for the active mode, an `ifdef`
directive can supply replacement text for features that are disabled in more
restrictive environments:

[,asciidoc]
----
ifdef::safe-mode-secure[]
Link to chapters instead of including them.
endif::safe-mode-secure[]
----

Under `secure` that line is emitted; under any lower mode it is dropped. This is
particularly handy for content displayed on GitHub, where the safe mode is set
to its most restrictive setting, xref:safe-modes.adoc[`secure`].

"#
    );

    let gated = "= Doc\n\nifdef::safe-mode-secure[]\nLink to chapters instead of including them.\nendif::safe-mode-secure[]\n";

    // Secure (the API default) emits the gated line.
    assert!(convert(gated).contains("Link to chapters instead of including them."));

    // A lower mode drops it.
    assert!(
        !convert_with(gated, &Options::new().safe_mode(SafeMode::Unsafe))
            .contains("Link to chapters instead of including them.")
    );
}

non_normative!(
    r#"
== Setting the safe mode

These attributes reflect the safe mode the processor was given; a document
cannot change them itself. Set the safe mode xref:cli:set-safe-mode.adoc[from the
CLI] with `-S`/`--safe-mode`, or xref:api:set-safe-mode.adoc[from the API] with
`Options::safe_mode`. The defaults differ to match Asciidoctor: the API defaults
to `secure`, while the `adoc` command defaults to `unsafe`.

== Known limitations

The attributes above are populated for every conversion, so referencing and
gating on the safe mode work today. The features whose availability the safe
mode ultimately governs in Asciidoctor -- include directives, `data-uri`, icons,
source highlighting, and `docinfo` -- are not surfaced by this renderer yet, so
there is not much to gate on beyond the stylesheet link-versus-embed choice
described on the xref:safe-modes.adoc[Safe Modes] page. As those features
arrive, they will honor the safe mode too.
"#
);
