//! Coverage of the AsciiDoc language description's *Troubleshooting Blocks*
//! page.
//!
//! The page documents the rule that a delimited block's opening and closing
//! delimiters must be the same length. This crate honors that rule — a matched
//! `****`/`****` pair renders as a sidebar, while a mismatched pair leaves the
//! rest of the document nested inside the unterminated block and emits an
//! `UnterminatedDelimitedBlock` warning — so both cases are verified (through
//! `convert` and `load().warnings()`). Only the title and section heading are
//! non-normative.

use asciidoc_parser::warnings::WarningType;

use crate::{
    convert, load,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/troubleshoot-blocks.adoc");

non_normative!(
    r#"
= Troubleshooting Blocks

== Opening and closing delimiters

"#
);

// A delimited block whose opening and closing delimiters are the same length is
// valid: a `****`/`****` pair renders as a sidebar.
#[test]
fn matched_delimiters_render_a_sidebar() {
    verifies!(
        r#"
The opening and closing delimiters of a delimited block must be the same length.
For example, a sidebar is specified by an opening delimiter of four asterisks (`+****+`).
Its closing delimiter must also be four asterisks (`+****+`).

Here's a sidebar using valid delimiter lengths:

----
****
This is a valid delimited block.
It will be styled as a sidebar.
****
----

"#
    );

    let output =
        convert("****\nThis is a valid delimited block.\nIt will be styled as a sidebar.\n****\n");
    assert_css(&output, "div.sidebarblock > div.content", 1);
}

// When the delimiters differ in length, the shorter line is not recognized as a
// closing delimiter; the processor treats it as content and runs the block to
// the end of the document, emitting an `UnterminatedDelimitedBlock` warning
// because no matching closing delimiter is ever found.
#[test]
fn mismatched_delimiters_leave_the_block_unterminated_and_warn() {
    verifies!(
        r#"
However, the delimiter lengths in the following delimited block are not equal in length and therefore invalid:

----
********
This is an invalid sidebar block because the delimiter lines are different lengths.
****
----

When an AsciiDoc processor encounters the previous example, it will put the remainder of the content in the document inside the delimited block.
As far as the processor is concerned, the closing delimiter is just a line of content.
However, the processor will issue a warning if a matching closing delimiter is never found.

"#
    );

    let source =
        "********\nThis is an invalid sidebar block because the delimiter lines are different lengths.\n****\n";
    let output = convert(source);

    // The shorter `****` is treated as an opening delimiter rather than a close,
    // so the remainder nests inside the outer (unterminated) sidebar.
    assert_css(&output, "div.sidebarblock div.sidebarblock", 1);

    let doc = load(source);
    assert!(
        doc.warnings()
            .any(|w| w.warning == WarningType::UnterminatedDelimitedBlock),
        "an unterminated delimited block should emit a warning",
    );
}

// A concluding restatement of the same-length rule, already verified above.
non_normative!(
    r#"
If you want the processor to recognize a closing delimiter, it must be the same length as the opening delimiter.
"#
);
