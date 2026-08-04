//! Coverage of the AsciiDoc language description's *Assign Author and Email
//! with Attribute Entries* page.
//!
//! A single author's information can be set with `:author:` and `:email:`
//! attribute entries instead of an author line. Matching Asciidoctor 2.0.26,
//! this crate resolves those attributes into the same byline spans an author
//! line produces. That behavior is verified through `convert`; the surrounding
//! prose and the screenshot are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/author-attribute-entries.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the section heading. Descriptive
// prose.
non_normative!(
    r#"
= Assign Author and Email with Attribute Entries

Instead of using an author line, a single author's information can be set and assigned with attribute entries in the document header.

== author and email attribute syntax

"#
);

// The `author` and `email` attribute entries populate the same byline spans the
// author line does: an `<span id="author">` and an `<span id="email">` holding
// a `mailto:` link.
#[test]
fn author_and_email_attribute_entries_populate_the_byline() {
    verifies!(
        r#"
The built-in attributes `author` and `email` can be explicitly set and assigned values in the document header using attribute entries.

.Set author and email attributes
[source#ex-entries]
----
= The Intrepid Chronicles
:author: Kismet R. Lee <.>
:email: kismet@asciidoctor.org <.>
----
<.> The author's name is assigned to the built-in attribute `author`
<.> The author's email is assigned to the built-in attribute `email`

When the default stylesheet is applied, the author information assigned to these attributes is displayed on the byline.
The result of <<ex-entries>> is displayed below.

"#
    );

    let source = "\
= The Intrepid Chronicles
:author: Kismet R. Lee
:email: kismet@asciidoctor.org

Body.
";

    let output = convert(source);
    assert_xpath(
        &output,
        r#"//span[@id="author"][@class="author"][text()="Kismet R. Lee"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@id="email"][@class="email"]/a[@href="mailto:kismet@asciidoctor.org"][text()="kismet@asciidoctor.org"]"#,
        1,
    );
}

// The screenshot, the note that attribute entries can't set multiple authors,
// and the pointer to referencing these attributes. Descriptive prose.
non_normative!(
    r#"
image::author-and-email-attributes.png["Byline containing author information from the explicitly set author and email attributes",role=screenshot]

NOTE: You can't set the built-in attributes for multiple authors (e.g., `author_2`, `email_3`) using attribute entries.
xref:multiple-authors.adoc[Multiple authors] can only be set using the author line.

These attributes can also be xref:reference-author-attributes.adoc[referenced in the document].
"#
);
