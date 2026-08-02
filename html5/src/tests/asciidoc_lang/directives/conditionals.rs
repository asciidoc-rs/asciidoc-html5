//! Coverage of the AsciiDoc language description's *Conditionals* page — the
//! overview of the `ifdef`, `ifndef`, and `ifeval` conditional preprocessor
//! directives.
//!
//! The page's normative content is the rule that a conditional's condition
//! decides whether the enclosed lines are kept or skipped, and that a
//! backslash-escaped directive is not processed (even inside a verbatim block).
//! Both are verified through `convert`. The introduction, the cross-reference
//! list, the hidden author note, and the "it's a preprocessor directive, not a
//! macro" discussion are tracked as non-normative.

use super::convert_with_attrs;
use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/conditionals.adoc");

non_normative!(
    r#"
= Conditionals

You can include or exclude lines of text in your document using the following conditional preprocessor directives:

* xref:ifdef-ifndef.adoc#ifdef[ifdef]
* xref:ifdef-ifndef.adoc#ifndef[ifndef]
* xref:ifeval.adoc[ifeval]

"#
);

// The core rule shared by every conditional directive: the condition is
// evaluated, and the enclosed lines are kept when it is true and skipped when
// it is false. Verified here with the simplest condition — an `ifdef` on a
// single attribute — driving both outcomes; the per-directive pages exercise
// the value-based and multi-attribute conditions.
#[test]
fn condition_decides_whether_enclosed_lines_are_included() {
    verifies!(
        r#"
When the processor encounters one of these conditionals, it evaluates the specified condition.
The condition is based on the presence or value of one or more document attributes.
If the condition evaluates to true, the lines the conditional encloses are included.
Otherwise, the lines are skipped.

"#
    );

    // Condition true (the attribute is set): the enclosed line is included.
    let included = convert_with_attrs(
        "ifdef::flag[]\nconditional line\nendif::flag[]",
        &[("flag", "")],
    );
    assert!(included.contains("conditional line"), "{included}");

    // Condition false (the attribute is not set): the enclosed line is skipped.
    let skipped = convert_with_attrs("ifdef::flag[]\nconditional line\nendif::flag[]", &[]);
    assert!(!skipped.contains("conditional line"), "{skipped}");
}

// Non-normative: a hidden author note (inside an AsciiDoc `////` comment block,
// so it is never rendered), plus the discussion that a conditional is a
// preprocessor directive rather than a block macro — descriptive framing with
// no distinct rule to drive, and its `partial$` include cannot be resolved
// here.
non_normative!(
    r#"
////
For example, say you want to include a certain section of content only when converting to HTML.
Conditional preprocessor directives make this possible.
You simply check for the presence of the `basebackend-html` attribute using an `ifdef` directive.
Details of this example and more `ifdef` and `ifndef` examples, are described in the following sections.
See xref:ifeval.adoc[] for `ifeval` examples.
////

== Conditional processing

Although a conditional preprocessor directive looks like a block macro, *it's not a macro and therefore isn't processed like one*.
It's a preprocessor directive; it's important to understand the distinction.

include::partial$preprocessor.adoc[]
The conditional preprocessor directives determine which lines to add and which ones to take away based on the condition.

== Escape a conditional directive

"#
);

// Escaping a conditional directive with a leading backslash stops it from being
// processed, so the directive text is emitted literally. Because a conditional
// is a preprocessor directive with no awareness of the surrounding structure,
// the escape is required even inside a verbatim block — an *unescaped*
// directive there is still processed away.
#[test]
fn an_escaped_directive_is_not_processed() {
    verifies!(
        r#"
If you don't want a conditional preprocessor directive to be processed, you must escape it using a backslash.

// NOTE: the following listing uses indentation to prevent the directive from being processed
[source,indent=0]
----
 \ifdef::just-an-example[]
----

Escaping the directive is necessary _even if it appears in a verbatim block_ since it's not aware of the surrounding document structure.
"#
    );

    // An escaped directive at the top level is passed through verbatim.
    let escaped = convert_with_attrs(
        "\\ifdef::just-an-example[]\ncontent\n\\endif::just-an-example[]",
        &[],
    );
    assert!(escaped.contains("ifdef::just-an-example[]"), "{escaped}");

    // Inside a verbatim block the escape is still required: escaped, the
    // directive renders literally ...
    let escaped_verbatim = convert_with_attrs(
        "----\n\\ifdef::just-an-example[]\ncontent\n\\endif::just-an-example[]\n----",
        &[],
    );
    assert!(
        escaped_verbatim.contains("ifdef::just-an-example[]"),
        "{escaped_verbatim}"
    );

    // ... whereas an unescaped directive in the same block is processed away
    // before the block is parsed, leaving only its enclosed content.
    let unescaped_verbatim = convert_with_attrs(
        "----\nifdef::just-an-example[]\ncontent\nendif::just-an-example[]\n----",
        &[("just-an-example", "")],
    );
    assert!(
        !unescaped_verbatim.contains("ifdef::just-an-example[]"),
        "{unescaped_verbatim}"
    );
    assert!(
        unescaped_verbatim.contains("content"),
        "{unescaped_verbatim}"
    );
}
