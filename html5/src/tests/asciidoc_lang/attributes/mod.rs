//! Coverage of the AsciiDoc language description's `attributes` module pages.
//!
//! These pages specify document and element attributes. Attribute *parsing*
//! (names, values, precedence, the attrlist grammar) is the concern of
//! `asciidoc-parser`, which tracks the same pages against its own AST; here we
//! verify the *rendered* effect of an attribute wherever this renderer produces
//! one — a `role` becomes an HTML `class`, an `id` becomes an `id`, a counter
//! or attribute reference is substituted into the output, and so on. Claims
//! that describe only parser-internal behavior, with no distinct rendering, are
//! tracked as non-normative here and verified in `asciidoc-parser` instead.

mod attribute_entry_substitutions;
mod character_replacement_ref;
mod counters;
mod inline_attribute_entries;
mod options;
mod reference_attributes;
mod role;
mod unresolved_references;
mod unset_attributes;
mod wrap_values;
