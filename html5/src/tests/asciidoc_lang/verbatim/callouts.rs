//! Coverage of the AsciiDoc language description's *Callouts* page.
//!
//! A callout number annotates a line of a verbatim block: it appears once at
//! the end of the target line (rendered as a `<b class="conum">(N)</b>` conum)
//! and once below the block to introduce its annotation (rendered as an item of
//! a `div.colist` ordered list). This crate matches Asciidoctor 2.0.26 across
//! explicit numbering, automatic `<.>` numbering, mixed numbering, callouts
//! tucked behind line comments (with the `line-comment` attribute to customize
//! or disable the prefix), and XML-comment callouts. The examples and the
//! claims about them are verified through `convert`.
//!
//! Each page example is a `subs=-callouts` syntax display (callouts shown
//! literally as `<N>`) paired with a rendered result. The page draws them with
//! `include::example$callout.adoc`; the tests reproduce the tagged snippets
//! inline and assert on the rendered result — the observable behavior.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/callouts.adoc");

non_normative!(
    r#"
= Callouts

Callout numbers (aka callouts) provide a means to add annotations to lines in a verbatim block.

"#
);

// The core callout mechanism: each number appears twice — once at the end of
// the target line inside the block (a conum) and once below the block to define
// its annotation (a callout-list item). A source block with three callouts
// (two sharing a line) renders three conums and a three-item `colist`. The
// syntax display uses `subs=-callouts`, so its callouts show literally as
// `&lt;N&gt;` rather than as conums.
#[test]
fn callout_number_marks_a_target_and_defines_an_annotation() {
    verifies!(
        r#"
== Callout syntax

Each callout number used in a verbatim block must appear twice.
The first use, which goes within the verbatim block, marks the line being annotated (i.e., the target).
The second use, which goes below the verbatim block, defines the annotation text.
Multiple callout numbers may be used on a single line.

IMPORTANT: The callout number (at the target) must be placed at the end of the line.

Here's a basic example of a verbatim block that uses callouts:

.Callout syntax
[source#ex-basic,subs=-callouts]
....
include::example$callout.adoc[tag=basic]
....

The result of <<ex-basic>> is rendered below.

include::example$callout.adoc[tag=basic]

"#
    );

    // The rendered result of the `basic` snippet: three conums (two on one
    // line) and a three-item callout list.
    let result = convert(
        "[source,ruby]\n----\nrequire 'sinatra' <1>\n\nget '/hi' do <2> <3>\n  \"Hello World!\"\nend\n----\n<1> Library import\n<2> URL mapping\n<3> Response block\n",
    );
    assert_css(&result, "pre.highlight > code > b.conum", 3);
    assert_css(&result, "div.colist.arabic > ol > li", 3);
    assert_xpath(&result, r#"//code/b[@class="conum"][text()="(2)"]"#, 1);

    // The `subs=-callouts` syntax display shows the callouts literally, not as
    // conums.
    let display = convert(
        "[source#ex-basic,subs=-callouts]\n....\n[source,ruby]\n----\nrequire 'sinatra' <1>\n----\n<1> Library import\n....\n",
    );
    assert_css(&display, "div#ex-basic.listingblock", 1);
    assert!(display.contains("require 'sinatra' &lt;1&gt;"));
    assert_xpath(&display, "//b[@class=\"conum\"]", 0);
}

// The features that hide callout numbers are introduced next; descriptive
// overview with no example of its own.
non_normative!(
    r#"
Since callout numbers can interfere with the syntax of the code they are annotating, an AsciiDoc processor provides several features to hide the callout numbers from both the source and the converted document.
The sections that follow detail these features.

"#
);

// Automatic numbering: replacing each callout number with `<.>` lets the
// processor number them in sequence (scoped to the block), starting at 1. The
// result is identical to the explicitly numbered version above — three conums
// numbered (1), (2), (3) and a three-item list.
#[test]
fn automatic_numbering_with_a_dot() {
    verifies!(
        r#"
== Automatic numbering

Just like ordered lists, it's possible to allow the processor to automatically number the callouts.
To leverage this capability, you replace the numbers in each callout with a dot (e.g., `<.>`).
Each time the processor comes across a callout in either the verbatim block or the callout list, it selects the next number in the sequence (scoped to that block), starting from 1.

Let's return to the previous example to see how it looks if we use automatic numbering.

.Callout syntax with automatic numbering
[source#ex-auto,subs=-callouts]
....
include::example$callout.adoc[tag=auto]
....

The result is exactly the same as before.

"#
    );

    // The `auto` snippet numbers the `<.>` callouts 1, 2, 3 in sequence.
    let result = convert(
        "[,ruby]\n----\nrequire 'sinatra' <.>\n\nget '/hi' do <.> <.>\n  \"Hello World!\"\nend\n----\n<.> Library import\n<.> URL mapping\n<.> Response block\n",
    );
    assert_css(&result, "pre.highlight > code > b.conum", 3);
    assert_css(&result, "div.colist.arabic > ol > li", 3);
    assert_xpath(&result, r#"//code/b[@class="conum"][text()="(1)"]"#, 1);
    assert_xpath(&result, r#"//code/b[@class="conum"][text()="(3)"]"#, 1);
}

// Mixed numbering: `<.>` callouts are numbered among themselves, ignoring any
// explicit numbers. Repeating an explicit number creates an additional
// occurrence of that conum. The `auto-repeat` snippet has two `<.>` (numbered 1
// and 2) plus an explicit `<1>`, so three conums appear in the block but the
// callout list has only the two `<.>` items.
#[test]
fn mixed_numbering_with_explicit_repeats() {
    verifies!(
        r#"
=== Mixed numbering

The `<.>` callouts are automatically numbered based on their sequence among other `<.>`, not any callouts that have an explicit number.
In other words, the automatic numbering is not aware of any explicit numbering.
Therefore, you should generally avoid mixing them.

However, if you want to repeat a number in the verbatim block, then you can use an explicit number to create additional occurrences of that callout number.

Let's consider an example:

.Callout syntax with mixed numbering
[source#ex-auto-repeat,subs=-callouts]
....
include::example$callout.adoc[tag=auto-repeat]
....

The result of <<ex-auto-repeat>> is rendered below.

include::example$callout.adoc[tag=auto-repeat]

"#
    );

    // The `auto-repeat` snippet: two `<.>` (numbered 1, 2) and one explicit
    // `<1>` — three conums in the block, two items in the callout list.
    let result = convert(
        "[,ruby]\n----\nrequire 'asciidoctor' <.>\n\nputs Asciidoctor::VERSION <1>\n\nAsciidoctor.convert_file 'README.adoc' <.>\n----\n<.> The require statement exports the class from the gem to the global scope.\n<.> We can then call methods provided by that class.\n",
    );
    assert_css(&result, "pre.highlight > code > b.conum", 3);
    assert_css(&result, "div.colist.arabic > ol > li", 2);
}

non_normative!(
    r#"
The risk of this approach is that you have to keep track of which numbers are being assigned automatically.

"#
);

// The CSS rule that prevents callout conums from being selected (so they are
// not caught up in copied source) is a stylesheet concern, not part of the
// rendered block structure.
non_normative!(
    r#"
== Copy and paste friendly callouts

If you add callout numbers to example code in a verbatim (e.g., source) block, and a reader selects that source code in the generated HTML, we don't want the callout numbers to get caught up in the copied text.
If the reader pastes that example code into a code editor and tries to run it, the extra characters that define the callout numbers will likely lead to compile or runtime errors.
To mitigate this problem, and AsciiDoc processor uses a CSS rule to prevent the callouts from being selected.
That way, the callout numbers won't get copied.

"#
);

// Callouts can be tucked behind a trailing line comment. When font-based icons
// are not enabled — this crate's rendering — the line-comment characters are
// left in place, so the conum stays hidden behind the comment. Each of the
// four supported comment styles (`//`, `#`, `;;`, and the XML `<!--N-->`)
// yields a conum with its comment marker preserved in the text.
#[test]
fn callouts_tucked_behind_line_comments() {
    verifies!(
        r#"
On the other side of the coin, you don't want the callout annotations or CSS messing up your raw source code either.
You can tuck your callouts neatly behind line comments.
When font-based icons are enabled (e.g., `icons=font`), the AsciiDoc processor will recognize the line comments characters in front of a callout number--optionally offset by a space--and remove them when converting the document.
When font-based icons aren't enabled, the line comment characters are not removed so that the callout numbers remain hidden by the line comment.

Here are the line comments that are supported:

.Prevent callout copy and paste
[source#ex-prevent,subs=-callouts]
....
include::example$callout.adoc[tag=b-nonselect]
....

The result of <<ex-prevent>> is rendered below.

// The listing style must be added above the rendered block so it is rendered correctly because we're setting `source-language` in the component descriptor which automatically promotes unstyled listing blocks to source blocks.
[listing]
include::example$callout.adoc[tag=b-nonselect]

"#
    );

    // The `b-nonselect` snippet: four callouts behind four comment styles. Font
    // icons are not enabled, so the comment markers are preserved before each
    // conum.
    let result = convert(
        "----\nline of code // <1>\nline of code # <2>\nline of code ;; <3>\nline of code <!--4-->\n----\n<1> A callout behind a line comment for C-style languages.\n<2> A callout behind a line comment for Ruby, Python, Perl, etc.\n<3> A callout behind a line comment for Clojure.\n<4> A callout behind a line comment for XML or SGML languages like HTML.\n",
    );
    assert_css(&result, "div.listingblock > div.content > pre > b.conum", 4);
    assert_css(&result, "div.colist.arabic > ol > li", 4);

    // The comment markers are left in place, keeping the conum hidden behind
    // them.
    assert!(result.contains("line of code // "));
    assert!(result.contains("line of code # "));
    assert!(result.contains("line of code ;; "));
}

// The `line-comment` attribute customizes which prefix hides a callout, for a
// language whose comment marker the processor does not recognize by default.
non_normative!(
    r#"
=== Custom line comment prefix

An AsciiDoc processor recognizes the most ubiquitous line comment prefixes as a convenience.
If the source language you're embedding does not support one of these line comment prefixes, you can customize the prefix using the `line-comment` attribute on the block.

Let's say we want to tuck a callout behind a line comment in Erlang code.
In this case, we would set the `line-comment` character to `%`, as shown in this example:

"#
);

// Setting `line-comment=%` lets a callout hide behind an Erlang `%` comment:
// the `% <1>` renders a conum, and (font icons being off) the `%` is preserved.
#[test]
fn custom_line_comment_prefix() {
    verifies!(
        r#"
.Custom line comment prefix
[source#ex-line-comment,subs=-callouts]
....
include::example$callout.adoc[tag=line-comment]
....

The result of <<ex-line-comment>> is rendered below.

// The listing style must be added above the rendered block so it is rendered correctly because we're setting `source-language` in the component descriptor which automatically promotes unstyled listing blocks to source blocks.
[listing]
include::example$callout.adoc[tag=line-comment]

Even though it's not specified in the attribute, one space is still permitted immediately following the line comment prefix.

"#
    );

    // The `line-comment` snippet uses `line-comment=%` so the trailing `%`
    // comment hides the conum; the `%` stays in the rendered text.
    let result = convert(
        "[source,erlang,line-comment=%]\n----\n-module(hello_world).\n-compile(export_all).\n\nhello() ->\n    io:format(\"hello world~n\"). % <1>\n----\n<1> A callout behind a custom line comment prefix.\n",
    );
    assert_css(&result, "pre.highlight > code.language-erlang > b.conum", 1);
    assert_css(&result, "div.colist.arabic > ol > li", 1);
    assert!(result.contains("io:format(\"hello world~n\"). % "));
}

// Line-comment processing can be disabled entirely, for a language where the
// prefix would be misinterpreted (here an AsciiDoc open-block delimiter `--`).
non_normative!(
    r#"
=== Disable line comment processing

If the source language you're embedding does not support trailing line comments, or the line comment prefix is being misinterpreted, you can disable this feature using the `line-comment` attribute.

Let's say we want to put a callout at the end of a block delimiter for an open block in AsciiDoc.
In this case, the processor will think the double hyphen is a line comment, when in fact it's the block delimiter.
We can disable line comment processing by setting the `line-comment` character to an empty value, as shown in this example:

"#
);

// Setting `line-comment=` (empty) disables line-comment processing, so the
// `--` before the callout is treated as content (the open-block delimiter),
// not a comment prefix: the conum renders and the `--` remains.
#[test]
fn disable_line_comment_processing() {
    verifies!(
        r#"
.No line comment prefix
[source#ex-disable-line-comment,subs=-callouts]
....
include::example$callout.adoc[tag=disable-line-comment]
....

The result of <<ex-disable-line-comment>> is rendered below.

// The listing style must be added above the rendered block so it is rendered correctly because we're setting `source-language` in the component descriptor which automatically promotes unstyled listing blocks to source blocks.
[listing]
include::example$callout.adoc[tag=disable-line-comment]

Since the language doesn't support trailing line comments, there's no way to hide the callout number in the raw source.

"#
    );

    // The `disable-line-comment` snippet: `line-comment=` disables the feature,
    // so the leading `--` stays as content and the conum still renders.
    let result = convert(
        "[source,asciidoc,line-comment=]\n----\n-- <1>\nA paragraph in an open block.\n--\n----\n<1> An open block delimiter.\n",
    );
    assert_css(
        &result,
        "pre.highlight > code.language-asciidoc > b.conum",
        1,
    );
    assert_css(&result, "div.colist.arabic > ol > li", 1);
    assert!(result.contains("-- <b class=\"conum\">(1)</b>"));
}

// XML has no line comments, so a callout is embedded inside an XML comment
// (`<!--N-->`) placed at the end of the target line.
non_normative!(
    r#"
=== XML callouts

XML doesn't have line comments, so our "`tuck the callout behind a line comment`" trick doesn't work here.
To use callouts in XML, you must place the callout's angled brackets around the XML comment and callout number.

Here's how it appears in a listing:

"#
);

// The XML callout `<!--1-->` renders a conum, replacing the comment, with a
// one-item callout list.
#[test]
fn xml_comment_callouts() {
    verifies!(
        r#"
.XML callout syntax
[source#ex-xml,subs=-callouts]
....
include::example$callout.adoc[tag=source-xml]
....

The result of <<ex-xml>> is rendered below.

include::example$callout.adoc[tag=source-xml]

"#
    );

    // The `source-xml` snippet: an `<!--1-->` XML-comment callout becomes a
    // conum, and the surrounding XML tags are escaped in the source block.
    let result = convert(
        "[source,xml]\n----\n<section>\n  <title>Section Title</title> <!--1-->\n</section>\n----\n<1> The section title is required.\n",
    );
    assert_css(&result, "pre.highlight > code.language-xml > b.conum", 1);
    assert_css(&result, "div.colist.arabic > ol > li", 1);
    assert!(result.contains("&lt;title&gt;Section Title&lt;/title&gt;"));
}

// How the conum appears visually — a circled, unselectable number with font
// icons, versus a different, selectable rendering without — is a stylesheet
// concern beyond the rendered structure.
non_normative!(
    r#"
Notice the comment has been replaced with a circled number that cannot be selected (if not using font icons it will be
rendered differently and selectable).
Now both you and the reader can copy and paste XML source code containing callouts without worrying about errors.

"#
);

// The CSS-drawn callout icons enabled by `icons=font` are a stylesheet feature;
// what is observable here is that the example (a listing display of a document
// that itself contains two callouts) renders those callouts as conums.
non_normative!(
    r#"
== Callout icons

The font icons setting also enables callout icons drawn using CSS.

"#
);

// The final example is a listing display whose content is a small document
// carrying two `<.>` callouts; it renders verbatim as a listing block with two
// conums and a two-item callout list.
#[test]
fn callout_icons_example_is_shown_as_a_listing() {
    verifies!(
        r#"
----
include::example$callout.adoc[tag=co-icon]
----
<.> Activates the font-based icons in the HTML5 backend.
<.> Admonition block that uses a font-based icon.
"#
    );

    // The `co-icon` snippet, displayed inside a listing block: its two `<.>`
    // callouts render as conums with a two-item callout list.
    let result = convert(
        "----\n= Document Title\n:icons: font <.>\n\nNOTE: Asciidoctor supports font-based admonition\nicons, powered by Font Awesome! <.>\n----\n<.> Activates the font-based icons in the HTML5 backend.\n<.> Admonition block that uses a font-based icon.\n",
    );
    assert_css(&result, "div.listingblock > div.content > pre > b.conum", 2);
    assert_css(&result, "div.colist.arabic > ol > li", 2);
}
