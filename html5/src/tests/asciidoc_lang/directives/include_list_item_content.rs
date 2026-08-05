//! Coverage of the AsciiDoc language description's *Include List Item Content*
//! page.
//!
//! The page documents two techniques for sourcing a list item's content from an
//! include. The simple one — initiate the item with `{empty}`, then follow it
//! with an adjacent `include::` line — supplies the item's principal text. The
//! compound one — attach an open block to a `{empty}` item via a list
//! continuation — gives the item compound content. Both are verified through
//! `convert` and match Asciidoctor.

use super::convert_including;
use crate::tests::{assert_html::assert_css, sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-list-item-content.adoc");

non_normative!(
    r#"
= Include List Item Content

You can use the include directive to include the content of a list item from another file, but there are some things you need to be aware of.

Recall that the `include` directive must be defined on a line by itself.
This presents a challenge with lists since each list item must begin with the list marker.
"#
);

// The simple technique: because an `include::` must sit on its own line, a list
// item's marker is written with the built-in `{empty}` attribute as its
// (empty) principal text, and the very next line is the include directive that
// supplies the item's actual text.
#[test]
fn empty_attribute_lets_an_include_supply_a_list_items_text() {
    verifies!(
        r#"
We can solve this by using the built-in `empty` attribute to initiate the list item, then follow that line with the include directive to bring in the actual content.

Here's an example of how to use the `empty` attribute and the include directive to define a list item, then include the primary text from another file:

----
* {empty}
\include::item-text.adoc[]
----

"#
    );

    let output = convert_including("* {empty}\ninclude::item-text.adoc[]\n");
    assert_css(&output, ".ulist > ul > li", 1);
    assert!(
        output.contains("The primary text of the list item, kept in its own file."),
        "{output}"
    );
}

// The limitation of the simple technique, and the motivation for the compound
// one, is descriptive.
non_normative!(
    r#"
This technique works well if you control the contents of the included file and can ensure it only contains adjacent lines of text.
If a list item does not contain adjacent lines, the list may be terminated.
So we need a bit more syntax.

"#
);

// The compound technique: tuck the include inside an open block and attach it
// to the `{empty}` item with a list continuation (`+`). The item then carries
// an empty principal text plus the open block (and everything the include
// brought into it) as its content — matching Asciidoctor.
#[test]
fn empty_attribute_plus_continuation_attaches_a_compound_open_block() {
    verifies!(
        r#"
If you can't guarantee that all the included lines will be adjacent, you'll want to tuck the include directive inside an open block.
This keeps all the include lines together, enclosed inside the boundaries of the block.
You then attach this block to the list item using a list continuation (i.e., `+`).

Here's an example of how to include compound content from another file into a list item:

----
* {empty}
+
--
\include::complex-list-item.adoc[]
--
----

"#
    );

    let output = convert_including("* {empty}\n+\n--\ninclude::complex-list-item.adoc[]\n--\n");

    // A single list item, with an empty principal text and the open block
    // attached as compound content.
    assert_css(&output, ".ulist > ul > li", 1);
    assert_css(&output, ".ulist > ul > li > .openblock", 1);
    assert_css(&output, ".ulist > ul > li > .openblock .paragraph", 2);
    assert!(
        output.contains("The principal text of the item.")
            && output.contains("A second paragraph of item content."),
        "{output}"
    );
}

// The closing cross-reference is descriptive.
non_normative!(
    r#"
See xref:lists:continuation.adoc#drop-principal-text[dropping the principal text of a list item] for another example of this technique.
"#
);
