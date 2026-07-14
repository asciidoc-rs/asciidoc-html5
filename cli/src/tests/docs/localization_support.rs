use clap::Parser as _;

use crate::{run_with_input, tests::sdd::*, Cli};

track_file!("docs/modules/ROOT/pages/localization-support.adoc");

// This crate's own "Localization Support" page, tracked from the CLI crate. The
// prose is descriptive documentation, tracked as non-normative; the CLI-facing
// conversions are verified below by driving `adoc`. The `asciidoc-html5` crate
// tracks the same page and additionally verifies the Rust API example, which
// has no CLI counterpart and so is non-normative here. The sdd tool merges the
// two crates' coverage.

/// Pipes `source` through `adoc` with `args` (which select standard input),
/// returning the captured HTML. Drives the real stdin read path via
/// [`run_with_input`], the injectable-reader core of `run`.
fn run_piped(args: &[&str], source: &str) -> String {
    let cli = Cli::parse_from(args);
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("adoc converts");
    String::from_utf8(stdout).expect("adoc output is UTF-8")
}

non_normative!(
    r#"
= Localization Support
:navtitle: Localization Support
:description: How asciidoc-html5 handles non-English content, the lang attribute, and translating the built-in labels it emits.

`asciidoc-html5` is not restricted to English content. Like Asciidoctor, it
processes the full range of the UTF-8 character set, so you can write your
document in any language, save the source as UTF-8, and the renderer converts the
text unchanged.

Separately, the words the renderer _generates_ itself -- the built-in _labels_,
such as the "`version`" that precedes a revision number -- are English by
default. You localize them by overriding the attribute that controls each one.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

== UTF-8 content

"#
);

// UTF-8 content is passed through to the output verbatim (embedded output).
#[test]
fn utf8_content_is_rendered_verbatim() {
    verifies!(
        r#"
Document text is passed through to the output verbatim, whatever script it is
written in. This paragraph, mixing Latin-with-diacritics, CJK, Greek, and Arabic:

[,asciidoc]
----
Café, naïve, Über. 日本語. Ελληνικά. مرحبا.
----

converts unchanged (embedded output shown):

[,html]
----
<div class="paragraph">
<p>Café, naïve, Über. 日本語. Ελληνικά. مرحبا.</p>
</div>
----

"#
    );

    let html = run_piped(
        &["adoc", "-e", "-"],
        "Café, naïve, Über. 日本語. Ελληνικά. مرحبا.",
    );
    assert!(html.contains(
        "<div class=\"paragraph\">\n<p>Café, naïve, Über. 日本語. Ελληνικά. مرحبا.</p>\n</div>"
    ));
}

non_normative!(
    r#"
== The lang attribute

"#
);

// `-a lang=es` records the language on the root element of a standalone
// document.
#[test]
fn lang_attribute_sets_the_html_lang() {
    verifies!(
        r#"
Set the `lang` attribute to a language code to record it on the root element of a
standalone document. Given `-a lang=es`, the renderer emits:

[,html]
----
<html lang="es">
----

"#
    );

    let html = run_piped(&["adoc", "-a", "lang=es", "-"], "= Doc\n\nText.\n");
    assert!(html.contains("<html lang=\"es\">"));
}

non_normative!(
    r#"
As in Asciidoctor's `html5` backend, `lang` is only a hint: it *does not* enable
automatic translation of the built-in labels. If you want the labels in another
language, you must set the corresponding attributes yourself, either in the
document header or by passing them via the API or CLI. (Asciidoctor's
`lang`-driven `locale/attributes.adoc` include and its DocBook toolchain
translation are not implemented here -- see <<known-limitations>>.)

== Translating a built-in label

"#
);

// `-a version-label=…` retitles the revision label in the header; it defaults
// to the English-derived "version" and is rendered downcased.
#[test]
fn version_label_translates_the_revision_label() {
    verifies!(
        r#"
The one built-in label the renderer emits today is `version-label`, the word
that precedes the revision number in a document's header. It defaults to
_Version_ and is rendered downcased, matching Asciidoctor. Override it to
localize the header:

[,html]
----
<span id="revnumber">version 2.5,</span>
----

Setting `-a version-label=Ausgabe` (German) changes the label:

[,html]
----
<span id="revnumber">ausgabe 2.5,</span>
----

"#
    );

    let source = "= Titel\nAutor\nv2.5, 2026-01-04\n";

    let default_html = run_piped(&["adoc", "-"], source);
    assert!(default_html.contains("<span id=\"revnumber\">version 2.5,</span>"));

    let translated = run_piped(&["adoc", "-a", "version-label=Ausgabe", "-"], source);
    assert!(translated.contains("<span id=\"revnumber\">ausgabe 2.5,</span>"));
}

// The Rust API example has no CLI counterpart; it is verified from the
// `asciidoc-html5` crate and tracked as non-normative here.
non_normative!(
    r#"
From the API, supply the attribute through xref:api:index.adoc[`Options`]:

[,rust]
----
use asciidoc_html5::{convert_with, Options};

let html = convert_with(
    "= Titel\nAutor\nv2.5, 2026-01-04\n",
    &Options::new()
        .standalone(true)
        .attribute("version-label", "Ausgabe"),
);
----

"#
);

non_normative!(
    r#"
[#known-limitations]
== Known limitations

The renderer's localization support is limited to the behavior above, because it
is still at an early baseline. In particular:

* *Most built-in labels are not emitted yet.* Asciidoctor defines a table of
  caption and label attributes -- `appendix-caption`, `caution-caption`,
  `example-caption`, `figure-caption`, `note-caption`, `table-caption`,
  `toc-title`, `last-update-label`, and more. The constructs that use them
  (admonitions, examples, figures, tables, the table of contents, cross
  references, the footer timestamp) are not rendered yet, so setting those
  attributes has no visible effect. Only `version-label` is honored today.
* *No DocBook toolchain and no automatic label translation.* There is no DocBook
  backend, and `lang` does not translate labels or pull in a bundled
  `locale/attributes.adoc` translation file.
* *No right-to-left (RTL) layout.* Output is left-to-right, top-to-bottom, the
  same as Asciidoctor's `html5` backend.
* *No content translation.* Only the built-in labels are localizable; the
  renderer does not translate a document's prose.
"#
);
