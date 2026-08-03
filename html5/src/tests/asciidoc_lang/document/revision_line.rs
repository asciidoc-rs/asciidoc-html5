//! Coverage of the AsciiDoc language description's *Using the Revision Line*
//! page.
//!
//! The revision line, directly below the author line, sets the `revnumber`,
//! `revdate`, and `revremark` attributes. Matching Asciidoctor 2.0.26, this
//! crate renders them in the byline: `<span id="revnumber">` (prefixed by the
//! version label and with any leading letters stripped from the number), `<span
//! id="revdate">`, and `<span id="revremark">`. That behavior is verified
//! through `convert`; the surrounding prose, the syntax bullets, and the
//! `docdate`-dependent second example are non-normative.

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoc-lang/docs/modules/document/pages/revision-line.adoc");

// Renders a standalone document so the header byline is present in the output.
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().standalone(true))
}

// The document title, the definition of the revision line, the detection rules,
// and the syntax bullets. Descriptive prose and setup.
non_normative!(
    r#"
= Using the Revision Line

The revision attributes can be set and assigned values using the revision line.

[#revision-line]
== What's the revision line?

The [.term]*revision line* is the line directly after the author line in the document header.
When the content on this line is structured correctly, the processor assigns the content to the built-in `revnumber`, `revdate` and `revremark` attributes.

== When can I use the revision line?

In order for the processor to properly detect the revision line and assign the content to the correct attributes, all of the following criteria must be met:

. The document header must contain a xref:title.adoc[document title] and an author line.
. The revision information must be entered on the line directly beneath the xref:author-line.adoc[author line].
. The revision line must start with the revision number.
. The revision number must contain at least one number, but a number doesn't have to be the first character in the version.
. The values in the revision line must be placed in a specific order and separated with the correct syntax.

.Revision line structure
[source]
----
= Document Title
author <email>
revision number, revision date: revision remark
----

When using the revision line, the revision date and remark are optional.

* `pass:q[#v#]7.5` When the revision line only contains a revision number, prefix the number with a `v`.
* `7.5pass:q[#,#] 1-29-2020` When the revision line contains a version and a date, separate the version number from the date with a comma (`,`).
A `v` prefix before the version number is optional.
* `7.5pass:q[#:#] A new analysis` When the revision line contains a version and a remark, separate the version number from the remark with a colon (`:`).
A `v` prefix before the version number is optional.
* `7.5pass:q[#,#] 1-29-2020pass:q[#:#] A new analysis` When the revision line contains a version, date, and a remark, separate the version number from the date with a comma (`,`) and separate the date from the remark with a colon (`:`).
A `v` prefix before the version number is optional.

== Assign revision information using the revision line

"#
);

// A revision line carrying a number, date, and remark populates all three
// byline spans: the number (with the version label prefix), the date, and the
// remark.
#[test]
fn revision_line_populates_the_byline() {
    verifies!(
        r#"
The revision line in <<ex-line>> contains a revision number, date, and remark.

.Revision line with a version, date and remark
[source#ex-line]
----
= The Intrepid Chronicles
Kismet Lee <.>
2.9, October 31, 2021: Fall incarnation <.> <.> <.>
----
<.> The author line must be directly above the revision line.
<.> The revision line must begin with the revision number.
<.> The date is separated from the version by a comma (`,`).
The date can contain letters, numbers, symbols, and attribute references.
<.> The remark is separated from the date by a colon (`:`).

When the default stylesheet is applied, the revision information is displayed on the same line as the author information.
Note that the revision number is preceded with the word _Version_.
This label is automatically added by the processor.
It can be xref:version-label.adoc[changed or turned off with the version-label attribute].

"#
    );

    let source = "\
= The Intrepid Chronicles
Kismet Lee
2.9, October 31, 2021: Fall incarnation

Body.
";

    let output = convert(source);
    assert_xpath(
        &output,
        r#"//span[@id="revnumber"][text()="version 2.9,"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@id="revdate"][text()="October 31, 2021"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"//span[@id="revremark"][text()="Fall incarnation"]"#,
        1,
    );
}

// The first screenshot, then a second example whose date is a reference to the
// environment-derived `docdate` attribute (so its rendered byline is not
// reproducible in a test) and whose remark carries a Unicode glyph. The
// "leading letters are dropped from the version" rule it illustrates is
// verified through `revnumber` references on the revision-attribute pages.
non_normative!(
    r#"
image::revision-line.png["Byline with a version number, revision date, and revision remark",role=screenshot]

Let's look at another revision line.
In <<ex-prefix>>, the version starts with a letter, the date is a reference to the attribute `docdate`, and there's a Unicode glyph in the remark.

.Revision line with a version prefix, attribute reference and Unicode glyph
[source#ex-prefix]
----
include::example$revision-line-with-version-prefix.adoc[]
----

The result of <<ex-prefix>> is displayed below.

image::revision-line-with-version-prefix.png["Byline with the date derived from docdate and a remark with a Unicode glyph",role=screenshot]

_LPR_ was removed from the version because any letters or symbols that precede the revision number in the revision line are dropped.
To display the letters or symbols in front of a revision number, xref:revision-attribute-entries.adoc[set revnumber using an attribute entry].
"#
);
