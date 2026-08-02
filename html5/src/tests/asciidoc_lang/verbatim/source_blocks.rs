//! Coverage of the AsciiDoc language description's *Source Code Blocks* page.
//!
//! A source block is a listing block carrying the `source` style and a source
//! language. Matching Asciidoctor 2.0.26, this crate renders it as a
//! `div.listingblock` whose `pre.highlight` wraps a
//! `code.language-<lang>[data-lang=<lang>]`. The `source` style is optional:
//! the mere presence of a language on a listing block promotes it to a source
//! block. The examples and those claims are verified through `convert`.
//!
//! The page draws its example content with `include::example$source.adoc`
//! directives; the tests reproduce the relevant tagged snippets inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/source-blocks.adoc");

// The document title, URL attribute entries, and an editorial comment block —
// setup with nothing to render.
non_normative!(
    r#"
= Source Code Blocks
:table-caption: Table
:url-coderay: http://coderay.rubychan.de/
:url-highlightjs: https://highlightjs.org/
:url-pygments: https://pygments.org
:url-rouge: http://rouge.jneen.net
////
From user manual:
NOTE we really want this ID on the section, but there are problems w/ multiple IDs per section
[#source-code-blocks]
[#syntax-highlighting]
////

"#
);

// Conceptual introduction: a source block specializes a listing block, and
// AsciiDoc processors integrate syntax-highlighting libraries. These are
// descriptive claims about the ecosystem (Rouge, CodeRay, Pygments,
// highlight.js); the concrete rendering is verified below, and the highlighter
// behavior on the Source Highlighting page.
non_normative!(
    r#"
A source block is a specialization of a xref:listing-blocks.adoc[listing block].
Developers are accustomed to seeing source code colorized to emphasize the code's structure (i.e., keywords, types, delimiters, etc.).
This technique is known as [.term]*syntax highlighting*.
Since this technique is so prevalent, AsciiDoc processors will integrate at least one library to syntax highlight the source code blocks in your document.
For example, Asciidoctor provides integration with Rouge, CodeRay, Pygments, and highlight.js, as well as an adapter API to add support for additional libraries.

"#
);

// A listing block with the `source` style and a language (`ruby`) is a source
// block: it renders as a `div.listingblock` whose `pre.highlight` wraps a
// `code` carrying the `language-ruby` class and a `data-lang="ruby"` attribute.
#[test]
fn source_style_with_language_is_a_source_block() {
    verifies!(
        r#"
<<ex-source>> shows a listing block with the `source` style and language `ruby` applied to its content, hence a source block.

.Source block syntax
[source#ex-source]
....
include::example$source.adoc[tag=src-base]
....

The result of <<ex-source>> is rendered below.

include::example$source.adoc[tag=src-base]

"#
    );

    // The `tag=src-base` snippet from `example$source.adoc`.
    let output = convert(
        "[source,ruby]\n----\nrequire 'sinatra'\n\nget '/hi' do\n  \"Hello World!\"\nend\n----\n",
    );

    assert_css(
        &output,
        r#"div.listingblock > div.content > pre.highlight > code.language-ruby"#,
        1,
    );
    assert_xpath(&output, r#"//code[@data-lang="ruby"]"#, 1);
}

// The `source` style is optional: a language alone on a listing block promotes
// it to a source block. `[,ruby]` (an empty style position followed by the
// language) produces the same source-block markup as the explicit form above.
#[test]
fn language_alone_implies_a_source_block() {
    verifies!(
        r#"
Since a `source` block is most often used to designate a block with source code of a particular language, the `source` style itself is optional.
The mere presence of the language on a listing block automatically promotes it to a source block.

<<ex-implied-source>> shows a listing block implied to be a source block because a language is specified.

.Implied source block
[source#ex-implied-source]
....
include::example$source.adoc[tag=src-implied]
....

"#
    );

    // The `tag=src-implied` snippet: a language with no explicit `source` style.
    let output = convert(
        "[,ruby]\n----\nrequire 'sinatra'\n\nget '/hi' do\n  \"Hello World!\"\nend\n----\n",
    );

    assert_css(
        &output,
        r#"div.listingblock > div.content > pre.highlight > code.language-ruby"#,
        1,
    );
    assert_xpath(&output, r#"//code[@data-lang="ruby"]"#, 1);
}

// The `source-language` document attribute is documented in full on the Source
// Highlighting page; this cross-referencing prose has no rendered behavior of
// its own here.
non_normative!(
    r#"
This shorthand also works if the `source-language` attribute is set on the document, which serves as the default language for source blocks.
If the `source-language` attribute is set on the document and you want to make a regular listing block, add the `listing` style to the block.

"#
);

// The page shows the *syntax* of pulling source into a block with an include
// directive. The displayed snippet escapes the include (`\include::`) so it is
// shown literally rather than resolved: the block renders as a listing block
// whose `pre` displays the four source lines verbatim, the leading backslash
// removed.
#[test]
fn include_directive_syntax_is_shown_verbatim() {
    verifies!(
        r#"
== Using include directives in source blocks

You can use an xref:directives:include.adoc[include directive] to insert source code into an AsciiDoc document directly from a file.

.Code inserted from another file
[listing#ex-include]
....
include::example$source.adoc[tag=src-inc]
....

//TODO mention the use of AsciiDoc tags to include code snippets and the indent flag to reset indentation

"#
    );

    // The `tag=src-inc` snippet, displayed by the surrounding `[listing]` block.
    // The escaped `\include::` is shown literally, its backslash removed.
    let output = convert("[listing]\n....\n[,ruby]\n----\n\\include::app.rb[]\n----\n....\n");

    assert_css(&output, "div.listingblock > div.content > pre", 1);
    assert!(output.contains("[,ruby]\n----\ninclude::app.rb[]\n----"));
}

// A tip about preserving syntax highlighting when customizing substitutions,
// plus a trailing editorial comment; no rendered behavior of its own.
non_normative!(
    r#"
TIP: If you specify custom substitutions on the source block using the `subs` attribute, make sure to include the `specialcharacters` substitution if you want to preserve syntax highlighting.
However, if you do plan to modify the substitutions, we recommend using xref:subs:apply-subs-to-blocks.adoc#incremental[incremental substitutions] instead.

// Highlight PHP sidebar was taken from end of page and made its own page
"#
);
