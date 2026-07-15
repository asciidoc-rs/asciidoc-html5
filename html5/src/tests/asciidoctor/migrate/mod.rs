//! Coverage of Asciidoctor's `migrate` documentation module under
//! `ref/asciidoctor/docs/modules/migrate/pages/`.
//!
//! These pages help authors move content into AsciiDoc from other formats
//! (Markdown, Confluence XHTML, DocBook XML, MS Word, the legacy AsciiDoc.py
//! processor) or upgrade between Asciidoctor releases. They document external
//! conversion tooling and authoring- and processor-migration workflows, none of
//! which states a rule for this HTML5 renderer to satisfy, so every page is
//! tracked wholesale as non-normative.

mod asciidoc_py;
mod confluence_xhtml;
mod docbook_xml;
mod markdown;
mod ms_word;
mod upgrade;
