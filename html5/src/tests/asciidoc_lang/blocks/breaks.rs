//! Coverage of the AsciiDoc language description's *Breaks* page.
//!
//! The page itself is only an aggregator: it includes the *Thematic breaks* and
//! *Page breaks* partials. Those partials are not measured as spec sources
//! (only `pages/` content is), so this file verifies the two behaviors they
//! document against `convert` — a thematic break (`'''`) renders as an `<hr>`,
//! and a page break (`<<<`) renders as a `page-break-after` div — and attaches
//! each `include::` line to the behavior it pulls in. Only the page title is
//! non-normative.

use crate::{
    convert,
    tests::{assert_html::assert_xpath, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/breaks.adoc");

non_normative!(
    r#"
= Breaks

"#
);

// A thematic break — a line of three single quotation marks (`'''`), offset
// from the surrounding content by blank lines — renders as an HTML `<hr>`.
#[test]
fn thematic_break_renders_as_hr() {
    verifies!(
        r#"
include::partial$thematic-breaks.adoc[]

"#
    );

    let output = convert("Before.\n\n'''\n\nAfter.\n");
    assert_xpath(&output, "//hr", 1);
}

// A page break — a line of three less-than characters (`<<<`) — renders as a
// `page-break-after: always` div, a hint the converter emits for page-oriented
// output.
#[test]
fn page_break_renders_as_page_break_div() {
    verifies!(
        r#"
include::partial$page-breaks.adoc[]
"#
    );

    let output = convert("Before.\n\n<<<\n\nAfter.\n");
    assert_xpath(&output, r#"//div[@style="page-break-after: always;"]"#, 1);
}
