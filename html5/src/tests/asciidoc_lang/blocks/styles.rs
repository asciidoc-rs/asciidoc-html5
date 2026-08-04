//! Coverage of the AsciiDoc language description's *Block Style* page.
//!
//! This page is archived: it carries no rules of its own, only a redirect to
//! the block-style section of the delimited-blocks index. The whole page is
//! therefore tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/blocks/pages/styles.adoc");

// Archived redirect stub — the page states only that its former content moved
// to `index.adoc#block-style`, so there is no behavior to verify here.
non_normative!(
    r#"
= Block Style
:page-archived:

Moved xref:index.adoc#block-style[here].
"#
);
