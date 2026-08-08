use asciidoc_parser::document::InterpretedValue;

use crate::{tests::sdd::*, Options};

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/skip-front-matter.adoc");

// Asciidoctor's "Skip Front Matter" page. It documents the
// `skip-front-matter` attribute, which strips a leading YAML-style front
// matter block (as static site generators like Jekyll use) before parsing,
// and stores the stripped content in the `front-matter` attribute.
//
// `asciidoc-parser`'s `Parser::skip_front_matter` implements this in full, so
// both halves are verifiable through this crate's `Options`/`load_with`: with
// the attribute set, front matter is consumed and the header parses normally;
// without it, the leading `---` lines throw off header parsing, exactly as
// the page's introduction warns. The one claim with no counterpart is front
// matter removal from *include* files — this crate's include handling does
// not call the parser's front-matter step for included content — so that
// sentence stays non-normative.
//
// Tracked from the library crate only: every verifiable claim is about how
// `asciidoc_html5::load`/`load_with` parses the document. Where the page
// drives behavior with a CLI `-a` attribute, we supply the same attribute
// through `Options`; the `adoc -a` option is a thin forwarder onto that API,
// covered by the CLI crate's own tests.

non_normative!(
    r#"
= Skip Front Matter

== Front matter for static site generators

Many static site generators (i.e., Jekyll, Middleman) rely on [.term]*front matter* added to the top of the document to determine how to convert the content.
Front matter typically starts on the first line of a file and is bounded by block delimiters (e.g., `+---+`).

Here's an example of a document that contains front matter:

[,asciidoc]
----
---  <.>
layout: default <.>
---  <.>
= Document Title

content
----
<.> Front matter opening delimiter
<.> Front matter data
<.> Front matter closing delimiter

"#
);

const FRONT_MATTER_SOURCE: &str = "---\nlayout: default\n---\n= Document Title\n\ncontent";

// Without `skip-front-matter`, the leading front-matter lines throw off
// header parsing: the `---` lines are not a valid document header, so no
// doctitle is recognized.
#[test]
fn without_the_attribute_front_matter_throws_off_the_processor() {
    verifies!(
        r#"
The static site generator removes these lines before passing the document to the AsciiDoc processor to be converted.
Outside of the tool, however, these extra lines can throw off the processor.

"#
    );

    let doc = crate::load(FRONT_MATTER_SOURCE);
    assert_eq!(doc.doctitle(), None);
}

// Setting `skip-front-matter` (via the API or CLI) makes the parser recognize
// and consume the front matter before parsing the document, so the header
// (and the doctitle it carries) parses normally.
#[test]
fn skip_front_matter_attribute_consumes_the_front_matter() {
    verifies!(
        r#"
== skip-front-matter attribute

If the `skip-front-matter` attribute is set via the API or CLI (e.g., `-a skip-front-matter`), Asciidoctor will recognize the front matter and consume it before parsing the document.
"#
    );

    // The page sets `skip-front-matter` from the API or CLI (`-a
    // skip-front-matter`); we supply it the same way — as an external
    // attribute — through `Options::set`, the API the `adoc -a` option feeds
    // into.
    let doc = crate::load_with(
        FRONT_MATTER_SOURCE,
        &Options::new().set("skip-front-matter"),
    );
    assert_eq!(doc.doctitle(), Some("Document Title"));
}

// The extracted front matter (delimiters excluded) is stored in the
// `front-matter` document attribute.
#[test]
fn extracted_front_matter_is_stored_in_the_front_matter_attribute() {
    verifies!(
        r#"
Asciidoctor stores the content it extracts in the `front-matter` attribute to make it available for integrations.
"#
    );

    let doc = crate::load_with(
        FRONT_MATTER_SOURCE,
        &Options::new().set("skip-front-matter"),
    );
    assert_eq!(
        doc.attribute_value("front-matter"),
        InterpretedValue::Value("layout: default".to_string())
    );
}

// Front matter removal from *include* files has no counterpart in this
// crate's include handling, so it carries no rule to verify.
non_normative!(
    r#"
Asciidoctor also removes front matter when reading xref:asciidoc:directives:include.adoc[include files].
"#
);
