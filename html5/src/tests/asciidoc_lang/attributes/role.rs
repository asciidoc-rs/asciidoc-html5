//! Coverage of the AsciiDoc language description's *Role Attribute* page.
//!
//! The page's central claim — "the `role` attribute in AsciiDoc always gets
//! mapped to the `class` attribute in the HTML output" — is exactly what this
//! renderer produces, so each syntax the page documents (shorthand dot and
//! formal `role=`, on blocks and on formatted inline elements, single and
//! multiple) is verified by converting the shown source and checking that the
//! roles appear as HTML classes. Only the introduction, the section headings,
//! and a couple of tangential asides are tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/role.adoc");

non_normative!(
    r#"
= Role Attribute
:page-aliases: roles.adoc

You can assign one or more roles to blocks and most inline elements using the `role` attribute.
The `role` attribute is a xref:positional-and-named-attributes.adoc#named[named attribute].
Even though the attribute name is singular, it may contain multiple (space-separated) roles.
Roles may also be defined using a shorthand (dot-prefixed) syntax.

A role:

. adds additional semantics to an element
. can be used to apply additional styling to a group of elements (e.g., via a CSS class selector)
. may activate additional behavior if recognized by the converter

TIP: The `role` attribute in AsciiDoc always get mapped to the `class` attribute in the HTML output.
In other words, role names are synonymous with HTML class names, thus allowing output elements to be identified and styled in CSS using class selectors (e.g., `sidebarblock.role1`).

"#
);

mod assign_roles_to_blocks {
    non_normative!(
        r#"
== Assign roles to blocks

"#
    );

    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
You can assign roles to blocks using the shorthand dot (`.`) syntax or the longhand (`role=`) syntax.

"#
    );

    #[test]
    fn shorthand_role_syntax_single() {
        verifies!(
            r#"
=== Shorthand role syntax for blocks

To assign a role to a block, prefix the value with a dot (`.`) in style style position of an attribute list.
The dot implicitly sets the `role` attribute.

.Sidebar block with a role assigned using the shorthand dot
[source#ex-block]
----
[.rolename]
****
This is a sidebar with a role assigned to it, rolename.
****
----

"#
        );

        let output = convert(
            "[.rolename]\n****\nThis is a sidebar with a role assigned to it, rolename.\n****\n",
        );

        // The dot shorthand sets the `role`, which the renderer emits as an HTML
        // class alongside the block's context class.
        assert_css(&output, "div.sidebarblock.rolename", 1);
    }

    #[test]
    fn shorthand_role_syntax_multiple() {
        verifies!(
            r#"
You can assign multiple roles to a block by prefixing each value with a dot (`.`).

.Sidebar with two roles assigned using the shorthand dot
[source#ex-two-roles]
----
[.role1.role2]
****
This is a sidebar with two roles assigned to it, role1 and role2.
****
----

The role values are turned into a space-separated list of values, `role1 role2`.

"#
        );

        let output = convert(
            "[.role1.role2]\n****\nThis is a sidebar with two roles assigned to it, role1 and role2.\n****\n",
        );

        // Each dotted value becomes its own class: `class="sidebarblock role1 role2"`.
        assert_css(&output, "div.sidebarblock.role1.role2", 1);
    }

    #[test]
    fn formal_role_syntax_single() {
        verifies!(
            r#"
=== Formal role syntax for blocks

You can define the roles using a named attribute instead, which is the longhand syntax for adding roles to an element.
When using this syntax, add the attribute name `role` followed by the equals sign (`=`) then the role name or names to any position in the block attribute list.

.Sidebar block with a role assigned using the formal syntax
[source#ex-block-formal]
----
[role=rolename]
****
This is a sidebar with one role assigned to it, rolename.
****
----

"#
        );

        let output = convert(
            "[role=rolename]\n****\nThis is a sidebar with one role assigned to it, rolename.\n****\n",
        );

        // The formal `role=` syntax produces the same class as the shorthand dot.
        assert_css(&output, "div.sidebarblock.rolename", 1);
    }

    #[test]
    fn formal_role_syntax_multiple() {
        verifies!(
            r#"
Separate multiple role values using spaces.
Since the value has spaces, it's easier to read if enclosed in quotes, though the quotes are not strictly required.

.Sidebar with two roles assigned using the formal syntax
[source#ex-two-roles-formal]
----
[role="role1 role2"]
****
This is a sidebar with two roles assigned to it, role1 and role2.
****
----

"#
        );

        let output = convert(
            "[role=\"role1 role2\"]\n****\nThis is a sidebar with two roles assigned to it, role1 and role2.\n****\n",
        );

        // A space-separated `role=` value becomes one class per role.
        assert_css(&output, "div.sidebarblock.role1.role2", 1);
    }

    non_normative!(
        r#"
In this form, the value of the role attribute is already in the right form to be passed through to the output.
No additional processing is done on it.

This longhand syntax can also be used on inline macros, but it cannot be used with formatted (aka quoted) text.

"#
    );
}

mod assign_roles_to_formatted_inline_elements {
    use crate::{
        convert,
        tests::{assert_html::assert_css, sdd::*},
    };

    non_normative!(
        r#"
== Assign roles to formatted inline elements

"#
    );

    #[test]
    fn assign_roles_to_inline_elements() {
        verifies!(
            r#"
You can assign roles to inline elements that are enclosed in formatting syntax, such as bold (`+*+`), italic (`+_+`), and monospace (`++`++`).
To assign a role to an inline element that's enclosed in formatting syntax block, prefix the value with a dot (`.`) inside the boxed attrlist.

.Inline role assignments using shorthand syntax
[source#ex-role-dot]
----
This sentence contains [.application]*bold inline content* that's assigned a role.

This sentence contains [.varname]`monospace text` that's assigned a role.
----

IMPORTANT: The boxed attrlist on formatted text only supports the attribute shorthand syntax.
It does not support named attributes (e.g. `name=value`).

The HTML source code that is output from <<ex-role-dot>> is shown below.

.HTML source code produced by <<ex-role-dot>>
[source#ex-role-html,html]
----
<p>This sentence contains <strong class="application">bold inline content</strong> that&#8217;s assigned a role.</p>

<p>This sentence contains <code class="varname">monospace text</code> that&#8217;s assigned a role.</p>
</div>
----

As you can see from this output, roles in AsciiDoc are translated to CSS class names in HTML.
Thus, roles are an ideal way to annotated elements in your document so you can use CSS to uniquely style them.

"#
        );

        let output = convert(
            "This sentence contains [.application]*bold inline content* that's assigned a role.\n\n\
             This sentence contains [.varname]`monospace text` that's assigned a role.\n",
        );

        // The dotted role becomes a class on the formatting element itself:
        // `<strong class="application">` and `<code class="varname">`.
        assert_css(&output, "strong.application", 1);
        assert_css(&output, "code.varname", 1);
    }

    non_normative!(
        r#"
The role is often used on a phrase to represent semantics you might have expressed using a dedicated element in DocBook or DITA.

"#
    );

    #[test]
    fn assign_multiple_roles() {
        verifies!(
            r#"
If you need to assign multiple roles, you must join them together in a series:

.Formatted text with multiple roles
[source#ex-roles]
----
This [.rolename1.rolename2]#formatted text# has two roles.
----

The roles can also be accompanied by an ID assignment.
"#
        );

        let output = convert("This [.rolename1.rolename2]#formatted text# has two roles.\n");

        // Unconstrained formatted text (`#…#`) with two roles renders as a
        // `<span>` carrying both role classes.
        assert_css(&output, "span.rolename1.rolename2", 1);
    }
}
