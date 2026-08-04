//! Coverage of the AsciiDoc language description's *Using the Author Line*
//! page.
//!
//! The author line, directly below the document title, sets the `author` and
//! `email` attributes. Matching Asciidoctor 2.0.26, this crate renders that
//! information in the header byline: the author name in `<span id="author"
//! class="author">` and, when an email follows in angle brackets, a `mailto:`
//! link in `<span id="email" class="email">`. That behavior is verified
//! through `convert`; the surrounding prose, the syntax templates, and the
//! screenshot are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/author-line.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the definition of the author line, and the rules and
// syntax template for using it. Descriptive prose and setup.
non_normative!(
    r#"
= Using the Author Line

The author attributes can be implicitly set and assigned values using the author line.

[#author-line]
== What's the author line?

The [.term]*author line* is directly after the document title line in the document header.
When the content on this line is structured correctly, the processor assigns the content to the built-in `author` and `email` attributes.

== When can I use the author line?

In order for the processor to properly detect the author line and assign the content to the correct attributes, all of the following criteria must be met:

. The header must contain a xref:title.adoc[document title].
. The author information must be entered on the line directly beneath the document title.
. The author line must start with an author name.
. The content in the author line must be placed in a specific order and separated with the correct syntax.

.Author line structure for single author
[source]
----
= Document Title
firstname middlename lastname <email>
----

The author's middle name is optional.
An email following the author's last name is also optional.
If included, the email address must be enclosed in a pair of angle brackets (`< >`).

TIP: The email can be replaced by a URL, though the value is still stored in the `email` attribute.

The author line also accepts xref:multiple-authors.adoc[multiple authors].

"#
);

// The author line assigns the author name and (bracketed) email, which the
// byline renders as an `<span id="author">` and a `<span id="email">` holding a
// `mailto:` link.
#[test]
fn author_line_populates_the_byline() {
    verifies!(
        r#"
== Assign an author and email

In <<ex-line>>, let's add an author and their email address using the author line.
The author line must be placed on the line directly below the xref:title.adoc[document title] and start with an author's name.

.Add an author and email using the author line
[source#ex-line]
----
= The Intrepid Chronicles
Kismet R. Lee <kismet@asciidoctor.org> <.> <.>
----
<.> Enter the author's name on the line below the document title.
<.> In a pair of angle brackets (`< >`), enter the author's email.

Remember, a middle name and email are optional.
The processor assigns the content on the author line to the built-in attributes using word position, word count, and syntax.

TIP: The email can be replaced by a URL, though the value is still stored in the `email` attribute.

When the default stylesheet is applied, the author information is displayed on the byline.
The [.term]*byline* displays the author information and the xref:revision-information.adoc[revision information] directly beneath the document's title.

"#
    );

    let source = "\
= The Intrepid Chronicles
Kismet R. Lee <kismet@asciidoctor.org>

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

// The screenshot and the sidebar cautioning against attribute references in the
// author line. Descriptive prose.
non_normative!(
    r#"
image::author-line-with-author-and-email.png[Author and email information displayed on the byline,role=screenshot]

.Using attribute references in the author line
****
The author line is not intended to support the arbitrary placement of attribute references.
While attribute references are replaced in the author line (as part of the header substitution group), they aren't substituted until after the line is parsed.
This ordering can sometimes produce undesirable results.
It's best to use the author line strictly as a shorthand for defining static author and email information.

If you do need to use attribute references in the author or email values, you should xref:author-attribute-entries.adoc[define the attributes explicitly using attribute entries].
****
"#
);
