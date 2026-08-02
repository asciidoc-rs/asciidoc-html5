//! Coverage of the AsciiDoc language description's *Description Lists* page.
//!
//! A description list pairs one or more terms with a description. Each term
//! becomes a `<dt class="hdlist1">` and each description a following `<dd>`,
//! wrapped in `<div class="dlist"><dl>`. The term delimiter (`::`, `:::`,
//! `::::`, `;;`) sets the nesting level: changing the delimiter starts a nested
//! description list, and a description may hold any block content (such as a
//! nested unordered list). This crate renders those forms, so every section's
//! example and claims — including the hybrid list that mixes all three list
//! types under nested (`:::`) description terms — are verified through
//! `convert`.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/description.adoc");

non_normative!(
    r#"
= Description Lists
:keywords: dlist, definition list, labeled list

A description list (often abbreviated as dlist in AsciiDoc) is an association list that consists of one or more terms (or sets of terms) that each have a description.
This list type is useful when you have a list of terms that you want to emphasize and describe with text or other supporting content.

NOTE: You may know this list variation by the antiquated term _definition list_.
The preferred term is now _description list_, which matches the terminology used by the https://html.spec.whatwg.org/multipage/grouping-content.html#the-dl-element[HTML specification^].

== Anatomy

A description list item marks the beginning of a description list.
Each item in a description list consists of:

"#
);

// Each item is a term followed by a delimiter (`::`), then the description
// (after a space or on the next line). The term renders as `<dt>` and the
// description as the following `<dd>` — whether the description sits on the
// term's line or the line below.
#[test]
fn an_item_pairs_a_term_with_a_description() {
    verifies!(
        r#"
* one or more terms, each followed by a term delimiter (typically a double colon, `::`, unless the list is nested)
* one space or newline character
* the description in the form of text, attached blocks, or both

"#
    );

    let output = convert("CPU:: The brain\nHard drive::\nPermanent storage.\n");
    assert_css(&output, "div.dlist > dl > dt.hdlist1", 2);
    assert_css(&output, "div.dlist > dl > dd", 2);
    // The description may follow the term on the same line or on the next line.
    assert_xpath(&output, r#"//dt[text()="CPU"]"#, 1);
    assert_xpath(&output, r#"//dd/p[text()="The brain"]"#, 1);
    assert_xpath(&output, r#"//dd/p[text()="Permanent storage."]"#, 1);
}

non_normative!(
    r#"
If a term has an anchor, the anchor must be defined at the start of the same line as the term.

The first term defines which term delimiter is used for the description list.
The terms for the remaining entries at that level must use the same delimiter.

"#
);

// Changing the term delimiter starts a nested description list. Each of the
// alternate delimiters (`:::`, `::::`, `;;`) used under a `::` term nests a
// fresh `div.dlist` inside the description.
#[test]
fn changing_the_delimiter_nests_a_description_list() {
    verifies!(
        r#"
The valid set of term delimiters is fixed.
When the term delimiter is changed, that term begins a new, nested description list (similar to how ordered and unordered lists work).
The available term delimiters you can use for this purpose are as follows:

* `::`
* `:::`
* `::::`
* `;;`

There's no direct correlation between the number of characters in the delimiter and the nesting level.
Each time you change delimiters (selected from this set), it introduces a new level of nesting.
This is how list depth is implied in a language with a left-aligned syntax.
It's customary to use the delimiters in the order shown above to provide a hint that the list is nested at a certain level.

"#
    );

    // A `::` term with no delimiter change stays a single, flat list.
    let flat = convert("Group:: value\n");
    assert_css(&flat, "div.dlist div.dlist", 0);

    // Each alternate delimiter, used under a `::` term, nests a description
    // list inside the description.
    for delimiter in [":::", "::::", ";;"] {
        let output = convert(&format!("Group::\nMember{delimiter} value\n"));
        assert_css(&output, ".dlist:root > dl > dd > div.dlist > dl > dt", 1);
        assert_xpath(&output, r#"//dd//dt[text()="Member"]"#, 1);
    }
}

non_normative!(
    r#"
== Basic description list

Here's an example of a description list that identifies parts of a computer:

"#
);

// A basic description list: the six-term computer-parts example renders six
// `<dt>`/`<dd>` pairs, each description below its term.
#[test]
fn basic_description_list_pairs_each_term_with_its_description() {
    verifies!(
        r#"
----
include::example$description.adoc[tag=base]
----

By default, the content of each item is displayed below the label when rendered.
Here's a preview of how this list is rendered:

.A basic description list
====
include::example$description.adoc[tag=base]
====

"#
    );

    let output = convert(
        "CPU:: The brain of the computer.\n\
         Hard drive:: Permanent storage for operating system and/or user files.\n\
         RAM:: Temporarily stores information the CPU uses during operation.\n\
         Keyboard:: Used to enter text or control items on the screen.\n\
         Mouse:: Used to point to and select items on your computer screen.\n\
         Monitor:: Displays information in visual form using text and graphics.\n",
    );
    assert_css(&output, "div.dlist > dl > dt.hdlist1", 6);
    assert_css(&output, "div.dlist > dl > dd", 6);
    assert_xpath(&output, r#"//dt[text()="CPU"]"#, 1);
    assert_xpath(&output, r#"//dd/p[text()="The brain of the computer."]"#, 1);
}

non_normative!(
    r#"
== Mixing lists

The content of a description list can be any AsciiDoc element.
For instance, we could split up a grocery list by aisle, using description list terms for the aisle names.

"#
);

// A description's content can be any block: here each term's description is an
// unordered list, so each `<dd>` holds a `div.ulist`.
#[test]
fn a_description_can_hold_another_list() {
    verifies!(
        r#"
----
include::example$description.adoc[tag=base-mix]
----

====
include::example$description.adoc[tag=base-mix]
====

"#
    );

    let output = convert("Dairy::\n* Milk\n* Eggs\nBakery::\n* Bread\nProduce::\n* Bananas\n");
    assert_css(&output, "div.dlist > dl > dt.hdlist1", 3);
    assert_css(&output, "div.dlist > dl > dd > div.ulist", 3);
    assert_xpath(
        &output,
        r#"//dt[text()="Dairy"]/following-sibling::dd[1]//li/p[text()="Milk"]"#,
        1,
    );
}

// Whitespace is lenient: spreading the items out with blank lines and indenting
// the nested content produces the same structure as the compact form.
#[test]
fn whitespace_around_items_is_lenient() {
    verifies!(
        r#"
Description lists are quite lenient about whitespace, so you can spread the items out and even indent the content if that makes it more readable for you:

----
include::example$description.adoc[tag=base-mix-alt]
----

"#
    );

    let output = convert(
        "Dairy::\n\n  * Milk\n  * Eggs\n\nBakery::\n\n  * Bread\n\nProduce::\n\n  * Bananas\n",
    );
    assert_css(&output, "div.dlist > dl > dt.hdlist1", 3);
    assert_css(&output, "div.dlist > dl > dd > div.ulist", 3);
}

non_normative!(
    r#"
== Nested description list

"#
);

// A hybrid list mixes all three list types: the `::` term holds a nested `:::`
// description list, whose terms hold ordered lists, whose items hold nested
// unordered lists.
#[test]
fn a_hybrid_list_mixes_all_three_list_types() {
    verifies!(
        r#"
[#three-hybrid]
Finally, you can mix and match the three list types within a single hybrid list.
The AsciiDoc syntax tries hard to infer the relationships between the items that are most intuitive to us humans.

Here's a list that mixes description, ordered, and unordered lists.
Notice how the term delimiter is changed from `::` to `:::` to create a nested description list.

----
include::example$description.adoc[tag=3-mix]
----

Here's how the list is rendered:

.A hybrid list
====
include::example$description.adoc[tag=3-mix]
====

You can include more xref:continuation.adoc[compound content in a list item] as well.
"#
    );

    let output = convert(
        "Operating Systems::\n  Linux:::\n    . Fedora\n      * Desktop\n    . Ubuntu\n      \
         * Desktop\n      * Server\n  BSD:::\n    . FreeBSD\n    . NetBSD\n\n\
         Cloud Providers::\n  PaaS:::\n    . OpenShift\n    . CloudBees\n  \
         IaaS:::\n    . Amazon EC2\n    . Rackspace\n",
    );

    // Two top-level description terms.
    assert_css(&output, ".dlist:root > dl > dt.hdlist1", 2);
    // Each holds a nested (`:::`) description list.
    assert_css(
        &output,
        ".dlist:root > dl > dd > div.dlist > dl > dt.hdlist1",
        4,
    );
    // A `:::` term (Linux) holds an ordered list …
    assert_xpath(
        &output,
        r#"//dt[text()="Linux"]/following-sibling::dd[1]/div[@class="olist arabic"]/ol/li/p[text()="Fedora"]"#,
        1,
    );
    // … whose first item (Fedora) holds a nested unordered list.
    assert_xpath(
        &output,
        r#"//dt[text()="Linux"]/following-sibling::dd[1]/div[@class="olist arabic"]/ol/li[1]/div[@class="ulist"]/ul/li/p[text()="Desktop"]"#,
        1,
    );
}
