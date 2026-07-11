use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("docs/modules/generate-html/pages/custom-stylesheet.adoc");

// This crate's "Apply a Custom Stylesheet" page, tracked from the CLI. Its
// prose and the API (Rust) invocations are non-normative here — the
// `asciidoc-html5` crate verifies those against the API, and the sdd tool
// merges the two crates by line. What this test suite verifies is the one
// `adoc` invocation the page shows: `adoc my-document.adoc` reads the custom
// stylesheet named in the header from the input file's directory and embeds it
// into the `<head>`, with no web fonts.

non_normative!(
    r#"
= Apply a Custom Stylesheet
:navtitle: Custom Stylesheet
:description: How asciidoc-html5 applies a custom stylesheet in place of the default, embedding or linking it per the safe mode.

In place of Asciidoctor's default stylesheet, you can tell `asciidoc-html5` to
apply a custom stylesheet of your own by setting the `stylesheet` document
attribute. Whether the stylesheet is _embedded_ or _linked_ follows the same
xref:ROOT:safe-modes.adoc[safe mode] rule as the
xref:default-stylesheet.adoc[default stylesheet].

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` and API
invocations it shows are normative: they are verified against the
implementation, so the documented behavior is guaranteed.
====

== Specify the custom stylesheet

Set the `stylesheet` attribute to the path of your stylesheet, relative to the
document. An empty value (the default) keeps the default stylesheet; any other
value selects a custom one.

Create a stylesheet next to your document -- say _my-theme.css_:

[,css]
----
body {
  color: #ff0000;
}
----

Then point the `stylesheet` attribute at it from the document header:

[,asciidoc]
----
= My Document
:stylesheet: my-theme.css

Hello.
----

"#
);

// `adoc my-document.adoc` embeds the header-named custom stylesheet, read from
// the input file's directory, into the `<head>` — self-contained output with no
// web fonts. This is the CLI counterpart to the API embedding the library crate
// verifies.
#[test]
fn adoc_embeds_a_custom_stylesheet_from_disk() {
    verifies!(
        r#"
Converting the file from the command line embeds the stylesheet's contents into
the `<head>`, so the output is self-contained:

 $ adoc my-document.adoc

Unlike the default stylesheet, a custom stylesheet does not pull in the Google
web fonts.

"#
    );

    let dir = std::env::temp_dir().join(format!("adoc-docs-css-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("my-document.adoc");
    std::fs::write(&input, "= My Document\n:stylesheet: my-theme.css\n\nHello.")
        .expect("write input");
    std::fs::write(dir.join("my-theme.css"), "body { color: #ff0000; }\n").expect("write css");

    let cli = Cli::parse_from([
        "adoc",
        input.to_str().expect("temp path is UTF-8"),
        "-o",
        "-",
    ]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the file");
    let _ = std::fs::remove_dir_all(&dir);

    let html = String::from_utf8(stdout).expect("stdout is UTF-8");
    assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
    assert!(!html.contains("fonts.googleapis.com"));
}

non_normative!(
    r##"
== Embed or link

Which form the `<head>` takes follows the xref:ROOT:safe-modes.adoc[safe mode],
exactly as for the default stylesheet:

* The `adoc` command (which runs `unsafe`) and any safe mode below `secure`
_embed_ the stylesheet's contents inline.
* The API default (`secure`), and any mode with `linkcss` set, _link_ to the
stylesheet.

Embedding reads the stylesheet from disk, so it resolves against a base
directory and, under a jailed safe mode (`safe` or `server`), is confined to it
-- the same rules as an `include::` target. The `adoc` command anchors the read
at the input file's directory; through the API, name the document with
`Options::input_file` (or set `Options::base_dir`).

If you already hold the stylesheet's contents -- for example, from a resource
you loaded yourself -- pass them with `Options::stylesheet_content` to embed them
without any file access:

[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let html = convert_with(
    "= My Document\n:stylesheet: my-theme.css\n\nHello.",
    &Options::new()
        .safe_mode(SafeMode::Unsafe)
        .stylesheet_content("body { color: #ff0000; }"),
);
assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
----

Linking needs only the path, not the file, so it works from a plain string.
Under the `secure` default the `<head>` links to the stylesheet at its
normalized web path:

[,rust]
----
let html = asciidoc_html5::convert("= My Document\n:stylesheet: my-theme.css\n\nHello.");
assert!(html.contains(r#"<link rel="stylesheet" href="./my-theme.css">"#));
----

== Configure the styles directory

When the stylesheet lives in a subdirectory, name the directory with the
`stylesdir` attribute. It is joined ahead of the `stylesheet` value both when
resolving the file to embed and when building the linked reference:

[,rust]
----
use asciidoc_html5::{convert_with, Options};

let html = convert_with(
    "= My Document\n:stylesdir: css\n:stylesheet: my-theme.css\n\nHello.",
    &Options::new().set("linkcss"),
);
assert!(html.contains(r#"<link rel="stylesheet" href="./css/my-theme.css">"#));
----

A stylesheet given as a URL (or an absolute path) is already a complete
reference, so it is linked as-is:

[,rust]
----
let html = asciidoc_html5::convert("= Doc\n:stylesheet: https://example.org/theme.css\n\nHi.");
assert!(html.contains(r#"<link rel="stylesheet" href="https://example.org/theme.css">"#));
----

== Known limitations

`asciidoc-html5` produces HTML but never writes companion files. With a _linked_
custom stylesheet you are responsible for placing the stylesheet where the HTML
references it; there is no `copycss` step that copies it into an output
directory. Embedding a _remote_ stylesheet (an `http`/`https` URL) is likewise
unsupported, since the library does not fetch over the network -- a remote
stylesheet can still be linked, as shown above. Both are tracked in
https://github.com/asciidoc-rs/asciidoc-html5/issues/39[issue #39].
"##
);
