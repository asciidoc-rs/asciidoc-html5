//! Coverage of the AsciiDoc language description's *Cross Reference Text and
//! Styles* page.
//!
//! By default a cross reference's text is the target's reftext (its title, or
//! an explicit `reftext`). The `xrefstyle` attribute (`full`/`short`/`basic`)
//! restyles a numbered reference — signifier + number + quoted/emphasized title
//! — and the `<type>-refsig` attributes customize (or drop) the signifier word.
//! All verified through `convert`, matching Asciidoctor 2.0.26.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/xref-text-and-style.adoc");

non_normative!(
    r#"
= Cross Reference Text and Styles

You can customize the style of the automatic cross reference text using the `xrefstyle` document attribute.
This customization brings the cross reference text formatting from the DocBook toolchain to AsciiDoc processing, specifically during conversion.

CAUTION: Since this is a newer feature of the AsciiDoc language, it may not be supported by all converters.
Where you can find support for it is in Asciidoctor's HTML, PDF, and EPUB 3 converters.
It's not supported by the DocBook converter since it's a feature the DocBook toolchain already provides.

"#
);

// By default the cross reference text is the target's title; an explicit
// `reftext` attribute overrides the title.
#[test]
fn default_styling() {
    verifies!(
        r#"
== Default styling

By default, the cross reference text matches the title of the referenced element.
For example, if you're linking to a section titled “Installation”, the text of the cross reference link appears as:

====
Installation
====

If the reftext attribute is specified on the referenced element, that value is preferred over its title.
For example, let's assume the section from the previous example was written as:

[source]
----
[reftext="Installation Procedure"]
=== Installation
----

In this case, the text of the cross reference link appears as:

====
Installation Procedure
====

Attribute references are substituted in the reftext during parsing and reftext substitutions (specialchars, quotes, and replacements) are applied to the value when it's used during conversion.

If the reftext is not specified, the text of the cross reference is automatically generated.
By default, this text is the title of the reference.

"#
    );

    // Default: the link text is the section title.
    assert!(
        convert("See <<install>>.\n\n[#install]\n== Installation\n\nBody.")
            .contains(r##"<a href="#install">Installation</a>"##)
    );

    // An explicit reftext is preferred over the title.
    assert!(
        convert("See <<install>>.\n\n[#install,reftext=\"Installation Procedure\"]\n== Installation\n\nBody.")
            .contains(r##"<a href="#install">Installation Procedure</a>"##)
    );
}

non_normative!(
    r#"
== Cross reference styles

The generated text of a cross reference is controlled by the xrefstyle.
It will also vary for different element types (section, figure, etc).
Let's consider the following document to learn how the xrefstyle value affects the generated text of a cross reference.

[,asciidoc]
----
== Installation

.Big Cats
image::big-cats.png[]
----

There are three built-in styles supported by the xrefstyle document attribute that you can choose from to customize the generated text of a cross reference.

"#
);

// The three `xrefstyle` values restyle a numbered section reference: `full`
// (signifier + number + quoted title), `short` (signifier + number), and
// `basic` (title only).
#[test]
fn cross_reference_styles() {
    verifies!(
        r#"
 :xrefstyle: full:: Uses the signifier for the reference followed by the reference number and emphasized (chapter or appendix) or title enclosed in quotes (e.g., Section 2.3, “Installation”) (e.g., Figure 1, “Big Cats”).

 :xrefstyle: short:: Uses the signifier for the reference followed by the reference number (e.g., Section 2.3) (e.g., Figure 1).

 :xrefstyle: basic:: Uses the title only, only applying emphasis if the reference is a chapter or appendix (e.g., Installation) (e.g., Big Cats).

The `xrefstyle` attribute can also be specified directly on the xref:xref.adoc[xref macro] to override the xrefstyle value for a single reference (e.g., `+xref:installation[xrefstyle=short]+`).
The element attribute supports the same three styles.

The xrefstyle formatting only applies to references that have both a title and number (or explicit caption), but no explicit reftext.
If the reference is a chapter or an appendix, the title is displayed in italics instead of quotes (even when the xrefstyle is basic).

Let's assume you want to reference a section titled “Installation” that has the number 2.3.
The *full* style is displayed as:

====
Section 2.3, “Installation”
====

The *short* style is displayed as:

====
Section 2.3
====

The *basic* style is displayed as:

====
Installation
====

The *full* and *short* styles only apply for references that have a caption.
Specifically, the corresponding `<context>-caption` attribute must be set for the target's block type (e.g., `listing-caption` for listing blocks, `example-caption` for example blocks, `table-caption` for tables, etc.).
Otherwise, the *basic* style is used.

"#
    );

    // A numbered section referenced with each style (here the section number is
    // simply `1`; the page's example uses `2.3`).
    let doc = |style: &str| {
        format!(":sectnums:\n:xrefstyle: {style}\n\n== Installation\n\nSee <<Installation>>.")
    };

    // `full`: signifier, number, and the title in typographic quotes.
    assert!(convert(&doc("full"))
        .contains(r##"<a href="#_installation">Section 1, &#8220;Installation&#8221;</a>"##));

    // `short`: signifier and number only.
    assert!(convert(&doc("short")).contains(r##"<a href="#_installation">Section 1</a>"##));

    // `basic`: the title only.
    assert!(convert(&doc("basic")).contains(r##"<a href="#_installation">Installation</a>"##));
}

non_normative!(
    r#"
== Reference signifiers

You can use document attributes to customize the signifier that is placed in front of the reference's number.
This [.term]*reference signifier* indicates the reference's type (e.g., Chapter or Section).

* `chapter-refsig` -- defines the signifier to use for a cross reference to a chapter (default: Chapter)
* `section-refsig` -- defines the signifier to use for a cross reference to a section (default: Section)
* `appendix-refsig` -- defines the signifier to use for a cross reference to an appendix (default: Appendix)

(The signifier attribute for a part cross reference will be introduced once numeration is supported for parts).

"#
);

// The `<type>-refsig` attribute customizes the signifier word; unsetting it
// drops the signifier from the cross reference text.
#[test]
fn reference_signifiers() {
    verifies!(
        r#"
For example, to customize the word “Section”, define the `section-refsig` attribute in the document header:

[source]
----
:section-refsig: Sect.
----

The *full* xrefstyle would then be displayed as:

====
Sect. 2.3, “Installation”
====

The *short* xrefstyle would be displayed as:

====
Sect. 2.3
====

If you unset the attribute, the signifier is dropped from the cross reference text.
For example:

[source]
----
:!section-refsig:
----

In this case, the *full* xrefstyle will display only the number and title:

====
2.3, “Installation”
====

The *short* xrefstyle will fall back to the number only:

====
2.3
====

The *basic* xrefstyle is unaffected by the value of the signifier.

"#
    );

    // A custom `section-refsig` replaces the "Section" signifier.
    assert!(convert(
        ":sectnums:\n:section-refsig: Sect.\n:xrefstyle: full\n\n== Installation\n\nSee <<Installation>>."
    )
    .contains(r##"<a href="#_installation">Sect. 1, &#8220;Installation&#8221;</a>"##));

    // Unsetting `section-refsig` drops the signifier, leaving number and title.
    assert!(convert(
        ":sectnums:\n:!section-refsig:\n:xrefstyle: full\n\n== Installation\n\nSee <<Installation>>."
    )
    .contains(r##"<a href="#_installation">1, &#8220;Installation&#8221;</a>"##));
}

non_normative!(
    r#"
Only the aforementioned styles are provided out of the box.
Support for a custom formatting string is planned.
Refer to https://github.com/asciidoctor/asciidoctor/issues/2212[#2212^] for details.
Until then, you can implement custom formatting in a custom converter or overriding the xreftext method on the node.
"#
);
