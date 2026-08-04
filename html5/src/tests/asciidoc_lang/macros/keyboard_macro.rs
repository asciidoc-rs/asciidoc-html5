//! Coverage of the AsciiDoc language description's *Keyboard Macro* page.
//!
//! The keyboard macro (`kbd:[…]`) is a UI macro, enabled by the `experimental`
//! document attribute. This crate renders it exactly like Asciidoctor 2.0.26: a
//! single key becomes a bare `<kbd>` element, while a key sequence becomes a
//! `<span class="keyseq">` wrapping one `<kbd>` per key. The plus and comma
//! separators, the backslash-plus-space and escaped-`]` edge cases, and the
//! rendered example table are all verified through `convert`; the
//! `experimental`-attribute disclaimer (an included partial) is non-normative.
//!
//! The page draws its rendered example with
//! `include::example$ui.adoc[tag=key]`; the test reproduces that tagged snippet
//! inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/keyboard-macro.adoc");

// The title, the macro's purpose, and the included `experimental`-attribute
// disclaimer. Descriptive prose (the disclaimer's requirement is exercised
// indirectly wherever a UI macro is verified with `:experimental:` set).
non_normative!(
    r#"
= Keyboard Macro

The keyboard macro allows to create a reference to a key or key sequence on a keyboard.
You can use this macro when you need to communicate to a reader what key or key sequence to press to perform a function.

include::partial$ui-macros-disclaimer.adoc[]

"#
);

// A single key renders as a bare `<kbd>`; a key sequence renders as a
// `<span class="keyseq">` with one `<kbd>` per key. Plus and comma both
// separate keys, and an alpha key's case is preserved as entered.
#[test]
fn single_key_and_key_sequence_syntax() {
    verifies!(
        r#"
== Keyboard macro syntax

The keyboard macro uses the short (no target) macro syntax `+kbd:[key(+key)*]+`.
Each key is displayed as entered in the document.
Multiple keys are separated by a plus (e.g., `Ctrl+T`) or a comma (e.g., `Ctrl,T`).
The plus is preferred.

It's customary to represent alpha keys in uppercase, though this is not enforced.

"#
    );

    // A single key is a bare `<kbd>`, no `keyseq` wrapper.
    let single = convert(":experimental:\n\nPress kbd:[F11] now.");
    assert_xpath(&single, r#"//kbd[text()="F11"]"#, 1);
    assert_css(&single, "span.keyseq", 0);

    // A two-key sequence wraps two `<kbd>` in a `keyseq` span.
    let plus = convert(":experimental:\n\nPress kbd:[Ctrl+T].");
    assert_css(&plus, "span.keyseq", 1);
    assert_css(&plus, "span.keyseq kbd", 2);

    // The comma separator yields the same key sequence as the plus.
    let comma = convert(":experimental:\n\nPress kbd:[Ctrl,T].");
    assert_css(&comma, "span.keyseq", 1);
    assert_css(&comma, "span.keyseq kbd", 2);

    // Each key is displayed as entered — a lowercase alpha key stays lowercase.
    let lower = convert(":experimental:\n\nkbd:[t]");
    assert_xpath(&lower, r#"//kbd[text()="t"]"#, 1);
}

// The two escaping edge cases: a trailing backslash key must be followed by a
// space, and a closing-bracket key must be backslash-escaped so the macro does
// not end early.
#[test]
fn backslash_and_closing_bracket_keys() {
    verifies!(
        r#"
If the last key is a backslash (`\`), it must be followed by a space.
Without this space, the processor will not recognize the macro.
If one of the keys is a closing square bracket (`]`), it must be preceded by a backslash.
Without the backslash escape, the macro will end prematurely.
You can find example of these cases in the example below.

"#
    );

    // A lone backslash key (`kbd:[\ ]`) renders a single `<kbd>` holding `\`.
    let backslash = convert(":experimental:\n\nkbd:[\\ ]");
    assert_xpath(&backslash, r#"//kbd[text()="\"]"#, 1);
    assert_css(&backslash, "span.keyseq", 0);

    // An escaped closing bracket (`kbd:[Ctrl+\]]`) keeps the `]` as the second
    // key of the sequence.
    let bracket = convert(":experimental:\n\nkbd:[Ctrl+\\]]");
    assert_css(&bracket, "span.keyseq kbd", 2);
    assert_xpath(&bracket, r#"//kbd[text()="]"]"#, 1);
}

// The rendered example — a shortcut table whose cells hold keyboard macros
// exercising every case above (single key, sequences, the backslash key, and
// the escaped bracket key).
#[test]
fn example_shortcut_table() {
    verifies!(
        r#"
.Using the keyboard macro syntax
[#ex-kbd]
----
include::example$ui.adoc[tag=key]
----

The result of <<ex-kbd>> is displayed below.

[%autowidth]
include::example$ui.adoc[tag=key]
"#
    );

    // The `tag=key` snippet from `example$ui.adoc`, rendered with `:experimental:`
    // set and the `%autowidth` table option from the page.
    let source = ":experimental:\n\n[%autowidth]\n\
        |===\n\
        |Shortcut |Purpose\n\n\
        |kbd:[F11]\n|Toggle fullscreen\n\n\
        |kbd:[Ctrl+T]\n|Open a new tab\n\n\
        |kbd:[Ctrl+Shift+N]\n|New incognito window\n\n\
        |kbd:[\\ ]\n|Used to escape characters\n\n\
        |kbd:[Ctrl+\\]]\n|Jump to keyword\n\n\
        |kbd:[Ctrl + +]\n|Increase zoom\n|===\n";
    let output = convert(source);

    // Eleven `<kbd>` elements across the six shortcut cells: F11 (1),
    // Ctrl+T (2), Ctrl+Shift+N (3), `\` (1), Ctrl+] (2), Ctrl + + (2).
    assert_css(&output, "kbd", 11);

    // Four of those cells are key sequences (two or more keys), each wrapped in
    // a `keyseq` span; the single-key `F11` and `\` cells are bare.
    assert_css(&output, "span.keyseq", 4);

    // The `%autowidth` option renders a fit-content table.
    assert_css(&output, "table.fit-content", 1);
}
