// Asciidoctor's "Docinfo Processor Extension Example" page. It walks through
// a DocinfoProcessor that injects content into the document header or footer.
//
// This crate implements no extension mechanism (none is planned for 1.0; see
// the workspace README), so the page states no rule this renderer can satisfy
// and is tracked in full as non-normative. See `sdd/README.md`.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/docs/modules/extensions/pages/docinfo-processor.adoc");

non_normative!(
    r#"
= Docinfo Processor Extension Example
:navtitle: Docinfo Processor

Purpose::
Appends the Google Analytics tracking code to the bottom of an HTML document.

== GoogleAnalyticsDocinfoProcessor

[,ruby]
----
class GoogleAnalyticsDocinfoProcessor < Asciidoctor::Extensions::DocinfoProcessor
  use_dsl
  at_location :footer
  def process document
    return unless (ga_account_id = document.attr 'google-analytics-account')
    %(<script>
(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){
(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),
m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)
})(window,document,'script','https://www.google-analytics.com/analytics.js','ga');
ga('create','#{ga_account_id}','auto');
ga('send','pageview');
</script>)
  end
end
----

== Usage

[,ruby]
----
Asciidoctor::Extensions.register do
  docinfo_processor GoogleAnalyticsDocinfoProcessor
end

Asciidoctor.convert_file 'sample.adoc', safe: :safe, attributes: 'google-analytics-account=UA-ABCXYZ123'
----
"#
);
