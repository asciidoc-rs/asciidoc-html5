//! Coverage of the AsciiDoc language description's *Icon Macro* page.
//!
//! The inline `icon:<target>[<attrlist>]` macro is resolved by
//! `asciidoc-parser` and emitted verbatim by this crate, so its rendered output
//! matches Asciidoctor 2.0.26 exactly. Every icon mode (text, image, font) and
//! every documented attribute (`role`, `title`, `link`/`window`, `alt`/`width`,
//! `size`/`rotate`/`flip`) is verified through `convert`. The DocBook converter
//! output and the missing-image fallback (which needs filesystem access) are
//! non-normative.

use crate::{convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/icon-macro.adoc");

// The image- and font-mode examples enable icons from the *document* header
// (`:icons:` / `:icons: font`), which only takes effect below the `Secure` safe
// mode: under `Secure` (the API default) Asciidoctor drops a document-set
// `icons` so an untrusted document cannot steer icon image sources through
// `iconsdir` (see #50). They therefore convert under `Server` — the highest
// mode that still lets the document enable icons. The text-mode examples set no
// `icons`, so they render identically here (an icon macro with icons off yields
// its alt text regardless of the safe mode).
fn convert(source: &str) -> String {
    convert_with(source, &Options::new().safe_mode(SafeMode::Server))
}

non_normative!(
    r#"
= Icon Macro

In addition to built-in icons, you can add icons anywhere in your content where macros are substituted using the icon macro.
This page covers the anatomy of the icon macro, how the target is resolved, and what features it support (subject to the icon mode).

"#
);

non_normative!(
    r#"
== Anatomy

The icon macro is an inline macro.
Like other inline macros, its syntax follows the familiar pattern of the macro name and target separated by a colon followed by an attribute list enclosed in square brackets.

[source]
----
icon:<target>[<attrlist>]
----

The `<target>` is the icon name or path.
The `<attrlist>` specifies various named attributes to configure how the icon is displayed.

"#
);

// The anatomy example. With `icons` unset (the default), the icon renders as
// bracketed fallback text carrying the role on its wrapping `<span>`.
#[test]
fn anatomy_example() {
    verifies!(
        r#"
For example:

[source]
----
icon:heart[2x,role=red]
----

"#
    );

    let output = convert("icon:heart[2x,role=red]");
    assert!(output.contains(r#"<span class="icon red">[heart&#93;</span>"#));
}

// The tags example. The spec's "not set or empty" wording is loose: with the
// `icons` attribute genuinely unset the icon renders as bracketed fallback text
// (see `resolves_by_mode` below); the `<img>` output shown here is produced
// when `icons` is *empty* (image mode). Asciidoctor uses `class="icon"` (as
// verified) rather than the `class="image"` printed in this spec example.
#[test]
fn tags_example() {
    verifies!(
        r#"
== Example

Here's an example that shows how to inserts an icon named _tags_ in front of a list of tag names.

[source]
----
icon:tags[] ruby, asciidoctor
----

Here's how the HTML converter converts an icon macro when the `icons` attribute is not set or empty.

.Result: HTML output
[source,html]
----
<div class="paragraph">
  <p><span class="image"><img src="./images/icons/tags.png" alt="tags"></span> ruby, asciidoctor</p>
</div>
----

"#
    );

    let output = convert(":icons:\n\nicon:tags[] ruby, asciidoctor");
    assert!(output.contains(
        r#"<span class="icon"><img src="./images/icons/tags.png" alt="tags"></span> ruby, asciidoctor"#
    ));
}

// The DocBook converter is not implemented by this crate, so the
// `<inlinemediaobject>` rendering is not verified. Likewise, the fallback when
// an icon image can't be located needs filesystem access this crate leaves to
// the caller.
non_normative!(
    r#"
Here's how the DocBook converter converts an icon macro.

[source,xml]
----
<inlinemediaobject>
  <imageobject>
    <imagedata fileref="./images/icons/tags.png"/>
  </imageobject>
  <textobject><phrase>tags</phrase></textobject>
</inlinemediaobject> ruby, asciidoctor
----

When the image for an icon can't be located in the icons directory, the AsciiDoc processor displays the icon macro's alt (i.e. fallback) text.

"#
);

// The three icon modes resolve the target differently: text mode brackets the
// name, image mode resolves it to a file in `iconsdir`, and font mode maps it
// to a `fa fa-<name>` glyph class.
#[test]
fn resolves_by_mode() {
    verifies!(
        r#"
== How the icon is resolved

The target of the icon macro is an icon name (or path).
How that target is resolved depends on the icon mode assigned to the xref:icons.adoc#icons-attribute[icons attribute].

text::
The icon name will be enclosed in square brackets (e.g., `[heart]`).

image::
The icon name will be resolved to a file in the `iconsdir` with the file extension specified by `icontype` (e.g., [.path]_./images/icons/heart.png_).

font::
The icon name will be resolved to a glyph in an icon font (as mapped by a CSS class) (e.g., `fa fa-heart`).

WARNING: If you include a file extension in the image target, the icon macro will not work correctly when using the font icon mode (i.e., `icons=font`).

"#
    );

    // text mode (the `icons` attribute is not set): the name is bracketed (the
    // `]` is emitted as `&#93;`).
    assert!(convert("icon:heart[]").contains(r#"<span class="icon">[heart&#93;</span>"#));

    // image mode: the name resolves to a file in `iconsdir` with the `icontype`
    // extension (default `png`).
    assert!(convert(":icons: image\n\nicon:heart[]")
        .contains(r#"<span class="icon"><img src="./images/icons/heart.png" alt="heart"></span>"#));

    // font mode: the name resolves to a `fa fa-<name>` glyph class.
    assert!(convert(":icons: font\n\nicon:heart[]")
        .contains(r#"<span class="icon"><i class="fa fa-heart"></i></span>"#));

    // WARNING: a file extension in the target breaks font icon mode, because the
    // extension is carried into the glyph class (`fa-heart.png`).
    assert!(convert(":icons: font\n\nicon:heart.png[]")
        .contains(r#"<span class="icon"><i class="fa fa-heart.png"></i></span>"#));
}

// The shared attributes (`role`, `title`, `link`, `window`). `role`, `link`,
// and `window` are demonstrated in their own examples below; here `title` adds
// a `title` attribute to the rendered image.
#[test]
fn shared_attributes_title() {
    verifies!(
        r#"
== Icon macro attributes (shared)

The following attributes of the icon macro are shared for all icon modes.

`role`::
The role applied to the element that surrounds the icon.

`title`::
The title of the image displayed when the mouse hovers over it.

`link`::
The URI target used for the icon, which will wrap the converted icon in a link.

`window`::
The target window of the link (when the `link` attribute is specified).

"#
    );

    assert!(convert(":icons: image\n\nicon:tags[title=Tags]").contains(
        r#"<span class="icon"><img src="./images/icons/tags.png" alt="tags" title="Tags"></span>"#
    ));
}

// `role` is appended to the class of the `<span>` surrounding the icon.
#[test]
fn role() {
    verifies!(
        r#"
=== Role

Here's an example of an icon that uses a role to specify the color.

[source]
----
icon:tags[role=blue] ruby, asciidoctor
----

"#
    );

    assert!(convert(":icons: image\n\nicon:tags[role=blue] ruby, asciidoctor").contains(
        r#"<span class="icon blue"><img src="./images/icons/tags.png" alt="tags"></span> ruby, asciidoctor"#
    ));
}

// `link` wraps the icon in an `<a class="image">`; `window=_blank` sets the
// link's `target` and adds `rel="noopener"`.
#[test]
fn link_and_window() {
    verifies!(
        r#"
=== Link and window

Here's an example of an icon with a link that targets a separate window:

[source]
----
icon:download[link=https://rubygems.org/downloads/whizbang-1.0.0.gem, window=_blank]
----

"#
    );

    let output = convert(
        ":icons: image\n\nicon:download[link=https://rubygems.org/downloads/whizbang-1.0.0.gem, window=_blank]",
    );
    assert!(output.contains(concat!(
        r#"<span class="icon">"#,
        r#"<a class="image" href="https://rubygems.org/downloads/whizbang-1.0.0.gem" target="_blank" rel="noopener">"#,
        r#"<img src="./images/icons/download.png" alt="download"></a></span>"#,
    )));
}

// The image-mode-only attributes `alt` and `width`.
#[test]
fn image_mode_alt_and_width() {
    verifies!(
        r#"
== Icon macro attributes (image mode only)

The attributes listed below only apply when using the image icon mode.

`alt`::
The alternative text on the `<img>` element (HTML output) or text of `<inlinemediaobject>` element (DocBook output)

`width`::
The width applied to the image.

For example, here's how to control the icon alt text and width when using the image icon mode:

[source]
----
icon:tags[Tags,width=16] ruby, asciidoctor
----

"#
    );

    // The `alt` named attribute sets the `<img>` alt text in image mode.
    assert!(convert(":icons: image\n\nicon:tags[alt=Tags]")
        .contains(r#"<span class="icon"><img src="./images/icons/tags.png" alt="Tags"></span>"#));

    // The `width` named attribute sets the `<img>` width.
    assert!(convert(":icons: image\n\nicon:tags[width=16]").contains(
        r#"<span class="icon"><img src="./images/icons/tags.png" alt="tags" width="16"></span>"#
    ));

    // The icon macro's sole positional attribute is `size` (a font-mode
    // attribute), so in `icon:tags[Tags,width=16]` the leading `Tags` is parsed
    // as `size`, not `alt`. Since `size` has no effect in image mode, the alt
    // text falls back to the target-derived default (`tags`) while `width=16`
    // applies.
    assert!(convert(":icons: image\n\nicon:tags[Tags,width=16] ruby, asciidoctor").contains(
        r#"<span class="icon"><img src="./images/icons/tags.png" alt="tags" width="16"></span> ruby, asciidoctor"#
    ));
}

non_normative!(
    r#"
The icon macro doesn't support any options to change its physical position (such as alignment).

"#
);

// The font-mode-only attributes `size`, `rotate`, and `flip`.
#[test]
fn font_mode_size_rotate_flip() {
    verifies!(
        r#"
== Icon macro attributes (font mode only)

The icon macro has a few attributes that modify the size and orientation of a font-based icon.
These attributes are only recognized in the font icon mode.

`size`::
First positional attribute; scales the icon; values: `1x` (default), `2x`, `3x`, `4x`, `5x`, `lg`, `fw`

`rotate`::
Rotates the icon; values: `90`, `180`, `270`

`flip`::
Flips the icon; values: `horizontal`, `vertical`

"#
    );

    // `size` scales the glyph via a `fa-<size>` class.
    assert!(convert(":icons: font\n\nicon:heart[2x]")
        .contains(r#"<span class="icon"><i class="fa fa-heart fa-2x"></i></span>"#));

    // `rotate` rotates the glyph via a `fa-rotate-<deg>` class.
    assert!(convert(":icons: font\n\nicon:shield[rotate=90]")
        .contains(r#"<span class="icon"><i class="fa fa-shield fa-rotate-90"></i></span>"#));

    // `flip` flips the glyph via a `fa-flip-<dir>` class.
    assert!(convert(":icons: font\n\nicon:shield[flip=vertical]")
        .contains(r#"<span class="icon"><i class="fa fa-shield fa-flip-vertical"></i></span>"#));
}

// The positional (`2x`) and named (`size=2x`) size forms are equivalent.
#[test]
fn font_mode_size() {
    verifies!(
        r#"
=== Size

To make the icon twice the size as the default, enter `2x` inside the square brackets.

[source]
----
icon:heart[2x]
----

or

[source]
----
icon:heart[size=2x]
----

"#
    );

    let expected = r#"<span class="icon"><i class="fa fa-heart fa-2x"></i></span>"#;
    assert!(convert(":icons: font\n\nicon:heart[2x]").contains(expected));
    assert!(convert(":icons: font\n\nicon:heart[size=2x]").contains(expected));
}

// The `fw` (fixed-width) size adds the `fa-fw` class.
#[test]
fn font_mode_fixed_width() {
    verifies!(
        r#"
[TIP]
====
If you want to line up icons so that you can use them as bullets in a list, use the `fw` size as follows:

----
[%hardbreaks]
icon:bolt[fw] bolt
icon:heart[fw] heart
----
====

"#
    );

    assert!(convert(":icons: font\n\nicon:bolt[fw] bolt")
        .contains(r#"<span class="icon"><i class="fa fa-bolt fa-fw"></i></span> bolt"#));
    assert!(convert(":icons: font\n\nicon:heart[fw] heart")
        .contains(r#"<span class="icon"><i class="fa fa-heart fa-fw"></i></span> heart"#));
}

// When both `rotate` and `flip` are supplied, `flip` takes precedence (matching
// Asciidoctor, which selects the flip class and ignores the rotate class).
#[test]
fn font_mode_rotate_and_flip() {
    verifies!(
        r#"
=== Rotate and flip

To rotate and flip an icon, specify these options using named attributes:

[source]
----
icon:shield[rotate=90, flip=vertical]
----
"#
    );

    let output = convert(":icons: font\n\nicon:shield[rotate=90, flip=vertical]");
    assert!(output
        .contains(r#"<span class="icon"><i class="fa fa-shield fa-flip-vertical"></i></span>"#));
    assert!(!output.contains("fa-rotate-90"));
}
