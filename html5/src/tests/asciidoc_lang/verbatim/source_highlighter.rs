//! Coverage of the AsciiDoc language description's *Source Highlighting* page.
//!
//! The `source-highlighter` attribute enables syntax highlighting of source
//! blocks. This crate models the two *client-side* highlighters (highlight.js
//! and prettify), which reshape each source block's `pre`/`code` classes and
//! inject a CDN `<link>`/`<script>`; the server-side highlighters (CodeRay,
//! Rouge, Pygments) run in an external toolchain this crate does not invoke, so
//! their tokenized output is not observable here. The page's observable claims
//! — the attribute enables highlighting, a language is required, the `source`
//! style is required for paragraphs, and the `source-language` attribute
//! promotes bare listings — are verified through `convert`.
//!
//! The page draws its example content with `include::example$source.adoc`
//! directives; the tests reproduce the relevant tagged snippets inline.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/source-highlighter.adoc");

// The document title and a caption attribute entry.
non_normative!(
    r#"
= Source Highlighting
:table-caption: Table

"#
);

// Conceptual introduction: highlighting applies to text with the `source`
// style and a language, defined on the block or inherited from
// `source-language`. The concrete attribute and language handling are verified
// below.
non_normative!(
    r#"
Source highlighting is applied to text that's assigned the `source` block style (either explicitly or implicitly) and a source language.
The source language is defined either on the block or inherited from the `source-language` document attribute.

"#
);

// Source highlighting is off by default and enabled by the `source-highlighter`
// document attribute. With no highlighter, a source block renders in its plain
// shape (`pre.highlight > code.language-ruby`, no `hljs` class, no CDN assets);
// setting `:source-highlighter: highlight.js` reshapes it (`pre.highlightjs`,
// `code.language-ruby.hljs`) and injects the highlight.js CDN `<link>`/
// `<script>`. The page's header snippets are displayed verbatim as listing
// blocks.
#[test]
fn source_highlighter_attribute_enables_highlighting() {
    verifies!(
        r#"
[#source-highlighter]
== source-highlighter attribute

Source highlighting isn't enabled by default.
To enable source highlighting, you must set the `source-highlighter` attribute in the document header using an attribute entry.

----
= Document Title
:source-highlighter: <value>
----

For example, here's how to enable syntax highlighting using Rouge:

----
= Document Title
:source-highlighter: rouge
----

You can also declare this attribute using the CLI or API.

"#
    );

    // With no highlighter set, the source block is in its default shape.
    let plain = convert("[,ruby]\n----\nputs 1\n----\n");
    assert_css(&plain, "pre.highlight > code.language-ruby", 1);
    assert!(!plain.contains("hljs"));
    assert!(!plain.contains("highlight.js"));

    // Setting `source-highlighter` to highlight.js enables client-side
    // highlighting: the classes change and the CDN assets are injected. A
    // document-set `source-highlighter` is honored only below the SERVER safe
    // mode (the renderer half of the safe-mode lock, #215), so this conversion
    // runs under `SafeMode::Safe`.
    // Standalone output so the highlighter's `<head>`/footer CDN assets appear.
    let hl = crate::convert_with(
        "= T\n:source-highlighter: highlight.js\n\n[,ruby]\n----\nputs 1\n----\n",
        &Options::new().safe_mode(SafeMode::Safe).standalone(true),
    );
    assert_css(
        &hl,
        "pre.highlightjs.highlight > code.language-ruby.hljs",
        1,
    );
    assert!(hl.contains("cdnjs.cloudflare.com/ajax/libs/highlight.js"));

    // The header snippets are displayed verbatim as listing blocks.
    let display = convert("----\n= Document Title\n:source-highlighter: <value>\n----\n");
    assert_css(&display, "div.listingblock > div.content > pre", 1);
    assert!(display.contains(":source-highlighter: &lt;value&gt;"));
}

// The recognized `source-highlighter` values and the toolchains that support
// them form a support matrix for CodeRay, highlight.js, Pygments, and Rouge —
// external libraries whose installation and availability are outside this
// crate. The client-side CDN loading described here is verified above.
non_normative!(
    r#"
[#available-source-highlighters]
== Available source highlighters

<<built-in-values>> lists the recognized values for the `source-highlighter` attribute and the toolchains that support the usage of the syntax highlighting libraries.

.Built-in source-highlighter values and the supporting toolchains
[#built-in-values%autowidth]
|===
|Library |Value |Toolchain

|CodeRay
|`coderay`
|Asciidoctor, AsciidoctorJ, Asciidoctor PDF

|highlight.js
|`highlight.js`
|Asciidoctor, AsciidoctorJ, Asciidoctor.js

|Pygments
|`pygments`
|Asciidoctor, Asciidoctor PDF

|Rouge
|`rouge`
|Asciidoctor, AsciidoctorJ, Asciidoctor PDF
|===

To use Rouge, CodeRay, or Pygments, you must have the appropriate library installed on your system.
See xref:asciidoctor:syntax-highlighting:rouge.adoc[], xref:asciidoctor:syntax-highlighting:coderay.adoc[], or xref:asciidoctor:syntax-highlighting:pygments.adoc[] for installation instructions.

If you're using the client-side library xref:asciidoctor:syntax-highlighting:highlightjs.adoc[], there's no need to install additional libraries.
The generated HTML will load the required source files from a CDN, custom URL, or file path.

"#
);

// A terminology sidebar distinguishing "source highlighter" (the attribute)
// from "syntax highlighter" (the library); explanatory prose with no rendered
// behavior of its own.
non_normative!(
    r#"
.Source Highlighter vs. Syntax Highlighter
****
You might notice that the `source-highlighter` attribute uses the term "`source highlighter`", whereas the library that performs the highlighting is referred to as a "`syntax highlighter`".
What's the difference?

* The generally accepted term for a syntax (aka code) highlighter is "`syntax highlighter`".
* The syntax highlighter is applied to source blocks in AsciiDoc, hence why we say "`source highlighter`".

In other words, the `source-highlighter` attribute means "`use this syntax highlighter to colorize source blocks`".
****

"#
);

// The applicability rules (a language is required; a paragraph or literal must
// also carry the `source` style; language values are defined by the highlighter
// library) are descriptive. The concrete cases — an implied source block and a
// source paragraph — are verified in the two tests that follow.
non_normative!(
    r#"
== Apply source highlighting

To apply highlighting to a block of source code, you must specify a source language.
If the block is a literal block or paragraph, you must also specify the `source` style.

The AsciiDoc language does not specify the list of valid source language values.
Instead, the available source language values are defined by the syntax highlighter library.

TIP: You can find the list of available languages supported by Rouge in the https://github.com/rouge-ruby/rouge/blob/master/docs/Languages.md[Rouge documentation].
You can print a list of available languages supported by Pygments by running `pygmentize -L formatters`.
The available languages supported by highlight.js depends on which bundle of highlight.js you are using.

Typically, the source language value is the proper name of the language in lowercase (e.g., `ruby`, `java`).
Most syntax highlighters also accept using the source file extension (e.g., `js`, `rb`), though it's important to be consistent.
If the syntax highlighter doesn't recognize or support the source language, the block will not be highlighted.

"#
);

// The `ex-code` example is a syntax display whose content carries automatic
// callouts (`<.>`), so the display renders as a source-styled listing (id
// `ex-code`) with numbered conums followed by a four-item callout list. The
// rendered result (`src-base-co-res`) is a `ruby` source block carrying the
// `hello` id from the `[#hello,ruby]` shorthand — the `source` style implied by
// the language.
#[test]
fn implied_source_block_carries_id_and_callouts() {
    verifies!(
        r#"
.Source block with ID and source highlighting
[source#ex-code,line-comment=]
....
include::example$source.adoc[tag=src-base-co]
....
<.> The block style `source` is implied since a source language is specified.
<.> An optional ID can be added to the block by appending it to style using the shorthand syntax (`#`) for `id`.
<.> Assign a source language to the second position.
<.> An implicit source block uses the listing structural container.

The result of <<ex-code>> is displayed below.

include::example$source.adoc[tag=src-base-co-res]

"#
    );

    // The `ex-code` syntax display: the `<.>` callouts in the shown source
    // render as conums, and the four annotations become a callout list.
    let display = convert(
        "[source#ex-code,line-comment=]\n....\n[#hello,ruby] <.> <.> <.>\n---- <.>\nrequire 'sinatra'\n----\n....\n<.> The block style source is implied.\n<.> An optional ID can be added.\n<.> Assign a source language.\n<.> An implicit source block uses the listing structural container.\n",
    );
    assert_css(&display, "div#ex-code.listingblock", 1);
    assert_css(&display, "pre.highlight > code > b.conum", 4);
    assert_css(&display, "div.colist.arabic > ol > li", 4);

    // The rendered result (`src-base-co-res`): a `ruby` source block with the
    // `hello` id — the `source` style is implied by the language alone.
    let result = convert(
        "[#hello,ruby]\n----\nrequire 'sinatra'\n\nget '/hi' do\n  \"Hello World!\"\nend\n----\n",
    );
    assert_css(
        &result,
        r#"div#hello.listingblock > div.content > pre.highlight > code.language-ruby"#,
        1,
    );
}

// A source *paragraph* requires the explicit `source` style (the comma
// shorthand alone does not promote a paragraph). `[source,xml]` above a
// paragraph produces an `xml` source block that ends at the blank line; the
// following text is a normal paragraph.
#[test]
fn source_paragraph_requires_the_source_style() {
    verifies!(
        r#"
.Source paragraph
[source#ex-style]
----
include::example$source.adoc[tag=src-para-co]
----
<.> Place the attribute list directly above the paragraph.
In this case, the `source` style is always required.
<.> Once an empty line is encountered the source block ends.

The result of <<ex-style>> is displayed below.

include::example$source.adoc[tag=src-para]

"#
    );

    // The `src-para` snippet: a source paragraph followed by a normal paragraph.
    let output = convert(
        "[source,xml]\n<meta name=\"viewport\"\n  content=\"width=device-width, initial-scale=1.0\">\n\nThis is normal content.\n",
    );

    // The source paragraph is an `xml` source block; it ends at the blank line.
    assert_css(
        &output,
        r#"div.listingblock > div.content > pre.highlight > code.language-xml"#,
        1,
    );
    assert!(output.contains("&lt;meta name=\"viewport\""));

    // The text after the blank line is a normal paragraph, not part of the block.
    assert_xpath(
        &output,
        r#"//div[@class="paragraph"]/p[text()="This is normal content."]"#,
        1,
    );
}

// The distinction between the `shell` and `console` languages is guidance for
// the syntax highlighter and does not affect this crate's rendering.
non_normative!(
    r#"
=== shell vs console

The source language for shell and console are often mixed up.
The language `shell` is intended for the contents of a shell script, often indicated by a shebang for the generic shell.
If the shell script is written for a particular shell, you might use that language instead (e.g., `bash` or `zsh`).
The language `console` is intended to represent text that's typed into a console (i.e., a terminal application).

"#
);

// The two `shell`/`console` examples are syntax displays: `[source]` blocks
// delimited by `....`, each rendered verbatim as a source-styled listing.
#[test]
fn shell_and_console_examples_are_shown_verbatim() {
    verifies!(
        r#"
Here's an example of when you would use `shell`:

[source]
....
[,shell]
----
#!/bin/sh

fail () {
    echo
    echo "$*"
    echo
    exit 1
} >&2

JAVACMD=java
which java >/dev/null 2>&1 || fail "ERROR: no 'java' command could be found in your PATH.

exec "$JAVACMD" "$@"
----
....

Here's an example of when you would use `console`:

[source]
....
[source,console]
$ asciidoctor -v
....

"#
    );

    let shell = convert("[source]\n....\n[,shell]\n----\n#!/bin/sh\n\nJAVACMD=java\n----\n....\n");
    assert_css(&shell, "div.listingblock > div.content > pre", 1);
    assert!(shell.contains("[,shell]\n----\n#!/bin/sh"));

    let console = convert("[source]\n....\n[source,console]\n$ asciidoctor -v\n....\n");
    assert_css(&console, "div.listingblock > div.content > pre", 1);
    assert!(console.contains("[source,console]\n$ asciidoctor -v"));
}

// How the highlighter treats a console prompt, and the advice to use a literal
// paragraph for simple console commands, are highlighter/authoring notes with
// no observable effect here.
non_normative!(
    r#"
Typically, the syntax highlighter will parse the prompt (e.g., `$`) at the start of each line, then handle the remaining text using the shell language.

Often times, a basic console command is represented using a literal paragraph since there isn't much to be gained from syntax highlighting in this case.

"#
);

// Line numbering is added by the syntax highlighter, not the converter, so the
// `linenums` option produces no observable line numbers in this crate (and
// highlight.js — the client-side highlighter modelled here — does not support
// it, matching Asciidoctor).
non_normative!(
    r#"
== Enable line numbering

Provided the feature is supported by the source highlighter, you can enable line numbering on a source block by setting the `linenums` option on the block.

IMPORTANT: Line numbering is added by the syntax highlighter, not the AsciiDoc converter.
Therefore, to get line numbering on a source block, you must have the `source-highlighter` attribute set and the library to which it refers must support line numbering.
When using Asciidoctor, the only syntax highlighter that does not support line numbering is highlight.js.

The `linenums` option can either be specified as a normal block option named `linenums`, or as the third positional attribute on the block.
The value of the positional attribute doesn't matter, though it's customary to use `linenums`.

"#
);

// The two line-numbering examples are syntax displays, rendered verbatim as
// source-styled listings carrying their block ids.
#[test]
fn linenums_examples_are_shown_verbatim() {
    verifies!(
        r#"
.Enable line numbering using the `linenums` option
[source#ex-linenums-option]
....
include::example$source.adoc[tag=linenums-option]
....

.Enable line numbering using the third positional attribute
[source#ex-linenums-posattr]
....
include::example$source.adoc[tag=linenums-posattr]
....

// We can't show the output since the source highlighter used for the site (highlight.js) isn't configure to support line numbering.
////
The result of both <<ex-linenums-option>> and <<ex-linenums-posattr>> is displayed below:

include::example$source.adoc[tag=linenums-option]
////

"#
    );

    // The `linenums-option` snippet, displayed with the `ex-linenums-option` id.
    let option = convert(
        "[source#ex-linenums-option]\n....\n[%linenums,ruby]\n----\nputs 1\nputs 2\nputs 3\n----\n....\n",
    );
    assert_css(
        &option,
        "div#ex-linenums-option.listingblock > div.content > pre",
        1,
    );
    assert!(option.contains("[%linenums,ruby]"));

    // The `linenums-posattr` snippet, displayed with the `ex-linenums-posattr` id.
    let posattr = convert(
        "[source#ex-linenums-posattr]\n....\n[,ruby,linenums]\n----\nputs 1\nputs 2\nputs 3\n----\n....\n",
    );
    assert_css(
        &posattr,
        "div#ex-linenums-posattr.listingblock > div.content > pre",
        1,
    );
    assert!(posattr.contains("[,ruby,linenums]"));
}

// Source highlighting is disabled for a block by specifying the language `text`
// (which still produces a `language-text` source block, but with nothing to
// colorize) or by removing the `source` style (leaving a plain listing block).
#[test]
fn highlighting_is_disabled_by_text_language_or_removing_source_style() {
    verifies!(
        r#"
== Disable source highlighting

To disable source highlighting for a given source block, specify the language as `text` or remove the `source` style.

"#
    );

    // Language `text`: a source block whose `code` is `language-text` (with
    // nothing a highlighter would colorize).
    let text = convert("[source,text]\n----\nputs 1\n----\n");
    assert_css(&text, "code.language-text", 1);

    // Removing the `source` style leaves a plain listing block: no `code`.
    let listing = convert("[listing]\n----\nputs 1\n----\n");
    assert_css(&listing, "div.listingblock > div.content > pre", 1);
    assert_xpath(&listing, "//pre/code", 0);
}

// The `source-language` document attribute supplies a default language and
// promotes bare `----` listings to source blocks. Setting it makes a bare
// listing an `xml`/`java` source block without a `source` style or language on
// the block; adding the `listing` style opts back out; a language on the block
// overrides the document default. The two examples are also shown verbatim.
#[test]
fn source_language_attribute_promotes_listings() {
    verifies!(
        r#"
[#source-language]
== source-language attribute

If the majority of your source blocks use the same source language, you can set the `source-language` attribute in the document header and assign a language to it.
Setting the `source-language` document attribute implicitly promotes listing blocks to source blocks.

.Set source-language attribute
[source#ex-language]
....
include::example$source.adoc[tag=src-lang]
....

Notice that it's not necessary to specify the `source` style or source language on the block.
To make a listing block in this situation, you must set the `listing` style on the block.

You can override the global source language on an individual block by specifying a source language directly on the block.

.Override source-language attribute
[source#ex-override]
....
include::example$source.adoc[tag=override]
....
"#
    );

    // The `src-lang` document: a bare `----` listing is promoted to a `java`
    // source block by the `source-language` attribute, with no style or
    // language of its own. (Pygments is server-side and not run here, so the
    // block renders in the default unhighlighted source shape.)
    let promoted = convert(
        "= Document Title\n:source-highlighter: pygments\n:source-language: java\n\n----\npublic void setAttributes(Attributes attributes) {\n    this.options.put(ATTRIBUTES, attributes.map());\n}\n----\n",
    );
    assert_css(&promoted, "pre.highlight > code.language-java", 1);

    // Adding the `listing` style opts back out to a plain listing block.
    let opted_out = convert("= T\n:source-language: java\n\n[listing]\n----\nx\n----\n");
    assert_css(&opted_out, "div.listingblock > div.content > pre", 1);
    assert_xpath(&opted_out, "//pre/code", 0);

    // A language declared on the block overrides the document default.
    let overridden = convert("= T\n:source-language: java\n\n[,ruby]\n----\nx\n----\n");
    assert_css(&overridden, "pre.highlight > code.language-ruby", 1);
    assert_xpath(&overridden, "//code[@data-lang=\"java\"]", 0);

    // The `ex-language` syntax display renders verbatim with its block id.
    let display = convert(
        "[source#ex-language]\n....\n= Document Title\n:source-language: java\n\n----\npublic void x() {}\n----\n....\n",
    );
    assert_css(
        &display,
        "div#ex-language.listingblock > div.content > pre",
        1,
    );
    assert!(display.contains(":source-language: java"));
}
