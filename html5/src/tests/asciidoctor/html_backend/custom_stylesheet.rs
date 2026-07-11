use crate::{convert_with, tests::sdd::*, Options};

track_file!("ref/asciidoctor/docs/modules/html-backend/pages/custom-stylesheet.adoc");

// Asciidoctor's "Apply a Custom Stylesheet" page. It documents applying a
// stylesheet other than the default via the `stylesheet`, `stylesdir`,
// `linkcss`, and `copycss` attributes.
//
// The verifiable claim reproduced here is the *linked* case, which is where
// Asciidoctor's own web-path computation lives: when a custom stylesheet is
// linked (the `secure` API default, or any mode with `linkcss` set), this crate
// emits a `<link>` to the stylesheet at the web path Asciidoctor would use,
// mirroring `stylesdir` into it. That path computation is pure string work, so
// it is reproduced faithfully here.
//
// Embedding a custom stylesheet read from disk is also supported now (resolved
// against the base directory under the safe mode's jail); the behavior and its
// `adoc`/API surface are verified on this project's own
// xref:generate-html:custom-stylesheet.adoc[] docs page (tracked from both
// crates) rather than duplicated here. The `copycss` file copy and remote
// (`http`) stylesheet fetch this crate does not perform stay non-normative;
// they are tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/39.
//
// Tracked from the library crate only: every verifiable claim is about the HTML
// `<head>` that `asciidoc_html5::convert`/`convert_with` emits. Where the page
// drives behavior with a CLI `-a` attribute, we supply the same attribute
// through `Options`; the `adoc -a` option is a thin forwarder onto that API,
// covered by the CLI crate's own tests.

non_normative!(
    r#"
= Apply a Custom Stylesheet

So far we've talked about different ways of using the default stylesheet.
In place of the default stylesheet, you can instruct Asciidoctor to use a custom stylesheet of your choosing.
On this page, you'll learn how to apply a custom stylesheet and, if necessary, how to get Asciidoctor to copy and link to that stylesheet into the correct location.
You'll also learn about some of the limitations of this feature when processing multiple documents.

== Specify the custom stylesheet

Asciidoctor looks for the stylesheet file specified by the `stylesheet` document attribute.
If the value of this attribute is empty (the default), Asciidoctor uses the default stylesheet.
Otherwise, Asciidoctor assumes the value is the path of a custom stylesheet file relative to the source document.
All the same behavior described in xref:stylesheet-modes.adoc[] still applies, except Asciidoctor uses the specified stylesheet instead of the default stylesheet.

To begin, you must create a stylesheet file.
For now, create this file adjacent to your AsciiDoc document.
To keep it simple, we'll just define a basic stylesheet that changes all text to red.
Create the file [.path]_my-stylesheet.css_ the directory with your AsciiDoc source files and populate it with the following contents:

.my-stylesheet.css
[,css]
----
body {
  color: #ff0000;
}
----

Now let's use the `stylesheet` attribute to apply it.

The `stylesheet` document attribute must be set by the end of the header to be effective.
One way to do that is to set the attribute in the document header:

.stylesheet attribute set in document header
[,asciidoc]
----
include::example$my-document.adoc[tag=title]
:stylesheet: my-stylesheet.css
include::example$my-document.adoc[tag=body]
----

You can also set `stylesheet` using the API or CLI (shown here):

 $ asciidoctor -a stylesheet=my-stylesheet.css my-document.adoc

When you view the generated HTML file [.path]_my-document.html_ in your browser, you'll see that all the text is red.

TIP: If you want to create a custom stylesheet by extending the default stylesheet, see xref:default-stylesheet.adoc#customize-extend[Extend the default stylesheet].

"#
);

// Setting `linkcss` links to the custom stylesheet instead of embedding it.
// This crate emits the same `<link>`; because the stylesheet sits in the same
// folder as the output, its href is the bare `./my-stylesheet.css`. (The "does
// not copy" aside is a file-system observation this string-to-string library
// has no part in.)
#[test]
fn linkcss_links_to_the_custom_stylesheet() {
    verifies!(
        r#"
As with the default stylesheet, you can set the `linkcss` document attribute and Asciidoctor will link to your stylesheet instead.
(Note that it doesn't copy it in this case since it's already in the same folder as the output file).

 $ asciidoctor -a stylesheet=my-stylesheet.css -a linkcss my-document.adoc

"#
    );

    // The page sets both attributes from the CLI (`-a stylesheet=… -a linkcss`);
    // we supply them the same way through `Options`, the API `adoc -a` feeds.
    let html = convert_with(
        "= Doc\n\ntext",
        &Options::new()
            .attribute("stylesheet", "my-stylesheet.css")
            .set("linkcss"),
    );
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./my-stylesheet.css\">"));
    // A custom stylesheet gets neither the default stylesheet nor the web fonts.
    assert!(!html.contains("./asciidoctor.css"));
    assert!(!html.contains("fonts.googleapis.com"));
}

non_normative!(
    r#"
You may not want to keep your stylesheet in the same directory as your AsciiDoc documents.
Let's see how to tell Asciidoctor to look for it elsewhere.

== Configure the styles directory

When your stylesheet is in a directory below your AsciiDoc document, you need to tell Asciidoctor where to look to find the stylesheet.
There are two equivalent ways to do so:

* Specify the name of the stylesheet file using the `stylesheet` attribute and the directory where it's located using the `stylesdir` attribute
* Include the directory where the stylesheet is located in the value of the `stylesheet` attribute (instead of using the `stylesdir` attribute)

The value of the `stylesdir` attribute can end with a trailing directory separator (`/`), but it's not required.

If the `linkcss` attribute is not set, the styles directory can be either relative or absolute.
The situation gets trickier when `linkcss` is set, which we'll <<stylesdir-and-linkcss,get to later>>.

Let's assume you want to put your stylesheet in the [.path]_my-styles_ folder relative to your AsciiDoc document(s).
Go ahead and create that folder and move your custom stylesheet into it for this example.
Now we need to tell Asciidoctor where it's located using `stylesdir`.

The `stylesdir` document attribute must be set by the end of the header to be effective.
One way to do that is to set the attribute in the document header:

.stylesdir attribute set in document header
[,asciidoc]
----
include::example$my-document.adoc[tag=title]
:stylesheet: my-stylesheet.css
:stylesdir: my-styles
include::example$my-document.adoc[tag=body]
----

You can also set `stylesdir` using the API or CLI (shown here):

 $ asciidoctor -a stylesdir=my-styles -a stylesheet=my-stylesheet.css my-document.adoc

Asciidoctor now looks for the stylesheet file [.path]_my-styles/my-stylesheet.css_ relative to the AsciiDoc document and embeds it into the HTML.

You can achieve the same result by including the directory in the value of the `stylesheet` attribute:

 $ asciidoctor -a stylesheet=my-styles/my-stylesheet.css my-document.adoc

If you set the value of `stylesdir` to an absolute directory, then Asciidoctor would still locate the stylesheet and embed it into the HTML.
But this can create problems if you've configured the converter to link to the stylesheet instead.
Let's look at those problem and ways to work through them.

[#stylesdir-and-linkcss]
== Styles directory and linkcss

Using `stylesdir` gets tricky when you're linking to the stylesheet instead of embedding it.
That's because we're now dealing with two different paths.
One is the path where the stylesheet is located on disk (either absolute or relative to the document).
The other is the path the browser uses to access the stylesheet.

"#
);

// When linking, Asciidoctor mirrors the `stylesdir` value into the link
// reference. This crate does the same: the linked href includes the styles
// directory ahead of the stylesheet file name.
#[test]
fn linked_path_mirrors_the_styles_directory() {
    verifies!(
        r#"
First, it's important to understand that Asciidoctor mirrors the `stylesdir` path in the link reference.
Let's use the previous example to demonstrate.
If you invoke Asciidoctor as follows:

 $ asciidoctor -a stylesdir=my-styles -a stylesheet=my-stylesheet.css -a linkcss my-document.adoc

"#
    );

    let html = convert_with(
        "= Doc\n\ntext",
        &Options::new()
            .attribute("stylesdir", "my-styles")
            .attribute("stylesheet", "my-stylesheet.css")
            .set("linkcss"),
    );

    // The styles directory is mirrored into the linked path, ahead of the
    // stylesheet file name.
    assert!(html.contains("<link rel=\"stylesheet\" href=\"./my-styles/my-stylesheet.css\">"));

    // The shown HTML snippet is illustrative and uses the *default* file name
    // (`asciidoctor.css`) rather than the custom one; the authoritative behavior
    // (mirrored `stylesdir` + the actual stylesheet name) is asserted above.
    non_normative!(
        r#"
Then when you inspect the HTML, you will see:

[,html]
----
<link rel="stylesheet" href="my-styles/asciidoctor.css">
----

Notice how the value of `stylesdir` ([.path]_my-styles_) is included in the referenced path.
Asciidoctor will also preserve this directory if it needs to copy the stylesheet file.

"#
    );
}

// The rest of the page covers absolute styles directories, output directories,
// the `copycss` copy-from/link-to split, and nested-document layouts — all
// file-system side effects and multi-file workflows this string-to-string
// library does not perform. They carry no rule to verify here.
non_normative!(
    r#"
There are certain situations where this isn't going to work (at least not when publishing the file to the public internet).
Let's consider one such case.
We'll specify the location of `stylesdir` as an absolute path and generate the output file into a separate output directory.

 $ asciidoctor -a stylesdir=`pwd`/my-styles -a stylesheet=my-stylesheet.css -a linkcss -D public my-document.adoc

Asciidoctor generates the HTML file into the [.path]_public_ folder, but it does not copy the stylesheet.
Furthermore, it uses an absolute path in the link to the stylesheet:

[,html]
----
<link rel="stylesheet" href="/path/to/documents/my-styles/asciidoctor.css">
----

What we want is for Asciidoctor to copy the stylesheet to the output directory and link to it using a relative path.

A similar problem comes up if you want to control the styles directory and stylesheet file referenced by the HTML independently of the location where they are taken.

In brief, we need to be able to decouple the path where the stylesheet is read from the location where the stylesheet is published and referenced.
That's where the `copycss` attribute comes back into play.

[#copy-link-split]
== Copy from one place, link to another

For complex combinations that involve `stylesdir`, `linkcss`, and an explicit output directory, the meaning of `stylesdir` is too overloaded and needs to be reconciled.
We can turn to the `copycss` attribute to clear this situation up.

NOTE: This situation is unique to when `linkcss` is set.
It's not a problem when the converter embeds the stylesheet since there is no secondary reference involved.

The `copycss` attribute can accept a non-empty value.
Asciidoctor uses this value as an override for where to look for the stylesheet to read.
The converter, on the other hand, does not use this value.
That means we can use `stylesdir` and `stylesheet` to assemble the path relative to the output directory where Asciidoctor should write and link to the stylesheet independent of the location where it reads the file.

Let's revisit the broken scenario from the previous section and use `copycss` to reconcile the problem:

 $ asciidoctor -a stylesdir=css -a stylesheet=default.css \
 -a copycss=$PWD/my-styles/my-stylesheet.css -a linkcss -D public my-document.adoc

Asciidoctor copies the stylesheet from the absolute path specified by the `copycss` attribute to the path [.path]_public/css/default.css_ and links to it using the path [.path]_css/default.css_.
Notice that we changed the name of the folder and stylesheet file in the output.
That demonstrates that we have decoupled the path where the stylesheet is read from the location where the stylesheet is published and referenced.

TIP: When running on JRuby, you can point `copycss` to a classloader URI to load a stylesheet from a JAR file (e.g., `uri:classloader:/css/styles.css`).
However, the HTML cannot link to that location.
Therefore, you must use the `stylesheet` attribute (and optionally the `stylesdir`) to specify a filename or relative path where Asciidoctor will write the stylesheet.
The HTML will then be able to link to that destination.

== Styles directory and nested documents when linking

When xref:cli:process-multiple-files.adoc[invoking Asciidoctor on a nested set of documents], it's currently not possible to specify a single relative path for the `stylesdir` attribute that works for all of the documents.
This is because the relative depth of the stylesheet's location differs for the documents in the subdirectories.
One way to solve this problem is to maintain the path to `stylesdir` in each document.

Let's say you have three AsciiDoc documents saved in the following directory structure:

....
/my-documents
  a.adoc
  b.adoc
  /my-nested-documents
    c.adoc
  /my-styles
....

For [.path]_a.adoc_ and [.path]_b.adoc_, set `stylesdir` to:

[,asciidoc]
----
:stylesdir: my-styles
----

For [.path]_c.adoc_, set `stylesdir` to:

[,asciidoc]
----
:stylesdir: ../my-styles
----

If you're serving your documents from a web server, you can solve this problem by providing an absolute path to the stylesheet.
You can also try to use the `copycss` per document to control where Asciidoctor looks for the stylesheet independent of where Asciidoctor copies it and the converter configured the HTML to reference it.
"#
);
