//! Coverage of the AsciiDoc language description's *Button and Menu UI Macros*
//! page.
//!
//! The `btn:[…]` and `menu:Menu[items]` UI macros are enabled by the
//! `experimental` document attribute and render exactly as Asciidoctor 2.0.26
//! does: a button becomes `<b class="button">`, and a menu path becomes a
//! `<span class="menuseq">` with `menu`/`submenu`/`menuitem`/`caret` parts.
//! Both are verified through `convert`. The *shorthand* menu syntax (a quoted
//! `"a > b > c"`) is deliberately unimplemented in `asciidoc-parser` (it is not
//! on a standards track), so the material describing it is non-normative.
//!
//! The page draws its rendered examples with `include::example$ui.adoc[tag=…]`;
//! the tests reproduce those tagged snippets inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/ui-macros.adoc");

// The title and the included `experimental`-attribute disclaimer. Descriptive
// prose (the requirement is exercised wherever a UI macro is verified below
// with `:experimental:` set).
non_normative!(
    r#"
= Button and Menu UI Macros

include::partial$ui-macros-disclaimer.adoc[]

"#
);

// The button-macro heading and its motivating prose. Descriptive.
non_normative!(
    r#"
== Button macro syntax

It can be difficult to communicate to the reader that they need to press a button.
They can't tell if you are saying "`OK`" or they are supposed to look for a button labeled *OK*.
It's all about getting the semantics right.
The `btn` macro to the rescue!

"#
);

// The button example renders each `btn:[label]` as `<b
// class="button">label</b>`.
#[test]
fn button_example() {
    verifies!(
        r#"
.Using the button macro syntax
[#ex-btn]
----
include::example$ui.adoc[tag=button]
----

The result of <<ex-btn>> is displayed below.

====
include::example$ui.adoc[tag=button]
====

"#
    );

    // The `tag=button` snippet from `example$ui.adoc`, rendered in an example
    // block with `:experimental:` set (as the UI macros require).
    let output = convert(
        ":experimental:\n\n\
         ====\n\
         Press the btn:[OK] button when you are finished.\n\n\
         Select a file in the file navigator and click btn:[Open].\n\
         ====",
    );

    assert_css(&output, "div.exampleblock b.button", 2);
    assert_xpath(&output, r#"//b[@class="button"][text()="OK"]"#, 1);
    assert_xpath(&output, r#"//b[@class="button"][text()="Open"]"#, 1);
}

// The menu-macro heading. Descriptive.
non_normative!(
    r#"
== Menu macro syntax

"#
);

// The menu example renders a `menu:Menu[items]` path as a `<span
// class="menuseq">` — a `menu` label, then `submenu`/`menuitem` parts joined by
// `caret` separators.
#[test]
fn menu_example() {
    verifies!(
        r#"
Trying to explain how to select a menu item can be a pain.
With the `menu` macro, the symbols do the work.

.Using the menu macro syntax
[#ex-menu]
----
include::example$ui.adoc[tag=menu]
----

The instructions in <<ex-menu>> appear below.

====
include::example$ui.adoc[tag=menu]
====

"#
    );

    // The `tag=menu` snippet: a bare menu item (`menu:File[Save]`) and a submenu
    // path (`menu:View[Zoom > Reset]`).
    let output = convert(
        ":experimental:\n\n\
         ====\n\
         To save the file, select menu:File[Save].\n\n\
         Select menu:View[Zoom > Reset] to reset the zoom level to the default setting.\n\
         ====",
    );

    assert_css(&output, "span.menuseq", 2);
    // The first path is a single item (menu + menuitem, no submenu).
    assert_xpath(
        &output,
        r#"//span[@class="menuseq"]/b[@class="menu"][text()="File"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@class="menuseq"]/b[@class="menuitem"][text()="Save"]"#,
        1,
    );
    // The second path has an intermediate submenu.
    assert_xpath(&output, r#"//b[@class="submenu"][text()="Zoom"]"#, 1);
    assert_xpath(&output, r#"//b[@class="menuitem"][text()="Reset"]"#, 1);
}

// The shorthand menu syntax (a quoted `"a > b > c"`). `asciidoc-parser` does
// not implement it and does not plan to — per the AsciiDoc language docs the
// shorthand is not on a standards track — so this material is non-normative:
// covered for completeness, with no behavior asserted.
non_normative!(
    r#"
If the menu has more than one item, it can be expressed using a shorthand.

IMPORTANT: The shorthand syntax for menu is not on a standards track.
You can use it for transient documents, but do not rely on it long term.

In the shorthand syntax:

* each item is separated by a greater than sign (`>`) with spaces on either side
* the whole expression must be enclosed in double quotes (`"`)

The text of the item itself may contain spaces.

.Using the shorthand menu syntax
[#ex-menu-short]
----
include::example$ui.adoc[tag=menu-short]
----

The shorthand syntax can be escaped by preceding the opening double quote with a backslash character.

"#
);

// The first menu item must begin with a word character or an ampersand (for a
// character reference); otherwise the text is not recognized as a menu macro.
#[test]
fn first_menu_item_must_start_with_word_char_or_ampersand() {
    verifies!(
        r#"
Both the menu macro and menu shorthand require the first menu item start with a word character (alphanumeric character or underscore) or ampersand (to accommodate a character reference).
If you need the first menu item to start with a non-word character, you will need to substitute it with the equivalent character reference.
For example, to make a menu item that starts with vertical ellipsis, you must use `\&#8942;`.

"#
    );

    // The menu name may begin with an ampersand-introduced character reference
    // (here, a vertical ellipsis).
    let charref = convert(":experimental:\n\nmenu:&#8942;[More Tools, Extensions]");
    assert_css(&charref, "span.menuseq", 1);
    assert_xpath(&charref, r#"//span[@class="menuseq"]/b[@class="menu"]"#, 1);

    // A menu name that begins with a non-word, non-ampersand character is not
    // recognized as a menu macro, so it is emitted as literal text.
    let literal = convert(":experimental:\n\nmenu:.hidden[Item]");
    assert_css(&literal, "span.menuseq", 0);
    assert_xpath(&literal, r#"//p[text()="menu:.hidden[Item]"]"#, 1);
}

// Subsequent menu items may start with any character, unlike the first.
#[test]
fn subsequent_menu_items_may_start_with_any_character() {
    verifies!(
        r#"
.Using a character reference at the start of the menu
[#ex-menu-short-charref]
----
include::example$ui.adoc[tag=menu-charref]
----

Subsequent menu items don't have this requirement and thus can start with any character.
"#
    );

    // The example above (`example$ui.adoc[tag=menu-charref]`) uses the
    // *shorthand* menu syntax, which this crate does not implement (see the note
    // on the shorthand block above), so it stays literal text rather than a menu.
    // Starting the menu with a character reference is verified for the macro form
    // in `first_menu_item_must_start_with_word_char_or_ampersand` above.
    let shorthand = convert(
        ":experimental:\n\nSelect \"&#8942; > More Tools > Extensions\" to find and enable extensions.",
    );
    assert_css(&shorthand, "span.menuseq", 0);

    // Unlike the first menu item, subsequent items (submenus and the final item)
    // may start with any character, including a non-word character.
    let output = convert(":experimental:\n\nmenu:View[.zoom > .reset]");
    assert_css(&output, "span.menuseq", 1);
    assert_xpath(&output, r#"//b[@class="submenu"][text()=".zoom"]"#, 1);
    assert_xpath(&output, r#"//b[@class="menuitem"][text()=".reset"]"#, 1);
}
