// Asciidoctor's "Include Processor Extension Example" page. It walks through
// an IncludeProcessor that resolves an include directive.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/include-processor.adoc");

non_normative!(
    r#"
= Include Processor Extension Example
:navtitle: Include Processor

Purpose::
Include a file from a URI.

TIP: Asciidoctor supports including content from a URI out of the box if you set the `allow-uri-read` attribute (not available if the safe mode is `secure`).

== sample-with-uri-include.adoc

[source,asciidoc]
....
:source-highlighter: coderay

.Gemfile
[,ruby]
----
\include::https://cdn.jsdelivr.net/gh/asciidoctor/asciidoctor/Gemfile[]
----
....

== UriIncludeProcessor

[,ruby]
----
class UriIncludeProcessor < Asciidoctor::Extensions::IncludeProcessor
  def handles? target
    target.start_with? 'https://', 'https://'
  end

  def process doc, reader, target, attributes
    content = (::OpenURI.open_uri target).readlines
    reader.push_include content, target, target, 1, attributes
    reader
  end
end
----

== Usage

[,ruby]
----
Asciidoctor::Extensions.register do
  include_processor UriIncludeProcessor
end

Asciidoctor.convert_file 'sample-with-uri-include.adoc', safe: :safe
----
"#
);
