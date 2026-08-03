//! Coverage of the AsciiDoc language description's *URL Macro* page.
//!
//! Appending a pair of square brackets to a URL turns it into an inline macro
//! (the URL scheme doubles as the macro name). With empty brackets it behaves
//! like an autolink (bare role); with text it uses custom link text; and it can
//! force parsing where an autolink would not be recognized (inside quotes) or
//! mark the URL boundary explicitly. Each form is verified through `convert`.
//!
//! Several examples are pulled in with `include::example$url.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/url-macro.adoc");

non_normative!(
    r#"
= URL Macro
// The term URL is now preferred over the term URI. See https://en.wikipedia.org/wiki/Uniform_Resource_Identifier#URLs_and_URNs

If you're familiar with the AsciiDoc syntax, you may notice that a URL almost looks like an inline macro.
All that's missing is the pair of the trailing square brackets.
In fact, if you add them, then a URL is treated as an inline macro.
We call this a URL macro.

This page introduces the URL macro, when you would want to use it, and how it differs from the link macro.

== From URL to macro

"#
);

// A URL with empty brackets behaves like an autolink: the URL is both the
// target and the text, and the link gets the `bare` role.
#[test]
fn bare_url_macro() {
    verifies!(
        r#"
To transform a URL into a macro, add a pair of square brackets to the end of the URL.
For example:

[source]
----
https://asciidoctor.org[]
----

Since no text is specified, this macro behaves the same as an autolink.
In this case, the link automatically gets assigned the "`bare`" role.

When the URL is followed by a pair of square brackets, the URL scheme dually serves as the macro name.
AsciiDoc recognizes all the xref:autolinks.adoc#schemes[URL schemes] for autolinks as macro names (e.g., `https`).
That's why we say "`URL macros`" and not just "`URL macro`".
It's a family of macros.
With the exception of the xref:mailto-macro.adoc[mailto macro], all the URL macros behave the same, and also behave the same as the xref:link-macro.adoc[link macro].

"#
    );

    assert!(convert("https://asciidoctor.org[]")
        .contains(r#"<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>"#));
}

// The URL macro forces the URL to be parsed where an autolink would not be
// recognized — for example, when the URL is enclosed in double quotes.
#[test]
fn forces_parsing_in_quotes() {
    verifies!(
        r#"
So why might you graduate from a URL to a URL macro?
One reason is to force the URL to be parsed when it would not normally be recognized, such as if it's enclosed in double quotes:

[source]
----
Type "https://asciidoctor.org[]" into the location bar of your browser.
----

"#
    );

    assert!(
        convert(r#"Type "https://asciidoctor.org[]" into the location bar of your browser."#)
            .contains(
                r#"<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>"#
            )
    );
}

// The brackets mark the boundary of the URL explicitly, so trailing content is
// not captured into the link target.
#[test]
fn marks_url_boundary() {
    verifies!(
        r#"
Another is when the URL of an autolink is being ended too soon.
The URL macro allows you to explicitly mark the boundary of the URL.

[source]
----
Use http://example.com?menu=<value>[] to open to the menu named `<value>`.
----

"#
    );

    assert!(
        convert("Use http://example.com?menu=<value>[] to open to the menu named `<value>`.")
            .contains(r#"<a href="http://example.com?menu=&lt;value&gt;" class="bare">http://example.com?menu=&lt;value&gt;</a>"#)
    );
}

non_normative!(
    r#"
A more common reason is to specify custom link text.

[#link-text]
== Custom link text

"#
);

// Text between the brackets becomes the link's display text.
#[test]
fn custom_link_text() {
    verifies!(
        r#"
Instead of displaying the URL, you can configure the link to display custom text.
When the reader clicks on the text, they are directed to the target of the link, the URL.

To customize the text of the link, insert that text between the square brackets of the URL macro.

[source]
----
include::example$url.adoc[tag=irc]
----

"#
    );

    // The `tag=irc` snippet.
    assert!(convert(
        "Chat with other Fedora users in the irc://irc.freenode.org/#fedora[Fedora IRC channel]."
    )
    .contains(r#"<a href="irc://irc.freenode.org/#fedora">Fedora IRC channel</a>"#));
}

// The link text is subject to normal substitutions, so it can carry formatting.
#[test]
fn formatting_in_link_text() {
    verifies!(
        r#"
Since the text is subject to normal substitutions, you can apply formatting to it.

[source]
----
include::example$url.adoc[tag=text]
----
"#
    );

    // The `tag=text` snippet: `*community chat*` renders bold link text.
    assert!(
        convert("Ask questions in the https://chat.asciidoc.org[*community chat*].")
            .contains(r#"<a href="https://chat.asciidoc.org"><strong>community chat</strong></a>"#)
    );
}

non_normative!(
    r#"

== Link attributes

"#
);

// The attribute list customizes the link — here a new window (`^`) and a role.
#[test]
fn link_attributes() {
    verifies!(
        r#"
You can use the attribute list to further customize the link, such as to make it target a new window and apply a role to it.

[source]
----
include::example$url.adoc[tag=css]
----

"#
    );

    // The `tag=css` snippet: `^` opens a new window (adds `target`/`rel`), and
    // `role=green` adds a CSS class.
    assert!(
        convert("Chat with other AsciiDoc users in the https://chat.asciidoc.org[*project chat*^,role=green].")
            .contains(r#"<a href="https://chat.asciidoc.org" class="green" target="_blank" rel="noopener"><strong>project chat</strong></a>"#)
    );
}

non_normative!(
    r#"
To understand how the text between the square brackets of a URL macro is parsed, see xref:link-macro-attribute-parsing.adoc[attribute parsing].
"#
);
