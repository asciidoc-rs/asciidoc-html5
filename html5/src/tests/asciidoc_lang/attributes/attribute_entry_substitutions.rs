//! Coverage of the AsciiDoc language description's *Attribute Entry
//! Substitutions* page.
//!
//! The header substitution group (special characters, then attribute
//! references) is applied to an attribute entry's value before assignment, so
//! inline formatting in a value is *not* interpreted. This renderer reflects
//! that, and it honors the inline pass macro used to change which substitutions
//! apply when a value is assigned (`pass:[]` for none, `pass:quotes[]` / its
//! `q` alias for the quotes substitution), as well as reordering the
//! substitutions at the point of reference. Each is verified through `convert`.
//!
//! The section on attributes defined *outside* the document is about CLI/API
//! attribute passing (where substitutions are not applied), and the
//! value-inspection trick relies on a passthrough block; both are tracked as
//! non-normative here.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/attribute-entry-substitutions.adoc");

/// Returns the rendered content of the first paragraph in `output`.
fn first_paragraph(output: &str) -> &str {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner)
        .unwrap_or_default()
}

non_normative!(
    r#"
= Attribute Entry Substitutions

The AsciiDoc processor automatically applies substitutions from the header substitution group to the value of an attribute entry prior to the assignment, regardless of where the attribute entry is declared in the document.
The header substitution group, which replaces xref:subs:special-characters.adoc[special characters] followed by xref:subs:attributes.adoc[attribute references], is applied to the values of attribute entries, regardless of whether the entries are defined in the header or in the document body.
This is the same group that gets applied to metadata lines (author and revision information) in the document header.

That means that any inline formatting in an attribute value isn't interpreted because:

. inline formatting is not applied when the AsciiDoc processor sets an attribute, and
. inline formatting is not applied when an attribute is referenced since the relevant substitutions come before attributes are resolved.

"#
);

mod change_substitutions_when_assigning_a_value {
    use super::*;
    use crate::convert;

    #[test]
    fn pass_macro_applies_no_substitutions() {
        verifies!(
            r#"
[#pass-macro]
== Change substitutions when assigning a value

If you want the value of an attribute entry to be used *as is* (not subject to substitutions), or you want to alter the substitutions that are applied, you can enclose the value in the xref:pass:pass-macro.adoc[inline pass macro] (i.e., `\pass:[]`).
The inline pass macro accepts a list of zero or more substitutions in the target slot, which can be used to control which substitutions are applied to the value.
If no substitutions are specified, no substitutions will be applied.

In order for the inline macro to work in this context, it must completely surround the attribute value.
If it's used anywhere else in the value, it will be ignored.

Here's how to prevent substitutions from being applied to the value of an attribute entry:

[source]
----
:cols: pass:[.>2,.>4]
----

This might be useful if you're referencing the attribute in a place that depends on the unaltered text, such as the value of the `cols` attribute on a table.

"#
        );

        // `pass:[]` with no substitutions stores the value verbatim, so the `>`
        // characters are not escaped when the value is referenced.
        assert_eq!(
            first_paragraph(&convert(":cols: pass:[.>2,.>4]\n\n{cols}")),
            ".>2,.>4",
        );
    }

    #[test]
    fn pass_macro_applies_quotes() {
        verifies!(
            r#"
Here's how we can apply the xref:subs:quotes.adoc[quotes substitution] to the value of an attribute entry:

[source]
----
:app-name: pass:quotes[MyApp^2^]
----

Internally, the value is stored as `MyApp<sup>2</sup>`.
"#
        );

        // `pass:quotes[]` applies the quotes substitution when the value is
        // assigned, so the `^2^` superscript is resolved and stored.
        assert_eq!(
            first_paragraph(&convert(":app-name: pass:quotes[MyApp^2^]\n\n{app-name}")),
            "MyApp<sup>2</sup>",
        );
    }

    // The value-inspection trick relies on a `[subs=attributes+]` passthrough
    // block to reveal the stored value; that display mechanism is incidental to
    // the substitution behavior, so it is tracked as non-normative.
    non_normative!(
        r#"
You can inspect the value stored in an attribute using this trick:

[source]
----
[subs=attributes+]
------
{app-name}
------
----

"#
    );

    #[test]
    fn pass_macro_q_alias() {
        verifies!(
            r#"
You can also specify the substitution using the single-character alias, `q`.

[source]
----
:app-name: pass:q[MyApp^2^]
----

"#
        );

        // The `q` alias is equivalent to `quotes`.
        assert_eq!(
            first_paragraph(&convert(":app-name: pass:q[MyApp^2^]\n\n{app-name}")),
            "MyApp<sup>2</sup>",
        );
    }

    non_normative!(
        r#"
The inline pass macro kind of works like an attribute value preprocessor.
If the processor detects that an inline pass macro completely surrounds the attribute value, it:

. reads the list of substitutions from the target slot of the macro
. unwraps the value from the macro
. applies the substitutions to the value

If the macro is absent, the value is processed with the header substitution group.

"#
    );
}

// This section concerns attributes passed into the processor from the CLI
// (`-a`) or API (`:attributes`), where substitutions are *not* applied to the
// value. That external-attribute handling is exercised by the CLI/API layers
// rather than this page's rendering, so the section is tracked as
// non-normative.
non_normative!(
    r#"
== Substitutions for attributes defined outside the document

Unlike attribute entries, substitutions are *not* applied to the value of an attribute passed in to the AsciiDoc processor.
An attribute can be passed into the AsciiDoc processor using the `-a` CLI option or the `:attributes` API option.
When attributes are defined external to the document, the value must be prepared so it's ready to be referenced as is.
If the value contains XML special characters, that means those characters must be pre-escaped.
The exception would be if you intend for XML/HTML tags in the value to be preserved.
If the value needs to reference other attributes, those values must be pre-replaced.

Let's consider the case when the value of an attribute defined external to the document contains an ampersand.
In order to reference this attribute safely in the AsciiDoc document, the ampersand must be escaped:

 $ asciidoctor -a equipment="a bat &amp; ball" document.adoc

You can reference the attribute as follows:

[,asciidoc]
----
To play, you'll need {equipment}.
----

If the attribute were to be defined in the document, this escaping would not be necessary.

[,asciidoc]
----
:equipment: a bat & ball
----

That's because, in contrast, substitutions are applied to the value of an attribute entry.

"#
);

mod change_substitutions_when_referencing_an_attribute {
    use super::*;
    use crate::convert;

    #[test]
    fn reorder_substitutions_at_reference() {
        verifies!(
            r#"
== Change substitutions when referencing an attribute

You can also change the substitutions that are applied to an attribute at the time it is resolved.
This is done by manipulating the substitutions applied to the text where it is referenced.
For example, here's how we could get the processor to apply quote substitutions to the value of an attribute:

[source]
----
:app-name: MyApp^2^

[subs="specialchars,attributes,quotes,replacements,macros,post_replacements"]
The application is called {app-name}.
----

Notice that we've swapped the order of the `attributes` and `quotes` substitutions.
This strategy is akin to post-processing the attribute value.
"#
        );

        // Reordering `attributes` before `quotes` lets the quotes substitution
        // run over the resolved value, so the `^2^` superscript is applied.
        let output = convert(
            ":app-name: MyApp^2^\n\n[subs=\"specialchars,attributes,quotes,replacements,macros,post_replacements\"]\nThe application is called {app-name}.",
        );
        assert_eq!(
            first_paragraph(&output),
            "The application is called MyApp<sup>2</sup>.",
        );
    }
}
