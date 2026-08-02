//! Coverage of the AsciiDoc language description's *Offset Section Levels*
//! page.
//!
//! The `leveloffset` attribute pushes an included document's headings down by
//! the given number of levels, so a standalone chapter (whose title is a
//! level-0 heading) folds into a primary document as a section. All three forms
//! the page shows are verified through `convert` against a chapter fixture:
//! `leveloffset` on the include directive, the `:leveloffset:` document
//! attribute set around a run of includes (and reset afterward), and an
//! absolute `:leveloffset:` value. The introduction, the empty-line advice, and
//! the hidden author note are non-normative.

use super::convert_including;
use crate::tests::{assert_html::assert_css, sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-with-leveloffset.adoc");

non_normative!(
    r#"
= Offset Section Levels
//Partitioning Large Documents and using leveloffset
// [#include-partitioning]

When your document gets large, you can split it up into subdocuments for easier editing.

----
= My book

\include::chapter01.adoc[]

\include::chapter02.adoc[]

\include::chapter03.adoc[]
----

TIP: Note the empty lines before and after the include directives.
This practice is recommended whenever including AsciiDoc content to avoid unexpected results (e.g., a section title getting interpreted as a line at the end of a previous paragraph).

== Manipulate heading levels with leveloffset

"#
);

// `leveloffset=+1` on the include directive pushes the included chapter's
// headings down one level: its level-0 document title becomes a level-1 section
// (an `<h2>`), and a level-1 heading within it becomes level 2 (an `<h3>`).
#[test]
fn leveloffset_on_the_directive_pushes_headings_down() {
    verifies!(
        r#"
The `leveloffset` attribute can help here by pushing all headings in the included document down by the specified number of levels.
This allows you to publish each chapter as a standalone document (complete with a document title), but still be able to include the chapters into a primary document (which has its own document title).

You can easily assemble your book so that the chapter document titles become level 1 headings using:

----
= My Book

\include::chapter01.adoc[leveloffset=+1]

\include::chapter02.adoc[leveloffset=+1]

\include::chapter03.adoc[leveloffset=+1]
----

Because the leveloffset is _relative_ (it begins with + or -), this works even if the included document has its own includes and leveloffsets.

"#
    );

    let output = convert_including("= My Book\n\ninclude::chapter.adoc[leveloffset=+1]\n");
    assert_css(&output, "h2#_chapter_one", 1);
    assert_css(&output, "h3#_a_sub_heading", 1);
}

// Setting `:leveloffset:` as a document attribute applies the offset to every
// following include; the closing `:leveloffset: -1` returns the offset to 0, so
// a section after the run sits at its natural level again.
#[test]
fn leveloffset_attribute_applies_around_a_run_of_includes() {
    verifies!(
        r#"
If you have lots of chapters to include and want them all to have the same offset, you can save some typing by setting `leveloffset` around the includes:

----
= My book

:leveloffset: +1

\include::chapter01.adoc[]

\include::chapter02.adoc[]

\include::chapter03.adoc[]

:leveloffset: -1
----

The final line returns the level offset to 0.

"#
    );

    let output = convert_including(
        "= My Book\n\n:leveloffset: +1\n\ninclude::chapter.adoc[]\n\n:leveloffset: -1\n\n== After\n\ntext\n",
    );

    // The offset applied inside the run ...
    assert_css(&output, "h2#_chapter_one", 1);
    assert_css(&output, "h3#_a_sub_heading", 1);

    // ... and the reset returned it to 0, so the trailing level-1 section is an
    // `<h2>` (it would be an `<h3>` had the offset still been in force).
    assert_css(&output, "h2#_after", 1);
}

// An absolute `:leveloffset:` value (no leading `+`/`-`) sets the offset
// outright, so `:leveloffset: 1` pushes the chapter title to a level-1 section.
#[test]
fn absolute_leveloffset_sets_the_offset_outright() {
    verifies!(
        r#"
Alternatively, you could use absolute levels:

----
:leveloffset: 1

//includes

:leveloffset: 0
----

"#
    );

    let output = convert_including("= My Book\n\n:leveloffset: 1\n\ninclude::chapter.adoc[]\n");
    assert_css(&output, "h2#_chapter_one", 1);
    assert_css(&output, "h3#_a_sub_heading", 1);
}

// Non-normative: the recommendation to prefer relative levels, and a hidden
// author note (inside an AsciiDoc `////` comment block) reiterating the
// empty-line-around-includes advice.
non_normative!(
    r#"
Relative levels are preferred.
Absolute levels become awkward when you have nested includes since they aren't context aware.

////
That's also why it's important to surround the include directive by empty lines if it imports in a discrete structure.

You only want to place include files directly adjacent to one another if the imported content should be directly adjacent.

IMPORTANT: Take note of the empty lines between the include directives.
The empty line between include directives prevents the first and last lines of the included files from being adjoined.
This practice is *strongly* encouraged when combining document parts.
If you don't include these empty lines, you might find that the AsciiDoc processor swallows section titles.
This happens because the leading section title can get interpreted as the last line of the final paragraph in the preceding include.
Only place include directives on consecutive lines if the intent is for the includes to run together (such as in a listing block).
////
"#
);
