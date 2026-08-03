//! Coverage of the AsciiDoc language description's *Add Multiple Authors to a
//! Document* page.
//!
//! Several authors are listed on the author line, separated by semicolons.
//! Matching Asciidoctor 2.0.26, this crate renders each author in its own
//! byline span — `id="author"`, `id="author2"`, `id="author3"`, … — with a
//! companion `id="email<n>"` span only for the authors that carry an email.
//! That behavior is verified through `convert`; the surrounding prose, the
//! syntax templates, the character-reference escaping notes, and the screenshot
//! are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/multiple-authors.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the multi-author syntax, and the character-reference
// escaping caveat (a known limitation, not a rendering rule). Descriptive prose
// and setup.
non_normative!(
    r#"
= Add Multiple Authors to a Document

The xref:author-line.adoc[author line] is the only way to assign more than one author to a document for display in the byline.
Additionally, only the HTML 5 and Docbook converters can convert documents with multiple authors.

== Multi-author syntax

The information for each author is concluded with a semicolon (`;`).

.Author line structure for multiple authors
[source]
----
= Document Title
firstname middlename lastname <email>; firstname middlename lastname <email>
----

Directly after each author's last name or optional email, enter a semicolon (`;`) followed by a space, and then enter the next author's information.

=== Escape a trailing character reference

If an author name segment ends with a character reference (e.g., `\&#174;`), you must escape it from processing.
One way to escape it is to add a trailing attribute reference (e.g., `\{empty}`).
If the character reference appears at the end of the last author name segment, you can use a second semicolon instead.

A better way of escaping the character reference is to replace it with an attribute reference (e.g., `\{reg}`).

Even if the character reference is escaped, the segments of the author name will not be processed.
Instead, the whole name will be assigned to the `author` and `firstname` attributes.
This limitation may be lifted in the future.

"#
);

// Each author on the line gets its own byline span (`author`, `author2`,
// `author3`), and only the authors with an email get an `email<n>` span — so
// the email-less second author has no `email2`.
#[test]
fn multiple_authors_each_render_their_own_byline_span() {
    verifies!(
        r#"
== List multiple authors on the author line

The author line in <<ex-line-multiple>> lists the information for three authors.
Each author's information is separated by a semicolon (`;`).
Notice that the author _B. Steppenwolf_ doesn't have an email, so the semicolon is placed at the end of their name.

.An author line with three authors and two email addresses
[source#ex-line-multiple]
----
include::example$multiple-authors.adoc[tag=header]
----

The result of <<ex-line-multiple>> is displayed below.

"#
    );

    // The `tag=header` snippet from `example$multiple-authors.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet R. Lee <kismet@asciidoctor.org>; B. Steppenwolf; Pax Draeke <pax@asciidoctor.org>

Body.
";

    let output = convert(source);
    assert_xpath(
        &output,
        r#"//span[@id="author"][text()="Kismet R. Lee"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@id="email"]/a[@href="mailto:kismet@asciidoctor.org"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@id="author2"][text()="B. Steppenwolf"]"#,
        1,
    );
    // The second author has no email, so no `email2` span is emitted.
    assert_xpath(&output, r#"//span[@id="email2"]"#, 0);
    assert_xpath(&output, r#"//span[@id="author3"][text()="Pax Draeke"]"#, 1);
    assert_xpath(
        &output,
        r#"//span[@id="email3"]/a[@href="mailto:pax@asciidoctor.org"]"#,
        1,
    );
}

// The screenshot, the pointer to referencing per-author attributes, and two
// further character-reference escaping recipes. Descriptive prose.
non_normative!(
    r#"
image::author-line-with-multiple-authors.png[Multiple authors and their emails displayed on the byline,role=screenshot]

The information for each author can also be xref:reference-author-attributes.adoc#reference-multiple-authors[referenced in the document] using their respective built-in attribute.

If an author name ends with with a character reference, you can preserve the semicolon in the character reference by adding a trailing attribute reference:

----
AsciiDoc&#174;{empty} WG; Another Author
----

Another solution entails moving the character reference to an attribute and inserting it using an attribute reference:

----
:reg: &#174;
AsciiDoc{reg} WG; Another Author
----

Even though the character reference is escaped, the segments of the author name will not be processed.
"#
);
