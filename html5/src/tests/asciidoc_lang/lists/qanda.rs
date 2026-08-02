//! Coverage of the AsciiDoc language description's *Question and Answer Lists*
//! page.
//!
//! A `[qanda]` description list renders as an ordered list (`<div
//! class="qlist qanda"><ol>…`): each entry's term(s) become the question,
//! emphasized and each on its own line, and the description becomes the answer.
//! This crate renders that form, so both the page's example and the claims it
//! makes about the output are verified through `convert`. Only the page title
//! and its section heading are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/qanda.adoc");

non_normative!(
    r#"
= Question and Answer Lists
:keywords: qanda, Q & A, Q and A

"#
);

// A `[qanda]` list is a description list whose block style makes it render as
// an ordered list. The `<ol>` carries no `start`, so its items number with
// Arabic numerals from 1 (the default numeration).
#[test]
fn qanda_renders_as_an_ordered_list() {
    verifies!(
        r#"
A question and answer (qanda) list is a special form of a description list that renders as an ordered list.
The entries are numbered using Arabic numerals starting at 1.

"#
    );

    let output = convert(
        "[qanda]\nWhat is the answer?::\nThis is the answer.\n\n\
         Are cameras allowed?::\nAre backpacks allowed?::\nNo.\n",
    );

    assert_css(&output, "div.qlist.qanda > ol", 1);
    assert_css(&output, "ol[start]", 0);
    assert_css(&output, "ol[type]", 0);
}

non_normative!(
    r#"
== Question and answer list syntax

"#
);

// Each description-list entry becomes one `<li>`: the term(s) render as the
// question (emphasized) and the description as the answer. When an entry has
// more than one term, each question is emphasized on its own line above the
// shared answer.
#[test]
fn each_entry_pairs_questions_with_an_answer() {
    verifies!(
        r#"
Each entry in the description list represents one question and answer combination.
The term or terms are used as the question and the description is used as the answer.
If an entry has multiple questions, each question is rendered on a new line.

----
include::example$description.adoc[tag=qa]
----

.Rendered qanda list
====
include::example$description.adoc[tag=qa]
====
"#
    );

    let output = convert(
        "[qanda]\nWhat is the answer?::\nThis is the answer.\n\n\
         Are cameras allowed?::\nAre backpacks allowed?::\nNo.\n",
    );

    // Two entries, one per question-and-answer combination.
    assert_css(&output, "div.qlist.qanda > ol > li", 2);

    // The first entry: a single emphasized question, then the answer.
    assert_xpath(
        &output,
        r#"(//ol/li)[1]/p/em[text()="What is the answer?"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"(//ol/li)[1]/p[text()="This is the answer."]"#,
        1,
    );

    // The second entry: two emphasized questions, each on its own line, then
    // the shared answer.
    assert_xpath(&output, r#"(//ol/li)[2]/p/em"#, 2);
    assert_xpath(
        &output,
        r#"(//ol/li)[2]/p/em[text()="Are cameras allowed?"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"(//ol/li)[2]/p/em[text()="Are backpacks allowed?"]"#,
        1,
    );
    assert_xpath(&output, r#"(//ol/li)[2]/p[text()="No."]"#, 1);
}
