use crate::{convert_outline, convert_outline_with, load, tests::sdd::*, Document, OutlineOptions};

track_file!("ref/asciidoctor/docs/modules/api/pages/generate-html-toc.adoc");

// Asciidoctor's "Generate an HTML TOC" page, tracked from the library crate. It
// documents `convert_outline`, the HTML5 converter's method for turning a
// document's section tree into the nested `<ul class="sectlevelN">` table of
// contents — usable, the page notes, as a general-purpose API. This crate has a
// direct analog: the free function `convert_outline` (and its
// `OutlineOptions`-aware form `convert_outline_with`) produces that same list
// from a loaded `Document`.
//
// The page frames the method through several Ruby-specific ways to *reach* the
// converter — resolving one with `Asciidoctor::Converter.create`, going through
// `document.converter`, running the composite converter chain via
// `convert(document, 'outline')`, and calling it inside a Slim/Haml/ERB
// template. This crate exposes a single free function instead of a converter
// object, so those resolution idioms have no analog and are non-normative; what
// they all ultimately invoke — generating the outline HTML — is what the
// verified tests drive. The two concrete, output-bearing examples (the default
// TOC and the depth-limited `toclevels: 1` TOC) are verified against
// `convert_outline`/`convert_outline_with`.

// The sample document the page loads (`document-with-sections.adoc`): three
// top-level sections, the second carrying one subsection. Its auto-generated
// section ids are `_section_a`, `_section_b`, `_subsection`, and `_section_c`.
const SAMPLE: &str = "\
= Document Title

== Section A

== Section B

=== Subsection

== Section C
";

// The TOC the page shows for that document.
const EXPECTED_OUTLINE: &str = r##"<ul class="sectlevel1">
<li><a href="#_section_a">Section A</a></li>
<li><a href="#_section_b">Section B</a>
<ul class="sectlevel2">
<li><a href="#_subsection">Subsection</a></li>
</ul>
</li>
<li><a href="#_section_c">Section C</a></li>
</ul>"##;

// Parses the sample the page's `load_file` snippet loads. The `safe: :safe`
// mode and the file read the snippet performs bear only on attributes like
// `docfile`, which do not affect the outline, so we load the same source
// directly.
fn sample_doc() -> Document<'static> {
    load(SAMPLE)
}

non_normative!(
    r#"
= Generate an HTML TOC

Asciidoctor's HTML5 converter has a built-in method for generating an HTML TOC.
This TOC generator can also be used as a general purpose API.
This logic is available via the `convert_outline` method (which is the convert method for the `outline` node) on the HTML5 converter.

== Usage

The `convert_outline` method accepts a Document object and an optional Hash of options and it returns HTML.
It can be resolved and invoked as a general purpose method using the following snippet of code:

"#
);

// The canonical usage: load a document with sections, call `convert_outline` on
// it, and get the nested `<ul>` TOC. Asciidoctor resolves the HTML5 converter
// with `Asciidoctor::Converter.create` and calls `convert_outline` on it; this
// crate's `convert_outline` free function is the direct analog, and it produces
// the exact list the page shows.
#[test]
fn convert_outline_generates_the_html_toc() {
    verifies!(
        r##"
[,ruby]
----
document = Asciidoctor.load_file 'document-with-sections.adoc', safe: :safe
html_toc = (Asciidoctor::Converter.create 'html5').convert_outline document
puts html_toc
----

Here's an example of what this method produces:

[.output,html]
----
<ul class="sectlevel1">
<li><a href="#_section_a">Section A</a></li>
<li><a href="#_section_b">Section B</a>
<ul class="sectlevel2">
<li><a href="#_subsection">Subsection</a></li>
</ul>
</li>
<li><a href="#_section_c">Section C</a></li>
</ul>
----

"##
    );

    let doc = sample_doc();
    assert_eq!(convert_outline(&doc), EXPECTED_OUTLINE);
}

// Reaching `convert_outline` through the loaded document's converter. This is a
// Ruby resolution idiom — `document.converter` hands back the converter
// instance the document was loaded with — with no analog here, where the
// outline is a free function rather than a method on a converter object.
// Non-normative.
non_normative!(
    r#"
You can also access the `convert_outline` method on the converter instance by way of the Document API:

[,ruby]
----
document = Asciidoctor.load_file 'document-with-sections.adoc', safe: :safe
html_toc = document.converter.convert_outline document
----

"#
);

// Running the call through a composite converter chain via the generic
// `convert(document, 'outline')`. The converter chain is a Ruby extension
// mechanism this crate does not model, so this dispatch form is non-normative;
// the outline it ultimately produces is the same one the verified tests cover.
non_normative!(
    r#"
If you're using a composite converter, you can use the generic `convert` method to ensure the call is run through the converter chain.
To do so, invoke the `convert` method and pass in the Document object and the node name `outline`.
This, in turn, will call `convert_outline` on the converter in the chain that responds to this method.

[,ruby]
----
document = Asciidoctor.load_file 'document-with-sections.adoc', safe: :safe
html_doc = document.converter.convert document, 'outline'
----

"#
);

// Embedding the TOC from inside a Ruby template engine (Slim, Haml, ERB). This
// crate has no template layer, so the idiom is non-normative.
non_normative!(
    r#"
You can also use this method inside any converter template (e.g., Slim, Haml, or ERB) to generate and embed a TOC:

[,ruby]
----
= converter.convert document, 'outline'
----

"#
);

// The Options section header and the option list. `sectnumlevels` and
// `toclevels` are the two options `convert_outline` accepts, each defaulting to
// the matching document attribute; `OutlineOptions` carries the same two. The
// list itself is descriptive prose (the depth-limiting behavior is verified by
// the example that follows), so it is non-normative.
non_normative!(
    r#"
== Options

The `convert_outline` method accepts the following options:

sectnumlevels:: the number of section levels to number (defaults to the value of the `sectnumlevels` attribute.
toclevels:: the depth of the TOC (defaults to the value of the `toclevels` attribute)

Here's an example of how you can generate an HTML TOC with the depth limited to 1 for the previously loaded document:

"#
);

// Limiting the TOC depth with the `toclevels` option. Asciidoctor passes
// `toclevels: 1` in the option hash; this crate carries it on `OutlineOptions`.
// With the depth capped at 1, the subsection under Section B is dropped and
// every top-level section renders as a leaf.
#[test]
fn toclevels_option_limits_the_depth() {
    verifies!(
        r#"
[,ruby]
----
html_toc = document.converter.convert_outline document, toclevels: 1
----
"#
    );

    let doc = sample_doc();
    let expected = "\
<ul class=\"sectlevel1\">
<li><a href=\"#_section_a\">Section A</a></li>
<li><a href=\"#_section_b\">Section B</a></li>
<li><a href=\"#_section_c\">Section C</a></li>
</ul>";
    assert_eq!(
        convert_outline_with(&doc, &OutlineOptions::new().toclevels(1)),
        expected
    );
}
