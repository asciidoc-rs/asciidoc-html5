//! Small helpers for assembling HTML output.
//!
//! The parser has already applied inline substitutions to block *content* and
//! *titles* (see [`crate`] module docs), so those strings are emitted verbatim.
//! These helpers only cover the text that this crate itself places into markup:
//! attribute values (ids, class lists) and the occasional literal that has not
//! passed through the parser's substitution pipeline.

/// Escapes `value` for inclusion inside a double-quoted HTML attribute.
///
/// Ids and roles come straight from the source and are dropped into `id="…"`
/// and `class="…"`. They are normally simple tokens, but we escape defensively
/// so a stray `"`, `&`, `<`, or `>` cannot break out of the attribute.
pub(crate) fn escape_attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Builds the ` id="…"` fragment for a block wrapper, or an empty string when
/// the block has no id. The leading space is included so call sites can splice
/// the result directly into an opening tag.
pub(crate) fn id_attribute(id: Option<&str>) -> String {
    match id {
        Some(id) => format!(" id=\"{}\"", escape_attribute(id)),
        None => String::new(),
    }
}

/// Builds the ` class="…"` fragment from a base class plus any author-supplied
/// roles, matching Asciidoctor's convention of appending roles as extra
/// classes (e.g. `class="paragraph lead"`). Passing no base and no roles yields
/// an empty string.
pub(crate) fn class_attribute(base: &str, roles: &[&str]) -> String {
    if base.is_empty() && roles.is_empty() {
        return String::new();
    }

    let mut classes = String::new();
    if !base.is_empty() {
        classes.push_str(base);
    }
    for role in roles {
        if !classes.is_empty() {
            classes.push(' ');
        }
        classes.push_str(&escape_attribute(role));
    }

    format!(" class=\"{classes}\"")
}
