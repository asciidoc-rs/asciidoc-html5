// Asciidoctor's "Compound Block Processor Example" page. It walks through a
// BlockProcessor that expands a block into a compound block of child blocks.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/compound-block-processor.adoc");

non_normative!(
    r#"
= Compound Block Processor Example
:navtitle: Compound Block Processor

Purpose::
Register a custom block named `collapsible` that transforms a listing block into a compound block composed of the following:

* an example block with the collapsible option enabled
* the original listing block
* the listing block is promoted to a source block if a language is specified using the second positional attribute.

.sample-with-collapsible-block.adoc
[source,asciidoc]
....
.Show JSON
[collapsible,json]
----
{
   "foo": "bar"
}
----
....

.collapsible-block.rb
[,ruby]
----
class CollapsibleBlock < Asciidoctor::Extensions::BlockProcessor
  enable_dsl
  on_context :listing
  positional_attributes 'language'

  def process parent, reader, attrs
    lang = attrs.delete 'language'
    attrs['title'] ||= 'Show Listing'
    example = create_example_block parent, [], attrs, content_model: :compound
    example.set_option 'collapsible'
    listing = create_listing_block example, reader.readlines, nil
    if lang
      listing.style = 'source'
      listing.set_attr 'language', lang
      listing.commit_subs
    end
    example << listing
    example
  end
end

Asciidoctor::Extensions.register do
  block CollapsibleBlock, :collapsible
end
----

.Usage
 $ asciidoctor -r ./collapsible-block.rb sample-with-collapsible-block.adoc

NOTE: This extension mimics the builtin `collapsible` option on the example block, but consolidates it to a single block.
The purpose of this extension is to show how to assemble a compound block in an extension.
"#
);
