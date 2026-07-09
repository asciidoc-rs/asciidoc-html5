use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/ROOT/pages/reference-safe-mode.adoc");

// Asciidoctor's "Safe Mode Specific Content" page, tracked from the library
// crate. The parser this crate builds on exposes the same built-in attributes —
// `safe-mode-name`, `safe-mode-level`, and the per-mode `safe-mode-<name>` flag
// — so a document can reference the current safe mode or gate content on it
// with `ifdef`. That is what the test below drives. The commented-out
// `allow-uri-read` / standalone design notes describe features outside this
// crate's scope, so they are non-normative.

non_normative!(
    r#"
= Safe Mode Specific Content
// anchor: set-safe-attrs

Asciidoctor provides access to the current safe mode through built-in attributes.
You can use these attributes to enable or disable content based on the current safe mode of the processor.

"#
);

// The current safe mode is reachable from a document three ways: the
// `safe-mode-name` value, the `safe-mode-level` value, and the presence of the
// `safe-mode-<name>` flag (usable with `ifdef`). The parser populates all
// three, so a document can reference or gate on them.
#[test]
fn a_document_can_reference_the_current_safe_mode() {
    verifies!(
        r#"
== Referencing safe modes

The xref:safe-modes.adoc[safe mode] can be referenced by one of the following document attributes:

* The value of the `safe-mode-name` attribute (e.g., unsafe, safe, etc.)
* The value of the `safe-mode-level` attribute (e.g., 0, 10, etc.)
* The presence of the `safe-mode-<name>` attribute, where `<name>` is the safe mode name.

The attributes in the next example define replacement text for features that are disabled in high security environments:

[,asciidoc]
----
\ifdef::safe-mode-secure[]
Link to chapters instead of including them.
\endif::safe-mode-secure[]
----

This feature is particularly handy for displaying content on GitHub, where the safe mode is set to its most restrictive setting, xref:safe-modes.adoc#secure[SECURE].

"#
    );

    // `safe-mode-name` and `safe-mode-level` resolve to the current mode.
    let secure = convert("= Doc\n\nname={safe-mode-name} level={safe-mode-level}");
    assert!(secure.contains("name=secure"));
    assert!(secure.contains("level=20"));

    let unsafe_mode = convert_with(
        "= Doc\n\nname={safe-mode-name} level={safe-mode-level}",
        &Options::new().safe_mode(SafeMode::Unsafe),
    );
    assert!(unsafe_mode.contains("name=unsafe"));
    assert!(unsafe_mode.contains("level=0"));

    // The presence of `safe-mode-<name>` gates content via `ifdef`: the secure
    // flag is set only under SECURE.
    let gated = "= Doc\n\nifdef::safe-mode-secure[]\nLink to chapters instead of including them.\nendif::safe-mode-secure[]\n";
    assert!(convert(gated).contains("Link to chapters instead of including them."));
    assert!(
        !convert_with(gated, &Options::new().safe_mode(SafeMode::Unsafe))
            .contains("Link to chapters instead of including them.")
    );
}

// The closing cross-references are navigational, and the commented-out design
// notes (URI includes with `allow-uri-read`, and the `standalone` default)
// describe features this crate does not model.
non_normative!(
    r#"
You can set the xref:cli:set-safe-mode.adoc[safe mode from the CLI] and the xref:api:set-safe-mode.adoc[API].

////
Allow the include directive to import a file from a URI.

Example:

 include::https://cdn.jsdelivr.net/gh/asciidoctor/asciidoctor/README.adoc[]

To be secure by default, the allow-uri-read attribute must be set in the API or CLI (not document) for this feature to be enabled. It's also completely disabled if the safe mode is SECURE or greater.
Since this is a potentially dangerous feature, it’s disabled if the safe mode is SECURE or greater. Assuming the safe mode is less than SECURE, you must also set the allow-uri-read attribute to permit Asciidoctor to read content from a URI.

I decided the following defaults for the standalone option make the most sense:

true if using the cli (use -s to disable, consistent with asciidoc)
false if using the API, unless converting directly to a file, in which case true is the default
The basic logic is that if you are writing to a file, you probably want to create a standalone document. If you are converting to a string, then you probably want an embedded document. Of course, you can always set it explicitly, this is just a default setting.

The reason I think the standalone default is important is because we don't want people switching from Markdown to AsciiDoc and be totally taken by surprise when they start getting a full HTML document. On the other hand, if you are converting to a file (or using the cli), then it makes a lot of sense to write a standalone document. To me, it just feels natural now.
////
"#
);
