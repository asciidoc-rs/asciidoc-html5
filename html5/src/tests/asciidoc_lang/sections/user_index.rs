//! Coverage of the AsciiDoc language description's *Index* page.
//!
//! Index terms form a controlled vocabulary for navigating a document. The
//! built-in HTML5 converter does not generate an index catalog (only
//! Asciidoctor PDF and the DocBook toolchain do), so the `[index]` seed section
//! and catalog population produce no HTML here and are tracked non-normatively.
//! The one behavior this backend does implement – the visibility of the two
//! index-term forms – is verified: a flow index term is rendered as visible
//! text, while a concealed index term (and the equivalent macros) produces no
//! visible output, matching Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/user-index.adoc");

// The Index catalog is not produced by the built-in HTML5 converter (only
// Asciidoctor PDF and the DocBook toolchain generate one), so the `[index]`
// seed section and its automatic population have no HTML output to verify here.
// This prose is descriptive.
non_normative!(
    r#"
= Index
:page-aliases: index.adoc

You can mark index terms explicitly in AsciiDoc content.
Index terms form a controlled vocabulary that can be used to navigate the document by keyword starting from an index.

== Index catalog

NOTE: Although index terms are always processed, only Asciidoctor PDF and the DocBook toolchain support creating an index catalog automatically.
The built-in HTML5 converter in Asciidoctor does not generate an index.

To create an index, define a level 1 section (`==`) marked with the style `index` at the end of your document.
(In a multipart book, the index can be the last level 0 section (`=`)).

[source]
----
[index]
== Index
----

Both Asciidoctor PDF and the DocBook toolchain will automatically populate an index into this seed section.

The index will consist of term entries that link to (or otherwise cite) the location of each marked index term.
You learn how to mark an index term in the next section.

"#
);

// Descriptive prose introducing the index-term section; it states that every
// occurrence must be marked but describes no HTML rendering rule.
non_normative!(
    r#"
== Index terms

Every index term, as well as every occurrence of that index term, must be explicitly marked in the AsciiDoc document.
It's not enough just to mark the first occurrence of an index term if you want every occurrence to appear in the index.
Instead, each occurrence you want to be cited in the index must be marked explicitly.

"#
);

// The two index-term forms differ in visibility: a flow index term (double
// parenthesis `((primary))` or the `indexterm2:[...]` macro) appears in the
// flow of text as a visible term, while a concealed index term (triple
// parenthesis `(((...)))` or the `indexterm:[...]` macro) appears only in the
// index and produces no visible output. This crate matches Asciidoctor 2.0.26
// for both.
#[test]
fn flow_terms_are_visible_and_concealed_terms_are_hidden() {
    verifies!(
        r#"
There are two types of index terms in AsciiDoc:

flow index term:: `\indexterm2:[<primary>]` +
`+((<primary>))+`
+
An index term that appears in the flow of text (i.e., a visible term) and in the index.
This type of index term can only be used to define a primary entry and is case sensitive.
If you want the entry to appear in the index using a different case, use an adjacent concealed index term, such as `+(((term)))Term+`.

concealed index term:: `\indexterm:[<primary>, <secondary>, <tertiary>]` +
`+(((<primary>, <secondary>, <tertiary>)))+`
+
A group of index terms that appear only in the index.
This type of index term can be used to define a primary entry as well as optional secondary and tertiary entries.

"#
    );

    // The double-parenthesis flow term is visible; the triple-parenthesis
    // concealed term produces no visible output.
    let html = convert("I, ((Arthur)), was to carry Excalibur(((Sword, Broadsword, Excalibur))).");

    assert!(html.contains("<p>I, Arthur, was to carry Excalibur.</p>"));
    assert!(!html.contains("Broadsword"));

    // The `indexterm2` macro mirrors the flow form (visible); the `indexterm`
    // macro mirrors the concealed form (hidden).
    let macros = convert(
        "indexterm2:[Lancelot] was one of the Knights of the Round Table.\nindexterm:[knight, Knight of the Round Table, Lancelot]",
    );

    assert!(macros.contains("Lancelot was one of the Knights of the Round Table."));
    assert!(!macros.contains("Knight of the Round Table"));
}

// The remainder of the page is descriptive: a source-display example of the two
// forms in use (with callouts), the rule for quoting a comma inside a concealed
// term, and placement guidance for hidden index terms shown as literal listing
// blocks. None of it introduces an HTML rendering rule beyond the term
// visibility verified above, and the index catalog itself is not produced by
// the HTML5 backend.
non_normative!(
    r#"
Here's an example that shows the two forms in use.

[source]
----
The Lady of the Lake, her arm clad in the purest shimmering samite,
held aloft Excalibur from the bosom of the water,
signifying by divine providence that I, ((Arthur)), <.>
was to carry Excalibur(((Sword, Broadsword, Excalibur))). <.>
That is why I am your king. Shut up! Will you shut up?!
Burn her anyway! I'm not a witch.
Look, my liege! We found them.

indexterm2:[Lancelot] was one of the Knights of the Round Table. <.>
indexterm:[knight, Knight of the Round Table, Lancelot] <.>
----
<.> The double parenthesis form adds a primary index term and includes the term in the generated output.
<.> The triple parenthesis form allows for an optional second and third index term and _does not_ include the terms in the generated output (i.e., concealed index term).
<.> The inline macro `\indexterm2:[primary]` is equivalent to the double parenthesis form.
<.> The inline macro `\indexterm:[primary, secondary, tertiary]` is equivalent to the triple parenthesis form.

If you're defining a concealed index term (i.e., the `indexterm` macro), and one of the terms contains a comma, you must surround that segment in double quotes so the comma is treated as content.
For example:

[source]
----
I, King Arthur.
indexterm:[knight, "Arthur, King"]
----

or

[source]
----
I, King Arthur.
(((knight, "Arthur, King")))
----

//Follow https://github.com/asciidoctor/asciidoctor/issues/450[Asciidoctor issue #450] to track the progress of this feature.

== Placement of hidden index terms

Hidden index entries should be directly adjacent to the paragraph content to which they apply.
<<ex-hidden-terms-correct>> shows where to place hidden index terms for a paragraph.

.Correct
[#ex-hidden-terms-correct]
----
=== Create a new Git repository

(((Repository, create)))
(((Create Git repository)))
To create a new git repository,
----

If the terms are offset from the paragraph content by an empty line, it will cause an empty paragraph to be created in the parsed document, thus leaving extra space in the generated output.
<<ex-hidden-terms-incorrect-1>> and <<ex-hidden-terms-incorrect-2>> show where you should not place hidden index terms for a paragraph.

.Incorrect
[#ex-hidden-terms-incorrect-1]
----
=== Create a new Git repository

(((Repository, create)))
(((Create Git repository)))

To create a new git repository,
----

.Also incorrect
[#ex-hidden-terms-incorrect-2]
----
=== Create a new Git repository
(((Repository, create)))
(((Create Git repository)))

To create a new git repository,
----
"#
);
