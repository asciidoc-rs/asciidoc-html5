//! Coverage of the AsciiDoc language description's *Positional and Named
//! Attributes* page.
//!
//! The renderer-observable claims are in the positional-attribute section: the
//! positional-to-named mapping on the image macro (so `[Sunset,300,400]` equals
//! `[alt=Sunset,width=300,height=400]`) and the first-positional block-style
//! shorthand for ID (`#`), role (`.`), and option (`%`) on sections, lists,
//! tables, and formatted text. Each is verified through `convert`. The named-
//! attribute grammar, the attrlist parsing rules, and the substitution rules
//! describe parser behavior with no distinct rendering and are tracked as
//! non-normative (they are verified in `asciidoc-parser`).

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/positional-and-named-attributes.adoc");

non_normative!(
    r#"
= Positional and Named Attributes

This page breaks down the difference between positional and named attributes on an element and the rules for parsing an attribute list.

"#
);

mod positional_attribute {
    use crate::{
        convert,
        tests::{
            assert_html::{assert_css, assert_xpath},
            sdd::*,
        },
    };

    #[test]
    fn image_macro_positional_mapping() {
        verifies!(
            r#"
[#positional]
== Positional attribute

// tag::pos[]
Entries in an attribute list that only consist of a value are referred to as positional attributes.
The position is the 1-based index of the entry once all named attributes have been removed (so they may be interspersed).

The positional attribute may be dually assigned to an implicit attribute name if the block or macro defines a mapping for positional attributes.
Here are some examples of those mappings:

* `icon:` 1 => size
* `image:` and `image::` 1 => alt (text), 2 => width, 3 => height
* Delimited blocks: 1 => block style and attribute shorthand
* Other inline quoted text: 1 => attribute shorthand
* `link:` and `xref:` 1 => text
* Custom blocks and macros can also specify positional attributes

For example, the following two image macros are equivalent.

[source]
----
image::sunset.jpg[Sunset,300,400]

image::sunset.jpg[alt=Sunset,width=300,height=400]
----

The second macro is the same as the first, but written out in longhand form.
// end::pos[]

"#
        );

        // Both macros map positional 1/2/3 to alt/width/height, producing the
        // same `<img>` element.
        let output = convert(
            "image::sunset.jpg[Sunset,300,400]\n\nimage::sunset.jpg[alt=Sunset,width=300,height=400]",
        );
        assert_css(
            &output,
            r#"img[src="sunset.jpg"][alt="Sunset"][width="300"][height="400"]"#,
            2,
        );
    }

    non_normative!(
        r#"
=== Block style and attribute shorthand

The first positional attribute on all blocks (including sections) is special.
It's used to define the xref:blocks:index.adoc#block-style[block style].
It also supports a shorthand notation for defining the ID, role, and options attributes.
This shorthand notation can also be used on formatted text, even though formatted text doesn't technically support attributes.

To be clear, the shorthand notation is allowed in two places:

* The first positional attribute in block attribute line (i.e., the location of the block style)
* The text inside the brackets of formatted text (which is otherwise treated as the role)

The attribute shorthand is inspired by the HAML and Slim template languages as a way of saving the author some typing.
Instead of having to use the longhand form of a name attribute, it's possible to compress the assignment to a value prefixed by a special marker.
The markers are mapped as follows:

* `#` - ID
* `.` - role
* `%` - option

Each shorthand entry is placed directly adjacent to previous one, starting immediately after the optional block style.
The order of the entries does not matter, except for the style, which must come first.
The typical order is style, ID, role(s), option(s).

All of these are valid:

* `#idname.rolename`
* `.rolename1.rolename2`
* `.rolename#idname`
* `#idname.rolename%optionname`

"#
    );

    #[test]
    fn section_id_shorthand() {
        verifies!(
            r#"
Here's an example that shows how to set an ID on a section using this shorthand notation:

----
[#custom-id]
== Section with Custom ID
----

The shorthand entry must follow the block style, if present.
"#
        );

        // The `#` shorthand sets the section's id.
        let output = convert("[#custom-id]\n== Section with Custom ID\n\nText.");
        assert_css(&output, "h2#custom-id", 1);
    }

    #[test]
    fn appendix_id_shorthand() {
        verifies!(
            r#"
Here's an example that shows how to set an ID on an appendix section using this shorthand notation:

----
[appendix#custom-id]
== Appendix with Custom ID
----

"#
        );

        // The block style (`appendix`) comes first, then the `#` shorthand id;
        // the appendix is labeled and keeps the custom id.
        let output = convert("[appendix#custom-id]\n== Appendix with Custom ID\n\nText.");
        assert_xpath(
            &output,
            r#"//h2[@id="custom-id"][text()="Appendix A: Appendix with Custom ID"]"#,
            1,
        );
    }

    #[test]
    fn list_shorthand_id_role_option() {
        verifies!(
            r#"
Here's an example of a block that uses the shorthand notation to set the ID, a role, and an option for a list.
Specifically, this syntax sets the ID to `rules`, adds the role `prominent`, and sets the option `incremental`.

----
[#rules.prominent%incremental]
* Work hard
* Play hard
* Be happy
----

"#
        );

        // `#rules` sets the id and `.prominent` the role class; the `incremental`
        // option has no rendered effect on the list container.
        let output =
            convert("[#rules.prominent%incremental]\n* Work hard\n* Play hard\n* Be happy");
        assert_css(&output, "div#rules.ulist.prominent", 1);
    }

    #[test]
    fn table_shorthand_options() {
        verifies!(
            r#"
A block can have multiple roles and options, so these shorthand entries may be repeated.
Here's an example that shows how to set several options on a table.
Specifically, this syntax sets the `header`, `footer`, and `autowidth` options.

----
[%header%footer%autowidth]
|===
|Header A |Header B
|Footer A |Footer B
|===
----

"#
        );

        // The repeated `%` options set `header`, `footer`, and `autowidth`,
        // producing a `<thead>`, a `<tfoot>`, and the `fit-content` class.
        let output = convert(
            "[%header%footer%autowidth]\n|===\n|Header A |Header B\n|Footer A |Footer B\n|===",
        );
        assert_css(&output, "table.tableblock.fit-content", 1);
        assert_css(&output, "table > thead", 1);
        assert_css(&output, "table > tfoot", 1);
    }

    #[test]
    fn formatted_text_shorthand() {
        verifies!(
            r#"
This shorthand notation also appears on formatted text.
Here's an example that shows how to set the ID and add a role to a strong phrase.
Specifically, this syntax sets the ID to `free-world` and adds the `goals` role.

----
[#free-world.goals]*free the world*
----

Formatted text does not support a style, so the first and only positional attribute is always the shorthand notation.

"#
        );

        // On formatted text the shorthand sets the id and role directly on the
        // formatting element.
        let output = convert("[#free-world.goals]*free the world*");
        assert_css(&output, "strong#free-world.goals", 1);
    }
}

non_normative!(
    r#"
[#named]
== Named attribute

// tag::name[]
A named attribute consists of a name and a value separated by an `=` character (e.g., `name=value`).

If the value contains a space, comma, or quote character, it must be enclosed in double or single quotes (e.g., `name="value with space"`).
In all other cases, the surrounding quotes are optional.

If the value contains the *same* quote character used to enclose the value, the quote character in the value must be escaped by prefixing it with a backslash (e.g., `value="the song \"Dark Horse\""`).

If enclosing quotes are used, they are dropped from the parsed value and the preceding backslash is dropped from any escaped quotes.

[#unset]
=== Unset a named attribute

To undefine a named attribute, set the value to `None` (case sensitive).
// end::name[]

== Attribute list parsing

The source text that's used to define attributes for an element is referred to as an [.term]*attrlist*.
An attrlist is always enclosed in a pair of square brackets.
This applies for block attributes as well as attributes on a block or inline macro.
The processor splits the attrlist into individual attribute entries, determines whether each entry is a positional or named attribute, parses the entry accordingly, and assigns the result as an attribute on the node.

The rules for what defines the boundaries of an individual attribute, and whether the attribute is positional or named, are defined below.
In these rules, `name` consists of a word character (letter or numeral) followed by any number of word or `-` characters (e.g., `see-also`).

* Attribute references are expanded before the attrlist is parsed (i.e., the attributes substitution is applied).
* Parsing an attribute proceeds from the beginning of the attribute list string or after a previously identified delimiter (`,`).
** The first character of an attribute list cannot be a tab or space.
For subsequent attributes, any leading space or tab characters are skipped.
* If a valid attribute name is found, and it is followed by an equals sign (=), then the parser recognizes this as a named attribute.
The text after the equals sign (=) and up to the next comma or end of list is taken as the attribute value.
Space and tab characters around the equals sign (=) and at the end of the value are ignored.
* Otherwise, this is a positional attribute with a value that ends at the next delimiter or end of list.
Any space or tab characters at the boundaries of the value are ignored.
* To parse the attribute value:
** If the first character is not a quote, the string is read until the next delimiter or end of string.
** If the first character is a double quote (i.e., `"`), then the string is read until the next unescaped double quote or, if there is no closing double quote, the next delimiter.
If there is a closing double quote, the enclosing double quote characters are removed and escaped double quote characters are unescaped; if not, the initial double quote is retained.
** If the next character is a single quote (i.e., `'`), then the string is read until the next unescaped single quote or, if there is no closing single quote, the next delimiter.
If there is a closing single quote, the enclosing single quote characters are removed and escaped single quote characters are unescaped; if not, the initial single quote is retained.
If there is a closing single quote, and the first character is not an escaped single quote, substitutions are performed on the value as described in <<Substitutions>>.

.When to escape a closing square bracket
****
Since the terminal of an attrlist is a closing square bracket, it's sometimes necessary to escape a closing square bracket if it appears in the value of an attribute.

In line-oriented syntax such as a block attribute list, a block macro, and an include directive, you do not have to escape closing square brackets that appear in the attrlist itself.
That's because the parser already knows to look for the closing square bracket at the end of the line.

If a closing square bracket appears in the attrlist of an inline element, such as an inline macro, it usually has to be escaped using a backslash or by using the character reference `+&#93;+`.
There are some exceptions to this rule, such as a link macro in a footnote, which are influenced by the substitution order.
****

== Substitutions

// tag::subs[]
Recall that attribute references are expanded before the attrlist is parsed.
Therefore, it's not necessary to force substitutions to be applied to a value if you're only interested in applying the attributes substitution.
The attributes substitution has already been applied at this point.

If the attribute name (in the case of a positional attribute) or value (in the case of a named attribute) is enclosed in single quotes (e.g., `+citetitle='Processed by https://asciidoctor.org'+`), and the attribute is defined in an attrlist on a block, then the xref:subs:index.adoc#normal-group[normal substitution group] is applied to the value at assignment time.
No special processing is performed, aside from the expansion of attribute references, if the value is not enclosed in quotes or is enclosed in double quotes.

If the value contains the same quote character used to enclose the value, escape the quote character in the value by prefixing it with a backslash (e.g., `+citetitle='A \'use case\' diagram, generated by https://plantuml.com'+`).
// end::subs[]
"#
);
