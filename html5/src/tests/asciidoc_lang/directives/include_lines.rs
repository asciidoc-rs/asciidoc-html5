//! Coverage of the AsciiDoc language description's *Include Content by Line
//! Ranges* page.
//!
//! The `lines` attribute selects portions of the target by line number. Each
//! form the page documents is verified through `convert` against a six-line
//! fixture: a single `start..end` range, multiple ranges separated by commas
//! (which must be quoted) or by semicolons (which need no quotes), the `-1`
//! sentinel for the last line, and an open-ended range that defaults its end to
//! `-1`. The listings the page shows resolve an Antora `example$` reference
//! that is unavailable here, so the behavior is driven with equivalent `lines=`
//! directives.

use super::convert_including;
use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-lines.adoc");

non_normative!(
    r#"
= Include Content by Line Ranges

"#
);

// Multiple ranges are separated by a comma or a semicolon; a comma-separated
// list must be quoted (commas otherwise separate attributes), while a
// semicolon-separated list needs no quotes. Both select the same lines.
#[test]
fn multiple_ranges_use_a_comma_when_quoted_or_a_semicolon_otherwise() {
    verifies!(
        r#"
The include directive supports selecting portions of the document to include.
Using the `lines` attribute, you can include ranges of line numbers.

When including multiple line ranges, each entry in the list must be separated by either a comma or a semicolon.
If commas are used, the entire value must be enclosed in quotes.
Using the semicolon as the data separator eliminates this requirement.

"#
    );

    let comma = convert_including("include::lines.txt[lines=\"1..2,4..5\"]\n");
    let semicolon = convert_including("include::lines.txt[lines=1..2;4..5]\n");

    for out in [&comma, &semicolon] {
        assert!(out.contains("line one"), "{out}");
        assert!(out.contains("line two"), "{out}");
        assert!(out.contains("line four"), "{out}");
        assert!(out.contains("line five"), "{out}");
        assert!(!out.contains("line three"), "{out}");
        assert!(!out.contains("line six"), "{out}");
    }
}

non_normative!(
    r#"
== Specifying line ranges

"#
);

// A single range is a start and an end line number separated by two dots.
#[test]
fn a_single_range_selects_start_through_end() {
    verifies!(
        r#"
To include content by line range, assign a starting line number and an ending line number separated by a pair of dots (e.g., `lines=1..5`) to the `lines` attribute.

----
include::example$include.adoc[tag=line]
----

"#
    );

    let output = convert_including("include::lines.txt[lines=2..3]\n");
    assert!(output.contains("line two"), "{output}");
    assert!(output.contains("line three"), "{output}");
    assert!(!output.contains("line one"), "{output}");
    assert!(!output.contains("line four"), "{output}");
}

// Multiple ranges separated by commas must be quoted.
#[test]
fn comma_separated_ranges_must_be_quoted() {
    verifies!(
        r#"
You can specify multiple ranges by separating each range by a comma.
Since commas are normally used to separate individual attributes, you must quote the comma-separated list of ranges.

----
include::example$include.adoc[tag=m-line-comma]
----

"#
    );

    let output = convert_including("include::lines.txt[lines=\"1..2,5..6\"]\n");
    assert!(output.contains("line one"), "{output}");
    assert!(output.contains("line two"), "{output}");
    assert!(output.contains("line five"), "{output}");
    assert!(output.contains("line six"), "{output}");
    assert!(!output.contains("line three"), "{output}");
    assert!(!output.contains("line four"), "{output}");
}

// Semicolons separate ranges without requiring quotes.
#[test]
fn semicolon_separated_ranges_need_no_quotes() {
    verifies!(
        r#"
To avoid having to quote the list of ranges, you can instead separate them using semicolons.

----
include::example$include.adoc[tag=m-line]
----

"#
    );

    let output = convert_including("include::lines.txt[lines=1..2;5..6]\n");
    assert!(output.contains("line one"), "{output}");
    assert!(output.contains("line two"), "{output}");
    assert!(output.contains("line five"), "{output}");
    assert!(output.contains("line six"), "{output}");
    assert!(!output.contains("line three"), "{output}");
    assert!(!output.contains("line four"), "{output}");
}

// The value `-1` refers to the last line of the target.
#[test]
fn negative_one_refers_to_the_last_line() {
    verifies!(
        r#"
If you don't know the number of lines in the document, or you don't want to couple the range to the length of the file, you can refer to the last line of the document using the value -1.

----
include::example$include.adoc[tag=last]
----

"#
    );

    let output = convert_including("include::lines.txt[lines=5..-1]\n");
    assert!(output.contains("line five"), "{output}");
    assert!(output.contains("line six"), "{output}");
    assert!(!output.contains("line four"), "{output}");
}

// Leaving the end of a range unspecified defaults it to `-1` (the last line).
#[test]
fn an_open_ended_range_defaults_its_end_to_negative_one() {
    verifies!(
        r#"
Alternately, you can leave the end range unspecified and it will default to -1.

----
include::example$include.adoc[tag=endless]
----
"#
    );

    let output = convert_including("include::lines.txt[lines=5..]\n");
    assert!(output.contains("line five"), "{output}");
    assert!(output.contains("line six"), "{output}");
    assert!(!output.contains("line four"), "{output}");
}
