//! Externally-supplied document attributes for a conversion.
//!
//! [`Options`] carries a set of document attributes to seed into a conversion
//! from *outside* the document source — the equivalent of Asciidoctor's
//! `-a name=value` CLI option and the `:attributes` API option. It is the
//! parameter accepted by the `_with` conversion entry points ([`convert_with`]
//! and [`convert_file_with`]).
//!
//! # Override vs. default (soft) precedence
//!
//! Each attribute is seeded with one of two precedences, matching Asciidoctor:
//!
//! - **Override** (the default here, and Asciidoctor's `-a name=value`): the
//!   external value wins and the document *cannot* change it — an assignment of
//!   the same name in the document header or body is ignored.
//! - **Default** (Asciidoctor's soft-set `-a name=value@`): the external value
//!   is only a starting point; an assignment of the same name in the document
//!   header or body overrides it.
//!
//! The precedence maps directly onto `asciidoc-parser`'s
//! [`ModificationContext`]: an override becomes [`ModificationContext::ApiOnly`]
//! (locked against the document), a default becomes
//! [`ModificationContext::Anywhere`] (the document may reassign it).
//!
//! ```
//! use asciidoc_html5::{convert_with, Options};
//!
//! // Override: the API value wins over the document header.
//! let opts = Options::new().attribute("webfonts", "Ubuntu+Mono:400");
//! let html = convert_with("= Doc\n:webfonts: ignored\n\nBody.", &opts);
//! assert!(html.contains("family=Ubuntu+Mono:400"));
//! ```
//!
//! [`convert_with`]: crate::convert_with
//! [`convert_file_with`]: crate::convert_file_with

use asciidoc_parser::{parser::ModificationContext, Parser};

/// A set of document attributes to supply to a conversion from outside the
/// document source.
///
/// `Options` is a builder: start from [`Options::new`] (or
/// [`Options::default`]) and chain one call per attribute. Each call records a
/// directive; the directives are applied in order when the options are handed
/// to a `_with` conversion entry point, so a later call for the same attribute
/// name supersedes an earlier one.
///
/// See the [module documentation](self) for override vs. default precedence.
///
/// # Examples
///
/// ```
/// use asciidoc_html5::{convert_with, Options};
///
/// let opts = Options::new().set("linkcss").unset("webfonts");
/// let html = convert_with("= Doc\n\nBody.", &opts);
/// assert!(html.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The attribute directives, in the order they were added.
    attributes: Vec<Directive>,
}

/// One recorded attribute directive: a name, what to do with it, and whether
/// the external value overrides the document or merely defaults it.
#[derive(Clone, Debug)]
struct Directive {
    /// The attribute name (lowercased, as `asciidoc-parser` stores names).
    name: String,

    /// Whether to assign a value, set the attribute, or unset it.
    action: Action,

    /// Whether the external value wins over the document (`Override`) or the
    /// document wins if it assigns the same name (`Default`).
    precedence: Precedence,
}

/// What a [`Directive`] does to its attribute.
#[derive(Clone, Debug)]
enum Action {
    /// Assign an explicit string value (`name=value`).
    Value(String),

    /// Set the attribute with no explicit value (`name`).
    Set,

    /// Unset the attribute (`name!`).
    Unset,
}

/// Whether an externally-supplied attribute overrides the document or only
/// provides a default the document may override.
#[derive(Clone, Copy, Debug)]
enum Precedence {
    /// The external value wins; the document cannot change it. Maps to
    /// [`ModificationContext::ApiOnly`].
    Override,

    /// The external value is a default; a document assignment of the same name
    /// wins. Maps to [`ModificationContext::Anywhere`].
    Default,
}

impl Precedence {
    /// The `asciidoc-parser` modification context this precedence seeds with.
    fn modification_context(self) -> ModificationContext {
        match self {
            Precedence::Override => ModificationContext::ApiOnly,
            Precedence::Default => ModificationContext::Anywhere,
        }
    }
}

impl Options {
    /// Creates an empty set of options — no attributes supplied. Converting
    /// with it is equivalent to calling [`convert`](crate::convert).
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the attribute `name` with an explicit string `value`.
    ///
    /// This is Asciidoctor's `-a name=value`: the value wins over any
    /// assignment of the same name in the document header or body. Use
    /// [`attribute_default`](Self::attribute_default) for the soft-set form the
    /// document can override.
    pub fn attribute<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        self.push(name, Action::Value(value.into()), Precedence::Override);
        self
    }

    /// Sets the attribute `name` (with no explicit value), overriding the
    /// document.
    ///
    /// This is Asciidoctor's `-a name`: the attribute is turned on and the
    /// document cannot change it. Use [`set_default`](Self::set_default) for
    /// the soft-set form.
    pub fn set<N: Into<String>>(mut self, name: N) -> Self {
        self.push(name, Action::Set, Precedence::Override);
        self
    }

    /// Unsets the attribute `name`, overriding the document.
    ///
    /// This is Asciidoctor's `-a name!`: the attribute is turned off and the
    /// document cannot turn it back on. Use
    /// [`unset_default`](Self::unset_default) for the soft-set form.
    pub fn unset<N: Into<String>>(mut self, name: N) -> Self {
        self.push(name, Action::Unset, Precedence::Override);
        self
    }

    /// Assigns `value` to the attribute `name` as a default the document may
    /// override.
    ///
    /// This is Asciidoctor's soft-set `-a name=value@`: the value applies only
    /// when the document does not assign the same name itself.
    pub fn attribute_default<N: Into<String>, V: Into<String>>(
        mut self,
        name: N,
        value: V,
    ) -> Self {
        self.push(name, Action::Value(value.into()), Precedence::Default);
        self
    }

    /// Sets the attribute `name` as a default the document may override.
    ///
    /// This is Asciidoctor's soft-set `-a name@`.
    pub fn set_default<N: Into<String>>(mut self, name: N) -> Self {
        self.push(name, Action::Set, Precedence::Default);
        self
    }

    /// Unsets the attribute `name` as a default the document may override.
    ///
    /// This is Asciidoctor's soft-set `-a name!@`.
    pub fn unset_default<N: Into<String>>(mut self, name: N) -> Self {
        self.push(name, Action::Unset, Precedence::Default);
        self
    }

    /// Records one directive. Names are lowercased to match how
    /// `asciidoc-parser` stores attribute names.
    fn push<N: Into<String>>(&mut self, name: N, action: Action, precedence: Precedence) {
        self.attributes.push(Directive {
            name: name.into().to_lowercase(),
            action,
            precedence,
        });
    }

    /// Seeds `parser` with the recorded attribute directives, returning the
    /// parser ready to parse. Directives are applied in order, so a later one
    /// for the same name wins.
    pub(crate) fn apply(&self, mut parser: Parser) -> Parser {
        for directive in &self.attributes {
            let context = directive.precedence.modification_context();
            parser = match &directive.action {
                Action::Value(value) => {
                    parser.with_intrinsic_attribute(&directive.name, value, context)
                }
                Action::Set => parser.with_intrinsic_attribute_bool(&directive.name, true, context),
                Action::Unset => {
                    parser.with_intrinsic_attribute_bool(&directive.name, false, context)
                }
            };
        }
        parser
    }
}

#[cfg(test)]
mod tests {
    use crate::{convert, convert_with, Options};

    // The default web-font family, present when `webfonts` is set with no value.
    const DEFAULT_FAMILY: &str = "Open+Sans:300,300italic,400,400italic,600,600italic%7CNoto+Serif:400,400italic,700,700italic%7CDroid+Sans+Mono:400,700";

    fn font_link(family: &str) -> String {
        format!(
            "<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com/css?family={family}\">"
        )
    }

    #[test]
    fn empty_options_match_plain_convert() {
        let source = "= Doc\n:webfonts: from-header\n\nBody.";
        assert_eq!(convert_with(source, &Options::new()), convert(source));
    }

    #[test]
    fn attribute_value_supplies_a_value_absent_from_the_document() {
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new().attribute("webfonts", "X:400"),
        );
        assert!(html.contains(&font_link("X:400")));
    }

    #[test]
    fn set_turns_an_attribute_on() {
        let html = convert_with("= Doc\n\nBody.", &Options::new().set("linkcss"));
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
        assert!(!html.contains("<style>"));
    }

    #[test]
    fn unset_turns_an_attribute_off() {
        let html = convert_with("= Doc\n\nBody.", &Options::new().unset("webfonts"));
        assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
        // The default stylesheet is still embedded.
        assert!(html.contains("<style>"));
    }

    // An override (`-a name=value`) wins over a document-header assignment of
    // the same name: the header value is ignored.
    #[test]
    fn override_beats_the_document_header() {
        let source = "= Doc\n:webfonts: from-header\n\nBody.";
        let html = convert_with(source, &Options::new().attribute("webfonts", "from-api"));
        assert!(html.contains(&font_link("from-api")));
        assert!(!html.contains("from-header"));
    }

    // An override to unset locks the attribute off even when the header sets it.
    #[test]
    fn override_unset_beats_a_header_value() {
        let source = "= Doc\n:webfonts: from-header\n\nBody.";
        let html = convert_with(source, &Options::new().unset("webfonts"));
        assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
    }

    // A soft-set default (`-a name=value@`) yields to a document-header
    // assignment of the same name.
    #[test]
    fn default_yields_to_the_document_header() {
        let source = "= Doc\n:webfonts: from-header\n\nBody.";
        let html = convert_with(
            source,
            &Options::new().attribute_default("webfonts", "from-api"),
        );
        assert!(html.contains(&font_link("from-header")));
        assert!(!html.contains("from-api"));
    }

    // A soft-set default still applies when the document does not assign the
    // same name.
    #[test]
    fn default_applies_when_the_document_is_silent() {
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new().attribute_default("webfonts", "from-api"),
        );
        assert!(html.contains(&font_link("from-api")));
    }

    // Later directives for the same name supersede earlier ones.
    #[test]
    fn a_later_directive_wins() {
        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new()
                .attribute("webfonts", "first")
                .attribute("webfonts", "second"),
        );
        assert!(html.contains(&font_link("second")));
    }

    // Attribute names are case-insensitive, matching how the parser stores them.
    #[test]
    fn attribute_names_are_lowercased() {
        let html = convert_with("= Doc\n\nBody.", &Options::new().unset("WebFonts"));
        assert!(!html.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));
    }

    // A bare `set` keeps the default family (Set, not an empty value).
    #[test]
    fn set_keeps_the_default_family() {
        let html = convert_with("= Doc\n\nBody.", &Options::new().set("webfonts"));
        assert!(html.contains(&font_link(DEFAULT_FAMILY)));
    }
}
