use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/ROOT/pages/safe-modes.adoc");

// This crate's "Safe Modes" page. It documents the four safe modes and their
// levels, how the safe mode is set, its effect on the default stylesheet, and
// the `safe-mode-*` attributes a document can reference. Every concrete claim
// is verified against `asciidoc_html5`. Tracked from the library crate; the
// CLI's own default (`unsafe`) is verified on xref cli:set-safe-mode.
//
// The known-limitations prose describes features this renderer does not surface
// yet, so it carries no rule to verify.

non_normative!(
    r#"
= Safe Modes
:navtitle: Safe Modes
:description: How asciidoc-html5's safe mode controls security-sensitive rendering and whether the default stylesheet is linked or embedded.

`asciidoc-html5` models Asciidoctor's _safe mode_: a security level that controls
how far a document may reach outside itself while it is processed. There are four
modes, in order of increasing safety -- `unsafe`, `safe`, `server`, and `secure`
-- and each includes the restrictions of the modes below it.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

"#
);

// The four modes and their numeric levels, and the API default (`secure`).
#[test]
fn the_four_modes_and_levels() {
    verifies!(
        r#"
== The four modes

Each mode has a name and a numeric level:

[cols="1,1,3"]
|===
|Mode |Level |Summary

|`unsafe` |0 |No safe-mode restrictions. The default for the `adoc` command.
|`safe` |1 |Keeps file access within the document's own directory tree.
|`server` |10 |A document may not change settings that affect its own conversion.
|`secure` |20 |The most restrictive mode; links the stylesheet. The API default.
|===

Set the safe mode xref:cli:set-safe-mode.adoc[from the CLI] with `-S`/`--safe-mode`,
or xref:api:set-safe-mode.adoc[from the API] with `Options::safe_mode`. The
defaults differ to match Asciidoctor: the API defaults to `secure`, while the
`adoc` command defaults to `unsafe`.

"#
    );

    assert_eq!(SafeMode::Unsafe as u8, 0);
    assert_eq!(SafeMode::Safe as u8, 1);
    assert_eq!(SafeMode::Server as u8, 10);
    assert_eq!(SafeMode::Secure as u8, 20);

    // The API default (no safe mode set) is `secure`, which links the stylesheet.
    assert!(convert("= Doc\n\nBody.").contains("./asciidoctor.css"));
}

// The stylesheet link-versus-embed effect: the exact snippet the page shows.
#[test]
fn the_safe_mode_controls_stylesheet_linking() {
    verifies!(
        r#"
== Effect on the default stylesheet

Today the safe mode's most visible effect is whether the
xref:generate-html:default-stylesheet.adoc[default stylesheet] is _linked_ or
_embedded_. Under `secure`, the converter links to _asciidoctor.css_ (a secure
processor does not read the stylesheet file in order to embed it); under a lower
mode, it embeds the stylesheet inline:

[,rust]
----
use asciidoc_html5::{convert, convert_with, Options, SafeMode};

// Secure (the API default) links the stylesheet.
assert!(convert("= Doc\n\nBody.").contains("./asciidoctor.css"));

// A lower mode embeds it inline.
let html = convert_with("= Doc\n\nBody.", &Options::new().safe_mode(SafeMode::Server));
assert!(html.contains("<style>"));
----

"#
    );

    // Secure (the API default) links the stylesheet.
    assert!(convert("= Doc\n\nBody.").contains("./asciidoctor.css"));

    // A lower mode embeds it inline.
    let html = convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Server),
    );
    assert!(html.contains("<style>"));
}

// A document can reference the current safe mode through the built-in
// attributes.
#[test]
fn a_document_can_reference_the_safe_mode() {
    verifies!(
        r#"
== Reference the safe mode from a document

The current safe mode is available to a document through three built-in
attributes, so content can adapt to it:

* `safe-mode-name` -- the mode name (`unsafe`, `safe`, `server`, or `secure`).
* `safe-mode-level` -- the numeric level (`0`, `1`, `10`, or `20`).
* `safe-mode-<name>` -- set only for the active mode, so `ifdef::safe-mode-secure[]` gates content on it.

For example, this document reports its level and shows a line only under `secure`:

[,asciidoc]
----
The current level is {safe-mode-level}.

ifdef::safe-mode-secure[]
Running securely.
endif::safe-mode-secure[]
----

"#
    );

    // `safe-mode-name` and `safe-mode-level` resolve to the current mode.
    let secure = convert("= Doc\n\nlevel={safe-mode-level} name={safe-mode-name}");
    assert!(secure.contains("level=20"));
    assert!(secure.contains("name=secure"));

    // `safe-mode-<name>` is set only for the active mode, so `ifdef` gates on it.
    let gated =
        "= Doc\n\nifdef::safe-mode-secure[]\nRunning securely.\nendif::safe-mode-secure[]\n";
    assert!(convert(gated).contains("Running securely."));
    assert!(
        !convert_with(gated, &Options::new().safe_mode(SafeMode::Unsafe))
            .contains("Running securely.")
    );
}

non_normative!(
    r#"
These attributes can only be established when the document is processed; a
document cannot change them itself.

== Known limitations

Most of what a safe mode governs in Asciidoctor concerns features this renderer
does not surface yet -- include directives, icons, `data-uri`, source
highlighting, `docinfo`, and backend locking. Today the safe mode's observable
effect is the stylesheet link-versus-embed choice described above; as those
other features arrive, they will honor the safe mode too.
"#
);
