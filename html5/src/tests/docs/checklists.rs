//! Coverage of this crate's *Checklists* documentation page.
//!
//! The page shows how `asciidoc-html5` renders checklists: the default form
//! (Unicode glyphs) and the `%interactive` form (HTML checkboxes with
//! `data-item-complete`). Each shown source-and-output pair is verified by
//! converting the source through `convert`; the surrounding prose is tracked as
//! non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("docs/modules/ROOT/pages/checklists.adoc");

non_normative!(
    r#"
= Checklists
:navtitle: Checklists
:description: How asciidoc-html5 renders checklists, including interactive checkboxes.

A checklist is an unordered list whose items begin with `[*]` or `[x]` (checked)
or `[ ]` (unchecked). `asciidoc-html5` matches Asciidoctor's `html5` backend: a
plain checklist renders each mark as a Unicode glyph, and the `interactive`
option renders a real HTML checkbox.

[NOTE]
====
The prose on this page is non-normative documentation. The AsciiDoc conversions
it shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Make a checklist

"#
);

// The default checklist: the wrapper gains the `checklist` class and each mark
// renders as a Unicode glyph in place of the bullet.
#[test]
fn a_default_checklist() {
    verifies!(
        r#"
Begin each item with a mark. `[*]` and `[x]` are checked; `[ ]` is unchecked:

[,asciidoc]
----
* [*] Done
* [ ] Todo
----

The wrapper gains the `checklist` class, and each mark renders as a Unicode glyph
(`&#10003;` for checked, `&#10063;` for unchecked) in place of the bullet:

[,html]
----
<div class="ulist checklist">
<ul class="checklist">
<li>
<p>&#10003; Done</p>
</li>
<li>
<p>&#10063; Todo</p>
</li>
</ul>
</div>
----

"#
    );

    let output = convert("* [*] Done\n* [ ] Todo\n");
    assert_css(&output, "div.ulist.checklist > ul.checklist > li", 2);
    assert_css(&output, "input", 0);
    assert_xpath(&output, r#"//li/p[starts-with(text(), "✓")]"#, 1);
    assert_xpath(&output, r#"//li/p[starts-with(text(), "❏")]"#, 1);
}

non_normative!(
    r#"
== Interactive checkboxes

"#
);

// The `%interactive` option renders HTML checkboxes carrying
// `data-item-complete` (and `checked` for a checked item).
#[test]
fn interactive_checkboxes() {
    verifies!(
        r#"
Add the `%interactive` option to render real HTML checkboxes instead of glyphs.
Each becomes an `<input type="checkbox">` carrying `data-item-complete` (`1` when
checked, `0` when not) and, for a checked item, the `checked` attribute:

[,asciidoc]
----
[%interactive]
* [*] Done
* [ ] Todo
----

[,html]
----
<div class="ulist checklist">
<ul class="checklist">
<li>
<p><input type="checkbox" data-item-complete="1" checked> Done</p>
</li>
<li>
<p><input type="checkbox" data-item-complete="0"> Todo</p>
</li>
</ul>
</div>
----

"#
    );

    let output = convert("[%interactive]\n* [*] Done\n* [ ] Todo\n");
    assert_css(
        &output,
        r#"li > p > input[type="checkbox"][data-item-complete="1"][checked]"#,
        1,
    );
    assert_css(
        &output,
        r#"li > p > input[type="checkbox"][data-item-complete="0"]"#,
        1,
    );
}

non_normative!(
    r#"
== Known limitations

Checklists are fully supported: the default Unicode-glyph form and the
`%interactive` checkbox form both match Asciidoctor 2.0.26's output. Font-based
checkbox icons are not rendered. For plain unordered lists, see
xref:unordered-lists.adoc[].
"#
);
