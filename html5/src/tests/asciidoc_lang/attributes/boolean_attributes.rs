//! Coverage of the AsciiDoc language description's *Set Boolean Attributes*
//! page.
//!
//! A boolean attribute is a built-in toggle: setting it (with an empty value)
//! turns its feature on. The page's example — turning on `sectanchors` — is
//! verified through `convert` by confirming the section anchor appears in the
//! output. The syntax description itself has no distinct rendering and is
//! tracked as non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/boolean-attributes.adoc");

non_normative!(
    r#"
= Set Boolean Attributes
// [#boolean-attribute]

A boolean attribute is a built-in attribute that acts like a toggle.
Its sole function is to turn on a feature or behavior.

== Boolean attribute entry syntax

A boolean attribute is set using an attribute entry in the header or body of a document.
The value of a boolean value is always empty because boolean attributes in AsciiDoc only accept an _empty string_ value.
In AsciiDoc, an attribute that is set, but has an empty value, is interpreted as the _true_ state and an attribute which is not set is interpreted as the _false_ state.
However, a processor may interpret a value of `true` as the true state as well.

[source]
----
:name-of-a-boolean-attribute: <.>
----
<.> On a new line, type a colon (`:`), directly followed by the attribute's name and then another colon (`:`).
After the closing colon, press kbd:[Enter].
The attribute is now set and its behavior will be applied to the document.

"#
);

#[test]
fn declare_a_boolean_attribute() {
    verifies!(
        r#"
== Declare a boolean attribute

Let's use an attribute entry to turn on the built-in boolean attribute named `sectanchors`.
When `sectanchors` is set, it xref:sections:title-links.adoc#anchor[activates an anchor in front of a section title] when a cursor hovers over it.

[source]
----
= Document Title
:sectanchors: <.>
----
<.> The value of `sectanchors` is always left empty because it's a boolean attribute.
"#
    );

    // Setting the boolean `sectanchors` (with an empty value) turns on the
    // feature: an anchor link is rendered in front of each section title.
    let output = convert("= Document Title\n:sectanchors:\n\n== First Section\n\nText.");
    assert_css(&output, "h2 a.anchor", 1);
}
