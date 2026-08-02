//! Coverage of the AsciiDoc language description's *Wrap Attribute Entry
//! Values* page.
//!
//! A long attribute value can be split across lines. A plain line continuation
//! (space + `\`) soft-wraps — the newline and following indentation fold into a
//! single space — while a hard line break replacement (space + `+`) placed in
//! front of the continuation preserves the newline. Both behaviors are observed
//! by setting the attribute the page shows, referencing it, and inspecting the
//! rendered value. The page title is tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/wrap-values.adoc");

/// Returns the rendered content of the first paragraph in `output`.
fn first_paragraph(output: &str) -> &str {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner)
        .unwrap_or_default()
}

non_normative!(
    r#"
= Wrap Attribute Entry Values

"#
);

mod soft_wrap {
    use super::*;
    use crate::convert;

    #[test]
    fn soft_wrap_folds_into_a_single_space() {
        verifies!(
            r#"
== Soft wrap attribute values

If the value of a document attribute is too long to fit on the screen, you can split the value across multiple lines with a line continuation to make it easier to read.

A [.term]*line continuation* consists of a space followed by a backslash character (`\`) at the end of the line.
The line continuation must be placed on every line of a multiline value except for the last line.
Lines that follow a line continuation character may be indented, but that indentation will not be included in the value.

When the processor reads the attribute value, it folds the line continuation, the newline, and any ensuing indentation into a single space.
In this case, we can say that the attribute value has soft wraps.

Let's assume we want to define an attribute named `description` that has a very long value.
We can split this attribute up across multiple lines by placing a line continuation at the end of each line of the value except for the last.

.A multiline attribute value with soft wraps
[source]
----
:description: If you have a very long line of text \
that you need to substitute regularly in a document, \
you may find it easier to split the value neatly in the header \
so it remains readable to folks looking at the AsciiDoc source.
----

If the line continuation is missing, the processor will assume it has found the end of the value and will not include subsequent lines in the value of the attribute.

"#
        );

        let output = convert(
            ":description: If you have a very long line of text \\\nthat you need to substitute regularly in a document, \\\nyou may find it easier to split the value neatly in the header \\\nso it remains readable to folks looking at the AsciiDoc source.\n\n{description}",
        );

        // The line continuations and following newlines fold into single spaces.
        assert_eq!(
            first_paragraph(&output),
            "If you have a very long line of text that you need to substitute regularly in a document, you may find it easier to split the value neatly in the header so it remains readable to folks looking at the AsciiDoc source.",
        );
    }
}

mod hard_wrap {
    use super::*;
    use crate::convert;

    #[test]
    fn hard_wrap_preserves_line_breaks() {
        verifies!(
            r#"
[#hard]
== Hard wrap attribute values

You can force an attribute value to hard wrap by inserting a hard line break replacement in front of the line continuation.
A hard line break replace is a space followed by a plus character (`+`).

As described in the previous section, the line continuation, newline, and ensuing indentation is normally replaced with a space.
This would prevent the hard line break replacement from being recognized.
However, the processor accounts for this scenario and leaves the newline intact.

Let's assume we want to define an attribute named `haiku` that requires hard line breaks.
We can split this attribute up across multiple lines and preserve those line breaks by placing a hard line break replacement followed by a line continuation at the end of each line of the value except for the last.

.A multiline attribute value with hard wraps
[source]
----
:haiku: Write your docs in text, + \
AsciiDoc makes it easy, + \
Now get back to work!
----

This syntax ensures that the newlines are preserved in the output as hard line breaks.
"#
        );

        let output = convert(
            ":haiku: Write your docs in text, + \\\nAsciiDoc makes it easy, + \\\nNow get back to work!\n\n{haiku}",
        );

        // Each `+ \` keeps the newline, which renders as a hard line break.
        assert_eq!(
            first_paragraph(&output),
            "Write your docs in text,<br>\nAsciiDoc makes it easy,<br>\nNow get back to work!",
        );
    }
}
