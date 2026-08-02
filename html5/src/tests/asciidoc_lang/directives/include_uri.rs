//! Coverage of the AsciiDoc language description's *Include Content by URI*
//! page.
//!
//! This page documents reading include content from a URI, gated on the
//! `allow-uri-read` attribute. Reading remote resources over the network is a
//! deliberate non-goal of this crate — neither the library nor the `adoc` CLI
//! performs network reads, and `allow-uri-read` is not implemented
//! (<https://github.com/asciidoc-rs/asciidoc-html5/issues/39>). The entire page
//! is therefore tracked as non-normative: there is no in-scope behavior to
//! verify.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-uri.adoc");

// Non-normative in full: every rule on this page concerns reading content from
// a URI via `allow-uri-read`, a networked capability this crate does not
// provide (remote fetch is a non-goal). Nothing here maps to behavior the
// renderer can verify.
non_normative!(
    r#"
= Include Content by URI
//aka Include Content from a URI
:url-http: https://www.w3.org/Protocols/rfc2616/rfc2616-sec13.html
:url-uri: https://en.wikipedia.org/wiki/Uniform_Resource_Identifier

== Reference include content by URI

The include directive recognizes when the target is a URI and can include the content referenced by that URI.
This example demonstrates how to include an AsciiDoc file from a GitHub repository directly into your document.

----
include::example$include.adoc[tag=uri]
----

For security reasons, this capability is *not enabled by default*.
To allow content to be read from a URI, you must enable the URI read permission by:

. running Asciidoctor in `SERVER` mode or less and
. setting the `allow-uri-read` attribute securely from the CLI or API

Here's an example that shows how to run Asciidoctor from the console so it can read content from a URI:

 $ asciidoctor -a allow-uri-read filename.adoc

Remember that Asciidoctor executes in `UNSAFE` mode by default when run from the command line.

Here's an example that shows how to run Asciidoctor from the API so it can read content from a URI:

[source,ruby]
----
Asciidoctor.convert_file 'filename.adoc', safe: :safe, attributes: { 'allow-uri-read' => '' }
----

WARNING: Including content from sources outside your control carries certain risks, including the potential to introduce malicious behavior into your documentation.
Because `allow-uri-read` is a potentially dangerous feature, it is forcefully disabled when the safe mode is `SECURE` or higher.

.URI vs URL
****
URI stands for {url-uri}[Uniform Resource Identifier^].
When we talk about a URI, we're usually talking about a URL, or Uniform Resource Locator.
A URL is simply a URI that points to a resource over a network, or web address.

As far as Asciidoctor is concerned, all URIs share the same restriction, whether or not it's actually local or remote, or whether it points to a web address (http or https prefix), FTP address (ftp prefix), or some other addressing scheme.
****

The same restriction described in this section applies when embedding an image referenced from a URI, such as when `data-uri` is set or when converting to PDF using Asciidoctor PDF.

=== Caching URI content

Reading content from a URI is obviously much slower than reading it from a local file.

Asciidoctor provides a way for the content read from a URI to be cached, which is highly recommended.

To enable the built-in cache, you must:

. Install the `open-uri-cached` gem.
. Set the `cache-uri` attribute in the document.

When these two conditions are satisfied, Asciidoctor caches content read from a URI according the to {url-http}[HTTP caching recommendations^].
"#
);
