//! Coverage of the AsciiDoc language description's *Author Information* page.
//!
//! Author information is spread across a family of built-in attributes, most of
//! them derived from `author`. Matching Asciidoctor 2.0.26, this crate derives
//! `firstname`, `middlename`, `lastname`, and `authorinitials` from the author
//! name (applying the "more than three names collapse into `firstname`" rule of
//! the author line, and honoring underscore-adjoined name parts), and exposes
//! the `_<n>`-suffixed variants for additional authors. Those derivations are
//! verified through `convert`; the descriptive definitions are otherwise
//! non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/author-information.adoc");

// Renders a standalone document so the byline email span is present; the author
// references in the body are resolved at parse time regardless.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the introduction, and the section heading. Descriptive
// prose.
non_normative!(
    r#"
= Author Information

Adding author information to your document is optional.
A document's author information is assigned to multiple built-in attributes.
These optional attributes can be set and assigned values xref:author-line.adoc[using the author line] or xref:author-attribute-entries.adoc[using attribute entries] in a document's header.

== Author and email attributes

"#
);

// `firstname`, `middlename`, `lastname`, and `authorinitials` are derived from
// `author`; on the author line more than three space-separated names collapse
// into `firstname`; and underscores adjoin the words of a single name part.
#[test]
fn author_attribute_derives_the_name_parts() {
    verifies!(
        r#"
author::
The `author` attribute represents the author's full name.
The attributes `firstname`, `middlename`, `lastname`, and `authorinitials` are automatically derived from the value of the `author` attribute.
When assigned implicitly via the author line, the value includes all of the characters and words prior to the semicolon (`;`), angle bracket (`<`), or the end of the line.
Note that when using the implicit author line, the full name can have a maximum of three space-separated names.
If it has more, then the full name is assigned to the `firstname` attribute.
You can adjoin names using an underscore (`_`) character.

"#
    );

    // A three-part name derives each part and the initials.
    let output = convert(
        "= T\n:author: Kismet R. Lee\n\nfn={firstname} mn={middlename} ln={lastname} ai={authorinitials}\n",
    );
    assert!(output.contains("fn=Kismet mn=R. ln=Lee ai=KRL"));

    // On the author line, more than three names collapse into `firstname`
    // (and `lastname` is left unset, so the reference stays literal).
    let output = convert("= T\nAnn B C D\n\nauthor={author} fn={firstname} ln={lastname}\n");
    assert!(output.contains("author=Ann B C D fn=Ann B C D ln={lastname}"));

    // Underscores adjoin the words of a compound name part.
    let output = convert("= T\nAnn_Marie B_C\n\nfn={firstname} ln={lastname}\n");
    assert!(output.contains("fn=Ann Marie ln=B C"));
}

// The `email` attribute holds the first author's email (or URL), which the
// byline renders as a `mailto:` link.
#[test]
fn email_attribute_holds_the_first_authors_email() {
    verifies!(
        r#"
email::
The `email` attribute represents an email address or URL associated with the first author (`author`).
When assigned via the author line, it's enclosed in a pair of angle brackets (`< >`).
A URL can be used in place of the email address.

"#
    );

    let output = convert("= T\nJane Doe <jane@example.org>\n\nBody.\n");
    assert!(output.contains(
        r#"<span id="email" class="email"><a href="mailto:jane@example.org">jane@example.org</a></span>"#
    ));
}

// The precise derivation of each name part and the initials.
#[test]
fn name_and_initials_attributes_derive_from_author() {
    verifies!(
        r#"
=== Name and initials attributes

firstname::
The `firstname` attribute represents the first, forename, or given name of the author.
The first space-separated name in the value of the `author` attribute is automatically assigned to `firstname`.

lastname::
The `lastname` attribute represents the last, surname, or family name of the author.
If `author` contains more than one space-separated name, the third name and any names after that are assigned to the `lastname` attribute.

middlename::
The `lastname` attribute represents the middle name or initial of the author.
If `author` contains more than two space-separated names, the second name is assigned to the `middlename` attribute.

authorinitials::
The first character of the `firstname`, `middlename`, and `lastname` attribute values are assigned to the `authorinitials` attribute.
The value of the `authorinitials` attribute will consist of three characters or less depending on how many parts are in the author's name.

"#
    );

    // Three parts: firstname is the first name, lastname the third, middlename
    // the second, and the initials are their first characters.
    let output = convert(
        "= T\n:author: Kismet R. Lee\n\nfn={firstname} mn={middlename} ln={lastname} ai={authorinitials}\n",
    );
    assert!(output.contains("fn=Kismet mn=R. ln=Lee ai=KRL"));

    // Two parts: no middle name is derived, so `authorinitials` is two
    // characters.
    let output =
        convert("= T\n:author: Jane Doe\n\nfn={firstname} ln={lastname} ai={authorinitials}\n");
    assert!(output.contains("fn=Jane ln=Doe ai=JD"));
}

// Additional authors expose the same attributes under a numeric suffix.
#[test]
fn multiple_author_attributes_use_a_numeric_suffix() {
    verifies!(
        r#"
== Multiple author attributes

author_<n>:: An `author_<n>` attribute represents each additional author's full name, where `<n>` is the 1-based index of all of the authors listed on the author line (e.g., `author_2`, `author_3`).
The attributes `firstname_<n>`, `middlename_<n>`, `lastname_<n>`, and `authorinitials_<n>` are automatically derived from `author_<n>`.
xref:multiple-authors.adoc[Additional authors can only be assigned via the author line].
Each author's full name includes all of the characters and words directly after a semicolon (`;`) but prior to the angle bracket (`<`), next semicolon (`;`), or the end of the line.
The full name can have a maximum of three space-separated names.
If it has more, then the full name is assigned to the `firstname_<n>` attribute.
You can adjoin names using an underscore (`_`) character.

email_<n>::
The `email_<n>` attribute represents an email address associated with xref:multiple-authors.adoc[each additional author] (`author_<n>`).
It's enclosed in a pair of angle brackets (`< >`) on the author line.
A URL can be used in place of the email address.

firstname_<n>::
The first space-separated name in the value of the `author_<n>` attribute is automatically assigned to `firstname_<n>`.

lastname_<n>::
If `author_<n>` contains more than one space-separated name, the third name and any names after that are assigned to the `lastname_<n>` attribute.

middlename_<n>::
If `author_<n>` contains more than two space-separated names, the second name is assigned to the `middlename_<n>` attribute.

authorinitials_<n>::
The first character of the `firstname_<n>`, `middlename_<n>`, and `lastname_<n>` attribute values.
The value of the `authorinitials_<n>` attribute will consist of three characters or less depending on how many parts are in the author's name.
"#
    );

    // The second author on the line populates the `_2` attributes, derived the
    // same way as the first author's.
    let output = convert(
        "= T\nKismet Lee <k@x.org>; Pax B Draeke <pax@y.org>\n\na2={author_2} fn2={firstname_2} mn2={middlename_2} ln2={lastname_2} ai2={authorinitials_2} e2={email_2}\n",
    );
    // `{email_2}` autolinks in body text, so it renders as a `mailto:` anchor.
    assert!(output.contains(
        r#"a2=Pax B Draeke fn2=Pax mn2=B ln2=Draeke ai2=PBD e2=<a href="mailto:pax@y.org">pax@y.org</a>"#
    ));
}
