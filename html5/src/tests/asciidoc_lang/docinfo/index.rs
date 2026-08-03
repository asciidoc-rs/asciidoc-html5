//! Coverage of the AsciiDoc language description's *Docinfo Files* page.
//!
//! Docinfo lets a document splice custom content into three fixed positions of
//! the HTML output: the bottom of the `<head>`, immediately before the header
//! `<div>`, and immediately after the footer `<div>`. `asciidoc-parser` selects
//! which docinfo files apply — from the `docinfo`, `docinfodir`, and
//! `docinfosubs` attributes and the document name — reads them through this
//! crate's [`FsDocinfoFileHandler`](crate::docinfo_handler), applies attribute
//! substitution, and this renderer places the result. Each of those
//! HTML-backend behaviors is verified end to end through `convert`: the
//! head/header/footer placement, the private and shared file-naming schemes,
//! the `docinfo` enablement values, `docinfodir` relocation, and `docinfosubs`
//! attribute resolution.
//!
//! The DocBook `<info>` examples, the PDF caveat, and the illustrative element
//! lists describe backends and prose this HTML renderer does not cover, and are
//! tracked as non-normative.

use crate::{convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoc-lang/docs/modules/docinfo/pages/index.adoc");

/// Writes `files` (name → content) into a fresh temp directory and converts
/// `source` as a *standalone* document under `safe`, anchored at that directory
/// with a primary file of `mydoc.adoc` — so both shared (`docinfo.html`) and
/// private (`mydoc-docinfo.html`) docinfo files resolve. Returns the rendered
/// HTML; the temp directory is removed before returning.
///
/// A file name may contain a `/` to place it in a subdirectory (used to
/// exercise `docinfodir`). `tag` names the temp directory so concurrent tests
/// do not collide.
fn with_docinfo(tag: &str, source: &str, safe: SafeMode, files: &[(&str, &str)]) -> String {
    let dir = std::env::temp_dir().join(format!("adoc-lang-docinfo-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create subdir");
        }
        std::fs::write(path, content).expect("write scratch file");
    }

    let html = convert_with(
        source,
        &Options::new()
            .standalone(true)
            .safe_mode(safe)
            .input_file(dir.join("mydoc.adoc")),
    );

    let _ = std::fs::remove_dir_all(&dir);
    html
}

// The head docinfo example this page displays, injected verbatim to verify that
// a raw fragment (with no enclosing `<head>`) is spliced into the head as-is.
const HEAD_EXAMPLE: &str = "\
<meta name=\"keywords\" content=\"open source, documentation\">
<meta name=\"description\" content=\"The dangerous and thrilling adventures of an open source documentation team.\">
<link rel=\"stylesheet\" href=\"basejump.css\">
<script src=\"map.js\"></script>";

// The page title, the attribute entries for the reference links, the conceptual
// introduction, and the PDF caveat. Descriptive prose with nothing to render.
non_normative!(
    r#"
= Docinfo Files
:url-docbook-info-ref: https://tdg.docbook.org/tdg/5.0/info.html
:url-docinfo-example: https://github.com/clojure-cookbook/clojure-cookbook/blob/master/book-docinfo.xml

Docinfo is a feature of AsciiDoc that allows you to insert custom content into the head, header, or footer of the output document.
This custom content is read from files known as docinfo files by the converter.
Docinfo files are intended as convenient way to supplement the output produced by a converter.
Examples include injecting auxiliary metadata, stylesheets, and scripting logic not already provided by the converter.

The docinfo feature does not apply to all backends.
While it works when converting to output formats such as HTML and DocBook, it does not work when converting to PDF using Asciidoctor PDF.

"#
);

// The docinfo feature is opt-in: a docinfo file in the base directory is read
// only once the `docinfo` attribute enables it.
#[test]
fn docinfo_must_be_explicitly_enabled() {
    verifies!(
        r#"
The docinfo feature must be explicitly enabled using the `docinfo` attribute (see <<enable>>).
Which docinfo files are consumed depends on the value of the `docinfo` attribute as well as the backend.

"#
    );

    // Without the `docinfo` attribute, the file sitting in the base directory is
    // ignored.
    let off = with_docinfo(
        "enable-off",
        "= Doc\n\nBody.",
        SafeMode::Safe,
        &[("docinfo.html", "<meta name=\"di\">")],
    );
    assert!(!off.contains("<meta name=\"di\">"), "{off}");

    // Enabling `docinfo` consumes the file.
    let on = with_docinfo(
        "enable-on",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo.html", "<meta name=\"di\">")],
    );
    assert!(on.contains("<meta name=\"di\">\n</head>"), "{on}");
}

// The anchor and heading for the head docinfo section. A heading with nothing
// to render.
non_normative!(
    r#"
[#head]
== Head docinfo files

"#
);

// Head docinfo content is appended to the `<head>` element: it lands at the
// bottom of the head, just above the closing `</head>`.
#[test]
fn head_docinfo_is_appended_to_the_head() {
    verifies!(
        r#"
The content of head docinfo files gets injected into the top of the document.
For HTML, the content is append to the `<head>` element.
"#
    );

    let html = with_docinfo(
        "head-append",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo.html", "<meta name=\"di-head\" content=\"H\">")],
    );

    assert!(
        html.contains("<meta name=\"di-head\" content=\"H\">\n</head>"),
        "{html}"
    );
}

// The DocBook `<info>` target, the list of valid HTML head elements, and the
// caution against a `<title>` element. Describes DocBook output and enumerates
// permitted elements — nothing this renderer verifies.
non_normative!(
    r#"
For DocBook, the content is appended to the root `<info>` element.

The docinfo file for HTML output may contain valid elements to populate the HTML `<head>` element, including:

* `<base>`
* `<link>`
* `<meta>`
* `<noscript>`
* `<script>`
* `<style>`

CAUTION: Use of the `<title>` element is not recommended as it's already emitted by the converter.

"#
);

// The docinfo fragment need not include an enclosing `<head>` element: the raw
// example content is spliced into the head verbatim, as-is.
#[test]
fn head_docinfo_needs_no_enclosing_head_element() {
    verifies!(
        r#"
You do not need to include the enclosing `<head>` element as it's assumed to be the envelope.

Here's an example:

.A head docinfo file for HTML output
[source,html]
----
<meta name="keywords" content="open source, documentation">
<meta name="description" content="The dangerous and thrilling adventures of an open source documentation team.">
<link rel="stylesheet" href="basejump.css">
<script src="map.js"></script>
----

"#
    );

    let html = with_docinfo(
        "head-example",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo.html", HEAD_EXAMPLE)],
    );

    // The fragment is injected verbatim (no added `<head>` wrapper) and sits at
    // the bottom of the head.
    assert!(html.contains(&format!("{HEAD_EXAMPLE}\n</head>")), "{html}");
}

// The docinfo file for HTML output must carry the `.html` extension (matching
// the output file's `outfilesuffix`), so a `.xml`-named sibling is not read.
#[test]
fn html_docinfo_uses_the_html_extension() {
    verifies!(
        r#"
Docinfo files for HTML output must be saved with the `.html` file extension.
"#
    );

    // A `docinfo.xml` file is not consumed for HTML output; a `docinfo.html`
    // file is.
    let html = with_docinfo(
        "extension",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.xml", "<meta name=\"wrong-ext\">"),
            ("docinfo.html", "<meta name=\"right-ext\">"),
        ],
    );

    assert!(!html.contains("wrong-ext"), "{html}");
    assert!(
        html.contains("<meta name=\"right-ext\">\n</head>"),
        "{html}"
    );
}

// The see-also cross-reference and the DocBook head docinfo example (its
// `<info>`-child elements and the real-world Clojure Cookbook pointer). All
// DocBook, which this crate does not emit.
non_normative!(
    r#"
See <<naming>> for more details.

The docinfo file for DocBook 5.0 output may include any of the {url-docbook-info-ref}[<info> element's children^], such as:

* `<address>`
* `<copyright>`
* `<edition>`
* `<keywordset>`
* `<publisher>`
* `<subtitle>`
* `<revhistory>`

The following example shows some of the content a docinfo file for DocBook might contain.

.A docinfo file for DocBook 5.0 output
[source,xml]
----
<author>
  <personname>
    <firstname>Karma</firstname>
    <surname>Chameleon</surname>
  </personname>
  <affiliation>
    <jobtitle>Word SWAT Team Leader</jobtitle>
  </affiliation>
</author>

<keywordset>
  <keyword>open source</keyword>
  <keyword>documentation</keyword>
  <keyword>adventure</keyword>
</keywordset>

<printhistory>
  <para>April, 2021. Twenty-sixth printing.</para>
</printhistory>
----

Docinfo files for DocBook output must be saved with the `.xml` file extension.
See <<naming>> for more details.

You can find a real-world example of a docinfo file for DocBook in the source of the {url-docinfo-example}[Clojure Cookbook^].

"#
);

// The anchor and heading for the header docinfo section. A heading with nothing
// to render.
non_normative!(
    r#"
[#header]
== Header docinfo files

"#
);

// Header docinfo files are named with a `-header` suffix and their content is
// inserted immediately before the header `<div>`.
#[test]
fn header_docinfo_is_inserted_before_the_header_div() {
    verifies!(
        r#"
Header docinfo files are differentiated from head docinfo files by the addition of `-header` to the file name.
In the HTML output, the header content is inserted immediately before the header div (i.e., `<div id="header">`).
"#
    );

    let html = with_docinfo(
        "header-before",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo-header.html", "<div class=\"di-header\">HDR</div>")],
    );

    assert!(
        html.contains("<div class=\"di-header\">HDR</div>\n<div id=\"header\">"),
        "{html}"
    );
}

// The DocBook header docinfo placement (after the opening `<article>`/`<book>`
// tag). DocBook output, which this crate does not emit.
non_normative!(
    r#"
In the DocBook output, the header content is inserted immediately after the opening tag (e.g., `<article>` or `<book>`).

"#
);

// A header docinfo file is emitted even when the built-in header is suppressed
// with `noheader`, so it can replace the default header entirely.
#[test]
fn header_docinfo_survives_noheader() {
    verifies!(
        r#"
TIP: One possible use of the header docinfo file is to completely replace the default header in the standard stylesheet.
Just set the attribute `noheader`, then apply a custom header docinfo file.

"#
    );

    let html = with_docinfo(
        "header-noheader",
        "= Doc\n:docinfo: shared\n:noheader:\n\nBody.",
        SafeMode::Safe,
        &[("docinfo-header.html", "<div class=\"di-header\">HDR</div>")],
    );

    // The built-in header `<div>` is gone, but the docinfo header remains.
    assert!(!html.contains("<div id=\"header\">"), "{html}");
    assert!(
        html.contains("<div class=\"di-header\">HDR</div>"),
        "{html}"
    );
}

// The anchor and heading for the footer docinfo section. A heading with nothing
// to render.
non_normative!(
    r#"
[#footer]
== Footer docinfo files

"#
);

// Footer docinfo files are named with a `-footer` suffix and their content is
// inserted immediately after the footer `<div>`.
#[test]
fn footer_docinfo_is_inserted_after_the_footer_div() {
    verifies!(
        r#"
Footer docinfo files are differentiated from head docinfo files by the addition of `-footer` to the file name.
In the HTML output, the footer content is inserted immediately after the footer div (i.e., `<div id="footer">`).
"#
    );

    let html = with_docinfo(
        "footer-after",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo-footer.html", "<p class=\"di-footer\">FTR</p>")],
    );

    // The footer docinfo lands just after the closing footer `</div>`, before
    // `</body>`.
    assert!(
        html.contains("<p class=\"di-footer\">FTR</p>\n</body>"),
        "{html}"
    );
    let footer_div = html.find("<div id=\"footer\">").expect("footer div");
    let docinfo = html.find("di-footer").expect("footer docinfo");
    assert!(footer_div < docinfo, "{html}");
}

// The DocBook footer docinfo placement (before the closing
// `</article>`/`</book>` tag). DocBook output, which this crate does not emit.
non_normative!(
    r#"
In the DocBook output, the footer content is inserted immediately before the ending tag (e.g., `</article>` or `</book>`).

"#
);

// A footer docinfo file is emitted even when the built-in footer is suppressed
// with `nofooter`, so it can replace the default footer entirely.
#[test]
fn footer_docinfo_survives_nofooter() {
    verifies!(
        r#"
TIP: One possible use of the footer docinfo file is to completely replace the default footer in the standard stylesheet.
Just set the attribute `nofooter`, then apply a custom footer docinfo file.

"#
    );

    let html = with_docinfo(
        "footer-nofooter",
        "= Doc\n:docinfo: shared\n:nofooter:\n\nBody.",
        SafeMode::Safe,
        &[("docinfo-footer.html", "<p class=\"di-footer\">FTR</p>")],
    );

    // The built-in footer `<div>` is gone, but the docinfo footer remains.
    assert!(!html.contains("<div id=\"footer\">"), "{html}");
    assert!(html.contains("<p class=\"di-footer\">FTR</p>"), "{html}");
}

// An author's note (a line comment) followed by a commented-out (`////`) block
// of last-updated-label guidance. Non-rendering source, deliberately excluded
// from the output.
non_normative!(
    r#"
// Not here! Good info, but does nothing to clarify the previous paragraphs and could confuse.
////
TIP: To change the text in the "Last updated" line in the footer, set the text in the attribute `last-update-label` (for example, `:last-update-label: <your text> Last Updated`). +
To disable the "Last updated" line in the footer, unassign the attribute `last-update-label` (however, this leaves an empty footer div). +
To disable the footer completely, set the attribute `nofooter`. Then having a footer docinfo file effectively replaces the default footer with your custom footer.
////

"#
);

// The anchor and heading for the naming section. A heading with nothing to
// render.
non_normative!(
    r#"
[#naming]
== Naming docinfo files

"#
);

// The docinfo file extension must match the output file's extension
// (`outfilesuffix`, `.html` for HTML). The scope (private vs shared) selects
// which file name is sought; both are verified in detail below.
#[test]
fn docinfo_file_extension_matches_the_output() {
    verifies!(
        r#"
The file that gets selected to provide the docinfo depends on which converter is in use (html, docbook, etc) and whether the docinfo scope is configured for a specific document ("`private`") or for all documents in the same directory ("`shared`").
The file extension of the docinfo file must match the file extension of the output file (as specified by the `outfilesuffix` attribute, a value which always begins with a period (`.`)).

"#
    );

    // HTML output has an `outfilesuffix` of `.html`, so only the `.html`-named
    // docinfo file is read.
    let html = with_docinfo(
        "naming-ext",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.xml", "<meta name=\"docbook-ext\">"),
            ("docinfo.html", "<meta name=\"html-ext\">"),
        ],
    );

    assert!(!html.contains("docbook-ext"), "{html}");
    assert!(html.contains("<meta name=\"html-ext\">\n</head>"), "{html}");
}

// Private docinfo files carry the document name as a prefix —
// `<docname>-docinfo`, `<docname>-docinfo-header`, `<docname>-docinfo-footer` —
// and apply only to that document. With the primary file `mydoc.adoc`, the
// `mydoc-`-prefixed files fill the head, header, and footer respectively.
#[test]
fn private_docinfo_file_names() {
    verifies!(
        r#"
.Docinfo file naming
[cols="<10,<20,<30,<30"]
|===
|Mode |Location |Behavior |Docinfo file name

.3+|Private
|Head
|Adds content to `<head>`/`<info>` for <docname>.adoc files.
|`<docname>-docinfo<outfilesuffix>`

|Header
|Adds content to start of document for <docname>.adoc files.
|`<docname>-docinfo-header<outfilesuffix>`

|Footer
|Adds content to end of document for <docname>.adoc files.
|`<docname>-docinfo-footer<outfilesuffix>`

"#
    );

    let html = with_docinfo(
        "naming-private",
        "= Doc\n:docinfo: private\n\nBody.",
        SafeMode::Safe,
        &[
            ("mydoc-docinfo.html", "<meta name=\"p-head\">"),
            (
                "mydoc-docinfo-header.html",
                "<div class=\"p-header\">H</div>",
            ),
            ("mydoc-docinfo-footer.html", "<p class=\"p-footer\">F</p>"),
        ],
    );

    assert!(html.contains("<meta name=\"p-head\">\n</head>"), "{html}");
    assert!(
        html.contains("<div class=\"p-header\">H</div>\n<div id=\"header\">"),
        "{html}"
    );
    assert!(
        html.contains("<p class=\"p-footer\">F</p>\n</body>"),
        "{html}"
    );
}

// Shared docinfo files drop the document-name prefix — `docinfo`,
// `docinfo-header`, `docinfo-footer` — and apply to any document in the same
// directory. The same three positions are filled from the unprefixed files.
#[test]
fn shared_docinfo_file_names() {
    verifies!(
        r#"
.3+|Shared
|Head
|Adds content to `<head>`/`<info>` for any document in same directory.
|`docinfo<outfilesuffix>`

|Header
|Adds content to start of document for any document in same directory.
|`docinfo-header<outfilesuffix>`

|Footer
|Adds content to end of document for any document in same directory.
|`docinfo-footer<outfilesuffix>`
|===

"#
    );

    let html = with_docinfo(
        "naming-shared",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.html", "<meta name=\"s-head\">"),
            ("docinfo-header.html", "<div class=\"s-header\">H</div>"),
            ("docinfo-footer.html", "<p class=\"s-footer\">F</p>"),
        ],
    );

    assert!(html.contains("<meta name=\"s-head\">\n</head>"), "{html}");
    assert!(
        html.contains("<div class=\"s-header\">H</div>\n<div id=\"header\">"),
        "{html}"
    );
    assert!(
        html.contains("<p class=\"s-footer\">F</p>\n</body>"),
        "{html}"
    );
}

// The anchor and heading for the enabling section. A heading with nothing to
// render.
non_normative!(
    r#"
[#enable]
== Enabling docinfo

"#
);

// The `docinfo` attribute takes any combination of `{private,shared}-{head,
// header,footer}` values (plus the `private`/`shared` aliases). A granular
// value consumes only the file at that scope and location.
#[test]
fn docinfo_values_select_scope_and_location() {
    verifies!(
        r#"
To specify which file(s) you want to apply, set the `docinfo` attribute to any combination of these values:

* `private-head`
* `private-header`
* `private-footer`
* `private` (alias for `private-head,private-header,private-footer`)
* `shared-head`
* `shared-header`
* `shared-footer`
* `shared` (alias for `shared-head,shared-header,shared-footer`)

"#
    );

    // `private-head` consumes only the private head file; the private footer
    // file is left untouched.
    let private_head = with_docinfo(
        "enable-private-head",
        "= Doc\n:docinfo: private-head\n\nBody.",
        SafeMode::Safe,
        &[
            ("mydoc-docinfo.html", "<meta name=\"ph\">"),
            ("mydoc-docinfo-footer.html", "<p class=\"pf\">F</p>"),
        ],
    );
    assert!(
        private_head.contains("<meta name=\"ph\">\n</head>"),
        "{private_head}"
    );
    assert!(!private_head.contains("class=\"pf\""), "{private_head}");

    // `shared-footer` consumes only the shared footer file; the shared head file
    // is left untouched.
    let shared_footer = with_docinfo(
        "enable-shared-footer",
        "= Doc\n:docinfo: shared-footer\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.html", "<meta name=\"sh\">"),
            ("docinfo-footer.html", "<p class=\"sf\">F</p>"),
        ],
    );
    assert!(
        shared_footer.contains("<p class=\"sf\">F</p>\n</body>"),
        "{shared_footer}"
    );
    assert!(!shared_footer.contains("name=\"sh\""), "{shared_footer}");
}

// A `docinfo` attribute set with no value defaults to `private`.
#[test]
fn empty_docinfo_value_means_private() {
    verifies!(
        r#"
Setting `docinfo` with no value is equivalent to setting the value to `private`.

"#
    );

    // With an empty `:docinfo:`, the private (`mydoc-`prefixed) head file is
    // consumed while the shared file is ignored — proving the default is
    // `private`, not `shared`.
    let html = with_docinfo(
        "enable-empty",
        "= Doc\n:docinfo:\n\nBody.",
        SafeMode::Safe,
        &[
            ("mydoc-docinfo.html", "<meta name=\"priv\">"),
            ("docinfo.html", "<meta name=\"shared\">"),
        ],
    );

    assert!(html.contains("<meta name=\"priv\">\n</head>"), "{html}");
    assert!(!html.contains("name=\"shared\""), "{html}");
}

// A comma-separated `docinfo` value combines scopes and locations: `shared,
// private-footer` applies all three shared files plus the private footer, and
// nothing else.
#[test]
fn docinfo_values_combine() {
    verifies!(
        r#"
For example:

[source,asciidoc]
----
:docinfo: shared,private-footer
----

This docinfo configuration will apply the shared docinfo head, header and footer files, if they exist, as well as the private footer file, if it exists.

"#
    );

    let html = with_docinfo(
        "enable-combined",
        "= Doc\n:docinfo: shared,private-footer\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.html", "<meta name=\"s-head\">"),
            ("docinfo-header.html", "<div class=\"s-header\">H</div>"),
            ("docinfo-footer.html", "<p class=\"s-footer\">SF</p>"),
            ("mydoc-docinfo.html", "<meta name=\"p-head\">"),
            ("mydoc-docinfo-footer.html", "<p class=\"p-footer\">PF</p>"),
        ],
    );

    // The three shared files and the private footer are all applied.
    assert!(html.contains("name=\"s-head\""), "{html}");
    assert!(html.contains("class=\"s-header\""), "{html}");
    assert!(html.contains("class=\"s-footer\""), "{html}");
    assert!(html.contains("class=\"p-footer\""), "{html}");

    // The private head file is not requested (no `private-head` in the value),
    // so it is not applied.
    assert!(!html.contains("name=\"p-head\""), "{html}");
}

// A worked scenario naming two documents (adventure.adoc, insurance.adoc) that
// share a docinfo file, with one taking an additional private head file. It
// restates the `shared` and `shared,private-head` mechanics verified above and
// introduces no new rule this crate can check.
non_normative!(
    r#"
Let's apply this to an example:

You have two AsciiDoc documents, [.path]_adventure.adoc_ and [.path]_insurance.adoc_, saved in the same folder.
You want to add the same content to the head of both documents when they're converted to HTML.

. Create a docinfo file containing `<head>` elements.
. Save it as docinfo.html.
. Set the `docinfo` attribute in [.path]_adventure.adoc_ and [.path]_insurance.adoc_ to `shared`.

You also want to include some additional content, but only to the head of [.path]_adventure.adoc_.

. Create *another* docinfo file containing `<head>` elements.
. Save it as [.path]_adventure-docinfo.html_.
. Set the `docinfo` attribute in [.path]_adventure.adoc_ to `shared,private-head`

If other AsciiDoc files are added to the same folder, and `docinfo` is set to `shared` in those files, only the [.path]_docinfo.html_ file will be added when converting those files.

"#
);

// The anchor and heading for the locating section. A heading with nothing to
// render.
non_normative!(
    r#"
[#resolving]
== Locating docinfo files

"#
);

// By default, docinfo files are sought in the document's directory; the search
// directory can be redirected via the `base_dir` API option.
#[test]
fn docinfo_files_are_sought_in_the_document_directory() {
    verifies!(
        r#"
By default, docinfo files are searched for in the same directory as the document file (which can be overridden by setting the `:base_dir` API option / `--base-dir` CLI option).
"#
    );

    // The default: with the primary file `mydoc.adoc`, the sibling `docinfo.html`
    // in that same directory is found.
    let default = with_docinfo(
        "resolve-default",
        "= Doc\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[("docinfo.html", "<meta name=\"in-docdir\">")],
    );
    assert!(
        default.contains("<meta name=\"in-docdir\">\n</head>"),
        "{default}"
    );

    // The override: pointing `base_dir` at a directory (with no primary file)
    // makes it the search root for the shared docinfo file.
    let dir = std::env::temp_dir().join(format!("adoc-lang-docinfo-base-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create base dir");
    std::fs::write(dir.join("docinfo.html"), "<meta name=\"in-basedir\">").expect("write docinfo");

    let overridden = convert_with(
        "= Doc\n:docinfo: shared\n\nBody.",
        &Options::new()
            .standalone(true)
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone()),
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        overridden.contains("<meta name=\"in-basedir\">\n</head>"),
        "{overridden}"
    );
}

// The `docinfodir` attribute relocates the search: a relative value is appended
// to the document directory; an absolute value is used as is.
#[test]
fn docinfodir_relocates_the_search() {
    verifies!(
        r#"
If you want to load them from another location, set the `docinfodir` attribute to the directory where the files are located.
If the value of the `docinfodir` attribute is a relative path, that value is appended to the document directory.
If the value is an absolute path, that value is used as is.

[source,asciidoc]
----
:docinfodir: common/meta
----

"#
    );

    // A relative `docinfodir` is resolved as a subdirectory of the document
    // directory, so `meta/docinfo.html` is read.
    let relative = with_docinfo(
        "docinfodir-relative",
        "= Doc\n:docinfo: shared\n:docinfodir: meta\n\nBody.",
        SafeMode::Safe,
        &[("meta/docinfo.html", "<meta name=\"in-subdir\">")],
    );
    assert!(
        relative.contains("<meta name=\"in-subdir\">\n</head>"),
        "{relative}"
    );

    // An absolute `docinfodir` is used as is. The jailed safe modes recover an
    // absolute path back into the base directory, so this uses `Unsafe`, where an
    // absolute path outside the document directory is honored.
    let base =
        std::env::temp_dir().join(format!("adoc-lang-docinfo-abs-base-{}", std::process::id()));
    let elsewhere =
        std::env::temp_dir().join(format!("adoc-lang-docinfo-abs-else-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let _ = std::fs::remove_dir_all(&elsewhere);
    std::fs::create_dir_all(&base).expect("create base dir");
    std::fs::create_dir_all(&elsewhere).expect("create elsewhere dir");
    std::fs::write(
        elsewhere.join("docinfo.html"),
        "<meta name=\"absolute-dir\">",
    )
    .expect("write docinfo");

    let absolute = convert_with(
        &format!(
            "= Doc\n:docinfo: shared\n:docinfodir: {}\n\nBody.",
            elsewhere.display()
        ),
        &Options::new()
            .standalone(true)
            .safe_mode(SafeMode::Unsafe)
            .input_file(base.join("mydoc.adoc")),
    );
    let _ = std::fs::remove_dir_all(&base);
    let _ = std::fs::remove_dir_all(&elsewhere);
    assert!(
        absolute.contains("<meta name=\"absolute-dir\">\n</head>"),
        "{absolute}"
    );
}

// With `docinfodir` set, only that directory is searched: a docinfo file in the
// document directory is no longer found.
#[test]
fn docinfodir_replaces_the_document_directory() {
    verifies!(
        r#"
Note that if you use this attribute, only the specified folder will be searched; docinfo files in the document directory will no longer be found.

"#
    );

    // `docinfo.html` sits in both the document directory and the `meta`
    // subdirectory named by `docinfodir`; only the latter is read.
    let html = with_docinfo(
        "docinfodir-only",
        "= Doc\n:docinfo: shared\n:docinfodir: meta\n\nBody.",
        SafeMode::Safe,
        &[
            ("docinfo.html", "<meta name=\"in-docdir\">"),
            ("meta/docinfo.html", "<meta name=\"in-subdir\">"),
        ],
    );

    assert!(
        html.contains("<meta name=\"in-subdir\">\n</head>"),
        "{html}"
    );
    assert!(!html.contains("in-docdir"), "{html}");
}

// The anchor and heading for the attribute-substitution section. A heading with
// nothing to render.
non_normative!(
    r#"
[#attribute-substitution]
== Attribute substitution in docinfo files

"#
);

// Docinfo files may include attribute references. Which substitutions apply is
// controlled by `docinfosubs`: unset, it defaults to `attributes` (references
// are resolved); an explicit list that omits `attributes` leaves them
// unresolved.
#[test]
fn docinfosubs_controls_the_applied_substitutions() {
    verifies!(
        r#"
Docinfo files may include attribute references.
Which substitutions get applied is controlled by the `docinfosubs` attribute, which takes a comma-separated list of substitution names.
If this attribute is not set, it has an implied default value of `attributes` (i.e., attribute references are resolved).

"#
    );

    let docinfo = &[("docinfo.html", "<meta name=\"app\" content=\"{project}\">")];

    // Unset: the implied default `attributes` applies, so the reference resolves.
    let default = with_docinfo(
        "subs-default",
        "= Doc\n:docinfo: shared\n:project: Widgets\n\nBody.",
        SafeMode::Safe,
        docinfo,
    );
    assert!(
        default.contains("<meta name=\"app\" content=\"Widgets\">"),
        "{default}"
    );

    // An explicit `docinfosubs` list controls which substitutions apply: naming
    // only `replacements` (which omits `attributes`) leaves the attribute
    // reference unresolved.
    let explicit = with_docinfo(
        "subs-explicit",
        "= Doc\n:docinfo: shared\n:docinfosubs: replacements\n:project: Widgets\n\nBody.",
        SafeMode::Safe,
        docinfo,
    );
    assert!(
        explicit.contains("<meta name=\"app\" content=\"{project}\">"),
        "{explicit}"
    );
}

// The DocBook attribute-substitution example: a `docinfo.xml` referencing
// `{revnumber}`, the source document that supplies it, and the resulting
// DocBook `<info>`. DocBook output, which this crate does not emit.
non_normative!(
    r#"
For example, if you created the following docinfo file:

.Docinfo file (docinfo.xml) containing a revnumber attribute reference
[source,xml]
----
<edition>{revnumber}</edition>
----

And this source document:

.Source document including a revision number
[,asciidoc]
----
= Document Title
Author Name
v1.0, 2020-12-30
:doctype: book
:backend: docbook
:docinfo: shared
----

Then the converted DocBook output would be:

.Converted DocBook containing the docinfo
[,xml]
----
<?xml version="1.0" encoding="UTF-8"?>
<book xmlns="http://docbook.org/ns/docbook" xmlns:xl="http://www.w3.org/1999/xlink" version="5.0" xml:lang="en">
<info>
<title>Document Title</title>
<date>2020-12-30</date>
<author>
<personname>
<firstname>Author</firstname>
<surname>Name</surname>
</personname>
</author>
<authorinitials>AN</authorinitials>
<revhistory>
<revision>
<revnumber>1.0</revnumber>
<date>2020-12-30</date>
<authorinitials>AN</authorinitials>
</revision>
</revhistory>
<edition>1.0</edition> <!--.-->
</info>
</book>
----
<.> The revnumber attribute reference in `docinfo.xml` was replaced by the source document's revision number in the converted output.

"#
);

// The HTML attribute-substitution example: a `docinfo.html` license `<link>`
// referencing `{license-url}`/`{license-title}`, the source document that
// defines them, and the resolved `<link>` in the rendered head.
#[test]
fn docinfo_html_attribute_substitution() {
    verifies!(
        r#"
Another example is if you want to define the license link tag in the HTML head.

.Docinfo file (docinfo.html) containing a license meta tag
[,html]
----
<link rel="license" href="{license-url}" title="{license-title}">
----

Now define these attributes in your AsciiDoc source:

.Source document that defines license attributes
[,asciidoc]
----
= Document Title
:license-url: https://mit-license.org
:license-title: MIT
:docinfo: shared
----

Then the `<head>` tag in the converted HTML would include:

.Rendered license link tag in HTML output
[,html]
----
<link rel="license" href="https://mit-license.org" title="MIT">
----
"#
    );

    let html = with_docinfo(
        "subs-license",
        "= Document Title\n:license-url: https://mit-license.org\n:license-title: MIT\n:docinfo: shared\n\nBody.",
        SafeMode::Safe,
        &[(
            "docinfo.html",
            "<link rel=\"license\" href=\"{license-url}\" title=\"{license-title}\">",
        )],
    );

    assert!(
        html.contains("<link rel=\"license\" href=\"https://mit-license.org\" title=\"MIT\">"),
        "{html}"
    );
}
