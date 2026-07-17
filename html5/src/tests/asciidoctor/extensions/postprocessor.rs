// Asciidoctor's "Postprocessor Extension Example" page. It walks through a
// Postprocessor that rewrites the converted output before it is written.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/postprocessor.adoc");

non_normative!(
    r#"
= Postprocessor Extension Example
:navtitle: Postprocessor

Purpose::
Insert copyright text in the footer.

== CopyrightFooterPostprocessor

[,ruby]
----
class CopyrightFooterPostprocessor < Asciidoctor::Extensions::Postprocessor
  def process document, output
    content = (document.attr 'copyright') || 'Copyright Acme, Inc.'
    if document.basebackend? 'html'
      replacement = %(<div id="footer-text">\\1<br>\n#{content}\n</div>)
      output = output.sub(/<div id="footer-text">(.*?)<\/div>/m, replacement)
    elsif document.basebackend? 'docbook'
      replacement = %(<simpara>#{content}</simpara>\n\\1)
      output = output.sub(/(<\/(?:article|book)>)/, replacement)
    end
    output
  end
end
----

== Usage

[,ruby]
----
Asciidoctor::Extensions.register do
  postprocessor CopyrightFooterPostprocessor
end

Asciidoctor.convert_file 'sample-with-copyright-footer.adoc', safe: :safe
----
"#
);
