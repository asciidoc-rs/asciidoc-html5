//! Coverage of the AsciiDoc language description's *Hide Special Section
//! Titles* page.
//!
//! The `notitle` (formerly `untitled`) option to hide a special section's title
//! is a DocBook-only feature: per the section-styles reference table it is only
//! honored by converters that support it, and the HTML5 backend does not. This
//! crate matches Asciidoctor 2.0.26, which still renders the `<h2>` title for a
//! `[dedication%notitle]` section, so the page describes behavior that is out
//! of scope here and the whole page is tracked non-normatively.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/special-section-titles.adoc");

// The `notitle`/`untitled` option only hides a special section's title in the
// DocBook backend; the HTML5 backend (this crate, matching Asciidoctor 2.0.26)
// still renders the title's `<h2>`. DocBook output is out of scope, so the
// entire page describes behavior that cannot be verified here.
non_normative!(
    r#"
= Hide Special Section Titles

If supported by the converter, the title of a xref:styles.adoc[special section], such as the Dedication, can be turned off by setting the `notitle` option (e.g., `%notitle` or `opts=notitle`) (previously `untitled`) on the section.

----
[dedication%notitle]
== Dedication

include::example$dedication.adoc[tag=body]
----

Although the title is hidden in the output document, it still needs to be specified in the AsciiDoc source for the purpose of referencing.
The title will be used as the reftext of a cross reference, just as with any section.
"#
);
