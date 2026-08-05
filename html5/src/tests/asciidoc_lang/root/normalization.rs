//! Coverage of the AsciiDoc language description's *Normalization* page.
//!
//! The page describes a preprocessor step: before the document is parsed, each
//! line is forced to UTF-8 and stripped of trailing whitespace, and the lines
//! are later rejoined on `\n`. That normalization belongs to `asciidoc-parser`,
//! which reads the source before this crate ever sees a parsed document — this
//! crate has no separate normalization stage of its own to exercise. The page
//! specifies no rendering behavior, so it is tracked as non-normative here; the
//! trailing-space and encoding handling it describes is verified in the parser
//! crate.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/ROOT/pages/normalization.adoc");

// The entire page describes the preprocessor's line-normalization step, which
// runs inside `asciidoc-parser` ahead of parsing. There is no rendering
// contract for this crate to verify.
non_normative!(
    r#"
= Normalization

When an AsciiDoc processor reads the AsciiDoc source, the first thing it does is normalize the lines.
(This operation can be performed up front or as each line is visited).

Normalization consists of the following operations:

* Force the encoding to UTF-8 (An AsciiDoc processor always assumes the content is UTF-8 encoded)
* Strip trailing spaces from each line (including any end of line character)

This normalization is performed independent of any structured context.
It doesn't matter if the line is part of a literal block or a regular paragraph. All lines get normalized.

Normalization is only applied in certain cases to the lines of an include file.
Only include files that have a recognized AsciiDoc extension are normalized as described above.
For all other files, only the trailing end of line character is removed.
Include files can also have a different encoding, which is specified using the encoding attribute.
If the encoding attribute is not specified, UTF-8 is assumed.

When the AsciiDoc processor brings the lines back together to produce the rendered document (HTML, DocBook, etc), it joins the lines on the line feed character (`\n`).
"#
);
