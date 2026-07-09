//! Per-conversion options: externally-supplied document attributes and the
//! [safe mode].
//!
//! [`Options`] carries the settings applied to a conversion from *outside* the
//! document source: a set of document attributes (the equivalent of
//! Asciidoctor's `-a name=value` CLI option and the `:attributes` API option)
//! and the [`SafeMode`] under which the document is processed (Asciidoctor's
//! `:safe` option). It is the parameter accepted by the `_with` conversion
//! entry points ([`convert_with`] and [`convert_file_with`]).
//!
//! # Safe mode
//!
//! The safe mode governs security-sensitive rendering. Following Asciidoctor,
//! it also decides whether the default stylesheet is *linked* or *embedded*:
//! under [`SafeMode::Secure`] (the default here, matching Asciidoctor's API)
//! the converter links to `./asciidoctor.css` unless the caller sets `linkcss`
//! explicitly; under a lower mode it embeds the stylesheet inline. See
//! [`Options::safe_mode`].
//!
//! [safe mode]: SafeMode
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

use asciidoc_parser::{parser::ModificationContext, Parser, SafeMode};

/// The options to supply to a conversion from outside the document source: a
/// set of document attributes and the [safe mode](SafeMode).
///
/// `Options` is a builder: start from [`Options::new`] (or
/// [`Options::default`]) and chain one call per attribute, plus an optional
/// [`safe_mode`](Self::safe_mode). Each attribute call records a directive; the
/// directives are applied in order when the options are handed to a `_with`
/// conversion entry point, so a later call for the same attribute name
/// supersedes an earlier one.
///
/// See the [module documentation](self) for override vs. default precedence and
/// how the safe mode gates stylesheet embedding.
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

    /// The safe mode to process the document under. `None` defaults to
    /// [`SafeMode::Secure`], matching Asciidoctor's API default.
    safe_mode: Option<SafeMode>,
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
    /// Creates an empty set of options — no attributes supplied and the default
    /// safe mode. Converting with it is equivalent to calling
    /// [`convert`](crate::convert).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`SafeMode`] the document is processed under.
    ///
    /// This is Asciidoctor's `:safe` option. When left unset, conversion uses
    /// [`SafeMode::Secure`], the most conservative mode and Asciidoctor's API
    /// default. Following Asciidoctor, `Secure` links the default stylesheet
    /// (to `./asciidoctor.css`) instead of embedding it, unless the caller sets
    /// `linkcss` explicitly; lower modes embed it inline.
    ///
    /// # Examples
    ///
    /// ```
    /// use asciidoc_html5::{convert_with, Options, SafeMode};
    ///
    /// // A mode below `Secure` embeds the default stylesheet inline.
    /// let opts = Options::new().safe_mode(SafeMode::Server);
    /// let html = convert_with("= Doc\n\nBody.", &opts);
    /// assert!(html.contains("<style>"));
    /// ```
    pub fn safe_mode(mut self, safe: SafeMode) -> Self {
        self.safe_mode = Some(safe);
        self
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

    /// Seeds `parser` with the safe mode and the recorded attribute directives,
    /// returning the parser ready to parse. Directives are applied in order, so
    /// a later one for the same name wins.
    pub(crate) fn apply(&self, mut parser: Parser) -> Parser {
        // The safe mode is established first. `with_safe_mode` also populates
        // the `safe-mode-*` intrinsic attributes, which a bare `Parser` does
        // not set on its own.
        let mode = self.safe_mode.unwrap_or(SafeMode::Secure);
        parser = parser.with_safe_mode(mode);

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

        // Matching Asciidoctor: in `Secure` (or greater), `linkcss` defaults on
        // — the converter links the stylesheet instead of embedding it — unless
        // the caller supplied `linkcss` from the API/CLI. Seeding it as an
        // override (`ApiOnly`) also locks it, so a document `:linkcss!:` cannot
        // turn embedding back on, again matching Asciidoctor.
        if mode >= SafeMode::Secure && !self.mentions("linkcss") {
            parser =
                parser.with_intrinsic_attribute_bool("linkcss", true, ModificationContext::ApiOnly);
        }

        parser
    }

    /// Whether any recorded directive names `name` (already lowercased). Used
    /// to decide whether the caller has taken control of an attribute that
    /// the safe mode would otherwise default.
    fn mentions(&self, name: &str) -> bool {
        self.attributes.iter().any(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use crate::{convert, convert_with, Options, SafeMode};

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
        // The default stylesheet is still present — linked, under the default
        // (Secure) safe mode.
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
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

    // A soft-set (`set_default`) turns an attribute on when the document is
    // silent, but yields to a document assignment of the same name.
    #[test]
    fn set_default_is_soft() {
        // Applies when the document does not touch `linkcss`.
        let applied = convert_with("= Doc\n\nBody.", &Options::new().set_default("linkcss"));
        assert!(applied.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));

        // Yields to the document, which turns `linkcss` back off.
        let overridden = convert_with(
            "= Doc\n:linkcss!:\n\nBody.",
            &Options::new().set_default("linkcss"),
        );
        assert!(!overridden.contains("./asciidoctor.css"));
        assert!(overridden.contains("<style>"));
    }

    // A soft-unset (`unset_default`) turns an attribute off when the document is
    // silent, but yields to a document assignment of the same name.
    #[test]
    fn unset_default_is_soft() {
        // Applies when the document does not touch `webfonts`.
        let applied = convert_with("= Doc\n\nBody.", &Options::new().unset_default("webfonts"));
        assert!(!applied.contains("<link rel=\"stylesheet\" href=\"https://fonts.googleapis.com"));

        // Yields to the document, which assigns `webfonts` a value.
        let overridden = convert_with(
            "= Doc\n:webfonts: X:400\n\nBody.",
            &Options::new().unset_default("webfonts"),
        );
        assert!(overridden.contains(&font_link("X:400")));
    }

    // The default safe mode is `Secure` (matching Asciidoctor's API), which
    // links the default stylesheet instead of embedding it.
    #[test]
    fn secure_is_the_default_and_links_the_stylesheet() {
        let html = convert("= Doc\n\nBody.");
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
        assert!(!html.contains("<style>"));
    }

    // A safe mode below `Secure` embeds the default stylesheet inline, the way
    // the `adoc` CLI (which defaults to `Unsafe`) does.
    #[test]
    fn a_lower_safe_mode_embeds_the_stylesheet() {
        for mode in [SafeMode::Unsafe, SafeMode::Safe, SafeMode::Server] {
            let html = convert_with("= Doc\n\nBody.", &Options::new().safe_mode(mode));
            assert!(html.contains("<style>"), "{mode:?} should embed");
            assert!(
                !html.contains("./asciidoctor.css"),
                "{mode:?} should not link"
            );
        }
    }

    // Under `Secure`, `linkcss` is locked on: a document `:linkcss!:` cannot
    // turn embedding back on (parity with Asciidoctor's api_test).
    #[test]
    fn secure_locks_linkcss_against_the_document() {
        let html = convert_with("= Doc\n:linkcss!:\n\nBody.", &Options::new());
        assert!(html.contains("<link rel=\"stylesheet\" href=\"./asciidoctor.css\">"));
        assert!(!html.contains("<style>"));
    }

    // An API-level `linkcss` unset wins over the `Secure` default, so the
    // stylesheet is embedded even under `Secure`.
    #[test]
    fn an_api_linkcss_unset_beats_the_secure_default() {
        let html = convert_with("= Doc\n\nBody.", &Options::new().unset("linkcss"));
        assert!(html.contains("<style>"));
        assert!(!html.contains("./asciidoctor.css"));
    }
}
