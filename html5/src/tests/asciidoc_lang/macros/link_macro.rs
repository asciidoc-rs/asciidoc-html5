//! Coverage of the AsciiDoc language description's *Link Macro* page.
//!
//! The `link:<target>[<text>]` macro is the explicit way to make a link, needed
//! when autolinks and URL macros are insufficient — a relative file target, a
//! target needing a passthrough to escape special characters, a URL macro not
//! bounded by a word break, or a target whose scheme AsciiDoc doesn't
//! recognize. Each case is verified through `convert`, matching Asciidoctor
//! 2.0.26.
//!
//! Two examples are pulled in with `include::example$url.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/link-macro.adoc");

non_normative!(
    r#"
= Link Macro

The link macro is the most explicit method of making a link in AsciiDoc.
It's only necessary when the behavior of autolinks and URL macros proves insufficient.
This page covers the anatomy of the link macro, when it's required, and how to use it.

== Anatomy

"#
);

non_normative!(
    r#"
The link macro is an inline macro.
Like other inline macros, its syntax follows the familiar pattern of the macro name and target separated by a colon followed by an attribute list enclosed in square brackets.

[source]
----
link:<target>[<attrlist>]
----

The `<target>` becomes the target of the link.
the `<attrlist>` is the link text unless a named attribute is detected.
See xref:link-macro-attribute-parsing.adoc[link macro attribute list] to learn how the `<attrlist>` is parsed.

"#
);

// A leading backslash escapes the macro, leaving it as literal text.
#[test]
fn escaped_with_leading_backslash() {
    verifies!(
        r#"
Like all inline macros, the link macro can be escaped using a leading backslash (`\`).

"#
    );

    let output = convert("\\link:report.pdf[Get Report]");
    assert!(!output.contains("<a "));
    assert!(output.contains("link:report.pdf[Get Report]"));
}

non_normative!(
    r#"
== Link to a relative file

If you want to link to a non-AsciiDoc file that is relative to the current document, use the `link` macro in front of the file name.

TIP: To link to a relative AsciiDoc file, use the xref:inter-document-xref.adoc[xref macro] instead.

Here's an example that demonstrates how to use the link macro to link to a relative file path:

"#
);

// The link macro creates a link to a relative file target (not a URL).
#[test]
fn relative_file() {
    verifies!(
        r#"
[source]
----
include::example$url.adoc[tag=link]
----

The AsciiDoc processor will create a link to _report.pdf_ with the text "Get Report", even though the target is not a URL.

"#
    );

    // The `tag=link` snippet.
    assert!(convert("link:downloads/report.pdf[Get Report]")
        .contains(r#"<a href="downloads/report.pdf">Get Report</a>"#));
}

// A hash appended to the file target links to an anchor within it.
#[test]
fn hash_anchor() {
    verifies!(
        r#"
If the target file is an HTML file, and you want to link directly to an anchor within that document, append a hash (`#`) followed by the name of the anchor after the file name:

[source]
----
include::example$url.adoc[tag=hash]
----

Note that when linking to a relative file, even if it's an HTML file, the link text is required.
"#
    );

    // The `tag=hash` snippet.
    assert!(convert("link:tools.html#editors[]")
        .contains(r#"<a href="tools.html#editors" class="bare">tools.html#editors</a>"#));
}

non_normative!(
    r#"

// FIXME: this feels like it needs subsections
== When to use the link macro

Since AsciiDoc provides autolinks and URL macros, the link macro is not often needed.
Here are the few cases when the link macro is necessary:

"#
);

non_normative!(
    r#"
* The target is not a URL (e.g., a relative path)
* The target must be enclosed in a passthrough to escape characters with special meaning
* The URL macro is not bounded by spaces, brackets, or quotes.
* The target is a URL that does not start with a scheme recognized by AsciiDoc

"#
);

// The most common case: the target is not a URL (a relative path).
#[test]
fn target_not_a_url() {
    verifies!(
        r#"
The most common situation is when the target is not a URL.
For example, you would use the link macro to create a link to a relative path.

[source]
----
link:report.pdf[Get Report]
----

"#
    );

    assert!(
        convert("link:report.pdf[Get Report]").contains(r#"<a href="report.pdf">Get Report</a>"#)
    );
}

non_normative!(
    r#"
TIP: If the relative path is another AsciiDoc file, you should use the xref:inter-document-xref.adoc[xref macro] instead.

"#
);

// A space in the target prevents macro recognition, so it must be escaped or
// encoded — via a passthrough, a character reference, or URL encoding.
#[test]
fn escape_space_in_target() {
    verifies!(
        r#"
You may also discover that spaces are not permitted in the target of the link macro, at least not in the AsciiDoc source.
The space character in the target prevents the parser from recognizing the macro.
So it's necessary to escape or encode it.
Here are three ways to do it:

.Escape a space using a passthrough
[source]
----
link:pass:[My Documents/report.pdf][Get Report]
----

.Encode a space using a character reference (\&#32;)
[source]
----
link:My&#32;Documents/report.pdf[Get Report]
----

.Encode a space using URL encoding (%20)
[source]
----
link:My%20Documents/report.pdf[Get Report]
----

"#
    );

    // A passthrough keeps the literal space in the target.
    assert!(convert("link:pass:[My Documents/report.pdf][Get Report]")
        .contains(r#"<a href="My Documents/report.pdf">Get Report</a>"#));

    // A character reference encodes the space.
    assert!(convert("link:My&#32;Documents/report.pdf[Get Report]")
        .contains(r#"<a href="My&#32;Documents/report.pdf">Get Report</a>"#));

    // URL encoding encodes the space (visible in the target text).
    assert!(convert("link:My%20Documents/report.pdf[Get Report]")
        .contains(r#"<a href="My%20Documents/report.pdf">Get Report</a>"#));
}

non_normative!(
    r#"
Escaping or encoding the space ensures that the space does not prevent the link macro from being recognized.
The downside of using URL encoding is that it will be visible in the automatic link text since the browser does not decode it in that location.
In this case, the character reference is preferable.

There are other characters that are not permitted in a link target as well, such as a colon.
You can escape those using the same technique.

"#
);

// A colon in the target is likewise URL-encoded (`%3A`).
#[test]
fn encode_colon() {
    verifies!(
        r#"
.Encode a colon using URL encoding (%3A)
[source]
----
link:Avengers%3A%20Endgame.html[]
----

"#
    );

    assert!(convert("link:Avengers%3A%20Endgame.html[]").contains(
        r#"<a href="Avengers%3A%20Endgame.html" class="bare">Avengers%3A%20Endgame.html</a>"#
    ));
}

// A passthrough (`++…++`) escapes special characters (repeating underscores) so
// the target is not mangled by inline formatting.
#[test]
fn passthrough_escape_underscores() {
    verifies!(
        r#"
Another common case is when you need to use a passthrough to escape characters with special meaning.
In this case, the AsciiDoc processor will not recognize the target as a URL, and thus the link macro is necessary.
An example is when the URL contains repeating underscore characters.

[source]
----
link:++https://example.org/now_this__link_works.html++[]
----

"#
    );

    assert!(convert("link:++https://example.org/now_this__link_works.html++[]").contains(
        r#"<a href="https://example.org/now_this__link_works.html" class="bare">https://example.org/now_this__link_works.html</a>"#
    ));
}

// The `link:` prefix raises precedence so a URL macro is recognized even when
// it is not bounded by spaces, brackets, or quotes (here, adjacent to `|`).
#[test]
fn not_bounded_by_word_break() {
    verifies!(
        r#"
A similar situation is when the URL macro is not bounded by spaces, brackets, or quotes.
In this case, the link macro prefix is required to increase the precedence so that the macro can be recognized.

[source]
----
|link:https://asciidoctor.org[]|
----

"#
    );

    assert!(convert("|link:https://asciidoctor.org[]|")
        .contains(r#"<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>"#));
}

// When the target's scheme is not recognized as a URL (e.g. `file:`), the link
// macro is required.
#[test]
fn unrecognized_scheme() {
    verifies!(
        r#"
Finally, if the target is not recognized as a URL by AsciiDoc, the link macro is necessary.
For example, you might use the link macro to make a file link.

[source]
----
link:file:///home/username[Your files]
----

"#
    );

    assert!(convert("link:file:///home/username[Your files]")
        .contains(r#"<a href="file:///home/username">Your files</a>"#));
}

non_normative!(
    r#"
== Final word

The general rule of thumb is that you should only put the `link:` macro prefix in front of the target if the target is _not_ a URL.
Otherwise, the prefix just adds verbosity.
"#
);
