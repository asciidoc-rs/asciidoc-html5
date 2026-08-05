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

track_file!("ref/asciidoctor/docs/modules/ROOT/pages/safe-modes.adoc");

// Asciidoctor's "Safe Modes" page, tracked from the library crate. This crate
// models the same four safe modes through [`SafeMode`], with the same integer
// levels and the same default (`SECURE`) for the API. The tests below verify,
// from the page, the integer level of each mode, the default stylesheet
// link-vs-embed choice (`SECURE` "prevents access to stylesheets" and so links
// it, while a lower mode embeds it inline), and the `docfile`/`docdir`
// concealment under `SERVER`/`SECURE`. Other mode effects this crate honors —
// the `docinfo`, `backend`, `doctype`, and `source-highlighter` restrictions,
// and (through asciidoc-parser's own safe mode, which this crate sets; see #37)
// include directives and URI reads – are exercised by unit tests elsewhere, so
// their spans stay non-normative here (the `source-highlighter` bullet excepted
// – it is verified below). The SVG interactive/inline referencing modes are
// likewise surfaced and honor the safe mode (below `SECURE` they render an
// `<object>`/`<svg>`; at `SECURE` a plain `<img>`), exercised by unit tests
// elsewhere. What remains unsurfaced – icons and `data-uri` – is likewise
// non-normative.

non_normative!(
    r#"
= Safe Modes

Asciidoctor provides security levels that control the read and write access of attributes, include directives, macros, and scripts while a document is processing.
Each level includes the restrictions enabled in the prior security level.

.Security assumptions
[#security-assumptions]
****
Asciidoctor's safe modes are primarily focused on what the processor is permitted to do.
The safe modes do not provide a comprehensive security framework.
In particular, there's no safe mode that restricts the kind of content the author can pass through to the output document.
In other words, the safe mode setting does not provide a way to sanitize the output.

Asciidoctor performs sensible escaping to allow an author to safely input text, but does not limit the content that can be included in the output using passthrough blocks or custom substitutions.
The reason for this policy is that we assume the document will be passed through a sanitizer if the HTML must be embedded directly into a web page, precisely what GitHub and GitLab do.
This postprocessing (which could be done using a postprocessor extension) is better handled by a separate tool since
there are many permutations to consider and only a separate tool would know which restrictions to apply for a given situation.
****

The safe mode can be set from the xref:cli:set-safe-mode.adoc[CLI] and the xref:api:set-safe-mode.adoc[API].
You can also xref:reference-safe-mode.adoc[enable or disable content based on the current safe mode].

"#
);

// UNSAFE disables the security features and has integer level 0. This crate's
// `SafeMode::Unsafe` has level 0, and converting under it embeds the stylesheet
// (nothing prevents reading/embedding it).
#[test]
fn unsafe_disables_security_and_is_level_0() {
    verifies!(
        r#"
[#unsafe]
== UNSAFE

The `UNSAFE` safe mode level disables any security features enforced by Asciidoctor.
Ruby is still subject to its own restrictions.

*This is the default safe mode for the CLI.*
Its integer value is `0`.

"#
    );

    assert_eq!(SafeMode::Unsafe as u8, 0);
    assert!(convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Unsafe)
    )
    .contains("<style>"));
}

// SAFE allows assets such as the stylesheet to be embedded and has integer
// level
// 1. `SafeMode::Safe` has level 1, and converting under it embeds the
//    stylesheet.
#[test]
fn safe_embeds_assets_and_is_level_1() {
    verifies!(
        r#"
[#safe]
== SAFE

The `SAFE` safe mode level prevents access to files which reside outside of the parent directory of the source file.
Include directives (`+include::[]+`) are enabled, but paths to include files must be within the parent directory.
This mode allows assets (such as the stylesheet) to be embedded in the document.

Its integer value is `1`.

"#
    );

    assert_eq!(SafeMode::Safe as u8, 1);
    assert!(
        convert_with("= Doc\n\nBody.", &Options::new().safe_mode(SafeMode::Safe))
            .contains("<style>")
    );
}

// SERVER's host-revealing intrinsics `docfile` and `docdir` are enforced here:
// `docfile` is trimmed to its basename and `docdir` is emptied by
// `Options::apply`, verified below. Docinfo, backend, doctype, and
// source-highlighter are likewise enforced. A document `:docinfo:` or
// `:source-highlighter:` is ignored under SERVER and above, so only an API
// value enables either. Backend and doctype go further than the page: because
// html5 is the only backend this crate produces and `article` the only doctype
// it models, `backend` is pinned to `html5` and `doctype` to `article`, each
// locked against the document (and the API) in *every* safe mode – subsuming
// SERVER's restriction rather than merely matching it. Docinfo, backend,
// doctype, source-highlighter, docfile, and docdir are all covered by unit
// tests in `options.rs`; the source-highlighter lock (#215, the renderer half
// of #56) is also verified from the `setting …` bullet just below.
non_normative!(
    r#"
[#server]
== SERVER

The `SERVER` safe mode level disallows the document from setting attributes that would affect conversion of the document.
"#
);

// SERVER trims `docfile` to a relative path so neither the document nor its
// output can reveal the source file's absolute location. This crate reduces
// `docfile` to its basename; a document reference resolves to just the file
// name.
#[test]
fn server_trims_docfile_to_a_relative_path() {
    verifies!(
        r#"
This level trims `docfile` to its relative path and prevents the document from:

"#
    );

    let html = convert_with(
        "= Doc\n\nfile={docfile}",
        &Options::new()
            .safe_mode(SafeMode::Server)
            .input_file("/docs/guide/main.adoc"),
    );
    assert!(html.contains("<p>file=main.adoc</p>"), "{html}");
    assert!(!html.contains("/docs/guide"), "{html}");
}

// SERVER prevents the document from *setting* attributes that affect
// conversion. `doctype`/`docinfo`/`backend` are enforced and covered by the
// `Options` tests; `source-highlighter` is now enforced too (#215) – a document
// that enables a highlighter under SERVER is ignored, so no highlighter class
// or asset `<script>` is emitted and its origin cannot be steered by the
// document.
#[test]
fn server_prevents_the_document_from_setting_source_highlighter() {
    verifies!(
        r#"
* setting `source-highlighter`, `doctype`, `docinfo` and `backend`
"#
    );

    let html = convert_with(
        "= Doc\n:source-highlighter: highlightjs\n:highlightjsdir: https://evil.example.com\n\n\
         [source,ruby]\n----\nputs 1\n----",
        &Options::new().safe_mode(SafeMode::Server),
    );
    // The source block falls back to the default unhighlighted shape, and no
    // highlighter asset – from the attacker origin or the CDN – is emitted. (The
    // default stylesheet names a `pre.highlightjs` CSS rule, so the checks target
    // the highlighter markup itself, not the bare word.)
    assert!(
        html.contains(
            "<pre class=\"highlight\">\
             <code class=\"language-ruby\" data-lang=\"ruby\">puts 1</code></pre>"
        ),
        "{html}"
    );
    assert!(!html.contains("highlightjs highlight"), "{html}");
    assert!(!html.contains("evil.example.com"), "{html}");
    assert!(!html.contains("cdnjs.cloudflare.com"), "{html}");
}

// SERVER hides `docdir` from the document entirely, since it can expose the
// host filesystem layout. This crate empties it, so a document reference
// resolves to nothing.
#[test]
fn server_hides_docdir_from_the_document() {
    verifies!(
        r#"
* seeing `docdir` (as it can reveal information about the host filesystem)

"#
    );

    let html = convert_with(
        "= Doc\n\ndir=[{docdir}]",
        &Options::new()
            .safe_mode(SafeMode::Server)
            .input_file("/docs/guide/main.adoc"),
    );
    assert!(html.contains("<p>dir=[]</p>"), "{html}");
    assert!(!html.contains("/docs/guide"), "{html}");
}

// SERVER allows `linkcss` (so the stylesheet is not forced to link, and embeds
// by default) and has integer level 10.
#[test]
fn server_allows_linkcss_and_is_level_10() {
    verifies!(
        r#"
It allows `icons` and `linkcss`.

Its integer value is `10`.

"#
    );

    assert_eq!(SafeMode::Server as u8, 10);
    assert!(convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Server)
    )
    .contains("<style>"));
}

// SECURE inherits SERVER's `docdir`/`docfile` concealment — `docdir` emptied,
// `docfile` reduced to its basename — enforced in `Options::apply` and verified
// below. Docinfo, backend, and doctype are likewise surfaced: SECURE disables
// docinfo (no docinfo file is read), forces the backend to `html5`, and pins
// the doctype to `article`, each locked against the document (covered by unit
// tests in `options.rs`). SECURE also disables the interactive
// (`opts=interactive`) and inline (`opts=inline`) SVG modes — an SVG image
// renders as a plain `<img>` — surfaced and covered by unit tests (in
// `substitutions_test.rs` and the renderer's block-image tests). The remaining
// SECURE restrictions are not surfaced by this renderer yet, each tracked for
// later implementation: icons (https://github.com/asciidoc-rs/asciidoc-html5/issues/50), `data-uri`
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/51), and source
// highlighting (https://github.com/asciidoc-rs/asciidoc-html5/issues/45).
// Include directives and URI reads are already gated by asciidoc-parser's safe
// mode, which this crate now sets (see #37). SECURE also "prevents access to
// stylesheets," which is why it links the stylesheet rather than embedding it —
// verified in the next test (custom stylesheets are
// https://github.com/asciidoc-rs/asciidoc-html5/issues/36).
non_normative!(
    r#"
[#secure]
== SECURE

The `SECURE` safe mode level disallows the document from attempting to read files from the file system and including their contents into the document.
Additionally, it:

* disables icons
* disables include directives (`+include::[]+`)
* data can not be retrieved from URIs
* prevents access to stylesheets and JavaScript files
* sets the backend to `html5`
* disables `docinfo` files
* disables `data-uri`
* disables interactive (`opts=interactive`) and inline (`opts=inline`) modes for SVGs
"#
);

// SECURE inherits SERVER's concealment of `docdir` and `docfile`: `docdir` is
// emptied and `docfile` reduced to its basename, so neither exposes the host
// filesystem. The page says "disables"; Asciidoctor — the parity oracle —
// conceals rather than fully unsets (it keeps `docfile`'s basename), which is
// what this crate reproduces.
#[test]
fn secure_conceals_docdir_and_docfile() {
    verifies!(
        r#"
* disables `docdir` and `docfile` (as these can reveal information about the host filesystem)
"#
    );

    // Secure is the API default (no safe mode set).
    let html = convert_with(
        "= Doc\n\nfile=[{docfile}] dir=[{docdir}]",
        &Options::new().input_file("/docs/guide/main.adoc"),
    );
    assert!(html.contains("<p>file=[main.adoc] dir=[]</p>"), "{html}");
    assert!(!html.contains("/docs/guide"), "{html}");
}

non_normative!(
    r#"
* disables source highlighting

xref:extensions:index.adoc[Asciidoctor extensions] may still embed content into the document depending whether they honor the safe mode setting.

"#
);

// SECURE is the API default and has integer level 20. This crate defaults the
// API to `SafeMode::Secure` (level 20), which links the default stylesheet
// instead of embedding it — the observable effect of "prevents access to
// stylesheets."
#[test]
fn secure_is_the_api_default_and_is_level_20() {
    verifies!(
        r#"
*This is the default safe mode for the API.*
Its integer value is `20`.

"#
    );

    assert_eq!(SafeMode::Secure as u8, 20);

    // The API default (no safe mode set) is `Secure`, which links the stylesheet.
    let linked = convert("= Doc\n\nBody.");
    assert!(linked.contains("./asciidoctor.css"));
    assert!(!linked.contains("<style>"));
}

non_normative!(
    r#"
TIP: GitHub processes AsciiDoc files using the `SECURE` mode.

////
|===

|{empty} |Unsafe |Safe |Server |Secure

|URI access
|system access
|base directory access
|docdir
|docfile
|docinfo
|backend
|doctype
|source-highlighter
|macros
|include
|data-uri
|linkcss
|icons

|===
////
"#
);
