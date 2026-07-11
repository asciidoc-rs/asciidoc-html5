use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/ROOT/pages/safe-modes.adoc");

// Asciidoctor's "Safe Modes" page, tracked from the library crate. This crate
// models the same four safe modes through [`SafeMode`], with the same integer
// levels and the same default (`SECURE`) for the API. Most of what a mode
// governs in Asciidoctor — include directives, icons, `data-uri`,
// `source-highlighter`, `docinfo`, backend locking, SVG modes — is not
// something this renderer surfaces yet, so those spans are non-normative. What
// is observable here is the integer level of each mode and its effect on the
// default stylesheet: `SECURE` links it (it "prevents access to stylesheets"),
// while a lower mode embeds it inline.

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

// The restrictions SERVER places on document-set attributes are not modeled by
// this renderer yet; each is tracked for later implementation:
// source-highlighter (https://github.com/asciidoc-rs/asciidoc-html5/issues/45), doctype
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/46), backend
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/47), docdir
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/48), and docfile
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/49). Docinfo is the
// exception: its SERVER restriction *is* enforced — a document `:docinfo:` is
// ignored under SERVER and above, so only an API value enables docinfo.
// Enforcing the same for the attributes above is tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/56.
non_normative!(
    r#"
[#server]
== SERVER

The `SERVER` safe mode level disallows the document from setting attributes that would affect conversion of the document.
This level trims `docfile` to its relative path and prevents the document from:

* setting `source-highlighter`, `doctype`, `docinfo` and `backend`
* seeing `docdir` (as it can reveal information about the host filesystem)

"#
);

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

// Most of SECURE's restrictions are not surfaced by this renderer yet, each
// tracked for later implementation: icons
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/50), `data-uri`
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/51), interactive/inline
// SVG modes (https://github.com/asciidoc-rs/asciidoc-html5/issues/52), backend
// locking (https://github.com/asciidoc-rs/asciidoc-html5/issues/47), `docdir`
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/48), `docfile`
// (https://github.com/asciidoc-rs/asciidoc-html5/issues/49), and source
// highlighting (https://github.com/asciidoc-rs/asciidoc-html5/issues/45).
// Docinfo is surfaced: SECURE disables it (no docinfo file is read).
// Include directives and URI reads are already gated by asciidoc-parser's safe
// mode, which this crate now sets (see #37). The one restriction observable
// here is that SECURE "prevents access to stylesheets," which is why it links
// the stylesheet rather than embedding it — verified in the next test (custom
// stylesheets are https://github.com/asciidoc-rs/asciidoc-html5/issues/36).
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
* disables `docdir` and `docfile` (as these can reveal information about the host filesystem)
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
