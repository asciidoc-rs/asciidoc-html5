//! Coverage of the AsciiDoc language description's *Comments* page.
//!
//! Comments are the one construct on this page with observable behavior: the
//! processor drops them from the parsed document, so they never reach the
//! output. This crate delivers that for every documented form — the line
//! comment, the `////` comment block, the `[comment]` open block, and the
//! `[comment]` paragraph — including when a comment is attached to a list item
//! or sits inside an AsciiDoc table cell. Each of the page's syntax examples is
//! reproduced and driven through `convert` to confirm the comment leaves no
//! trace while the surrounding content still renders. The descriptive prose and
//! the syntax rules themselves are non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/ROOT/pages/comments.adoc");

// The page's framing: what a comment is, why you use one, that the processor
// ignores comments while still counting their lines, and that there are two
// styles (line and block). The concrete drop behavior is verified below.
non_normative!(
    r#"
= Comments

Like programming languages, AsciiDoc provides a way to add commentary to your document that's not carried over into the published document.
This artifact is collectively known as a [.term]*comment*.
Putting text in a comment is often referred to as "`commenting it out`".

A comment is often used to insert a writer-facing notation or to hide draft content that's not ready for publishing.
In general, you can use comments anytime you want to hide lines from the processor.
Comments can also be useful as a processor hint to keep adjacent blocks of content separate.

The AsciiDoc processor will ignore comments during parsing and, thus, will not include them in the parsed document.
It will, however, account for the lines when mapping line numbers back to the source document.

AsciiDoc supports two styles of comments, line and block.
A line comment is for making comments line-by-line (i.e., comment line).
A block comment is for enclosing an arbitrary range of lines as a comment (i.e., comment block).

[#comment-lines]
== Comment lines

A comment line is any line outside of a verbatim block that begins with a double forward slash (`//`) that's not immediately followed by a third forward slash.
Following this prefix, the line may contain an arbitrary number of characters.
It's customary to insert a single space between the prefix and the comment text (e.g., `// line comment`).

"#
);

// A line comment is dropped: the processor ignores the line and continues as
// though it were not there, so the shown source produces no output at all.
#[test]
fn line_comment_is_dropped() {
    verifies!(
        r#"
.Line comment syntax
----
// A single-line comment.
----

When the processor encounters a line comment, it ignores the line and continues processing as though the line is not there.
Line comments are processed as lines are read, so they can be used where paragraph text is not permitted, such as between attribute entries in the document header.

"#
    );

    let output = convert("// A single-line comment.\n");
    assert!(output.trim().is_empty());
}

// A lone line comment between two lists acts as a block boundary: it drops out
// of the parsed document, but not before separating the source into two
// distinct lists rather than one.
#[test]
fn line_comment_separates_adjacent_lists() {
    verifies!(
        r#"
Line comments can be used strategically to separate blocks that otherwise have affinity, such as two adjacent lists.

.Line comment separating two lists
----
* first list

//

* second list
----

In this case, the single line comment effectively acts as an empty paragraph that's dropped from the parsed document.
But before then, it will have served its purpose as a block boundary.

"#
    );

    let output = convert("* first list\n\n//\n\n* second list\n");
    assert_css(&output, "div.ulist", 2);
    assert!(!output.contains("//"));
}

// A comment block is a delimited block bounded by `////` lines. Once read, it
// is dropped from the parsed document, so no AsciiDoc within it is interpreted
// and nothing reaches the output.
#[test]
fn comment_block_is_dropped() {
    verifies!(
        r#"
== Comment blocks

A comment block is a specialized delimited block.
It consists of an arbitrary number of lines bounded on either side by `////` delimiter lines.

A comment block can be used anywhere a delimited block is normal accepted.
The main difference is that once the block is read, it's dropped from the parsed document (effectively ignored).
Additionally, no AsciiDoc syntax within the delimited lines is interpreted, not even preprocessor directives.

.Block comment syntax
----
////
A comment block.

Notice it's a delimited block.
////
----

"#
    );

    let output = convert("////\nA comment block.\n\nNotice it's a delimited block.\n////\n");
    assert!(output.trim().is_empty());
}

// The comment style can also be applied to an open block; it behaves the same
// as a `////` comment block and is dropped from the output.
#[test]
fn comment_open_block_is_dropped() {
    verifies!(
        r#"
A comment block can also be written as an open block with the comment style:

.Alternate block comment syntax
----
[comment]
--
A comment block.

Notice it's a delimited block.
--
----

"#
    );

    let output = convert("[comment]\n--\nA comment block.\n\nNotice it's a delimited block.\n--\n");
    assert!(output.trim().is_empty());
}

// A single paragraph can carry the comment style. Its contiguous lines are
// dropped; a following paragraph, separated by an empty line, is not part of
// the comment and still renders.
#[test]
fn comment_paragraph_is_dropped() {
    verifies!(
        r#"
A comment block that can consists of a single paragraph can be written as a paragraph with the comment style:

.Comment paragraph syntax
----
[comment]
A paragraph comment.
Like all paragraphs, the lines must be contiguous.

Not a comment.
----

"#
    );

    let output = convert(
        "[comment]\nA paragraph comment.\nLike all paragraphs, the lines must be contiguous.\n\nNot a comment.\n",
    );
    assert_css(&output, "div.paragraph", 1);
    assert!(output.contains("Not a comment."));
    assert!(!output.contains("A paragraph comment."));
}

// A comment block attached to a list item (via a list continuation) is dropped,
// leaving the two list items intact.
#[test]
fn comment_block_attached_to_list_item_is_dropped() {
    verifies!(
        r#"
If comment blocks are used in a list item, they must be attached to the list item just like any other block.

.Block comment attached to list item
----
* first item
+
////
A comment block in a list.

Notice it's attached to the preceding list item.
////

* second item
----

"#
    );

    let output = convert(
        "* first item\n+\n////\nA comment block in a list.\n\nNotice it's attached to the preceding list item.\n////\n\n* second item\n",
    );
    assert_css(&output, "div.ulist li", 2);
    assert!(output.contains("first item"));
    assert!(output.contains("second item"));
    assert!(!output.contains("A comment block in a list."));
}

// Inside a table, a comment block may only appear in an AsciiDoc (`a`) cell.
// There it is dropped, leaving the cell's other content behind.
#[test]
fn comment_block_within_a_table_cell_is_dropped() {
    verifies!(
        r#"
Within a table, a comment block can only be used in an AsciiDoc table cell.

.Block comment within a table
----
|===
a|
cell text

////
A comment block in a table.

Notice the cell has the "a" (AsciiDoc) style.
////
|===
----

"#
    );

    let output = convert(
        "|===\na|\ncell text\n\n////\nA comment block in a table.\n\nNotice the cell has the \"a\" (AsciiDoc) style.\n////\n|===\n",
    );
    assert_css(&output, "table td", 1);
    assert!(output.contains("cell text"));
    assert!(!output.contains("A comment block in a table."));
}

// Closing recommendation — a use case, not a rendering rule.
non_normative!(
    r#"
Comment blocks can be very effective for commenting out sections of the document that are not ready for publishing or that provide background material or an outline for the text being drafted.
"#
);
