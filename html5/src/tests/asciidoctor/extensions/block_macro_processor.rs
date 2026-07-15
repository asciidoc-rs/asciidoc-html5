// Asciidoctor's "Block Macro Processor Extension Example" page. It walks
// through a BlockMacroProcessor that handles a custom block macro.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/block-macro-processor.adoc");

non_normative!(
    r#"
= Block Macro Processor Extension Example
:navtitle: Block Macro Processor

Purpose::
Create a block macro named `gist` for embedding a gist.

== sample-with-gist-macro.adoc

[,asciidoc]
----
.My Gist
gist::123456[]
----

== GistBlockMacro

[,ruby]
----
class GistBlockMacro < Asciidoctor::Extensions::BlockMacroProcessor
  use_dsl

  named :gist

  def process parent, target, attrs
    title_html = (attrs.has_key? 'title') ?
        %(<div class="title">#{attrs['title']}</div>\n) : nil

    html = %(<div class="openblock gist">
#{title_html}<div class="content">
<script src="https://gist.github.com/#{target}.js"></script>
</div>
</div>)

    create_pass_block parent, html, attrs, subs: nil
  end
end
----

== Usage

[,ruby]
----
Asciidoctor::Extensions.register do
  block_macro GistBlockMacro if document.basebackend? 'html'
end

Asciidoctor.convert_file 'sample-with-gist.adoc', safe: :safe
----
"#
);
