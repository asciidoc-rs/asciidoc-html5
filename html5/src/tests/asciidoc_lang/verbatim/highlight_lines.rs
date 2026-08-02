//! Coverage of the AsciiDoc language description's *Highlight Select Lines*
//! page.
//!
//! The `highlight` attribute emphasizes specific lines of a source block. The
//! emphasis itself is produced by a server-side syntax highlighter (CodeRay,
//! Rouge, or Pygments) — none of which this crate runs — so the highlighted
//! output is not observable here and those claims are tracked as non-normative.
//! What this crate can verify is that the attribute is accepted on a source
//! block (rendering a plain source block, the documented limitation) and that
//! the page's syntax-display blocks render verbatim.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/highlight-lines.adoc");

// Concept and cross-reference: line highlighting emphasizes chosen lines, as
// distinct from source (syntax) highlighting.
non_normative!(
    r#"
= Highlight Select Lines

Not to be confused with xref:source-highlighter.adoc[source highlighting], you can highlight (i.e., emphasize) specific lines in a source block in order to call attention to them.

"#
);

// The usage-criteria table enumerates, per syntax highlighter, what is needed
// to use the `highlight` attribute. This is a support matrix for CodeRay,
// Rouge, Pygments, and highlight.js — server-side (or highlight.js-specific)
// libraries this crate does not run — so none of it is observable in this
// crate's output.
non_normative!(
    r#"
== Usage criteria

Line highlighting can be applied to a source block if certain criteria are met.

[%autowidth]
|===
|`source-highlighter` |Criteria to use the `highlight` attribute on a source block

|`coderay`
a|
* The `linenums` option is enabled on the block.
* The `highlight` attribute is defined on the block.

*Line highlighting will only emphasize the line number itself.*

|`rouge`
a|
* The `highlight` attribute is defined on the block.
* The CSS to support line highlighting is supplied by docinfo.
(Needed even if `rouge-css=style`).

|`pygments`
a|* The `highlight` attribute is defined on the block.

|`highlight.js`
|Not applicable.

*Line highlighting isn't available when using highlight.js.*
|===

"#
);

// The `highlight` attribute's value grammar (a comma/semicolon list of line
// numbers and `..` ranges) is consumed by the syntax highlighter to decide
// which lines to emphasize. Because this crate does not run a server-side
// highlighter, the grammar has no observable effect on the rendered output.
non_normative!(
    r#"
== highlight attribute

Line highlighting is activated on a source block if the highlight attribute is defined and at least one of the line numbers falls in this range.

IMPORTANT: Keep in mind that some syntax highlighter libraries require additional options (e.g., CodeRay and Rouge), and some don't support line highlighting at all (e.g., highlight.js).

The `highlight` attribute accepts a comma or semicolon delimited list of line ranges.
The numbers correspond to the line numbers of the source block.
If the `start` attribute is not specified, line numbers of the source block start at 1.

Here are some examples:

* 1
* 2,4,6
* 3..5
* 2,7..9

A line range is represented by two numbers separated by a double period (e.g., `2..5`).
The range is inclusive.

"#
);

// The CodeRay example is shown as a syntax display (a `[source]` block with the
// `....` delimiters), which this crate renders verbatim as a listing block
// carrying the block id. Separately, converting the *actual* configuration
// confirms the documented limitation: this crate accepts `%linenums` and
// `highlight=2..5` and renders a plain `ruby` source block, without the CodeRay
// line-number table or emphasized lines a server-side run would produce.
#[test]
fn coderay_example_is_shown_verbatim_and_highlight_is_accepted() {
    verifies!(
        r#"
=== CodeRay

.Highlight select lines when source-highlighter=coderay
[source#ex-coderay]
....
= Document Title
:source-highlighter: coderay

[%linenums,ruby,highlight=2..5]
----
ORDERED_LIST_KEYWORDS = {
  'loweralpha' => 'a',
  'lowerroman' => 'i',
  'upperalpha' => 'A',
  'upperroman' => 'I',
}
----
....

"#
    );

    // The syntax display renders verbatim, carrying the `ex-coderay` id.
    let display = convert(
        "[source#ex-coderay]\n....\n= Document Title\n:source-highlighter: coderay\n\n[%linenums,ruby,highlight=2..5]\n----\nORDERED_LIST_KEYWORDS = {\n  'loweralpha' => 'a',\n}\n----\n....\n",
    );
    assert_css(
        &display,
        "div#ex-coderay.listingblock > div.content > pre",
        1,
    );
    assert!(display.contains(":source-highlighter: coderay"));
    assert!(display.contains("[%linenums,ruby,highlight=2..5]"));

    // The documented limitation: the `highlight` attribute is accepted, but with
    // no server-side highlighter this crate renders a plain `ruby` source block —
    // no CodeRay line-number table, no emphasized lines.
    let rendered = convert(
        "= Document Title\n:source-highlighter: coderay\n\n[%linenums,ruby,highlight=2..5]\n----\nORDERED_LIST_KEYWORDS = {\n  'loweralpha' => 'a',\n}\n----\n",
    );
    assert_css(&rendered, "pre.highlight > code.language-ruby", 1);
    assert!(!rendered.contains("highlighted"));
    assert!(!rendered.contains("line-numbers"));
}

// CodeRay-specific behavior — that `linenums` is required and only the line
// number is emphasized — is produced by CodeRay, which this crate does not run.
non_normative!(
    r#"
When using CodeRay as the source highlighter, the `linenums` option is required to use line highlighting.
That's because line highlighting is only applied to the line number, which is emphasized using bold text.
CodeRay does not shade the line of code itself.

"#
);

// The Rouge example is likewise a syntax display, rendered verbatim with the
// `ex-rouge` id.
#[test]
fn rouge_example_is_shown_verbatim() {
    verifies!(
        r#"
=== Rouge

.Highlight select lines when source-highlighter=rouge
[source#ex-rouge]
....
= Document Title
:source-highlighter: rouge
:docinfo: shared

[,ruby,highlight=2..5]
----
ORDERED_LIST_KEYWORDS = {
  'loweralpha' => 'a',
  'lowerroman' => 'i',
  'upperalpha' => 'A',
  'upperroman' => 'I',
}
----
....

"#
    );

    let display = convert(
        "[source#ex-rouge]\n....\n= Document Title\n:source-highlighter: rouge\n:docinfo: shared\n\n[,ruby,highlight=2..5]\n----\nORDERED_LIST_KEYWORDS = {\n  'loweralpha' => 'a',\n}\n----\n....\n",
    );
    assert_css(&display, "div#ex-rouge.listingblock > div.content > pre", 1);
    assert!(display.contains(":source-highlighter: rouge"));
    assert!(display.contains("[,ruby,highlight=2..5]"));
}

// The requirement that Rouge line-highlighting CSS be supplied via docinfo.
non_normative!(
    r#"
When using Rouge as the source highlighter, you must supply CSS to support line highlighting.
You can do so by storing the required line highlighting CSS in a docinfo file, then including it in the output document by setting the `docinfo` document attribute.

"#
);

// The docinfo CSS is presented as an HTML source block; this crate renders it
// verbatim, escaping the special characters, as a `language-html` source block.
#[test]
fn rouge_docinfo_css_is_shown_verbatim() {
    verifies!(
        r#"
.Docinfo file (docinfo.html) to support line highlighting with Rouge
[,html]
----
<style>
pre.rouge .hll {
  background-color: #ffc;
  display: block;
}
pre.rouge .hll * {
  background-color: initial;
}
</style>
----

"#
    );

    let output = convert(
        "[,html]\n----\n<style>\npre.rouge .hll {\n  background-color: #ffc;\n  display: block;\n}\npre.rouge .hll * {\n  background-color: initial;\n}\n</style>\n----\n",
    );
    assert_css(&output, "pre.highlight > code.language-html", 1);

    // The HTML tags are escaped in the verbatim display.
    assert!(output.contains("&lt;style&gt;"));
    assert!(output.contains("pre.rouge .hll {"));
    assert_xpath(&output, "//code/style", 0);
}

// The remaining note describes how the Rouge integration marks highlighted
// lines (the `hll` class) — behavior of the Rouge library, not this crate.
non_normative!(
    r#"
Note that this supplemental CSS is needed even when `rouge-css=style`.
The Rouge integration does not embed the CSS for highlighting lines into the style attribute of the tag for each line.
Instead, it sets the `hll` class on the tag (e.g., `<span class="hll">`).

"#
);

// The Pygments example is a final syntax display, rendered verbatim with the
// `ex-pygments` id.
#[test]
fn pygments_example_is_shown_verbatim() {
    verifies!(
        r#"
=== Pygments

.Highlight select lines when source-highlighter=pygments
[source#ex-pygments]
....
= Document Title
:source-highlighter: pygments

[,ruby,highlight=2..5]
----
ORDERED_LIST_KEYWORDS = {
  'loweralpha' => 'a',
  'lowerroman' => 'i',
  'upperalpha' => 'A',
  'upperroman' => 'I',
}
----
....
"#
    );

    let display = convert(
        "[source#ex-pygments]\n....\n= Document Title\n:source-highlighter: pygments\n\n[,ruby,highlight=2..5]\n----\nORDERED_LIST_KEYWORDS = {\n  'loweralpha' => 'a',\n}\n----\n....\n",
    );
    assert_css(
        &display,
        "div#ex-pygments.listingblock > div.content > pre",
        1,
    );
    assert!(display.contains(":source-highlighter: pygments"));
    assert!(display.contains("[,ruby,highlight=2..5]"));
}
