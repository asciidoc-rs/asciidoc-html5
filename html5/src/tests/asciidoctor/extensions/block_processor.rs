// Asciidoctor's "Block Processor Extension Example" page. It walks through a
// BlockProcessor that handles a block marked with a custom style.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/block-processor.adoc");

non_normative!(
    r#"
= Block Processor Extension Example
:navtitle: Block Processor

Purpose::
Register a custom block style named `shout` that uppercases all the words and converts periods to exclamation points.

== sample-with-shout-block.adoc

[,asciidoc]
----
[shout]
The time is now. Get a move on.
----

== ShoutBlock

[,ruby]
----
class ShoutBlock < Asciidoctor::Extensions::BlockProcessor
  PeriodRx = /\.(?= |$)/

  use_dsl

  named :shout
  on_context :paragraph
  name_positional_attributes 'vol'
  parse_content_as :simple

  def process parent, reader, attrs
    volume = ((attrs.delete 'vol') || 1).to_i
    create_paragraph parent, (reader.lines.map {|l| l.upcase.gsub PeriodRx, '!' * volume }), attrs
  end
end
----

== Usage

[,ruby]
----
Asciidoctor::Extensions.register do
  block ShoutBlock
end

Asciidoctor.convert_file 'sample-with-shout-block.adoc', safe: :safe
----
"#
);
