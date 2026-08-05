//! Coverage of the AsciiDoc language description's *Admonitions* page.
//!
//! The page documents the admonition paragraph (`LABEL:` prefix) and block
//! (`[LABEL]` masquerade) forms, plus the three ways the icon cell is rendered:
//! the text label by default, a Font Awesome `<i>` under `:icons: font`, and a
//! caption glyph supplied through the `<type>-caption` attribute. This crate
//! renders all of them, so each example is verified through `convert`. The
//! title, the introduction, the type list, the caution-vs-warning aside, the
//! section headings, and the GitHub-specific emoji notes are non-normative.
//!
//! Each `verifies!` block drives the exact source of the `admonition.adoc`
//! example tag the page includes in that section (from
//! `ref/asciidoc-lang/docs/modules/blocks/examples/admonition.adoc`).

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

// Converts under `Server`, a safe mode below `Secure` that still "allows icons"
// (per the Safe Modes page). The admonition-icon examples enable icons from the
// document header (`:icons:` / `:icons: font`), which this crate's default
// `Secure` mode drops (matching Asciidoctor), so those examples
// render under `Server` to reproduce Asciidoctor's icon-permitting CLI output.
fn convert_with_icons(source: &str) -> String {
    crate::convert_with(source, &Options::new().safe_mode(SafeMode::Server))
}

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/admonitions.adoc");

non_normative!(
    r#"
= Admonitions

There are certain statements you may want to draw attention to by taking them out of the content's flow and labeling them with a priority.
These are called admonitions.
This page introduces you to admonition types AsciiDoc provides, how to add admonitions to your document, and how to enhance them using icons or emoji.

NOTE: The examples on this page (and in these docs) use a visual theme that differs from the style provided by AsciiDoc processors such as Asciidoctor.
The AsciiDoc language does not require that the admonitions be rendered using a particular style.
The only requirement is that they be offset from the main text and labeled appropriately according to their admonition type.

== Admonition types

The rendered style of an admonition is determined by the assigned type (i.e., name).
The AsciiDoc language provides five admonition types represented by the following labels:

* `NOTE`
* `TIP`
* `IMPORTANT`
* `CAUTION`
* `WARNING`

The label is specified either as the block style or as a special paragraph prefix.
The label becomes visible to the reader unless icons are enabled, in which case the icon is shown in its place.

.Caution vs. Warning
[#caution-vs-warning]
****
When choosing the admonition type, you may find yourself getting confused between "`caution`" and "`warning`" as these words are often used interchangeably.
Here's a simple rule to help you differentiate the two:

* Use *CAUTION* to advise the reader to _act_ carefully (i.e., exercise care).
* Use *WARNING* to inform the reader of danger, harm, or consequences that exist.

The word caution in this context translates into attention in French, which is often a good reference for how it should be applied.

To find a deeper analysis, see https://www.differencebetween.com/difference-between-caution-and-vs-warning/.
****

== Admonition syntax

"#
);

// The admonition paragraph: a paragraph whose first line starts with an
// uppercase label followed by a colon renders as an `admonitionblock` of that
// type, with the label in the icon cell.
#[test]
fn admonition_paragraph_prefix_renders_an_admonitionblock() {
    verifies!(
        r#"
When you want to call attention to a single paragraph, start the first line of the paragraph with the label you want to use.
The label must be uppercase and followed by a colon (`:`).

.Admonition paragraph syntax
[#ex-label]
----
include::example$admonition.adoc[tag=para-c]
----
<.> The label must be uppercase and immediately followed by a colon (`:`).
<.> Separate the first line of the paragraph from the label by a single space.

The result of <<ex-label>> is displayed below.

include::example$admonition.adoc[tag=para]

"#
    );

    let output = convert(
        "WARNING: Wolpertingers are known to nest in server racks.\nEnter at your own risk.\n",
    );
    assert_css(&output, "div.admonitionblock.warning", 1);
    assert_xpath(
        &output,
        r#"//td[@class="icon"]/div[@class="title"][text()="Warning"]"#,
        1,
    );
}

// The admonition block: an uppercase label as the style attribute on a block
// (here an example block) masquerades that block as an admonition of the given
// type, preserving its title and compound content.
#[test]
fn admonition_block_masquerade_preserves_compound_content() {
    verifies!(
        r#"
When you want to apply an admonition to compound content, set the label as a style attribute on a block.
As seen in the next example, admonition labels are commonly set on example blocks.
This behavior is referred to as *masquerading*.
The label must be uppercase when set as an attribute on a block.

.Admonition block syntax
[#ex-block]
----
include::example$admonition.adoc[tag=bl-c]
----
<.> Set the label in an attribute list on a delimited block.
The label must be uppercase.
<.> Admonition styles are commonly set on example blocks.
Example blocks are delimited by four equal signs (`====`).

The result of <<ex-block>> is displayed below.

include::example$admonition.adoc[tag=bl-nest]

"#
    );

    let output = convert(
        "[IMPORTANT]\n.Feeding the Werewolves\n======\n\
         While werewolves are hardy community members, keep in mind the following dietary concerns:\n\n\
         . They are allergic to cinnamon.\n\
         . More than two glasses of orange juice in 24 hours makes them howl in harmony with alarms and sirens.\n\
         . Celery makes them sad.\n======\n",
    );
    assert_css(&output, "div.admonitionblock.important", 1);
    assert_xpath(
        &output,
        r#"//td[@class="content"]/div[@class="title"][text()="Feeding the Werewolves"]"#,
        1,
    );
    assert_css(
        &output,
        "div.admonitionblock.important td.content div.olist ol",
        1,
    );
}

non_normative!(
    r#"
== Enable admonition icons

"#
);

// Enabling font icons: with `:icons: font` set on the document, the icon cell
// holds a Font Awesome `<i class="fa icon-<type>" title="<Label>">` instead of
// the text label.
#[test]
fn icons_font_renders_a_font_awesome_icon() {
    verifies!(
        r#"
In the examples above, the admonition is rendered in a callout box with the style label in the gutter.
You can replace the textual labels with font icons by setting the `icons` attribute on the document and assigning it the value `font`.

.Admonition paragraph with icons set
[#ex-icon]
----
= Document Title
:icons: font

include::example$admonition.adoc[tag=para]
----

"#
    );

    let output = convert_with_icons(
        "= Document Title\n:icons: font\n\n\
         WARNING: Wolpertingers are known to nest in server racks.\nEnter at your own risk.\n",
    );
    assert_xpath(
        &output,
        r#"//td[@class="icon"]/i[@class="fa icon-warning"][@title="Warning"]"#,
        1,
    );
    assert_css(&output, "td.icon div.title", 0);
}

// Enabling image icons: with `icons` set to a value other than `font` (here the
// bare `:icons:`), the icon cell holds an `<img>` of the icon file, built from
// `iconsdir` and `icontype` (defaults `./images/icons` and `png`), instead of
// the text label or a font glyph. The page only shows the `font` mode, so this
// image mode is exercised here as a regression test rather than tied to a page
// line.
#[test]
fn icons_image_mode_renders_an_img_icon() {
    let output = convert_with_icons(
        "= Document Title\n:icons:\n\n\
         WARNING: Wolpertingers are known to nest in server racks.\n",
    );
    assert_xpath(
        &output,
        r#"//td[@class="icon"]/img[@src="./images/icons/warning.png"][@alt="Warning"]"#,
        1,
    );
    assert_css(&output, "td.icon div.title", 0);
    assert_css(&output, "td.icon i.fa", 0);
}

non_normative!(
    r#"
Learn more about using Font Awesome or custom icons with admonitions in xref:macros:icons-font.adoc[].

== Using emoji for admonition icons

"#
);

// Using a caption glyph: when icons are not enabled the label comes from the
// `<type>-caption` attribute, so assigning it a Unicode glyph makes that glyph
// the admonition's icon.
#[test]
fn type_caption_attribute_supplies_the_label_glyph() {
    verifies!(
        r#"
If image-based or font-based icons are not available, you can leverage the admonition caption to display an emoji (or any symbol from Unicode) in the place of the admonition label, thus giving you an alternative way to make admonition icons.

If the `icons` attribute is not set on the document, the admonition label is shown as text (e.g., CAUTION).
The text for this label comes from an AsciiDoc attribute.
The name of the attribute is `<type>-caption`, where `<type>` is the admonition type in lowercase.
For example, the attribute for a tip admonition is `tip-caption`.

Instead of a word, you can assign a Unicode glyph to this attribute:

----
:tip-caption: 💡

[TIP]
It's possible to use Unicode glyphs as admonition icons.
----

Here's the result you get in the HTML:

[,html]
----
<td class="icon">
<div class="title">💡</div>
</td>
----

"#
    );

    let output = convert(
        ":tip-caption: 💡\n\n[TIP]\nIt's possible to use Unicode glyphs as admonition icons.\n",
    );
    assert_xpath(
        &output,
        r#"//td[@class="icon"]/div[@class="title"][text()="💡"]"#,
        1,
    );
}

// A caption glyph supplied as a character reference: the passthrough keeps the
// reference intact in the attribute value, so it lands in the label and
// resolves to the same glyph (`&#128161;` is 💡).
#[test]
fn type_caption_accepts_a_character_reference() {
    verifies!(
        r#"
Instead of entering the glyph directly, you can enter a character reference instead.
However, since you're defining the character reference in an attribute entry, you (currently) have to disable substitutions on the value.

----
:tip-caption: pass:[&#128161;]

[TIP]
It's possible to use Unicode glyphs as admonition icons.
----

"#
    );

    let output = convert(
        ":tip-caption: pass:[&#128161;]\n\n[TIP]\nIt's possible to use Unicode glyphs as admonition icons.\n",
    );
    assert_xpath(
        &output,
        r#"//td[@class="icon"]/div[@class="title"][text()="💡"]"#,
        1,
    );
}

// GitHub-specific: on GitHub, `:tip-caption: :bulb:` (guarded by an env-github
// conditional) is postprocessed into a real emoji. That pipeline is outside
// this renderer.
non_normative!(
    r#"
On GitHub, the HTML output from the AsciiDoc processor is run through a postprocessing filter that substitutes emoji shortcodes with emoji symbols.
That means you can use these shortcodes instead in the value of the attribute:

----
\ifdef::env-github[]
:tip-caption: :bulb:
\endif::[]

[TIP]
It's possible to use emojis as admonition icons on GitHub.
----

When the document is processed through the GitHub interface, the shortcodes get replaced with real emojis.
This is the only known way to get admonition icons to work on GitHub.
"#
);
