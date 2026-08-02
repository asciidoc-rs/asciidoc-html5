//! Coverage of the AsciiDoc language description's *Indent Included Content*
//! page.
//!
//! The `indent` attribute normalizes the leading block indent of verbatim
//! content: `indent=0` strips it, and a positive `indent` strips it and then
//! re-adds that many columns. The page shows both with a Ruby source block (the
//! indent applies to the block's verbatim content whether or not it came from
//! an include, so it is verified directly on the block). The rule that the
//! attribute is ignored when a line is not indented is verified too. The
//! `tabsize` aside is non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-with-indent.adoc");

/// The verbatim text of the first `<pre>` in `html`, unwrapping an inner
/// `<code>` when present. The indent examples contain no markup characters, so
/// no entity decoding is needed.
fn verbatim_text(html: &str) -> String {
    let start = html.find("<pre").expect("a <pre>");
    let open_end = html[start..].find('>').expect("end of <pre> tag") + start + 1;
    let end = html[open_end..].find("</pre>").expect("closing </pre>") + open_end;
    let inner = &html[open_end..end];

    if let Some(rest) = inner.strip_prefix("<code") {
        let content_start = rest.find('>').expect("end of <code> tag") + 1;
        let content_end = rest.rfind("</code>").expect("closing </code>");
        rest[content_start..content_end].to_string()
    } else {
        inner.to_string()
    }
}

non_normative!(
    r#"
= Indent Included Content
// aka Normalize Block Indentation
// This content needs to be made applicable to includes...like add a step in the example flow to show it coming from an include file.

Source code snippets from external files are often padded with a leading block indent.
This leading block indent is relevant in its original context.
However, once inside the documentation, this leading block indent is no longer needed.

== The indent attribute

"#
);

// The `indent` attribute normalizes a verbatim block's leading indent:
// `indent=0` strips it entirely, and a positive value strips it and then
// re-adds that many columns.
#[test]
fn indent_strips_and_optionally_re_adds_the_leading_block_indent() {
    verifies!(
        r#"
The attribute `indent` allows the leading block indent to be stripped and, optionally, a new block indent to be set for blocks with verbatim content (listing, literal, source, verse, etc.).

* When `indent` is 0, the leading block indent is stripped
* When `indent` is > 0, the leading block indent is first stripped, then the content is indented by the number of columns equal to this value.

"#
    );

    let stripped = convert("[listing,indent=0]\n----\n    first\n      second\n----\n");
    assert_eq!(verbatim_text(&stripped), "first\n  second");

    let reindented = convert("[listing,indent=3]\n----\n    first\n      second\n----\n");
    assert_eq!(verbatim_text(&reindented), "   first\n     second");
}

// If any line of the verbatim content is not indented, the common leading
// indent is zero, so the `indent` attribute has no effect.
#[test]
fn indent_is_ignored_when_a_line_is_not_indented() {
    verifies!(
        r#"
WARNING: If any line in the verbatim content is not indented, the `indent` attribute is effectively ignored.

"#
    );

    let output = convert("[listing,indent=0]\n----\n    first\nflush\n    third\n----\n");
    assert_eq!(verbatim_text(&output), "    first\nflush\n    third");
}

// Non-normative: the `tabsize` interaction (a separate normalization applied to
// tabs regardless of `indent`), and the prose that frames the worked examples
// below.
non_normative!(
    r#"
If the `tabsize` attribute is set on the block or the document, tabs are also replaced with the number of spaces specified by that attribute, regardless of whether the `indent` attribute is set.

Let's consider a source block that has included content which is indented.
Only the result of the include directive is shown here to help illustrate the behavior.
When the `indent` attribute is used on the source block in the following AsciiDoc source:

"#
);

// The page's worked example for `indent=0`: the 4-column indent of the source
// is stripped, leaving the code flush against the left margin (with its own
// relative indentation intact).
#[test]
fn indent_zero_example_strips_the_leading_indent() {
    verifies!(
        r#"
[source]
....
[source,ruby,indent=0]
----
    def names
      @name.split ' '
    end
----
....

The processor produces:

....
def names
  @name.split ' '
end
....

"#
    );

    let output = convert(
        "[source,ruby,indent=0]\n----\n    def names\n      @name.split ' '\n    end\n----\n",
    );
    assert_eq!(verbatim_text(&output), "def names\n  @name.split ' '\nend");
}

// Non-normative: the transition into the second worked example.
non_normative!(
    r#"
On the other hand, this AsciiDoc source:

"#
);

// The page's worked example for `indent=2`: the leading indent is first
// removed, then re-added as two columns.
#[test]
fn indent_two_example_re_adds_two_columns() {
    verifies!(
        r#"
[source]
....
[source,ruby,indent=2]
----
    def names
      @name.split ' '
    end
----
....

Produces:

----
  def names
    @name.split ' '
  end
----

"#
    );

    let output = convert(
        "[source,ruby,indent=2]\n----\n    def names\n      @name.split ' '\n    end\n----\n",
    );
    assert_eq!(
        verbatim_text(&output),
        "  def names\n    @name.split ' '\n  end"
    );
}

non_normative!(
    r#"
Notice that when the `indent` attribute is positive, the block indentation is first removed, then readded using the specified amount.
"#
);
