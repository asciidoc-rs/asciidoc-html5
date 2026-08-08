//! Coverage of Asciidoctor's *Add a Favicon* page.
//!
//! This page documents the `favicon` document attribute, which adds a
//! `<link rel="icon">` reference to the `<head>` of a standalone document.

use crate::{tests::sdd::*, Options, SafeMode};

// These tests assert the standalone document shell, so they render in
// standalone mode explicitly. The string entry points default to embedded,
// body-only output.
fn convert(source: &str) -> String {
    crate::convert_with(source, &Options::new().standalone(true))
}

fn convert_with(source: &str, options: &Options) -> String {
    crate::convert_with(source, &options.clone().standalone(true))
}

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/favicon.adoc");

// This page is tracked from the library crate only: every verifiable claim is
// about the HTML `<head>` that `asciidoc_html5::convert`/`convert_with` emits.
// Where the page drives behavior with a document attribute, we supply the same
// attribute through the API; the `adoc` binary has no favicon-specific
// behavior of its own to track separately.

non_normative!(
    r#"
= Add a Favicon

When using Asciidoctor to generate a standalone HTML document (i.e., the `standalone` option is `true`), you can instruct the processor to include a link to a favicon by setting the favicon attribute in the document header.

"#
);

// A bare `:favicon:` (no value) defaults to a link referencing `favicon.ico`
// typed `image/x-icon`.
#[test]
fn bare_favicon_attribute_defaults_to_favicon_ico() {
    verifies!(
        r#"
[,asciidoc]
----
= Document Title
:favicon:
----

By default, the processor will add a link reference of type "icon" that refers to a file named _favicon.ico_ (relative to the HTML document):

[,html]
----
<link rel="icon" type="image/x-icon" href="favicon.ico">
----

"#
    );

    let html = convert("= Document Title\n:favicon:\n\nBody.");
    assert!(
        html.contains("<link rel=\"icon\" type=\"image/x-icon\" href=\"favicon.ico\">"),
        "{html}"
    );
}

// The favicon `<link>` is only ever added to a standalone document's `<head>`;
// an embeddable (non-standalone) conversion emits no `<head>` at all, so the
// `favicon` attribute has no effect there.
#[test]
fn favicon_link_is_not_added_to_embeddable_output() {
    verifies!(
        r#"
This reference gets added inside the HTML `<head>` element (which explains why this feature is not available when generating an embeddable HTML document).

"#
    );

    // Embedded (non-standalone) output — the crate's default entry point.
    let html = crate::convert("= Document Title\n:favicon:\n\nBody.");
    assert!(!html.contains("rel=\"icon\""), "{html}");
}

// Assigning a value to `favicon` uses it verbatim as the `href`, with the
// `<link>` `type` derived from the value's file extension.
#[test]
fn custom_favicon_path_sets_the_href_and_mimetype() {
    verifies!(
        r#"
To modify the name or location of the icon file, simply assign a value to the favicon attribute:

[,asciidoc]
----
= Document Title
:favicon: ./images/favicon/favicon.png
----

This will now produce the following HTML element:

[,html]
----
<link rel="icon" type="image/png" href="./images/favicon/favicon.png">
----

Notice the mimetype is automatically set based on the file extension of the image.

"#
    );

    let html = convert("= Document Title\n:favicon: ./images/favicon/favicon.png\n\nBody.");
    assert!(
        html.contains(
            "<link rel=\"icon\" type=\"image/png\" href=\"./images/favicon/favicon.png\">"
        ),
        "{html}"
    );
}

// `iconsdir` is not mirrored into the favicon path automatically — unlike the
// icons used in the document content — so a document that wants the favicon
// under `iconsdir` must reference the attribute explicitly in its value.
#[test]
fn iconsdir_is_not_prepended_unless_referenced_explicitly() {
    verifies!(
        r#"
The value of the `iconsdir` attribute is not prepended to the favicon path as it is with icons in the content.
If you want this directory to be included in the favicon path, you must reference it explicitly:

[,asciidoc]
----
:favicon: {iconsdir}/favicon.png
----

"#
    );

    // `iconsdir` set but not referenced: the favicon path is used verbatim,
    // with no `iconsdir` prefix.
    let html = convert("= Doc\n:iconsdir: ./images/icons\n:favicon: favicon.png\n\nBody.");
    assert!(
        html.contains("<link rel=\"icon\" type=\"image/png\" href=\"favicon.png\">"),
        "{html}"
    );

    // Referencing `{iconsdir}` explicitly in the `favicon` value includes it,
    // via the same attribute-reference substitution any attribute entry gets.
    let html =
        convert("= Doc\n:iconsdir: ./images/icons\n:favicon: {iconsdir}/favicon.png\n\nBody.");
    assert!(
        html.contains("<link rel=\"icon\" type=\"image/png\" href=\"./images/icons/favicon.png\">"),
        "{html}"
    );
}

non_normative!(
    r#"
TIP: If you're converting a set of AsciiDoc files in multiple directories for the purpose of making a website, and the favicon is located in a shared location, you'll likely want to use a forward slash (`/`) at the beginning of the favicon path.

"#
);

// A head docinfo file and the `favicon` attribute are independent: setting
// both adds both `<link rel="icon">` elements to the `<head>`, one from each
// source, rather than one overriding the other.
#[test]
fn docinfo_and_favicon_attribute_both_add_their_own_icon_link() {
    verifies!(
        r#"
If you're looking for more control over how the favicon is declared, you should use a xref:ROOT:docinfo.adoc#head[head docinfo file].
Keep in mind that if you add an icon link in a head docinfo file and also set the favicon attribute, you'll end up with two icon links in the generated HTML document.
"#
    );

    // Docinfo is disabled at `Secure` (the default), so — like `-a
    // docinfo=shared` on the `adoc` CLI — this converts under a lower safe
    // mode.
    let docinfo = "<link rel=\"icon\" type=\"image/png\" href=\"/img/favicon.png\">";
    let dir = std::env::temp_dir().join(format!("adoc-favicon-docinfo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    std::fs::write(dir.join("docinfo.html"), docinfo).expect("write docinfo.html");

    let html = convert_with(
        "= Document Title\n:favicon:\n\nBody.",
        &Options::new()
            .safe_mode(SafeMode::Server)
            .attribute("docinfo", "shared")
            .base_dir(dir.clone()),
    );

    let _ = std::fs::remove_dir_all(&dir);

    // Both icon links are present: the one from the `favicon` attribute and
    // the one from the docinfo file.
    assert!(
        html.contains("<link rel=\"icon\" type=\"image/x-icon\" href=\"favicon.ico\">"),
        "{html}"
    );
    assert!(html.contains(docinfo), "{html}");
}
