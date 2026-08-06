//! Coverage of Asciidoctor's *Highlight.js* page — the built-in client-side
//! syntax highlighter this crate models most fully.
//!
//! Every claim on this page has an observable counterpart in the converter's
//! output: activating the highlighter reshapes each source block's `pre`/`code`
//! classes and injects the highlight.js CDN `<link>`/`<script>`; the
//! `highlightjs-theme` attribute selects the stylesheet;
//! `highlightjs-languages` adds one language script per entry (standalone
//! output only); and `highlightjsdir` redirects every asset at a personal copy.
//! Each is verified against `convert`.
//!
//! A document-set `:source-highlighter:` is honored only *below* the `Server`
//! safe mode (the renderer's safe-mode lock), so these conversions run under
//! `SafeMode::Safe`. Standalone output is used throughout so the highlighter's
//! `<head>` stylesheet and footer scripts are present to inspect.

use crate::{
    convert_with,
    tests::{assert_html::assert_css, sdd::*},
    Options, SafeMode,
};

track_file!("ref/asciidoctor/docs/modules/syntax-highlighting/pages/highlightjs.adoc");

/// Converts `source` to a standalone document under `SafeMode::Safe`, the mode
/// that honors a document-set `:source-highlighter:` (a document cannot set it
/// from `Server` up). Standalone output surfaces the highlighter's `<head>`
/// stylesheet link and footer script tags.
fn convert_highlighted(source: &str) -> String {
    convert_with(
        source,
        &Options::new().safe_mode(SafeMode::Safe).standalone(true),
    )
}

// The page title, its URL attribute definitions, and the descriptive lead
// sentence: definitional prose with no rendered behavior of its own. The
// concrete activation and asset loading are verified in the sections below.
non_normative!(
    r#"
= Highlight.js
:url-highlightjs: https://highlightjs.org/
:url-highlightjs-lang: https://highlightjs.org/download/
:url-highlightjs-cdn: https://cdnjs.com/libraries/highlight.js

{url-highlightjs}[Highlight.js^] is a popular client-side syntax highlighter that supports a broad range of {url-highlightjs-lang}[languages^].

"#
);

// Activating highlight.js — setting `:source-highlighter: highlight.js` in the
// header — reshapes each source block (`pre.highlightjs.highlight`,
// `code.language-<lang>.hljs`) and, by default, links the library and its
// stylesheet from the cdnjs CDN.
#[test]
fn activate_links_the_library_and_stylesheet_from_cdnjs() {
    verifies!(
        r#"
== Activate highlight.js

To activate highlight.js, add the following attribute entry to the header of your AsciiDoc file:

[,asciidoc]
----
:source-highlighter: highlight.js
----

By default, Asciidoctor will link to the highlight.js library and stylesheet hosted on {url-highlightjs-cdn}[cdnjs^].
The version of the highlight.js library Asciidoctor loads from the CDN only includes support for languages in the common language bundle (apache, bash, coffeescript, cpp, cs, css, diff, http, ini, java, javascript, json, makefile, markdown, nginx, objectivec, perl, php, properties, python, ruby, shell, sql, xml, and yaml).

"#
    );

    let html = convert_highlighted(
        "= Doc\n:source-highlighter: highlight.js\n\n[,ruby]\n----\nputs \"Hello, World!\"\n----\n",
    );

    // The source block passes through unchanged, tagged for the highlighter.
    assert_css(
        &html,
        "pre.highlightjs.highlight > code.language-ruby.hljs",
        1,
    );

    // The library and its (default `github`) stylesheet are linked from cdnjs.
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/styles/github.min.css\">"
    ));
    assert!(html.contains(
        "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/highlight.min.js\"></script>"
    ));
}

// The theme is selected by the `highlightjs-theme` attribute and defaults to
// `github`; it becomes the basename of the linked stylesheet (`.min.css`
// appended), loaded from the same CDN.
#[test]
fn highlightjs_theme_selects_the_stylesheet() {
    verifies!(
        r#"
== Change the theme

The theme controls the colors that are used for the tokens (keywords, strings, methods, etc.) in the highlighted code.
By default, highlight.js is configured to use the github theme.
You can change the theme used by highlight.js by setting the `highlightjs-theme` attribute.

[,asciidoc]
----
:source-highlighter: highlight.js
:highlightjs-theme: monokai
----

The theme is loaded from the CDN, so any theme supported by the version of highlight.js that Asciidoctor uses is supported.
Refer to https://cdnjs.com/libraries/highlight.js/9.18.3 for a list of themes (filter by *Asset Type: Styling*).
The value of the `highlightjs-theme` attribute is the basename of the file minus the _.min.css_ file extension.

"#
    );

    // The default theme is `github`.
    let default = convert_highlighted(
        "= Doc\n:source-highlighter: highlight.js\n\n[,ruby]\n----\nputs 1\n----\n",
    );
    assert!(default.contains("/highlight.js/9.18.3/styles/github.min.css"));

    // Setting `highlightjs-theme: monokai` swaps the stylesheet basename.
    let monokai = convert_highlighted(
        "= Doc\n:source-highlighter: highlight.js\n:highlightjs-theme: monokai\n\n[,ruby]\n----\nputs 1\n----\n",
    );
    assert!(monokai.contains(
        "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/styles/monokai.min.css\">"
    ));
    assert!(!monokai.contains("styles/github.min.css"));
}

// Each comma-separated `highlightjs-languages` entry loads an extra language
// script from the CDN. The attribute applies only to standalone output — the
// scripts live in the document footer, which embedded output omits — so an
// embedded conversion emits no language scripts.
#[test]
fn highlightjs_languages_loads_extra_language_scripts() {
    verifies!(
        r#"
== Load support for additional languages

To load additional languages supported by highlight.js, list them in the value of the `highlightjs-languages` document attribute.
Separate each language by a comma followed by an optional space.

The common highlight.js bundle does not include support for Rust and Swift.
Let's set the `highlightjs-languages` attribute so the HTML converter loads support for them into the HTML page.

[,asciidoc]
----
:source-highlighter: highlight.js
:highlightjs-languages: rust, swift
----

The `highlightjs-languages` attribute only applies when generating a standalone HTML document (i.e., backend: html, standalone: true).
It does not work when generating embedded HTML, which is used by site generator integrations such as Antora.

"#
    );

    let source =
        "= Doc\n:source-highlighter: highlight.js\n:highlightjs-languages: rust, swift\n\n[,rust]\n----\nfn main() {}\n----\n";

    // Standalone output loads one script per language (the optional space after
    // the comma is tolerated).
    let standalone = convert_highlighted(source);
    assert!(standalone.contains(
        "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/languages/rust.min.js\"></script>"
    ));
    assert!(standalone.contains(
        "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/languages/swift.min.js\"></script>"
    ));

    // Embedded output has no footer, so the language scripts are not emitted.
    let embedded = convert_with(
        source,
        &Options::new().safe_mode(SafeMode::Safe).embedded(true),
    );
    assert!(!embedded.contains("languages/rust.min.js"));
    assert!(!embedded.contains("languages/swift.min.js"));
}

// A personal copy of highlight.js is selected with the `highlightjsdir`
// attribute: every asset — the library, its stylesheet, and any language
// scripts — is loaded from that directory instead of cdnjs. The procedural
// steps (bundling, unpacking, renaming) have no rendered output; the observable
// outcome is the redirected asset base, which is verified.
#[test]
fn highlightjsdir_redirects_assets_to_a_personal_copy() {
    verifies!(
        r#"
== Use a custom highlight.js library

If you'd rather use a personal copy of highlight.js instead of the one hosted on the CDN, follow these steps:

. Create your custom bundle on the {url-highlightjs-lang}[download page^].
. Download and unpack the zip into a folder called [.path]_highlight_ adjacent to your AsciiDoc file (or in the output directory, if different)
. Rename [.path]_highlight/highlight.pack.js_ to [.path]_highlight/highlight.min.js_
. Rename [.path]_highlight/styles/github.css_ to [.path]_highlight/styles/github.min.css_
** Replace `github` with the name of the `highlightjs-theme` you are using, if different.
. Add the attribute entry `:highlightjsdir: highlight` to the header of your AsciiDoc file.
** Alternatively, you can pass the `-a highlightjsdir=highlight` flag when invoking the Asciidoctor CLI.

The output file will use your personal copy of the highlight.js library and stylesheet instead of the one hosted on cdnjs.
"#
    );

    let html = convert_highlighted(
        "= Doc\n:source-highlighter: highlight.js\n:highlightjsdir: highlight\n\n[,ruby]\n----\nputs 1\n----\n",
    );

    // The stylesheet and library load from `highlight/…`, not cdnjs.
    assert!(html.contains("<link rel=\"stylesheet\" href=\"highlight/styles/github.min.css\">"));
    assert!(html.contains("<script src=\"highlight/highlight.min.js\"></script>"));
    assert!(!html.contains("cdnjs.cloudflare.com/ajax/libs/highlight.js"));
}
