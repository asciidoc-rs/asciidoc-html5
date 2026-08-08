use crate::{
    asset_writer::RecordingAssetWriter, convert_file_with_writer, convert_with_writer,
    tests::sdd::*, DirAssetWriter, Options, SafeMode,
};

// These tests assert the standalone document shell (the stylesheet lives in
// `<head>`), so they render in standalone mode explicitly. The string entry
// points default to embedded, body-only output.
fn convert(source: &str) -> String {
    crate::convert_with(source, &Options::new().standalone(true))
}

fn convert_with(source: &str, options: &Options) -> String {
    crate::convert_with(source, &options.clone().standalone(true))
}

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/stylesheet-modes.adoc");

// Asciidoctor's "Stylesheet Modes" page. It documents the safe mode and the
// `linkcss`/`copycss`/`stylesheet` attributes that control whether the
// stylesheet is embedded, linked (and optionally copied to the output
// directory), or left out altogether.
//
// This crate implements all of that: the embed-vs-link split follows the safe
// mode (`Options::safe_mode`) the same way as the default and custom
// stylesheet pages already verify, `linkcss`/`copycss` are honored, and the
// library reports a linked stylesheet to copy through an `AssetWriter` (a
// `DirAssetWriter` drives the actual file write, matching what `adoc` does
// under the hood — verified end to end by the CLI crate, which reproduces
// this same page).
//
// Tracked from the library crate only for the safe-mode/attribute claims that
// have a direct API counterpart; the `copycss` file-copy claims are verified
// both here (through `AssetWriter`) and, end to end via the `adoc` binary, by
// the CLI crate.

non_normative!(
    r#"
= Stylesheet Modes

//When applying a stylesheet to the generated HTML, you can configure the HTML converter to either embed the CSS directly into the HTML or link to the stylesheet file.
The HTML converter can be configured to embed the CSS of the stylesheet directly into the HTML, link to the stylesheet file, or disable it entirely.
These modes are available regardless of whether you're using the xref:default-stylesheet.adoc[default stylesheet] or a xref:custom-stylesheet.adoc[custom stylesheet].
This page covers the document attributes that control how the stylesheet is applied.

"#
);

// The IMPORTANT admonition's claim: the stylesheet only appears in standalone
// output, because it lives in the `<head>` element that only standalone
// output carries.
#[test]
fn the_stylesheet_only_applies_to_standalone_documents() {
    verifies!(
        r#"
IMPORTANT: The HTML converter will only apply a stylesheet when generating a standalone HTML document.
That's because the stylesheet goes inside the HTML `<head>` element, and the converter only generates that element for standalone output.

"#
    );

    // Embedded (non-standalone) output carries neither an embedded `<style>`
    // nor a linked stylesheet — there is no `<head>` to put either in.
    let html = crate::convert("= Doc\n\nBody.");
    assert!(!html.contains("<style>"));
    assert!(!html.contains("<link rel=\"stylesheet\""));
}

// Under a safe mode of `server` or lower, the stylesheet is embedded inline in
// a `<style>` element; under `secure`, it is linked instead.
#[test]
fn embed_the_stylesheet() {
    verifies!(
        r#"
[#embed]
== Embed the stylesheet

When the xref:ROOT:safe-modes.adoc[safe mode] is server or lower, the default behavior of the HTML converter is to read the stylesheet file, enclose its contents in a `<style>` element, and embed it directly into the `<head>` element of the generated HTML.
This default makes the HTML more portable since you don't lose the stylesheet if you move the file.

However, if the safe mode is secure, the converter will <<link,link to the stylesheet file>> instead.
If you see a link to the stylesheet file in the generated HTML where you expect the stylesheet to be embedded, check your safe mode setting.

The same two rules apply regardless of whether you're using the default stylesheet or a custom stylesheet.

"#
    );

    let embedded = convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Server),
    );
    assert!(embedded.contains("<style>"));
    // Not linked (the web-fonts `<link>` the default stylesheet also carries
    // is unrelated to the stylesheet itself, so check for its absence
    // specifically).
    assert!(!embedded.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));

    let secure = convert_with(
        "= Doc\n\nBody.",
        &Options::new().safe_mode(SafeMode::Secure),
    );
    assert!(secure.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!secure.contains("<style>"));
}

// Setting `linkcss` links the stylesheet instead of embedding it, at
// `./asciidoctor.css` for the default stylesheet.
#[test]
fn link_to_the_stylesheet() {
    verifies!(
        r#"
[#link]
== Link to the stylesheet

You already know that the HTML converter will link to the stylesheet when the safe mode is secure.
However, it's possible to enable this behavior independent of the safe mode.
This can be beneficial if you're converting numerous AsciiDoc documents to HTML and want them all to share the same stylesheet.
It can also make inspecting the HTML a little simpler.

If the `linkcss` document attribute is set, the converter will link to the stylesheet instead of embedding it.
To link to the stylesheet, the converter uses a `<link>` element specialized by the `rel="stylesheet"` attribute.
The `href` attribute will reference the stylesheet using a relative path.

The `linkcss` document attribute must be set by the end of the header to be effective.
One way to do that is to set the attribute in the document header:

.linkcss attribute set in document header
[,asciidoc]
----
include::example$my-document.adoc[tag=title]
:linkcss:
include::example$my-document.adoc[tag=body]
----

You can also set `linkcss` using the API or CLI (shown here):

 $ asciidoctor -a linkcss my-document.adoc

In either case, if you inspect the `<head>` element in the output file [.path]_my-document.html_, you'll see that the HTML links to the stylesheet.

.my-document.html
[,html]
----
<link rel="stylesheet" href="./asciidoctor.css">
----

Since we didn't specify a stylesheet, the converter links to the default stylesheet.
But where is this stylesheet located?
Let's find out.

"#
    );

    // The page sets `linkcss` from the document header or CLI (`-a linkcss`);
    // we supply it the same way — as an external attribute — through
    // `Options::set`, the API the `adoc -a` option feeds into.
    let html = convert_with("= Doc\n\nBody.", &Options::new().set("linkcss"));
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
}

// Under a safe mode of `server` or lower with `linkcss` set (and `copycss` at
// its on-by-default value), the library reports the linked stylesheet to copy
// through an `AssetWriter`; under `secure`, it does not.
#[test]
fn copy_the_stylesheet_to_the_output_directory() {
    verifies!(
        r#"
[#copy]
== Copy the stylesheet to the output directory

If you're linking to a stylesheet file, the stylesheet file has to be available at the referenced path so the browser can access it.
For simple cases, Asciidoctor takes care of this for you.

If the safe mode is server or lower, and the `linkcss` document attribute is set, Asciidoctor will copy the stylesheet to the output directory so the HTML can link to it.
When using the default stylesheet, Asciidoctor writes the CSS to the file [.path]_asciidoctor.css_ in the output directory.
If you specify a custom stylesheet, Asciidoctor will copy that file instead, retaining the name of the file.
This utility works even if you specify an xref:cli:output-file.adoc[output file in a different directory] from the source file.

"#
    );

    // The "shared responsibility" sidebar: the library never writes files
    // itself (it renders HTML to a string); it reports the stylesheet to copy
    // through an `AssetWriter` that the caller drives. The end-to-end file
    // write (a `DirAssetWriter` rooted at the output directory) is what the
    // `verifies!` assertions below exercise.
    non_normative!(
        r#"
.A shared responsibility
****
While the converter handles the task of embedding or linking to the stylesheet file, it's the processor itself (not the converter) that handles copying the stylesheet.
****

"#
    );

    verifies!(
        r#"
If the safe mode is secure, Asciidoctor will not copy the stylesheet file and, thus, the link to it will be broken (unless, of course, you copy the file separately).

Let's revisit the previous example:

 $ asciidoctor -a linkcss my-document.adoc

After running this command, the stylesheet file [.path]_asciidoctor.css_ is copied to the same directory as the generated HTML file [.path]_my-document.html_.
Type `ls` to view the files in the directory.
You should see a file named [.path]_asciidoctor.css_.

 $ ls
 asciidoctor.css  my-document.adoc  my-document.html

"#
    );

    let dir = std::env::temp_dir().join(format!(
        "adoc-html-backend-stylesheet-modes-copy-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    std::fs::write(dir.join("my-document.adoc"), "= My Document\n\nHello.").expect("write adoc");

    let out_dir = dir.join("output");
    let mut writer = DirAssetWriter::new(&out_dir);
    let html = convert_file_with_writer(
        dir.join("my-document.adoc"),
        &Options::new().safe_mode(SafeMode::Server).set("linkcss"),
        &mut writer,
    )
    .expect("convert");
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(out_dir.join("asciidoctor.css").is_file());

    // Under `secure`, the same conversion links the stylesheet but writes
    // nothing to the output directory.
    let secure_dir = dir.join("secure-output");
    let mut secure_writer = DirAssetWriter::new(&secure_dir);
    let secure = convert_file_with_writer(
        dir.join("my-document.adoc"),
        &Options::new().safe_mode(SafeMode::Secure),
        &mut secure_writer,
    )
    .expect("convert");
    assert!(secure.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!secure_dir.join("asciidoctor.css").exists());

    let _ = std::fs::remove_dir_all(&dir);

    non_normative!(
        r#"
When you view the HTML file in your browser, you should observe that the default stylesheet is applied.

"#
    );
}

// Unsetting `copycss` (here from the document header) prevents the copy: the
// writer is never called, even though the HTML still links the stylesheet.
#[test]
fn to_copy_or_not_to_copy() {
    verifies!(
        r#"
=== To copy or not to copy

Whether Asciidoctor copies the stylesheet to the output directory is controlled by the `copycss` document attribute.
The `copycss` attribute is set by default unless the safe mode is secure.

To prevent Asciidoctor from copying the stylesheet independent of safe mode, unset the `copycss` document attribute.

The `copycss` document attribute must be unset by the end of the header to be effective.
One way to do that is to unset the attribute in the document header:

.copycss attribute unset in document header
[,asciidoc]
----
include::example$my-document.adoc[tag=title]
:linkcss:
:!copycss:
include::example$my-document.adoc[tag=body]
----

You can also unset `copycss` using the API or CLI (shown here):

 $ asciidoctor -a linkcss -a copycss! my-document.adoc

In either case, if you inspect the output directory, you will see that the stylesheet file [.path]_asciidoctor.css_ is missing (unless it was already there).

"#
    );

    let mut writer = RecordingAssetWriter::default();
    let html = convert_with_writer(
        "= My Document\n:!copycss:\n\nHello.",
        &Options::new()
            .standalone(true)
            .safe_mode(SafeMode::Server)
            .set("linkcss"),
        &mut writer,
    )
    .expect("convert");
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(writer.written.is_empty());

    non_normative!(
        r#"
We'll see the `copycss` attribute come up again on the xref:custom-stylesheet.adoc[custom stylesheet page] as a means of xref:custom-stylesheet.adoc#copy-link-split[overriding the location of the stylesheet to copy].

"#
    );
}

// Unsetting `stylesheet` leaves out the stylesheet altogether: no `<style>`
// and no `<link>`.
#[test]
fn disable_the_stylesheet() {
    verifies!(
        r#"
[#disable]
== Disable the stylesheet

The stylesheet is effectively disabled when generating embedded HTML, since the embedded HTML does not include the `<head>` element.
If you don't want the converter to include a stylesheet in the standalone HTML, unset the `stylesheet` attribute using the CLI.

 $ asciidoctor -a stylesheet! my-document.adoc

The reason you have to unset the `stylesheet` attribute is because it is set by default (to an empty value).
When the `stylesheet` attribute is set, but empty, the HTML converter uses the default stylesheet.
By unsetting this attribute, we're telling the converter not to use a stylesheet at all.

"#
    );

    let html = convert("= My Document\n:stylesheet!:\n\nHello.");
    assert!(!html.contains("<style>"));
    assert!(!html.contains("asciidoctor.css"));

    non_normative!(
        r#"
When you view the generated HTML file, [.path]_my-document.html_, you'll see bare HTML without any styles applied, as shown here:

image::no-stylesheet.png[]

"#
    );

    verifies!(
        r#"
NOTE: When the `stylesheet` attribute is unset, the `linkcss` and `copycss` attributes are ignored.
"#
    );

    // The closing note: with `stylesheet` unset, `linkcss` has no effect —
    // still no `<link>` — even under a safe mode that would otherwise link a
    // stylesheet.
    let with_linkcss = convert_with(
        "= My Document\n:stylesheet!:\n\nHello.",
        &Options::new().set("linkcss"),
    );
    assert!(!with_linkcss.contains("<link rel=\"stylesheet\""));

    non_normative!(
        r#"

Now that you have a clean slate, let's learn how to apply a custom stylesheet of your very own.
"#
    );
}
