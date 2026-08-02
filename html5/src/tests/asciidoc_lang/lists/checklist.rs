//! Coverage of the AsciiDoc language description's *Checklists* page.
//!
//! A checklist is an unordered list whose items begin with `[*]`/`[x]`
//! (checked) or `[ ]` (unchecked). This crate matches Asciidoctor 2.0.26: a
//! plain checklist renders each mark as a Unicode glyph (`&#10003;` /
//! `&#10063;`) in place of the bullet, and the `interactive` option renders an
//! HTML `<input type="checkbox">` carrying `data-item-complete`. The page's two
//! examples and the verifiable claims about them are checked through `convert`.
//!
//! The page also states, as general behavior, that checklist marks always
//! become HTML checkboxes and that the checkbox is rendered `disabled`. Neither
//! holds for the parity oracle (Asciidoctor 2.0.26), whose *default* checklist
//! uses a Unicode glyph and whose *interactive* checkbox carries no `disabled`
//! attribute, so those lines are tracked as non-normative with the reason
//! noted inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/checklist.adoc");

non_normative!(
    r#"
= Checklists

"#
);

// A checklist is an unordered list (`div.ulist.checklist`) whose items open
// with `[*]`/`[x]` (checked) or `[ ]` (unchecked). Matching Asciidoctor 2.0.26,
// a plain (non-interactive) checklist renders the checked mark as `&#10003;`
// (✓) and the unchecked mark as `&#10063;` (❏) in place of the bullet; a plain
// list item without a mark is left untouched.
#[test]
fn checklist_marks_items_checked_or_unchecked() {
    verifies!(
        r#"
List items can be marked complete using checklists.

Checklists (i.e., task lists) are unordered lists that have items marked as checked (`[*]` or `[x]`) or unchecked (`[ ]`).
Here's an example:

.Checklist syntax
[#ex-syntax]
----
include::example$checklist.adoc[tag=check]
----

The result of <<ex-syntax>> is displayed below.

include::example$checklist.adoc[tag=check]

TIP: Not all items in the list have to be checklist items, as <<ex-syntax>> shows.

"#
    );

    let output =
        convert("* [*] checked\n* [x] also checked\n* [ ] not checked\n* normal list item\n");

    assert_css(&output, "div.ulist.checklist > ul.checklist > li", 4);

    // Both `[*]` and `[x]` render the checked glyph (`&#10003;`, ✓); `[ ]`
    // renders the unchecked glyph (`&#10063;`, ❏).
    assert_xpath(&output, r#"//li/p[starts-with(text(), "✓")]"#, 2);
    assert_xpath(&output, r#"//li/p[starts-with(text(), "❏")]"#, 1);

    // The plain item carries no mark, confirming not every item must be a
    // checklist item.
    assert_xpath(&output, r#"//li/p[text()="normal list item"]"#, 1);
}

// The blanket statement that a checklist mark is converted to an HTML checkbox
// describes the AsciiDoc spec behavior. The parity oracle (Asciidoctor 2.0.26)
// only emits an `<input>` when the `interactive` option is set; its default
// checklist uses the Unicode glyph verified above. Tracked as non-normative.
non_normative!(
    r#"
When checklists are converted to HTML, the checkbox markup is transformed into an HTML checkbox with the appropriate checked state.
"#
);

// The `interactive` option makes each item an HTML `<input type="checkbox">`
// used in place of the bullet, carrying `data-item-complete="1"` (and the
// `checked` attribute) for a checked item and `data-item-complete="0"` for an
// unchecked one.
#[test]
fn interactive_checkbox_carries_data_item_complete() {
    verifies!(
        r#"
The `data-item-complete` attribute on the checkbox is set to 1 if the item is checked, 0 if not.
The checkbox is used in place of the item's bullet.

"#
    );

    let output = convert(
        "[%interactive]\n* [*] checked\n* [x] also checked\n* [ ] not checked\n* normal list item\n",
    );

    assert_css(
        &output,
        r#"li > p > input[type="checkbox"][data-item-complete="1"][checked]"#,
        2,
    );
    assert_css(
        &output,
        r#"li > p > input[type="checkbox"][data-item-complete="0"]"#,
        1,
    );
}

// Asciidoctor 2.0.26 renders neither a `disabled` attribute on the interactive
// checkbox nor an `<input>` at all for the default checklist (it uses the
// Unicode glyph), so this claim about a disabled checkbox does not hold for the
// parity oracle. Tracked as non-normative.
non_normative!(
    r#"
Since HTML generated from AsciiDoc is typically static, the checkbox is set as disabled to make it appear as a simple mark.
"#
);

// The `interactive` option (shown with the `%` shorthand) turns the whole
// checklist into HTML checkboxes; a plain item still renders without one.
#[test]
fn interactive_option_renders_input_checkboxes() {
    verifies!(
        r#"
If you want to make the checkbox interactive (i.e., clickable), add the `interactive` option to the checklist (shown here using the shorthand syntax for the xref:attributes:options.adoc[]):

.Checklist with interactive checkboxes
[#ex-interactive]
----
include::example$checklist.adoc[tag=check-int]
----

The result of <<ex-interactive>> is displayed below.

include::example$checklist.adoc[tag=check-int]

"#
    );

    let output = convert(
        "[%interactive]\n* [*] checked\n* [x] also checked\n* [ ] not checked\n* normal list item\n",
    );

    assert_css(&output, "div.ulist.checklist > ul.checklist > li", 4);
    assert_css(&output, r#"li > p > input[type="checkbox"]"#, 3);
    // The plain list item has no checkbox.
    assert_xpath(&output, r#"//li/p[text()="normal list item"]"#, 1);
}

non_normative!(
    r#"
////
This example doesn't seem quite right since nothing about it indicates font based icons.

As a bonus, if you enable font-based icons, the checkbox markup (in non-interactive lists) is transformed into a font-based icon!

.Checklist with font-based checkboxes
[source]
----
include::{partialsdir}/ex-ulist.adoc[tag=check-icon]
----
////
"#
);
