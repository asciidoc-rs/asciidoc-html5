//! Coverage of the AsciiDoc language description's *Reference the Author
//! Information* page.
//!
//! The built-in author attributes can be referenced anywhere in the document.
//! Matching Asciidoctor 2.0.26, this crate resolves `{author}`, `{firstname}`,
//! `{email}`, `{middlename}`, and `{authorinitials}` for the first author, and
//! the `_<n>`-suffixed forms (`{author_2}`, `{lastname_2}`, `{firstname_3}`, …)
//! for the subsequent authors. Those substitutions are verified through
//! `convert`; the surrounding prose and the screenshots are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/reference-author-attributes.adoc");

// Renders a standalone document; the author references are resolved in the body
// regardless of standalone mode, but the example documents have section titles
// worth exercising in a full document.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the section heading. Descriptive prose.
non_normative!(
    r#"
= Reference the Author Information

[#reference-author]
== Referencing the author attributes

"#
);

// The first author's attributes resolve wherever they are referenced:
// `{author}` (full name), `{firstname}`, `{email}` (as a `mailto:` link),
// `{middlename}`, and `{authorinitials}`.
#[test]
fn first_author_attributes_resolve_when_referenced() {
    verifies!(
        r#"
You can reference the built-in author attributes in your document regardless of whether they're set via the author line or attribute entries.
In <<ex-reference>>, the `author` and `email` attributes are assigned using attribute entries.

.Reference the author attributes
[source#ex-reference]
----
include::example$reference-author.adoc[]
----

The result of <<ex-reference>> is displayed below.

"#
    );

    // The full contents of `example$reference-author.adoc`.
    let source = "\
= The Intrepid Chronicles
:author: Kismet R. Lee
:email: kismet@asciidoctor.org

== About {author}

You can contact {firstname} at {email}.

P.S. Don't ask what the {middlename} stands for; it's a secret.
{authorinitials}
";

    let output = convert(source);
    // `{author}` -> the full name, in the section title.
    assert_xpath(&output, r#"//h2[text()="About Kismet R. Lee"]"#, 1);
    // `{email}` -> a `mailto:` link.
    assert_xpath(
        &output,
        r#"//p/a[@href="mailto:kismet@asciidoctor.org"]"#,
        1,
    );
    // `{firstname}` -> Kismet, `{middlename}` -> R., `{authorinitials}` -> KRL.
    assert!(output.contains("You can contact Kismet at"));
    assert!(output.contains("what the R. stands for"));
    assert!(output.contains("KRL"));
}

// The screenshot and the multiple-author section heading. Descriptive prose.
non_normative!(
    r#"
image::reference-author.png[Reference the built-in attributes for an author,role=screenshot]

[#reference-multiple-authors]
== Referencing information for multiple authors

"#
);

// Subsequent authors are referenced with `_<n>`-suffixed attribute names:
// `{author_2}`/`{lastname_2}` for the second author, `{author_3}`/
// `{firstname_3}`/`{authorinitials_3}` for the third, while the unsuffixed
// names still resolve to the first author.
#[test]
fn subsequent_author_attributes_use_a_numeric_suffix() {
    verifies!(
        r#"
The first author in an author line is assigned to the built-in attributes `author`, `email`, `firstname`, etc.
Subsequent authors are assigned to the built-in author attributes, but the attribute names are appended with an underscore (`+_+`) and the numeric position of the author in the author line.
For instance, the author _B. Steppenwolf_ in <<ex-reference-multiple>> is the second author in the author line.
The built-in attributes used to reference their information are appended with the number _2_, e.g., `author_2`, `email_2`, `lastname_2`, etc.

.Reference the built-in attributes for multiple authors
[source#ex-reference-multiple]
----
include::example$multiple-authors.adoc[tag=doc]
----

The result of <<ex-reference-multiple>> is displayed below.

"#
    );

    // The `tag=doc` snippet from `example$multiple-authors.adoc`.
    let source = "\
= The Intrepid Chronicles
Kismet R. Lee <kismet@asciidoctor.org>; B. Steppenwolf; Pax Draeke <pax@asciidoctor.org>

.About {author_2}
Mr. {lastname_2} lives in the Rocky Mountains.

.About {author_3}
{firstname_3}, also known as {authorinitials_3}, loves to surf.

.About {author}
You can contact {firstname} at {email}.
";

    let output = convert(source);
    // `{author_2}` / `{lastname_2}` -> B. Steppenwolf.
    assert!(output.contains("About B. Steppenwolf"));
    assert!(output.contains("Mr. Steppenwolf lives in the Rocky Mountains."));
    // `{author_3}` / `{firstname_3}` / `{authorinitials_3}` -> Pax Draeke / Pax /
    // PD.
    assert!(output.contains("About Pax Draeke"));
    assert!(output.contains("Pax, also known as PD, loves to surf."));
    // The unsuffixed names still resolve to the first author.
    assert!(output.contains("About Kismet R. Lee"));
    assert!(output.contains("You can contact Kismet at"));
}

// The screenshot and a commented-out alternative example. Descriptive prose.
non_normative!(
    r#"
image::reference-multiple-authors.png[Reference the built-in attributes for multiple author,role=screenshot]

////
.Set the author and email attributes in the document header
[source]
----
= The Intrepid Chronicles
:author: Kismet R. Lee <1>
:email: kismet@asciidoctor.org <2>
:author_2: Mara_Moss Wirribi <3>
:email_2: mmw@asciidoctor.org <4>
:author_3: B. Steppenwolf <5>
:email_3: https://twitter.com/asciidoctor[@asciidoctor] <6>
----
<1> The first author is assigned to the built-in attribute `author`.
<2> The first author's email is assigned to the built-in attribute `email`.
<3> The second author is assigned to the built-in attribute `author_2`.
The underscore (`+_+`) between _Mara_ and _Moss_ tells the processor that the xref:compound-author-name.adoc[author has a double first name].
<4> The second author's email is assigned to the built-in attribute `email_2`.
<5> The third author is assigned to the built-in attribute `author_3`.
<6> The value of an author's email attribute, such as `email_3`, can contain an active URL and link text.
////
"#
);
