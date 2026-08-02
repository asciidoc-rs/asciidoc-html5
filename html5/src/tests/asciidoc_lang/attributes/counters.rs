//! Coverage of the AsciiDoc language description's *Counters* page.
//!
//! A counter is a specialized document attribute that the processor increments
//! and substitutes each time a `{counter:name}` reference is resolved. Every
//! sequence the page demonstrates — plain increments, seeding a start value,
//! the non-displaying `counter2` prefix, resetting by unsetting the attribute,
//! character sequences, and a counter driving part numbers in a table — is
//! observed here by converting the shown source and checking the substituted
//! values in the rendered HTML. Only the introduction and its warning are
//! tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/counters.adoc");

non_normative!(
    r#"
= Counters
// document attributes and counters are NOT the same thing, but modifying a document attribute with the same name as the counter modifies the counter at the same time.

Counters are used to store and display ad-hoc sequences of numbers or Latin characters.

WARNING: Counters are a poorly defined feature in AsciiDoc and should be avoided if possible.
If you do use counters, you should only used them for the most rudimentary use cases, such as making a sequence in a list, table column, or prose.
You should *not* use counters to build IDs (i.e., references) or reference text.
Using counters across the boundaries of a reference will very likely result in unexpected behavior.

"#
);

#[test]
fn declare_and_display_a_counter() {
    verifies!(
        r#"
A counter is implemented as a specialized document attribute.
You declare and display a counter using an attribute reference, where the attribute name is prefixed with `counter:` (e.g., `+{counter:name}+`).
Since counters are attributes, counter names follow the same rules as xref:names-and-values.adoc#user-defined[attribute names].
The most important rule to note is that letters in counter names _must be lowercase_.

The counter value is incremented and displayed every time the `counter:` attribute reference is resolved.
The term [.term]*increment* means to advance the attribute value to the next value in the sequence.
If the counter value is an integer, add 1.
If the counter value is a character, move to the next letter in the Latin alphabet (e.g., a -> b).
The default start value of a counter is 1.

To create a sequence starting at 1, use the simple form `+{counter:name}+` as shown here:

[source]
The salad calls for {counter:seq1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.

Here's the resulting output:

====
:!seq1:
The salad calls for {counter:seq1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.
====

"#
    );

    let output = convert(
        ":!seq1:\nThe salad calls for {counter:seq1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.",
    );

    // Each `{counter:seq1}` reference advances and displays the value: 1, 2, 3.
    assert_xpath(
        &output,
        "//p[text()=\"The salad calls for 1) apples, 2) oranges and 3) pears.\"]",
        1,
    );
}

#[test]
fn use_a_counter_in_a_section_title() {
    verifies!(
        r#"
If you want to use a counter value in a section title, you should define it first using an attribute reference.

----
:seq1: {counter:seq1}
== Section {seq1}

The sequence in this section is {seq1}.

:seq1: {counter:seq1}
== Section {seq1}

The sequence in this section is {seq1}.
----

Here's the resulting output:

====
:!seq1:

:seq1: {counter:seq1}
[discrete]
== Section {seq1}

The sequence in this section is {seq1}.

:seq1: {counter:seq1}
[discrete]
== Section {seq1}

The sequence in this section is {seq1}.
====

"#
    );

    let output = convert(
        ":!seq1:\n\n:seq1: {counter:seq1}\n[discrete]\n== Section {seq1}\n\nThe sequence in this section is {seq1}.\n\n:seq1: {counter:seq1}\n[discrete]\n== Section {seq1}\n\nThe sequence in this section is {seq1}.",
    );

    // Seeding `seq1` before each heading resolves the counter once per section:
    // the headings read "Section 1" / "Section 2" and the following paragraphs
    // reuse the same resolved value.
    assert_xpath(&output, "//h2[text()=\"Section 1\"]", 1);
    assert_xpath(&output, "//h2[text()=\"Section 2\"]", 1);
    assert_xpath(
        &output,
        "//p[text()=\"The sequence in this section is 1.\"]",
        1,
    );
    assert_xpath(
        &output,
        "//p[text()=\"The sequence in this section is 2.\"]",
        1,
    );
}

#[test]
fn counter2_increments_without_displaying() {
    verifies!(
        r#"
To increment the counter without displaying it (i.e., to skip an item in the sequence), use the `counter2` prefix instead:

[source]
{counter2:seq1}

WARNING: A `counter2` attribute reference on a line by itself will produce an empty paragraph.
You'll need to adjoin it to the nearest content to avoid this side effect.

"#
    );

    let output = convert("{counter2:seq1}\n\n{seq1}");

    // The `counter2` reference on its own line advances the counter but displays
    // nothing, leaving an empty paragraph; the later plain reference shows the
    // value it advanced to (1).
    assert_css(&output, "div.paragraph", 2);
    assert_xpath(&output, "//p[text()=\"1\"]", 1);
}

#[test]
fn display_the_current_value_without_incrementing() {
    verifies!(
        r#"
To display the current value of the counter without incrementing it, reference the counter name as you would any other attribute:

[source]
{counter2:pnum}This is paragraph {pnum}.

"#
    );

    let output = convert("{counter2:pnum}This is paragraph {pnum}.");

    // `counter2:pnum` advances `pnum` to 1 without displaying it; the plain
    // `{pnum}` reference then displays the current value without advancing it.
    assert_xpath(&output, "//p[text()=\"This is paragraph 1.\"]", 1);
}

#[test]
fn create_a_character_sequence_or_custom_start_value() {
    verifies!(
        r#"
To create a character sequence, or start a number sequence with a value other than 1, specify a start value by appending it to the first use of the counter:

[source]
Dessert calls for {counter:seq1:A}) mangoes, {counter:seq1}) grapes and {counter:seq1}) cherries.

CAUTION: Character sequences either run from a,b,c,...x,y,z,{,|... or A,B,C,...,X,Y,Z,[,... depending on the start value.
Therefore, they aren't really useful for more than 26 items.

"#
    );

    let output = convert(
        "Dessert calls for {counter:seq1:A}) mangoes, {counter:seq1}) grapes and {counter:seq1}) cherries.",
    );

    // The `:A` start value seeds a character sequence: A, B, C.
    assert_xpath(
        &output,
        "//p[text()=\"Dessert calls for A) mangoes, B) grapes and C) cherries.\"]",
        1,
    );
}

#[test]
fn reset_a_counter_by_unsetting_the_attribute() {
    verifies!(
        r#"
The start value of a counter is only recognized if the counter is _unset_ at that point in the document.
Otherwise, the start value is ignored.

To reset a counter attribute, unset the corresponding attribute using an attribute entry.
The attribute entry must be adjacent to a block or else it is ignored.

[source]
----
The salad calls for {counter:seq1:1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.

:!seq1:
Dessert calls for {counter:seq1:A}) mangoes, {counter:seq1}) grapes and {counter:seq1}) cherries.
----

This gives:

====
:!seq1:
The salad calls for {counter:seq1:1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.

:!seq1:
Dessert calls for {counter:seq1:A}) mangoes, {counter:seq1}) grapes and {counter:seq1}) cherries.
====

"#
    );

    let output = convert(
        ":!seq1:\nThe salad calls for {counter:seq1:1}) apples, {counter:seq1}) oranges and {counter:seq1}) pears.\n\n:!seq1:\nDessert calls for {counter:seq1:A}) mangoes, {counter:seq1}) grapes and {counter:seq1}) cherries.",
    );

    // Unsetting `seq1` between the two paragraphs lets the second start value
    // (`:A`) take effect, so each paragraph restarts its own sequence.
    assert_xpath(
        &output,
        "//p[text()=\"The salad calls for 1) apples, 2) oranges and 3) pears.\"]",
        1,
    );
    assert_xpath(
        &output,
        "//p[text()=\"Dessert calls for A) mangoes, B) grapes and C) cherries.\"]",
        1,
    );

    // The start value is ignored once the counter is set: re-seeding `seq1`
    // with a different value mid-sequence has no effect.
    let output = convert("{counter:seq1:5} {counter:seq1:9} {counter:seq1:9}");
    assert_xpath(&output, "//p[text()=\"5 6 7\"]", 1);
}

#[test]
fn use_a_counter_for_part_numbers_in_a_table() {
    verifies!(
        r#"
Here's a full example that shows how to use a counter for part numbers in a table.

[source]
----
include::example$counter.adoc[tag=base]
----

Here's the output of that table:

====
include::example$counter.adoc[tag=base]
====
"#
    );

    // The body of `example$counter.adoc` (tag `base`), inlined here so the test
    // does not depend on include resolution.
    let output = convert(
        ".Parts{counter2:index:0}\n|===\n|Part Id |Description\n\n|PX-{counter:index}\n|Description of PX-{index}\n\n|PX-{counter:index}\n|Description of PX-{index}\n|===",
    );

    // The table is numbered with the document-wide table counter, while the
    // per-part `index` counter is seeded to 0 (without being displayed) by the
    // title and then advances 1, 2 across the rows.
    assert_xpath(&output, "//caption[text()=\"Table 1. Parts\"]", 1);
    assert_xpath(&output, "//p[text()=\"PX-1\"]", 1);
    assert_xpath(&output, "//p[text()=\"Description of PX-1\"]", 1);
    assert_xpath(&output, "//p[text()=\"PX-2\"]", 1);
    assert_xpath(&output, "//p[text()=\"Description of PX-2\"]", 1);
}
