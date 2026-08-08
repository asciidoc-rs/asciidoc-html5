use clap::Parser as _;

use crate::{run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/stylesheet-modes.adoc");

// Asciidoctor's "Stylesheet Modes" page, tracked from the CLI crate. It
// verifies the `adoc` invocations for each mode — embedding by default (and
// linking under `-S secure`), linking with `-a linkcss`, copying the linked
// stylesheet next to the output, skipping the copy when `copycss` is unset,
// and disabling the stylesheet. The API (Rust) invocations the
// `asciidoc-html5` crate verifies against are the same claims reproduced
// there; the sdd tool merges the two crates' coverage by line.

/// Runs `adoc` with `args` and returns its standard output as a string.
fn adoc_stdout(args: &[&str]) -> String {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts the document");
    String::from_utf8(stdout).expect("stdout is UTF-8")
}

/// Creates a fresh temp directory named after `tag` holding a
/// `my-document.adoc` for the copy tests to convert into an output file.
fn scratch(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "adoc-html-backend-modes-{}-{tag}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    std::fs::write(dir.join("my-document.adoc"), "= My Document\n\nHello.").expect("write adoc");
    dir
}

non_normative!(
    r#"
= Stylesheet Modes

//When applying a stylesheet to the generated HTML, you can configure the HTML converter to either embed the CSS directly into the HTML or link to the stylesheet file.
The HTML converter can be configured to embed the CSS of the stylesheet directly into the HTML, link to the stylesheet file, or disable it entirely.
These modes are available regardless of whether you're using the xref:default-stylesheet.adoc[default stylesheet] or a xref:custom-stylesheet.adoc[custom stylesheet].
This page covers the document attributes that control how the stylesheet is applied.

IMPORTANT: The HTML converter will only apply a stylesheet when generating a standalone HTML document.
That's because the stylesheet goes inside the HTML `<head>` element, and the converter only generates that element for standalone output.

"#
);

// `adoc my-document.adoc` runs `unsafe` by default, so the stylesheet is
// embedded inline in a `<style>` element; `-S secure` links it instead.
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

    let dir = scratch("embed");
    let path = dir.join("my-document.adoc");
    let path = path.to_str().expect("path is UTF-8");

    let html = adoc_stdout(&[path, "-o", "-"]);
    assert!(html.contains("<style>"));
    // Not linked (the web-fonts `<link>` the default stylesheet also carries
    // is unrelated to the stylesheet itself, so check for its absence
    // specifically).
    assert!(!html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));

    let secure = adoc_stdout(&[path, "-o", "-", "-S", "secure"]);
    assert!(secure.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    assert!(!secure.contains("<style>"));

    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a linkcss` links to the default stylesheet at `./asciidoctor.css`
// instead of embedding it.
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

    let dir = scratch("link");
    let html = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        "-",
        "-a",
        "linkcss",
    ]);
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
    let _ = std::fs::remove_dir_all(&dir);
}

// `adoc -a linkcss` with a real output file copies _asciidoctor.css_ into the
// output directory next to the HTML; `-S secure` does not.
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

    let dir = scratch("copy");
    let _ = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        dir.join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
    ]);
    assert!(dir.join("asciidoctor.css").is_file());

    // "If you specify a custom stylesheet, Asciidoctor will copy that file
    // instead, retaining the name of the file": a custom `stylesheet` is
    // copied under its own name, not `asciidoctor.css`. The output goes to a
    // separate `dist` directory so the copy destination differs from the
    // source, proving a copy actually happened rather than the source file
    // simply already sitting where the assertion looks.
    let custom_dir = scratch("copy-custom");
    std::fs::write(custom_dir.join("my-theme.css"), "body { color: teal; }")
        .expect("write custom css");
    let dist_dir = custom_dir.join("dist");
    let _ = adoc_stdout(&[
        custom_dir
            .join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        dist_dir
            .join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
        "-a",
        "stylesheet=my-theme.css",
    ]);
    assert_eq!(
        std::fs::read_to_string(dist_dir.join("my-theme.css")).expect("read copied css"),
        "body { color: teal; }"
    );
    assert!(!dist_dir.join("asciidoctor.css").exists());

    // The `secure` half of the claim above: the same command under `-S secure`
    // does not write the stylesheet to the output directory.
    let secure_dir = scratch("copy-secure");
    let _ = adoc_stdout(&[
        secure_dir
            .join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        secure_dir
            .join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
        "-S",
        "secure",
    ]);
    assert!(!secure_dir.join("asciidoctor.css").exists());

    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&secure_dir);

    non_normative!(
        r#"
When you view the HTML file in your browser, you should observe that the default stylesheet is applied.

"#
    );
}

// `adoc -a linkcss -a copycss!` links the stylesheet but does not copy it: no
// _asciidoctor.css_ is written to the output directory.
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

    let dir = scratch("nocopy");
    let _ = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        dir.join("my-document.html")
            .to_str()
            .expect("path is UTF-8"),
        "-a",
        "linkcss",
        "-a",
        "copycss!",
    ]);
    assert!(!dir.join("asciidoctor.css").exists());
    let _ = std::fs::remove_dir_all(&dir);

    non_normative!(
        r#"
We'll see the `copycss` attribute come up again on the xref:custom-stylesheet.adoc[custom stylesheet page] as a means of xref:custom-stylesheet.adoc#copy-link-split[overriding the location of the stylesheet to copy].

"#
    );
}

// `adoc -a stylesheet!` leaves out the stylesheet altogether: no `<style>`
// and no `<link>`, and a `linkcss` alongside it changes nothing.
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

    let dir = scratch("disable");
    let html = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        "-",
        "-a",
        "stylesheet!",
    ]);
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

    // The closing note: with `stylesheet` unset, `-a linkcss` has no effect —
    // still no `<link>`.
    let with_linkcss = adoc_stdout(&[
        dir.join("my-document.adoc")
            .to_str()
            .expect("path is UTF-8"),
        "-o",
        "-",
        "-a",
        "stylesheet!",
        "-a",
        "linkcss",
    ]);
    assert!(!with_linkcss.contains("<link rel=\"stylesheet\""));

    let _ = std::fs::remove_dir_all(&dir);

    non_normative!(
        r#"

Now that you have a clean slate, let's learn how to apply a custom stylesheet of your very own.
"#
    );
}
