use std::fs;

use asciidoc_parser::Parser;

use crate::{convert, convert_document, convert_file, tests::sdd::*};

track_file!("ref/asciidoctor/docs/modules/api/pages/index.adoc");

// Asciidoctor's "Process AsciiDoc Using the API" page. It documents the Ruby
// `Asciidoctor` API — its purpose, how to `require` it, and its four Ruby
// entrypoints (`load`, `load_file`, `convert`, `convert_file`). Most of the
// page is Ruby-specific prose (the gem, `require`, `Asciidoctor::VERSION`, the
// Hash vs. shorthand attribute forms, safe modes) with no counterpart in this
// crate, so it is non-normative here. Two claims do have counterparts: that the
// API can run the load and convert steps together or separately, and that its
// entrypoints parse a string or file into a document model and convert it. This
// crate verifies those against its own API — `convert`/`convert_file` for the
// combined path and `asciidoc_parser`'s `Parser` (load) plus `convert_document`
// (convert) for the separate path. The page is purely about the API, so —
// unlike the shared introduction and get-started pages — it is tracked only
// from this crate.

non_normative!(
    r#"
= Process AsciiDoc Using the API
:url-api: {url-api-gems}/asciidoctor/{release-version}

Asciidoctor provides a {url-api}[Ruby API^] named `Asciidoctor` for parsing, analyzing, and converting AsciiDoc content.
This API is intended for use in applications, scripts, integrations with other Ruby software, such as Rails, GitHub, and GitLab, and by other languages, such as Java (via xref:asciidoctorj::index.adoc[AsciidoctorJ]) and JavaScript (via xref:asciidoctor.js::index.adoc[Asciidoctor.js]).

This page examines the purpose of the API and how it can be used, shows how to get started with it, introduces its primary entry points, and links to where you can find more information to start putting it into practice.

[#when-to-use]
== When to use the API

If all you need to do is convert an AsciiDoc file to a publishable format such as HTML, then the xref:cli:index.adoc[`asciidoctor`] CLI should suit your needs.
Here's an example showing how to convert an AsciiDoc document to HTML using the CLI:

 $ asciidoctor -d book -n -a toc=left -a source-highlighter=highlight.js document.adoc

Here's the same example showing how to convert the document to HTML using the API:

[,ruby]
----
require 'asciidoctor'

Asciidoctor.convert_file 'document.adoc', doctype: :book, safe: :unsafe,
  attributes: { 'toc' => 'left', 'sectnums' => '', 'source-highlighter' => 'highlight.js' }
----

IMPORTANT: When passing attributes using the Hash format, the attribute name *must* be specified as a String, not a Symbol.
In other words, `toc: 'left'` is not recognized as an attribute entry (since `toc:` creates a Symbol key).
It must be specified as `'toc' \=> 'left'`, as shown in the previous example.

This API call can be abbreviated by declaring the document attributes using shorthand form:

[,ruby]
----
require 'asciidoctor'

Asciidoctor.convert_file 'document.adoc', doctype: :book, safe: :unsafe,
  attributes: 'toc=left sectnums source-highlighter=highlight.js'
----

In contrast to the CLI, the API enables you to break down the processing into smaller steps.
By doing so, you can capture the result of a single step, analyze the parsed document, interface with the document model in an extension, or just gain more control over the processing in general.

When using the API, we talk about two main steps:

load:: In this step, the AsciiDoc content is parsed into a document object model.
This model represents an in-memory hierarchy of the AsciiDoc content as a tree of Ruby objects.
These objects represent the elements in the AsciiDoc document, down to the block level, along with any metadata, such as document and block attributes.

convert:: In this step, the previously prepared document object model is converted into the specified output format using a converter.
In coordination with the converter, the processor passes each node in the document object model to the converter to be converted.
The result is then combined and either returned or written to a file, depending on the options passed to the API.

"#
);

// The API can run the two steps together or separately. Together maps to this
// crate's `convert` (string) and `convert_file` (file). Separately maps to
// loading with `asciidoc_parser`'s `Parser` and then calling `convert_document`
// on the resulting document — the analog of `#convert` on the document object.
#[test]
fn steps_together_or_separately() {
    verifies!(
        r#"
You can use the API to perform these two steps together (like the CLI) using `convert` and `convert_file`, or separately using `load` and `load_file` followed by calling `#convert` on the document object.
The API can load and convert AsciiDoc files (`load_file` and `convert_file`) or strings (`load` and `convert`).
The <<API entrypoints>> section introduces these methods in more detail.
"#
    );

    let source = "= Hello\n\nWorld.";

    // Together: `convert` loads and converts a string in a single call.
    let together = convert(source);

    // Separately: load the string into a document model with the parser, then
    // render that model with `convert_document` (this crate's `#convert` on the
    // document). The combined and separate paths agree.
    let doc = Parser::default().parse(source);
    assert_eq!(together, convert_document(&doc));

    // The file counterpart, `convert_file`, loads and converts a file; for the
    // same source it matches the string path.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-steps-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");
    let from_file = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);
    assert_eq!(from_file, together);

    assert!(together.starts_with("<!DOCTYPE html>"));
    assert!(together.contains("<title>Hello</title>"));
}

non_normative!(
    r#"

The API is the main way to interface with Asciidoctor in an application or integration library.
By using the API, you avoid having to invoke Asciidoctor using a subprocess.
You'll find that it also gives you a lot more control over the processing than what the CLI affords.
The API is also a useful tool in scripts, where you may need to dive deeper into the document model beyond where extensions allow you to go.
It is also instrumental in the development of extensions.

Let's learn how to get started with the API by requiring the Asciidoctor library.

[#require]
== Start with `require`

To get started with the Asciidoctor API, you first need to xref:install:index.adoc[install the gem].
The gem provides both the CLI and the API.
In fact, the CLI uses the API to handle AsciiDoc conversion.

Whereas with the CLI, you interact with Asciidoctor using the `asciidoctor` command, when using the API, you interact with Asciidoctor using methods on the `Asciidoctor` module.

In order to make the `Asciidoctor` module available in a Ruby script or application, you must require the `asciidoctor` gem using Ruby's `require` function:

[,ruby]
----
require 'asciidoctor'
----

This one statement makes all of the {url-api}[public APIs in Asciidoctor^] available to your Ruby code.
For example, we can check which version of Asciidoctor was required by Ruby using the following statement:

[,ruby,subs=attributes+]
----
puts Asciidoctor::VERSION
# => {release-version}
----

If the `require` statement fails, check to make sure that you have the gem installed (and available on the GEM_PATH).

[#entrypoints]
== API entrypoints

The CLI is primarily intended for converting AsciiDoc.
While the API can do that as well, it can do so much more.
Instead of just converting AsciiDoc, the API allows you to load the AsciiDoc into a document model.
From there, you can either convert the document model to an output format, or you can pause to analyze or modify its structure.

"#
);

// The four Ruby entrypoints. `load`/`load_file` parse source or a file into an
// `Asciidoctor::Document`; this crate's load step is `asciidoc_parser`'s
// `Parser`, which yields a `Document`. `convert`/`convert_file` parse and
// convert a string or file to the output format — HTML5 here, the only backend
// this crate provides.
#[test]
fn four_entrypoints() {
    verifies!(
        r#"
There are four main entrypoints in the Asciidoctor API:

`Asciidoctor.load`:: parses the AsciiDoc source (down to the block level) into an `Asciidoctor::Document` object
`Asciidoctor.load_file`:: parses the contents of the AsciiDoc source file (down to the block level) into an `Asciidoctor::Document` object
`Asciidoctor.convert`:: parses and converts the AsciiDoc source to the output format determined by the specified backend
`Asciidoctor.convert_file`:: parses and converts the contents of the AsciiDoc source file to the output format determined by the specified backend
"#
    );

    let source = "= Hello\n\nWorld.";

    // `load`: parse the source into a document model (a `Document`, the analog of
    // an `Asciidoctor::Document`).
    let doc = Parser::default().parse(source);

    // `convert`: parse and convert a string to the output format. Rendering the
    // separately loaded document gives the same HTML5.
    let html = convert(source);
    assert_eq!(html, convert_document(&doc));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Hello</title>"));

    // `convert_file`: parse and convert the contents of a file; its output
    // matches converting the same source held in memory.
    let path = std::env::temp_dir().join(format!(
        "asciidoc-html5-api-entrypoints-{}.adoc",
        std::process::id()
    ));
    fs::write(&path, source).expect("write temp input");
    let from_file = convert_file(&path).expect("convert_file reads and renders");
    let _ = fs::remove_file(&path);
    assert_eq!(from_file, html);
}

non_normative!(
    r#"

If you're processing a file, you'd typically use a method that ends with `_file`.
Otherwise, you'd use its complement, which accepts a String, an array of Strings, or an IO object.

By default, the output of the convert methods follows the input.
If you convert a String, the method will output a String.
If you convert a file, the method will output to a file.
However, these defaults can be changed using the options (e.g., `:to_file`).

When calling Asciidoctor using the API, you can access xref:options.adoc[additional options that control processing] which are not available when using the CLI.
For example, you can pass in extensions, parse only the header, enable the sourcemap, or catalog assets.

In addition to the main entrypoints, the API also provides a mechanism to register and develop extensions.
To learn more about extensions, see xref:extensions:index.adoc[].

== Next steps

* xref:convert-files.adoc[Load and convert AsciiDoc from a file]
* xref:convert-strings.adoc[Load and convert AsciiDoc from a string]
* xref:convert-strings.adoc#embedded-output[Generating embedded vs standalone output]
* xref:convert-strings.adoc#convert-inline-markup-only[Convert inline markup only]
* xref:generate-html-toc.adoc[Generate an HTML TOC]
* xref:load-templates.adoc[Load custom templates]
* xref:extensions:index.adoc[Create extensions]
* xref:options.adoc[API options]
* {url-api}[Ruby API docs^]
"#
);
