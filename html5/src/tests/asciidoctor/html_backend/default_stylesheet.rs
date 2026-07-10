use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/default-stylesheet.adoc");

// Asciidoctor's "Default Stylesheet" page. It documents the stylesheet that the
// `html5` backend embeds into (or links from) a standalone document, the
// web-font `<link>` it relies on, and the attributes that control both. This
// crate produces the same standalone `<head>`, so its behavior is what we
// verify here: that a converted document carries the default stylesheet, links
// the Google web fonts, and honors `linkcss`, the safe mode's embed-vs-link
// default, `:webfonts!:`, and a custom `webfonts` value. We also verify the
// page's concrete "why a stylesheet is required" claims by checking that the
// companion CSS classes it names (built-in roles, list markers, table borders,
// TOC position) are present in the embedded stylesheet, and the `id`/`role`
// addressability claims against the block wrapper this crate emits.
//
// This page is tracked from the library crate only: every verifiable claim is
// about the HTML `<head>` that `asciidoc_html5::convert` (or `convert_with`)
// emits. Where the page drives behavior with a CLI `-a` attribute, we supply
// the same attribute through `Options`; the `adoc` binary's `-a` option is a
// thin forwarder onto that API and is covered by the CLI crate's own tests, so
// tracking this page from the CLI crate too would only duplicate the full-page
// reproduction with no independent claim to verify.
//
// The rest is non-normative here — features this crate does not implement (it
// converts a string to a string and embeds only the default stylesheet), each
// carrying no rule to verify:
// - the `@import`/custom-stylesheet "extend" recipe (custom stylesheets are
//   tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/36);
// - the `copycss` file copy (tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/39);
// - the external Asciidoctor Skins themes (out of scope; third-party).
//
// The page's docinfo recipe *is* verified: this crate injects head docinfo
// below the default stylesheet, so we supply the page's `docinfo.html`
// `<style>` through a `DocinfoFileHandler` and check its placement (issue #40).

// The renderer embeds `html5/assets/asciidoctor-default.css`; the definitive
// copy is the Asciidoctor stylesheet vendored under `ref/`. Guard against the
// two drifting apart. The `ref/` tree ships with the repository but not with
// the published crate, so this comparison is skipped when it is absent.
#[test]
fn embedded_stylesheet_matches_the_reference_copy() {
    let embedded = include_str!("../../../../assets/asciidoctor-default.css");
    let reference_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ref/asciidoctor/data/stylesheets/asciidoctor-default.css"
    );
    if let Ok(reference) = std::fs::read_to_string(reference_path) {
        assert_eq!(
            embedded, reference,
            "html5/assets/asciidoctor-default.css has drifted from the vendored ref/ copy"
        );
    }
}

/// Converts `source` under a safe mode below `Secure`, so the default
/// stylesheet is embedded inline (`<style>`) rather than linked. The default
/// (`Secure`) mode links it — the behavior asserted separately below — but the
/// stylesheet *content* is the same either way, so the CSS-content claims below
/// exercise the embed branch to read it back.
fn embed(source: &str) -> String {
    convert_with(source, &Options::new().safe_mode(SafeMode::Unsafe))
}

/// The embedded default stylesheet, as it appears inside the `<style>` element
/// of a converted standalone document. Several claims below assert that a
/// companion CSS class is present, so pull the stylesheet text out once.
fn embedded_stylesheet(html: &str) -> String {
    let start = html.find("<style>\n").expect("embedded <style>") + "<style>\n".len();
    let end = start + html[start..].find("\n</style>").expect("closing </style>");
    html[start..end].to_string()
}

non_normative!(
    r#"
= Default Stylesheet
:url-default-stylesheet: https://cdn.jsdelivr.net/gh/asciidoctor/asciidoctor@{page-component-version}/data/stylesheets/asciidoctor-default.css
:url-default-stylesheet-source: https://github.com/asciidoctor/asciidoctor/blob/v{page-component-version}.x/src/stylesheets/asciidoctor.css

When you use the HTML converter to generate a standalone HTML document, Asciidoctor includes a default stylesheet to ensure the HTML looks presentable right out of the box.
This feature gets you up and running quickly by giving you a result you can preview or publish without having to do any additional work.

This page covers why the default is necessary, how to apply it, and how to build on it so you don't have to create a stylesheet from scratch.

NOTE: The default stylesheet that Asciidoctor provides is just that, _a default_.
If you prefer a different style, you can customize it, extend it, or replace it with an entirely different one.
When replacing the default stylesheet, it's important to understand that it does provide support for numerous features in AsciiDoc, as covered in the next section.
You'll need to include these required styles when developing your own stylesheet if you want these features to continue to work.

// TODO: we probably need a page to defines what styles any stylesheet must provide to be fully compatible with AsciiDoc
== Why provide a default?

Asciidoctor includes a default stylesheet to provide a nice out-of-the-box experience when generating HTML from AsciiDoc.
But there's more to it.
There are elements of AsciiDoc that require stylesheet support.

"#
);

// A built-in role only takes effect when the stylesheet carries a companion CSS
// class. The embedded default stylesheet supplies that class — e.g. the
// `text-center` role is backed by a `.text-center` rule.
#[test]
fn built_in_roles_have_a_companion_class_in_the_stylesheet() {
    verifies!(
        r#"
One example is to honor *built-in roles*, such as `text-center`.
In order for a role to take effect, it needs a companion CSS class in the stylesheet.
To satisfy the expectations of a built-in role, a stylesheet is required.

"#
    );

    let css = embedded_stylesheet(&embed("= Doc\n\nBody."));
    assert!(css.contains(".text-center{text-align:center!important}"));
}

non_normative!(
    r#"
You may have noticed the floating anchors next to section titles when you hover over them.
Although the HTML to make them is there, it's the stylesheet that brings them to life.

"#
);

// List marker styles such as `loweralpha` are not applied by HTML on their own;
// the stylesheet provides them. The embedded stylesheet carries the matching
// `list-style-type` rule.
#[test]
fn list_marker_styles_come_from_the_stylesheet() {
    verifies!(
        r#"
Another example is to implement *list marker styles*.
AsciiDoc allows you to specify the marker for a list using a block style (e.g., `loweralpha`).
However, HTML does not apply these markers by default.
Rather, it's something that the stylesheet provides.

"#
    );

    let css = embedded_stylesheet(&embed("= Doc\n\nBody."));
    assert!(css.contains("loweralpha{list-style-type:lower-alpha}"));
}

// Table cell borders and shading (the frame/grid/stripes combinations) are
// supplied by the stylesheet; the embedded copy carries those grid rules.
#[test]
fn table_borders_and_shading_come_from_the_stylesheet() {
    verifies!(
        r#"
The default stylesheet also applies *borders and shading to table cells* to support all combinations of the frame, grid, and stripes attributes.

"#
    );

    let css = embedded_stylesheet(&embed("= Doc\n\nBody."));
    assert!(css.contains("grid-all"));
}

// Positioning the TOC as a left/right sidebar requires a page-layout change
// that only the stylesheet can make; the embedded copy carries the `toc2`
// layout.
#[test]
fn toc_position_comes_from_the_stylesheet() {
    verifies!(
        r#"
Yet another example is the *TOC position*.
To position the TOC on the left or right requires help from the stylesheet to change the layout of the page so the TOC appears as a sidebar.
It's the stylesheet that handles that task.

"#
    );

    let css = embedded_stylesheet(&embed("= Doc\n\nBody."));
    assert!(css.contains("toc2"));
}

non_normative!(
    r#"
In order for the AsciiDoc experience to be complete when generating HTML, a stylesheet is required.
The default stylesheet not only completes this experience, but also serves as a reference for the styles a custom stylesheet must provide.

=== Web fonts

The default stylesheet ensures that the same fonts are selected across all platforms.

By default, the browser relies on system fonts.
But system fonts vary widely by platform, so users end up getting a very different experience.
That's where web fonts come in.

"#
);

// When the default stylesheet is used, the converter adds HTML that loads the
// web fonts from Google Fonts, and the stylesheet prefers those fonts. This
// crate emits the same `<link>` naming Noto Serif, Open Sans, and Droid Sans
// Mono, and the embedded stylesheet references those families.
#[test]
fn the_converter_loads_and_the_stylesheet_prefers_the_web_fonts() {
    verifies!(
        r#"
When the default stylesheet is used, the converter adds additional HTML to load open source fonts from Google Fonts.
The default stylesheet, in turn, specifies a preference for these fonts.

The web fonts used by the default stylesheet are as follows:

Noto Serif:: body text (default)
Open Sans:: headings
Droid Sans Mono:: monospaced phrases and verbatim blocks

Loading and preferring these web fonts ensures everyone sees the same result.

"#
    );

    let html = embed("= Doc\n\nBody.");

    // The converter adds a Google Fonts <link> naming all three families.
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700\">"
    ));

    // The stylesheet, in turn, prefers those fonts.
    let css = embedded_stylesheet(&html);
    assert!(css.contains("Noto Serif"));
    assert!(css.contains("Open Sans"));
    assert!(css.contains("Droid Sans Mono"));
}

non_normative!(
    r#"
== Usage

"#
);

// The headline behavior: generating standalone HTML applies the default
// stylesheet into the `<head>` with no extra effort. Like the `asciidoctor`
// command (which runs unsafe), this crate embeds the stylesheet as a `<style>`
// element under a safe mode below `Secure`.
#[test]
fn generating_html_embeds_the_default_stylesheet_in_the_head() {
    verifies!(
        r#"
When generating HTML, there's nothing special you need to do to apply the default stylesheet.
Asciidoctor automatically embeds the default stylesheet into the `<head>` of the generated HTML when you run the `asciidoctor` command.

 $ asciidoctor document.adoc

Since no stylesheet is specified, Asciidoctor uses the default stylesheet (which is located at [.path]_data/stylesheets/asciidoctor.css_ inside the installed gem).

"#
    );

    let html = embed("= Doc\n\nBody.");
    let head = &html[..html.find("</head>").expect("head")];
    assert!(head.contains(
        "<style>\n/*! Asciidoctor default stylesheet | MIT License | https://asciidoctor.org */"
    ));
    assert!(head.contains("{padding:0}}\n</style>"));
}

non_normative!(
    r#"
When you view the generated HTML file, [.path]_document.html_, you'll see styled HTML, as shown here:

image::default-stylesheet.png[]

"#
);

// Setting `linkcss` makes the converter link to the default stylesheet instead
// of embedding it. This crate emits the same `<link>` to `./asciidoctor.css` in
// place of the inline `<style>`. (The companion `copycss`, which copies the
// stylesheet file into the output directory, is a file-system side effect this
// string-to-string library does not perform; it is tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/39.)
#[test]
fn linkcss_links_the_stylesheet_instead_of_embedding_it() {
    verifies!(
        r#"
If you want Asciidoctor to generate HTML that links to the default stylesheet instead of embedding it in the HTML, you can instruct it to do so by setting the `linkcss` and `copycss` attributes as follows:

 $ asciidoctor -a linkcss -a copycss document.adoc

"#
    );

    // The page sets `linkcss` from the CLI (`-a linkcss`); we supply it the same
    // way — as an external attribute — through `Options::set`, the API the
    // `adoc -a` option feeds into.
    let html = convert_with("= Doc\n\nBody.", &Options::new().set("linkcss"));
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!html.contains("<style>"));
}

// The API links the default stylesheet by default because its default safe mode
// is `secure`; a safe mode of server or lower embeds it instead. This crate now
// models safe mode, so both halves are verifiable. The `copycss` file copy this
// crate does not perform (tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/39) and the Ruby snippet
// stay non-normative.
#[test]
fn the_api_links_by_default_and_a_lower_safe_mode_embeds() {
    verifies!(
        r#"
When using the API, Asciidoctor already links to the stylesheet by default instead of embedding it (due to the default safe mode).
"#
    );

    // The default safe mode (`Secure`, matching Asciidoctor's API) links the
    // default stylesheet rather than embedding it.
    let linked = convert("= Doc\n\nBody.");
    assert!(linked.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!linked.contains("<style>"));

    non_normative!(
        r#"
However, Asciidoctor does not copy the stylesheet to the output directory.
You would have to put it there yourself.
Otherwise, the browser will not be able to find the stylesheet.

"#
    );

    verifies!(
        r#"
To solve this problem, set the safe mode to server or lower (e.g., server, safe, or unsafe) and Asciidoctor will embed the default stylesheet, like when using the `asciidoctor` command.

"#
    );

    // A safe mode of server or lower embeds the stylesheet inline instead.
    for mode in [SafeMode::Server, SafeMode::Safe, SafeMode::Unsafe] {
        let embedded = convert_with("= Doc\n\nBody.", &Options::new().safe_mode(mode));
        assert!(embedded.contains("<style>"), "{mode:?} should embed");
        assert!(
            !embedded.contains("./asciidoctor.css"),
            "{mode:?} should not link"
        );
    }

    non_normative!(
        r#"
[,ruby]
----
require 'asciidoctor'

Asciidoctor.convert_file 'document.adoc', safe: :safe
----

== Disable or modify the web fonts

"#
    );
}

// Unsetting `webfonts` drops the Google Fonts `<link>` while keeping the
// default stylesheet. This crate honors `:webfonts!:` the same way.
#[test]
fn unsetting_webfonts_disables_the_font_link() {
    verifies!(
        r#"
When the default stylesheet is used, the converter adds a `<link>` element specialized by the attribute `rel="stylesheet"` to load web fonts from Google Fonts.
You can disable this link by unsetting the `webfonts` document attribute from the CLI, API, or document header.

 $ asciidoctor -a webfonts! document.adoc

"#
    );

    // The page shows `-a webfonts!` on the CLI; we unset it the same way — as an
    // external attribute — through `Options::unset`, the API the `adoc -a`
    // option feeds into. Under an embedding safe mode so the stylesheet stays
    // inline (the point here is the absent font link, not embed-vs-link).
    let html = convert_with(
        "= Doc\n\nBody.",
        &Options::new().unset("webfonts").safe_mode(SafeMode::Unsafe),
    );

    // No emitted web-font <link> (the embedded CSS names Google Fonts only in a
    // commented-out @import, so match on the <link> tag itself).
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

    // The default stylesheet is still embedded.
    assert!(html.contains("<style>\n/*! Asciidoctor default stylesheet"));
}

non_normative!(
    r#"
With the web fonts absent, the browser will drop back to the fallback system fonts specified in the stylesheet.
But this also provides an opportunity to use <<customize-docinfo,docinfo>> to load the web fonts from a different source.

"#
);

// A `webfonts` value replaces the default font family in the font-loader URL.
// This crate substitutes the value verbatim into the `family` query parameter.
#[test]
fn a_webfonts_value_changes_the_font_family_in_the_url() {
    verifies!(
        r#"
Rather than disabling the link, you can also use the `webfonts` attribute to change which fonts are loaded.
When set, the value of the `webfonts` attribute is used as the value of the `family` query string parameter in the font-loader URL.

Let's say you want to use Ubuntu Mono instead of Droid Sans Mono for monospaced text.
You would set the `webfonts` attribute as follows:

 $ asciidoctor \
 -a webfonts="Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CUbuntu+Mono:400" \
 document.adoc

"#
    );

    // The page sets `webfonts` from the CLI (`-a webfonts=...`); we supply it the
    // same way — as an external attribute — through `Options::attribute`, the API
    // the `adoc -a` option feeds into.
    let webfonts =
        "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CUbuntu+Mono:400";
    let html = convert_with(
        "= Doc\n\nBody.",
        &Options::new().attribute("webfonts", webfonts),
    );
    assert!(html.contains(&format!(
        "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family={webfonts}\">"
    )));
}

non_normative!(
    r#"
In this case, you would still need to use <<customize-docinfo,docinfo>> to instruct the stylesheet to use the new font.

== Customize the default stylesheet

What if the default stylesheet is not exactly to your liking, but you don't want to go off and create a custom stylesheet from scratch?
Can you customize it?
Indeed, you can.

There are at least two ways to customize the default stylesheet.
One way is to add auxiliary styles using docinfo.
Another way is to create a custom stylesheet, but import the default stylesheet as a starting point.

"#
);

// The docinfo recipe: create a `docinfo.html` head docinfo file carrying a
// `<style>` element, then load it with `-a docinfo=shared`. The setup is prose
// and a file example (nothing to verify); the placement claim that follows it
// is verified below.
non_normative!(
    r#"
[#customize-docinfo]
=== Auxiliary styles with docinfo

Adding auxiliary styles is a great use case for xref:ROOT:docinfo.adoc[docinfo].
The docinfo feature in AsciiDoc allows you to inject auxiliary content from a file into various places in the HTML output.
In this case, we're interested in the "head" position, which injects content at the bottom of the `<head>` element.

Let's say you want to change the color of headings (and other heading-like titles) to match the color of paragraph text.
Start by creating a file named [.path]_docinfo.html_ (head is the default location) and populate it with a `<style>` element with the necessary styles.

.docinfo.html
[,html]
----
<style>
h1, h2, h3, h4, h5, h6, #toctitle,
.sidebarblock > .content > .title {
  color: rgba(0, 0, 0, 0.8);
}
</style>
----

Now tell Asciidoctor to look for and load the docinfo file using the `docinfo` attribute:

 $ asciidoctor -a docinfo=shared document.adoc

"#
);

// A head docinfo file's content is injected at the bottom of the `<head>`,
// directly below the default stylesheet. Docinfo is disabled at `Secure` (the
// default), so — like `-a docinfo=shared` on the `adoc` CLI — this converts
// under a lower safe mode, which also embeds the default stylesheet inline.
#[test]
fn head_docinfo_is_inserted_below_the_default_stylesheet() {
    verifies!(
        r#"
The `<style>` element in your docinfo file will be inserted directly below the default stylesheet in the generated HTML.

"#
    );

    // The page's `docinfo.html`, written to a temp directory as the shared head
    // docinfo file and loaded via the base directory.
    let docinfo = "<style>\nh1, h2, h3, h4, h5, h6, #toctitle,\n.sidebarblock > .content > .title {\n  color: rgba(0, 0, 0, 0.8);\n}\n</style>";
    let dir = std::env::temp_dir().join(format!("adoc-ds-docinfo-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    std::fs::write(dir.join("docinfo.html"), docinfo).expect("write docinfo.html");

    let html = convert_with(
        "= Document Title\n:docinfo: shared\n\nBody.",
        &Options::new()
            .safe_mode(SafeMode::Server)
            .base_dir(dir.clone()),
    );
    let _ = std::fs::remove_dir_all(&dir);

    // The default stylesheet's closing `</style>` is immediately followed by the
    // docinfo `<style>`, which in turn is immediately followed by `</head>`: the
    // docinfo styles sit directly below the default stylesheet, at the bottom of
    // the head.
    assert!(html.contains(&format!("</style>\n{docinfo}\n</head>")));
}

non_normative!(
    r#"
[#customize-targets]
=== Make more elements addressable from CSS

If you want to style specific elements in your content, you need to make them addressable from CSS.
In other words, it must be possible to target them using a CSS selector.
There are two mechanisms in AsciiDoc that enable you to do this:

"#
);

// The two addressability mechanisms describe how AsciiDoc attributes map to
// HTML: the ID attribute becomes the HTML `id` attribute, and the role
// attribute becomes the HTML `class` attribute. This crate emits both on the
// block wrapper it renders.
#[test]
fn id_maps_to_the_id_attribute_and_role_maps_to_the_class_attribute() {
    verifies!(
        r#"
id:: You can add an ID to almost any element in AsciiDoc using the xref:asciidoc:attributes:ids.adoc[ID attribute].
The ID attribute in AsciiDoc translates to the `id` attribute in HTML.
You can then target that element (and only that element) from CSS in order to modify its style using the syntax `#<id>`, where `<id>` is the value you specify.
Each ID can only be used once in a document.

role:: You can add a role to almost any element in AsciiDoc using the xref:asciidoc:attributes:role.adoc[role attribute].
The role attribute in AsciiDoc translates to the `class` attribute in HTML.
You can then target that element (and any other elements that share the same role) from CSS in order to modify its style using the syntax `.<role>`, where `<role>` is the value you specify.
A role can be used many times in the document.
You can even target different elements that share the same role individually in the stylesheet by adding the element name (e.g., `span.appname`) or additional roles (e.g., `.varname.global`).

For any ID or role you introduce, you must provide custom styles for it in order for it to have any visual effect.

"#
    );

    // The ID attribute becomes the HTML `id` attribute.
    assert!(convert("= Doc\n\n[#my_id]\nText.").contains("id=\"my_id\""));

    // The role attribute becomes an HTML `class` token on the wrapper.
    assert!(convert("= Doc\n\n[.my_role]\nText.").contains("class=\"paragraph my_role\""));
}

non_normative!(
    r#"
[#customize-extend]
=== Extend the default stylesheet

Instead of writing a custom stylesheet from scratch, you can import the default stylesheet and add overrides for any styles you want to change (leveraging the cascading nature of CSS).
This is also a good way to use the default stylesheet, but load web fonts from a different CDN.

Let's again change the color of headings (and other heading-like titles) to match the color of paragraph text.
Start by creating a stylesheet named [.path]_my-asciidoctor.css_.
Next, add an `@import` declaration to import the default stylesheet.
We use a CDN here to pull the default stylesheet directly out of the repository, but you can put it anywhere the browser can access it.
Then, add another `@import` declaration to import the web fonts the default stylesheet uses (which are not imported by the default stylesheet).
Finally, add your overrides below those `@import` directives.
Here's how that looks altogether.

[,css,subs=attributes+]
----
@import "https://fonts.googleapis.com/css?family=Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700";
@import "{url-default-stylesheet}";

h1, h2, h3, h4, h5, h6, #toctitle,
.sidebarblock > .content > .title {
  color: rgba(0, 0, 0, 0.8);
}
----

Now tell Asciidoctor to use your custom stylesheet instead of the default one:

 $ asciidoctor -a stylesheet=my-asciidoctor.css document.adoc

Asciidoctor will now embed the contents of your custom stylesheet instead of the default one.
However, Asciidoctor will not embed the contents of the default stylesheet.
Instead, the browser will fetch it from the location specified by the `@import` directive.
You can avoid this network call by putting the default stylesheet in the same directory as your custom stylesheet and linking to it using `@import "asciidoctor.css"`.

To obtain the compiled default stylesheet, you can either {url-default-stylesheet}[download it^] from the source repository, or you can use the following `asciidoctor` command (or equivalent) to write it to the current directory:

 $ echo | asciidoctor -o $TMPDIR/out.html -a linkcss -a copycss - && cp $TMPDIR/asciidoctor.css .

Alternately, you can use this script to write the default stylesheet to the working directory:

[,ruby]
----
require 'asciidoctor'

Asciidoctor::Stylesheets.instance.write_primary_stylesheet '.'
----

You can also download the {url-default-stylesheet-source}[source of the default stylesheet^] if you want to use it as a starting point for developing a custom stylesheet.

To learn more about how to apply a custom stylesheet, see xref:custom-stylesheet.adoc[].

== Are there different themes?

The default stylesheet does not provide different themes.
You may be interested in trying the themes provided by the https://github.com/darshandsoni/asciidoctor-skins[Asciidoctor Skins^] project.
These stylesheets take the approach of loading the default stylesheet (from a CDN), then overlaying additional styles to produce a variety of themes.
You also have the option of downloading the {url-default-stylesheet-source}[source of the default stylesheet^] and customizing it to suit your needs.

CAUTION: The Asciidoctor Skins project is hosted outside of the Asciidoctor organization.
As such, it's not guaranteed to be compatible with the latest Asciidoctor release.
If there are problems with the stylesheets it provides, please report it to that project.

To learn how to apply a custom stylesheet, see xref:custom-stylesheet.adoc[].
"#
);
