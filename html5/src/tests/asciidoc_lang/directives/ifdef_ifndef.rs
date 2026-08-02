//! Coverage of the AsciiDoc language description's *ifdef and ifndef
//! Directives* page.
//!
//! Every rule the page states about attribute-presence conditionals is verified
//! through `convert`: `ifdef` includes its content when the attribute is set,
//! `ifndef` when it is not; the single-line (bracketed) and long-form (named
//! `endif`) variants; and the "or" (comma) and "and" (plus) combinators for
//! multiple attributes, including how `ifndef` negates each. Only the section
//! headings, the syntax restatements, and the framing prose are non-normative.

use super::convert_with_attrs;
use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/ifdef-ifndef.adoc");

non_normative!(
    r#"
= ifdef and ifndef Directives

[#ifdef]
== ifdef directive

"#
);

// `ifdef` includes the content between it and its `endif` when the named
// attribute is set, and drops it when the attribute is not set.
#[test]
fn ifdef_includes_content_when_the_attribute_is_set() {
    verifies!(
        r#"
Content between the `ifdef` and `endif` directives gets included if the specified attribute is set:

.ifdef example
----
\ifdef::env-github[]
This content is for GitHub only.
\endif::[]
----

"#
    );

    let src = "ifdef::env-github[]\nThis content is for GitHub only.\nendif::[]";

    let set = convert_with_attrs(src, &[("env-github", "")]);
    assert!(set.contains("This content is for GitHub only."), "{set}");

    let unset = convert_with_attrs(src, &[]);
    assert!(
        !unset.contains("This content is for GitHub only."),
        "{unset}"
    );
}

// Non-normative: a restatement of the start-directive syntax.
non_normative!(
    r#"
The syntax of the start directive is `ifdef::<attribute>[]`, where `<attribute>` is the name of an attribute.

"#
);

// The enclosed content is not limited to a single line: any amount of content
// may sit between `ifdef` and `endif`, and the long-form directive names the
// attribute again on the closing `endif` for readability.
#[test]
fn ifdef_content_is_not_limited_to_a_single_line() {
    verifies!(
        r#"
Keep in mind that the content is not limited to a single line.
You can have any amount of content between the `ifdef` and `endif` directives.

If you have a large amount of content inside the `ifdef` directive, you may find it more readable to use the long-form version of the directive, in which the attribute (aka condition) is referenced again in the `endif` directive.

.ifdef long-form example
----
\ifdef::env-github[]
This content is for GitHub only.

So much content in this section, I'd get confused reading the source without the closing `ifdef` directive.

It isn't necessary for short blocks, but if you are conditionally including a section it may be something worth considering.

Other readers reviewing your docs source code may go cross-eyed when reading your source docs if you don't.
\endif::env-github[]
----

"#
    );

    let src = "ifdef::env-github[]\nFirst paragraph of GitHub-only content.\n\n\
               Second paragraph of GitHub-only content.\nendif::env-github[]\n\nAlways shown.";

    let set = convert_with_attrs(src, &[("env-github", "")]);
    assert!(
        set.contains("First paragraph of GitHub-only content."),
        "{set}"
    );
    assert!(
        set.contains("Second paragraph of GitHub-only content."),
        "{set}"
    );

    // The named `endif` closes the region: the trailing line falls outside it
    // and is shown regardless of the attribute.
    let unset = convert_with_attrs(src, &[]);
    assert!(
        !unset.contains("First paragraph of GitHub-only content."),
        "{unset}"
    );
    assert!(unset.contains("Always shown."), "{unset}");
}

// The single-line form puts the content directly in the brackets and drops the
// `endif`; it is equivalent to the formal three-line directive. Attribute
// references inside the content are still substituted.
#[test]
fn ifdef_single_line_form_is_equivalent_to_the_formal_form() {
    verifies!(
        r#"
If you're only dealing with a single line of text, you can put the content directly inside the square brackets and drop the `endif` directive.

.ifdef single line example
----
\ifdef::revnumber[This document has a version number of {revnumber}.]
----

The single-line block above is equivalent to this formal `ifdef` directive:

----
\ifdef::revnumber[]
This document has a version number of {revnumber}.
\endif::[]
----

"#
    );

    let expected = "This document has a version number of 2.9.";

    let single = convert_with_attrs(
        "ifdef::revnumber[This document has a version number of {revnumber}.]",
        &[("revnumber", "2.9")],
    );
    assert!(single.contains(expected), "{single}");

    let formal = convert_with_attrs(
        "ifdef::revnumber[]\nThis document has a version number of {revnumber}.\nendif::[]",
        &[("revnumber", "2.9")],
    );
    assert!(formal.contains(expected), "{formal}");
}

non_normative!(
    r#"
[#ifndef]
== ifndef directive

"#
);

// `ifndef` is the logical opposite of `ifdef`: it includes its content only
// when the attribute is *not* set.
#[test]
fn ifndef_includes_content_only_when_the_attribute_is_not_set() {
    verifies!(
        r#"
`ifndef` is the logical opposite of `ifdef`.
Content between `ifndef` and `endif` gets included only if the specified attribute is _not_ set:

.ifndef example
----
\ifndef::env-github[]
This content is not shown on GitHub.
\endif::[]
----

"#
    );

    let src = "ifndef::env-github[]\nThis content is not shown on GitHub.\nendif::[]";

    let unset = convert_with_attrs(src, &[]);
    assert!(
        unset.contains("This content is not shown on GitHub."),
        "{unset}"
    );

    let set = convert_with_attrs(src, &[("env-github", "")]);
    assert!(
        !set.contains("This content is not shown on GitHub."),
        "{set}"
    );
}

// Non-normative: the `ifndef` start-directive syntax, its shared variants, and
// the framing of the multiple-attribute section.
non_normative!(
    r#"
The syntax of the start directive is `ifndef::<attribute>[]`, where `<attribute>` is the name of an attribute.

The `ifndef` directive supports the same single-line and long-form variants as `ifdef`.

== Checking multiple attributes

Both the `ifdef` and `ifndef` directives accept multiple attribute names.
The combinator can be "`and`" or "`or`".
The two combinators cannot be combined in the same expression.

=== ifdef with multiple attributes

"#
);

// With `ifdef`, a comma between attribute names is the "or" combinator: the
// content is included if *any* of the listed attributes is set.
#[test]
fn ifdef_with_comma_includes_when_any_attribute_is_set() {
    verifies!(
        r#"
If any attribute is set (or)::
Multiple attribute names must be separated by commas (`,`).
If one or more of the attributes are set, the content is included.
Otherwise, the content is not included.
+
.If any attribute example
----
\ifdef::backend-html5,backend-docbook5[Only shown if converting to HTML (backend-html5 is set) or DocBook (backend-docbook5 is set).]
----

"#
    );

    // The page's example names `backend-html5`, which this renderer always sets;
    // to exercise both branches of the "or" cleanly, use two attributes that are
    // not set by default.
    let src = "ifdef::feature-preview,feature-beta[Shown when either feature attribute is set.]";

    let one = convert_with_attrs(src, &[("feature-preview", "")]);
    assert!(
        one.contains("Shown when either feature attribute is set."),
        "{one}"
    );

    let none = convert_with_attrs(src, &[]);
    assert!(
        !none.contains("Shown when either feature attribute is set."),
        "{none}"
    );
}

// With `ifdef`, a plus between attribute names is the "and" combinator: the
// content is included only if *all* of the listed attributes are set.
#[test]
fn ifdef_with_plus_includes_only_when_all_attributes_are_set() {
    verifies!(
        r#"
If all attributes are set (and)::
Multiple attribute names must be separated by pluses (`+`).
If all the attributes are set, the content is included.
Otherwise, the content is not included.
+
.If all attributes example
----
\ifdef::backend-html5+env-github[Only shown when converting to HTML (backend-html5 is set) on GitHub (env-github is set).]
----

"#
    );

    let src = "ifdef::backend-html5+env-github[Only shown when converting to HTML (backend-html5 is set) on GitHub (env-github is set).]";

    let all = convert_with_attrs(src, &[("backend-html5", ""), ("env-github", "")]);
    assert!(all.contains("Only shown when converting to HTML"), "{all}");

    let one = convert_with_attrs(src, &[("backend-html5", "")]);
    assert!(!one.contains("Only shown when converting to HTML"), "{one}");
}

// Non-normative: the framing that `ifndef` negates the multi-attribute
// expression, read with an "unless" prefix.
non_normative!(
    r#"
=== ifndef with multiple attributes

The `ifndef` directive negates the results of the expression.
When using the `ifndef` directive, the expression should be read with the prefix "`unless`".

"#
);

// With `ifndef`, a comma is still the "or" combinator, but the result is
// negated: the content is dropped if *any* attribute is set, and included only
// when none are.
#[test]
fn ifndef_with_comma_drops_when_any_attribute_is_set() {
    verifies!(
        r#"
Unless any attribute is set (or)::
Multiple attribute names must be separated by commas (`,`).
If one or more of the attributes are set, the content is not included.
Otherwise, the content is included.
+
.Unless any attribute example
----
\ifndef::profile-production,env-site[Not shown if profile-production or env-site is set.]
----

"#
    );

    let src =
        "ifndef::profile-production,env-site[Not shown if profile-production or env-site is set.]";

    let none = convert_with_attrs(src, &[]);
    assert!(none.contains("Not shown if profile-production"), "{none}");

    let one = convert_with_attrs(src, &[("env-site", "")]);
    assert!(!one.contains("Not shown if profile-production"), "{one}");
}

// With `ifndef`, a plus is the "and" combinator, negated: the content is
// dropped only if *all* attributes are set, and included otherwise.
#[test]
fn ifndef_with_plus_drops_only_when_all_attributes_are_set() {
    verifies!(
        r#"
Unless all attributes are set (and)::
Multiple attribute names must be separated by pluses (`+`).
If all of the attributes are set, the content is not included.
Otherwise, the content is included.
+
.Unless all attributes example
----
\ifndef::profile-staging+env-site[Not shown if profile-staging and env-site are set.]
----
"#
    );

    let src =
        "ifndef::profile-staging+env-site[Not shown if profile-staging and env-site are set.]";

    let all = convert_with_attrs(src, &[("profile-staging", ""), ("env-site", "")]);
    assert!(!all.contains("Not shown if profile-staging"), "{all}");

    let one = convert_with_attrs(src, &[("env-site", "")]);
    assert!(one.contains("Not shown if profile-staging"), "{one}");
}
