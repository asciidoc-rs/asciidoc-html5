//! Coverage of the AsciiDoc language description's *Document Attribute
//! Assignment Precedence* page.
//!
//! An attribute set via the API/CLI overrides one set in the document, unless
//! the API/CLI assignment carries the `@` soft-set modifier, in which case the
//! document wins. This crate models both forms with [`Options::attribute`] (a
//! hard override) and [`Options::attribute_default`] (the soft-set form), so
//! the `imagesdir` precedence examples are verified through `convert_with`. The
//! CLI syntax details and the soft-unset rule are tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/assignment-precedence.adoc");

non_normative!(
    r#"
= Document Attribute Assignment Precedence
:navtitle: Attribute Assignment Precedence

"#
);

mod default_attribute_value_precedence {
    use super::*;
    use crate::{convert_with, tests::assert_html::assert_css, Options};

    #[test]
    fn api_value_overrides_document_value() {
        verifies!(
            r#"
== Default attribute value precedence

The attribute assignment precedence, listed from highest to lowest, is:

. An attribute defined using the API or CLI
. An attribute defined in the document
. The default value of the attribute, if applicable

Let's use the `imagesdir` attribute to show how precedence works.

The default value for the `imagesdir` attribute is an empty string.
Therefore, if the `imagesdir` attribute is not assigned a value (either in the document, API, or CLI), the processor will assign it the default value of empty string.
If the `imagesdir` attribute is set in the document (meaning assigned a new value, such as `images`), that value will override the default value.
Finally, if a value is assigned to the `imagesdir` attribute via the API or CLI, that value will override both the default value and the value assigned in the document.

It's possible to alter this order of precedence using a modifier, covered in the next section.

"#
        );

        // The document sets `imagesdir`, but the API assignment (a hard
        // override) wins, so the image path uses the API value.
        let src = "= T\n:imagesdir: images\n\nimage::a.png[X]";
        let output = convert_with(src, &Options::new().attribute("imagesdir", "from-api"));
        assert_css(&output, r#"img[src="from-api/a.png"]"#, 1);
    }
}

mod altering_the_assignment_precedence {
    use super::*;
    use crate::{convert_with, tests::assert_html::assert_css, Options};

    #[test]
    fn soft_set_lets_the_document_win() {
        verifies!(
            r#"
== Altering the assignment precedence

You can allow the document to reassign an attribute that is defined via the API or CLI by adding the `@` precedence modifier to the end of the attribute value or the end of the attribute name.
Adding this modifier lowers the precedence so that an assignment in the document still wins out.
We sometimes refer to this as "`soft setting`" the attribute.
This feature can be useful for assigning default values for attribute, but still letting the document control its own fate.

NOTE: The `@` modifier is removed before the assignment is made.

Here's an example that shows how to set the `imagesdir` from the CLI with a lower precedence:

 $ asciidoctor -a imagesdir=images@ doc.adoc

Alternately, you can place the modifier at the end of the attribute name:

 $ asciidoctor -a imagesdir@=images doc.adoc

It's now possible to override the value of the `imagesdir` attribute from within the document:

[source]
----
= Document Title
:imagesdir: new/path/to/images
----

"#
        );

        // The API soft-sets `imagesdir` (the `@` form), so the document's own
        // assignment takes precedence and the image path uses the document value.
        let src = "= Document Title\n:imagesdir: new/path/to/images\n\nimage::a.png[X]";
        let output = convert_with(
            src,
            &Options::new().attribute_default("imagesdir", "from-api"),
        );
        assert_css(&output, r#"img[src="new/path/to/images/a.png"]"#, 1);
    }

    non_normative!(
        r#"
To soft unset an attribute from the CLI or API, you can use the following syntax:

 !name=@

The leading `!` unsets the attribute while the `@` lowers the precedence of the assignment.
This assignment is almost always used to unset a default value while still allowing the document to assign a new one.
One such example is `sectids`, which is enabled by default.
`!sectids=@` switches the setting off.

Let's update the attribute assignment precedence list defined earlier to reflect this additional rule:

. An attribute passed to the API or CLI whose value does not end in `@`
. An attribute defined in the document
. An attribute passed to the API or CLI whose value or name ends in `@`
. The default value of the attribute, if applicable

Regardless of whether the precedence modifier is applied, an attribute assignment always overrides the default value.
"#
    );
}
