//! Coverage of the AsciiDoc language description's *Customize the Substitutions
//! Applied to Blocks* page.
//!
//! A block's `subs` attribute replaces (or, with `+`/`-` modifiers,
//! incrementally amends) the block's default substitutions. This crate
//! implements every worked example the page shows through `convert`, so those
//! examples are verified. The definition list of step/group names and the
//! `+`/`-` modifier definitions are descriptive, so they are tracked as
//! non-normative. The examples are pulled in with
//! `include::example$subs.adoc[…]`; the reproduced lines keep the include
//! directives, while the tests drive the included example text directly.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/subs/pages/apply-subs-to-blocks.adoc");

non_normative!(
    r#"
= Customize the Substitutions Applied to Blocks

Each block context is associated with a set default substitutions that best suit the content model.
However, there are situations where you may need a different set of substitutions to be applied.
For example, you may want the AsciiDoc processor to substitute attribute references in a listing block.
Therefore, the AsciiDoc language provides a mechanism for altering the substitutions on a block.

== The subs attribute

The substitutions that get applied to a block (and to certain inline elements) can be changed or modified using the `subs` element attribute.
This attribute accepts a comma-separated list of substitution steps or groups.
The list _replaces_ the substitutions normally applied to a block unless <<incremental,incremental substitutions>> are specified.

The names of those substitution steps and groups are as follows:

[#subs-groups]
`none`:: Substitution group that disables all substitutions.

`normal`:: Substitution group that performs all substitution types except callouts.

`verbatim`:: Substitution group that replaces special characters and processes callouts.

`specialchars`:: Substitution step that replaces `<`, `>`, and `&` with their corresponding entities.
For source blocks, this substitution step enables syntax highlighting as well.

`callouts`:: Substitution step that processes callouts in literal, listing, and source blocks.

`quotes`:: Substitution step that applies inline text formatting.

`attributes`:: Substitution step that replaces attribute references.

`replacements`:: Substitution step that replaces hexadecimal Unicode code points and entity, HTML, and XML character references with the characters' decimal Unicode code point.
The output of `replacements` may depend on whether the `specialcharacters` substitution was previously applied.

`macros`:: Substitution step that processes inline and block macros.

`post_replacements`:: Substitution step that processes the line break character (`{plus}`).

If a `+` or `-` modifier is added to a step, the existing substitutions are modified accordingly (see <<incremental,incremental subs>>).
Otherwise, the existing substitutions are replaced.
The value also specifies the order in which the substitutions are applied.

NOTE: The `subs` element attribute does not inherit to nested blocks.
It can only be applied to a leaf block, which is any block that cannot have child blocks (e.g., a paragraph or a listing block).

== Set the subs attribute on a block

CAUTION: You should almost always prefer to use <<incremental,incremental substitutions>>.
Only switch to exact substitutions when you require very specific control.
That's because setting the `subs` attribute on a block _only_ uses the substitutions specified.
In contrast, incremental substitutions amends the default substitutions for that block.

"#
);

// Setting `subs="verbatim,quotes"` on a source block reinstates the verbatim
// group (so special characters are encoded) and adds `quotes`, so `*<name>*`
// becomes a `<strong>` element wrapping the encoded `<name>`.
#[test]
fn set_subs_verbatim_quotes() {
    verifies!(
        r#"
Let's look at an example where you want to process inline formatting markup in a source block.
By default, source blocks (as well as other verbatim blocks) are only subject to the verbatim substitution group (specialchars and callouts).
You can change this behavior by setting the `subs` attribute in the block's attribute list.

[source,asciidoc]
....
include::example$subs.adoc[tag=subs-in]
....
<.> The `subs` attribute is set in the attribute list and assigned the `verbatim` and `quotes` values.
It's important to reinstate the `verbatim` substitution step to ensure special characters are encoded (which, for source blocks, also enables syntax highlighting).
<.> The formatting markup in this line will be replaced when the `quotes` substitution step runs.

Here's the result.

====
include::example$subs.adoc[tag=subs-out]
====
<.> The `verbatim` value enables any special characters and callouts to be processed.
<.> The `quotes` value enables the bold text formatting to be processed.

"#
    );

    let out = convert(
        "[source,java,subs=\"verbatim,quotes\"]\n----\nSystem.out.println(\"Hello *<name>*\")\n----\n",
    );
    assert!(out.contains("System.out.println(\"Hello <strong>&lt;name&gt;</strong>\")"));
}

// Enabling `macros` instead lets a `pass:c,q[…]` macro inside the block apply
// the special characters and quotes steps locally, while the surrounding
// verbatim content keeps its markup literal.
#[test]
fn set_subs_verbatim_macros() {
    verifies!(
        r#"
If enabling the quotes substitution step on the whole block causes problems, you can instead enable the macros substitution step, then use the pass macro to enable the quotes substitution step locally.

[source,asciidoc]
....
[source,java,subs="verbatim,macros"]
----
System.out.println("No bold *here*");
pass:c,q[System.out.println("Hello *<name>*");] <1>
----
....
<1> The pass macro with the `c,q` target applies the specialchars and quotes substitution steps to the enclosed text.

"#
    );

    let out = convert(
        "[source,java,subs=\"verbatim,macros\"]\n----\nSystem.out.println(\"No bold *here*\");\n\
         pass:c,q[System.out.println(\"Hello *<name>*\");]\n----\n",
    );
    assert!(out.contains("System.out.println(\"No bold *here*\");"));
    assert!(out.contains("System.out.println(\"Hello <strong>&lt;name&gt;</strong>\");"));
}

non_normative!(
    r#"
You may be wondering why `verbatim` is specified in the previous examples since it's applied to literal blocks by default.
The reason is that when you specify substitutions without a modifier, it replaces all existing substitutions.
Therefore, it's necessary to start with `verbatim` in order to restore the default substitutions.
You can avoid having to do this by using incremental substitutions instead, which is covered in the next section.

[#incremental]
== Add and remove substitution types from a default substitution group

"#
);

// Setting `subs` without a modifier removes every default substitution, so a
// literal block set to `attributes` resolves references but no longer encodes
// special characters (the verbatim group is gone).
#[test]
fn incremental_replaces_defaults() {
    verifies!(
        r#"
When you set the `subs` attribute on a block, you automatically *remove* all of its default substitutions.
For example, if you set `subs` on a literal block, and assign it a value of `attributes`, only attribute references are substituted.
The `verbatim` substitution group will not be applied.
To remedy this situation, AsciiDoc provides a syntax to append or remove substitutions instead of replacing them outright.

"#
    );

    let out = convert(":v: RESOLVED\n[subs=\"attributes\"]\n....\n<x> {v}\n....\n");
    // `{v}` is resolved (attributes applied) but `<x>` is left unencoded
    // (verbatim/specialchars no longer applied).
    assert!(out.contains("<pre><x> RESOLVED</pre>"));
}

non_normative!(
    r#"
You can add or remove a substitution type from the default substitution group using the plus (`+`) and minus (`-`) modifiers.
These are known as [.term]*incremental substitutions*.

`<substitution>+`::
Prepends the substitution to the default list.

`+<substitution>`::
Appends the substitution to the default list.

`-<substitution>`::
Removes the substitution from the default list.

"#
);

// The `attributes+` modifier adds the attributes step to a source block's
// default group, so `{version}` is resolved while the block stays verbatim.
#[test]
fn incremental_add() {
    verifies!(
        r#"
For example, you can add the `attributes` substitution to the beginning of a listing block's default substitution group by placing the plus (`+`) modifier at the end of the `attributes` value.

.Add attributes substitution to default substitution group
[source]
....
include::example$subs.adoc[tag=subs-add]
....

"#
    );

    let out = convert(
        ":version: 1.0\n[source,xml,subs=\"attributes+\"]\n----\n<version>{version}</version>\n----\n",
    );
    assert!(out.contains("&lt;version&gt;1.0&lt;/version&gt;"));
}

// The `-callouts` modifier removes callout processing, so `<1>` is treated as
// ordinary (special-character-encoded) verbatim text rather than a callout.
#[test]
fn incremental_remove() {
    verifies!(
        r#"
Similarly, you can remove the `callouts` substitution from a block's default substitution group by placing the minus (`-`) modifier in front of the `callouts` value.

.Remove callouts substitution from default substitution group
[source,subs=-callouts]
....
include::example$subs.adoc[tag=subs-sub]
....

"#
    );

    let out = convert(
        "[source,xml,subs=\"-callouts\"]\n.An illegal XML tag\n----\n<1>\n  content inside \"1\" tag\n</1>\n----\n",
    );
    assert!(out.contains("&lt;1&gt;"));
    assert!(out.contains("&lt;/1&gt;"));
}

// A combined value adds `attributes` to the front, appends `replacements`, and
// removes `callouts`: `{version}` resolves, `(C)` becomes the copyright sign,
// and `<1>` stays literal.
#[test]
fn incremental_multi() {
    verifies!(
        r#"
You can also specify whether the substitution type is added to the end of the substitution group.
If a `{plus}` comes before the name of the substitution, then it's added to the end of the existing list, whereas if a `{plus}` comes after the name, it's added to the beginning of the list.

[source,subs=-callouts]
....
include::example$subs.adoc[tag=subs-multi]
....

In the above example, the `attributes` substitution step is added to the beginning of the default substitution group, the `replacements` step is added to the end of the group, and the `callouts` step is removed from the group.

"#
    );

    let out = convert(
        ":version: 1.0\n[source,xml,subs=\"attributes+,+replacements,-callouts\"]\n----\n\
         <version>{version}</version>\n<copyright>(C) ACME</copyright>\n<1>\n  content inside \"1\" tag\n</1>\n----\n",
    );
    assert!(out.contains("&lt;version&gt;1.0&lt;/version&gt;"));
    assert!(out.contains("&lt;copyright&gt;&#169; ACME&lt;/copyright&gt;"));
    assert!(out.contains("&lt;1&gt;"));
}

// Editorial placeholder comment in the source page; it carries no rule.
non_normative!(
    r#"
// NOTE: More examples are pending. Information about the callouts substitution also needs to be included here.

"#
);

// A reusable substitution value stored in an attribute entry (`+quotes`)
// applies just as if it were written inline, so the source block's bold markup
// is formatted.
#[test]
fn incremental_attr_entry() {
    verifies!(
        r#"
[TIP]
====
If you are applying the same set of substitutions to numerous blocks, you should consider making them an attribute entry to ensure consistency.

[source]
....
include::example$subs.adoc[tag=subs-attr]
....

Another way to ensure consistency and keep your documents clean and simple is to use the xref:asciidoctor:extensions:tree-processor.adoc[tree Processor extension].
====
"#
    );

    let out = convert(
        ":markup-in-source: +quotes\n\n[source,java,subs=\"{markup-in-source}\"]\n----\n\
         System.out.println(\"Hello *bold* text\").\n----\n",
    );
    assert!(out.contains("System.out.println(\"Hello <strong>bold</strong> text\")."));
}
