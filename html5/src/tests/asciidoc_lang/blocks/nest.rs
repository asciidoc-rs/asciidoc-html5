//! Coverage of the AsciiDoc language description's *Nesting Blocks* page.
//!
//! This page is archived: it carries no rules of its own, only a redirect to
//! the nesting section of the delimited-blocks page. The whole page is
//! therefore tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/nest.adoc");

// Archived redirect stub — the page states only that its former content moved
// to `delimited.adoc#nesting`, so there is no behavior to verify here.
non_normative!(
    r#"
= Nesting Blocks
:page-archived:

Moved xref:delimited.adoc#nesting[here].
"#
);
