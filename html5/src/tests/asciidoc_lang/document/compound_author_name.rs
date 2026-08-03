//! Coverage of the AsciiDoc language description's *Compound Author Names*
//! page.
//!
//! An underscore joins the words of a compound name part so the processor
//! assigns them to a single attribute. Matching Asciidoctor 2.0.26, this crate
//! drops the underscores when rendering the byline and resolves the correct
//! values for `{firstname}`, `{lastname_2}`, etc. Those behaviors are verified
//! through `convert`; the surrounding prose, the syntax template, and the
//! screenshots are non-normative.

use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/compound-author-name.adoc");

// Renders a standalone document so the byline (and the resolved references in
// the body) appear in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title and the explanation, with the syntax template, of why a
// compound name needs underscores. Descriptive prose and setup.
non_normative!(
    r#"
= Compound Author Names

When a name consists of multiple parts, such as a compound or composite surname, or a double middle name, the processor needs to be explicitly told which words should be assigned to a specific attribute.

== Connecting compound author names

If the parts of an author's name aren't assigned to the correct built-in attributes, they may output the wrong information if they're referenced in the body of the document.
For instance, if the name _Ann Marie Jenson_ was entered on the author line or assigned to the attribute `author`, the processor would assign _Ann_ to `firstname`, _Marie_ to `middlename`, and _Jenson_ to `lastname` based on the location and order of each word.
This assignment would be incorrect because the author's first name is _Ann Marie_.

When part of an author's name consists of more than one word, use an underscore (`+_+`) between the words to connect them.

.Compound name syntax
[source]
----
= Document Title
firstname_firstname lastname; firstname middlename_middlename lastname
----

If the more than three space-separated names (or initials) are entered in the implicit author line, the entire line (including the email portion) will be used as the author's full name and first name.
Thus, it's important to use the underscore separator to ensure there are no more than three space-separated names.

== Compound names in the author line

"#
);

// The underscores that join a compound first name (`Ann_Marie`) or a
// three-word surname (`López_del_Toro`) are stripped in the byline, which shows
// the names with plain spaces.
#[test]
fn compound_names_render_without_underscores_in_the_byline() {
    verifies!(
        r#"
In <<ex-line-compound>>, the first author has a compound first name and the second author has a compound surname.

.Assign compound names in the author line
[source#ex-line-compound]
----
= Drum and Bass Breakbeats
Ann_Marie Jenson; Tomás López_del_Toro <.> <.>
----
<.> To signal to the processor that _Ann Marie_ is the author's first name (instead of their first and middle names), type an underscore (`+_+`) between each part of the author's first name.
<.> The second author's last name consists of three words.
Type an underscore (`+_+`) between each word of the author's last name.

The result of <<ex-line-compound>> is displayed below.
Notice that the underscores (`+_+`) aren't displayed when the document is rendered.

"#
    );

    let source = "\
= Drum and Bass Breakbeats
Ann_Marie Jenson; Tomás López_del_Toro

Body.
";

    let output = convert(source);
    assert!(output.contains(r#"<span id="author" class="author">Ann Marie Jenson</span>"#));
    assert!(output.contains(r#"<span id="author2" class="author">Tomás López del Toro</span>"#));
}

// The screenshot. Nothing to render.
non_normative!(
    r#"
image::author-line-with-compound-names.png[Compound author names displayed in the byline,role=screenshot]

"#
);

// Referencing `{firstname}` and `{lastname_2}` in the body yields the compound
// values with the underscores removed.
#[test]
fn compound_names_resolve_without_underscores_when_referenced() {
    verifies!(
        r#"
The underscore between each word in a compound name ensures that the parts of an author's name are assigned correctly to the corresponding built-in attributes.
If you were to reference the first author's first name or the second author's last name in the document body, as shown in <<ex-reference-compound>>, the correct values would be displayed.

.Reference authors with compound names
[source#ex-reference-compound]
----
= Drum and Bass Breakbeats
Ann_Marie Jenson; Tomás López_del_Toro

The first author's first name is {firstname}.

The second author's last name is {lastname_2}.
----

Like in the byline, the underscores (`+_+`) aren't displayed when the document is rendered.

"#
    );

    let source = "\
= Drum and Bass Breakbeats
Ann_Marie Jenson; Tomás López_del_Toro

The first author's first name is {firstname}.

The second author's last name is {lastname_2}.
";

    let output = convert(source);
    assert!(output.contains("first name is Ann Marie."));
    assert!(output.contains("last name is López del Toro."));
}

// The screenshot and the section heading. Descriptive prose.
non_normative!(
    r#"
image::reference-compound-names.png[Compound author names displayed in the document body when referenced,role=screenshot]

== Compound names in the author attribute

"#
);

// A compound name assigned through the `author` attribute is split the same
// way: `{firstname}` resolves to the compound first name (_Mara Moss_) with the
// underscore removed.
#[test]
fn compound_name_in_the_author_attribute_resolves_firstname() {
    verifies!(
        r#"
An underscore (`+_+`) should also be placed between each part of a compound name when the author is assigned using the `author` attribute.

.Assign a compound name using the author attribute
[source#ex-compound]
----
= Quantum Networks
:author: Mara_Moss Wirribi <.>

== About {author}

{firstname} lives on the Bellarine Peninsula near Geelong, Australia. <.>
----
<.> Assign the author's name to the `author` attribute.
Enter an underscore (`+_+`) between each part of the author's first name.
This ensures that their full first name is correct when it's automatically assigned to `firstname` by the processor.
<.> The built-in attribute `firstname` is referenced in the document's body.
The author's first name is automatically extracted from the value of `author` and assigned to `firstname`.

The result of <<ex-compound>>, displayed below, shows that the processor assigned the correct words to the built-in attribute `firstname` since the author's full first name, _Mara Moss_, is displayed where `firstname` was referenced.

"#
    );

    let source = "\
= Quantum Networks
:author: Mara_Moss Wirribi

== About {author}

{firstname} lives on the Bellarine Peninsula near Geelong, Australia.
";

    let output = convert(source);
    assert!(output.contains("Mara Moss lives on the Bellarine Peninsula near Geelong, Australia."));
}

// The screenshot. Nothing to render.
non_normative!(
    r#"
image::author-attribute-with-compound-name.png[Compound author name displayed in the byline using the author attribute,role=screenshot]
"#
);
