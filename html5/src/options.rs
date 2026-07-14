//! Per-conversion options: externally-supplied document attributes and the
//! [safe mode].
//!
//! [`Options`] carries the settings applied to a conversion from *outside* the
//! document source: a set of document attributes (the equivalent of
//! Asciidoctor's `-a name=value` CLI option and the `attributes` API option)
//! and the [`SafeMode`] under which the document is processed (Asciidoctor's
//! `safe` API option). It is the parameter accepted by the `_with` conversion
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
//! // Override: the API value wins over the document header. (`webfonts` is a
//! // standalone-document `<head>` feature, so render standalone here.)
//! let opts = Options::new()
//!     .standalone(true)
//!     .attribute("webfonts", "Ubuntu+Mono:400");
//! let html = convert_with("= Doc\n:webfonts: ignored\n\nBody.", &opts);
//! assert!(html.contains("family=Ubuntu+Mono:400"));
//! ```
//!
//! [`convert_with`]: crate::convert_with
//! [`convert_file_with`]: crate::convert_file_with

use std::path::{Path, PathBuf};

use asciidoc_parser::{parser::ModificationContext, Parser, SafeMode};

use crate::{docinfo_handler::FsDocinfoFileHandler, include_handler::FsIncludeFileHandler};

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
/// let opts = Options::new()
///     .standalone(true)
///     .set("linkcss")
///     .unset("webfonts");
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

    /// The CSS to embed when the document selects a custom stylesheet and the
    /// stylesheet is embedded rather than linked. `None` leaves the library
    /// with nothing to embed for a custom stylesheet. See
    /// [`Options::stylesheet_content`].
    stylesheet_content: Option<String>,

    /// The base directory: the anchor for filesystem-relative resources
    /// (`include::` targets and docinfo files) and, under a jailed safe mode,
    /// the boundary reads may not cross. `None` leaves it to be derived from
    /// the primary file's directory (see [`base_dir`](Self::base_dir)).
    base_dir: Option<PathBuf>,

    /// The path of the primary document, used to name it for diagnostics, to
    /// anchor top-level `include::` resolution, and to derive the `docname`
    /// that names *private* docinfo files. Set by
    /// [`convert_file_with`](crate::convert_file_with) and by
    /// [`input_file`](Self::input_file).
    primary_file: Option<PathBuf>,

    /// Whether to render a standalone document (the full `<!DOCTYPE>`/`<head>`/
    /// `<body>` shell) or embedded, body-only output. `None` defers to the
    /// entry point's default — embedded for the string entry points,
    /// standalone for the file entry points — matching Asciidoctor's
    /// `:standalone` option. See [`standalone`](Self::standalone) /
    /// [`embedded`](Self::embedded).
    standalone: Option<bool>,
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
    /// This is Asciidoctor's `safe` API option. When left unset, conversion
    /// uses [`SafeMode::Secure`], the most conservative mode and
    /// Asciidoctor's API default. Following Asciidoctor, `Secure` links the
    /// default stylesheet (to `./asciidoctor.css`) instead of embedding it,
    /// unless the caller sets `linkcss` explicitly; lower modes embed it
    /// inline.
    ///
    /// # Examples
    ///
    /// ```
    /// use asciidoc_html5::{convert_with, Options, SafeMode};
    ///
    /// // A mode below `Secure` embeds the default stylesheet inline.
    /// let opts = Options::new().standalone(true).safe_mode(SafeMode::Server);
    /// let html = convert_with("= Doc\n\nBody.", &opts);
    /// assert!(html.contains("<style>"));
    /// ```
    pub fn safe_mode(mut self, safe: SafeMode) -> Self {
        self.safe_mode = Some(safe);
        self
    }

    /// Selects standalone output (`true`) or embedded, body-only output
    /// (`false`).
    ///
    /// This is Asciidoctor's `:standalone` option. A *standalone* document is
    /// the complete HTML5 file — the `<!DOCTYPE html>` declaration, `<html>`, a
    /// `<head>` (with the default stylesheet), and a `<body>` framing the
    /// header, content, and footer. *Embedded* output is the converted body
    /// on its own, with no shell, stylesheet, or header/footer frame —
    /// meant to be dropped into a surrounding template.
    ///
    /// When left unset, the output mode follows the entry point, matching
    /// Asciidoctor: the string entry points ([`convert`](crate::convert),
    /// [`convert_with`](crate::convert_with)) default to *embedded*, while the
    /// file entry points ([`convert_file`](crate::convert_file),
    /// [`convert_file_with`](crate::convert_file_with)) default to
    /// *standalone*. Setting this explicitly overrides that default for
    /// either kind of entry point.
    ///
    /// # Examples
    ///
    /// ```
    /// use asciidoc_html5::{convert_with, Options};
    ///
    /// // The string API is embedded by default; opt in to a full document.
    /// let opts = Options::new().standalone(true);
    /// let html = convert_with("= Doc\n\nBody.", &opts);
    /// assert!(html.starts_with("<!DOCTYPE html>"));
    /// ```
    pub fn standalone(mut self, yes: bool) -> Self {
        self.standalone = Some(yes);
        self
    }

    /// Selects embedded, body-only output (`true`) or standalone output
    /// (`false`) — the inverse of [`standalone`](Self::standalone).
    ///
    /// This is the spelling Asciidoctor's `-e`/`--embedded` CLI flag reaches
    /// for. `options.embedded(true)` is equivalent to
    /// `options.standalone(false)`. See [`standalone`](Self::standalone)
    /// for what each mode emits and how the unset default follows the entry
    /// point.
    ///
    /// # Examples
    ///
    /// ```
    /// use asciidoc_html5::{convert_file_with, Options};
    ///
    /// // The file API is standalone by default; opt in to body-only output.
    /// let opts = Options::new().embedded(true);
    /// # let _ = &opts;
    /// ```
    pub fn embedded(self, yes: bool) -> Self {
        self.standalone(!yes)
    }

    /// Supplies the CSS to embed when the document selects a *custom*
    /// stylesheet — that is, when the `stylesheet` attribute is set to a
    /// non-empty value other than `DEFAULT`.
    ///
    /// The library converts text to text and cannot read an external stylesheet
    /// file on its own, so a caller that wants a custom stylesheet *embedded*
    /// (`<style>…</style>`) must hand its contents in through this method. This
    /// is the string counterpart to Asciidoctor reading the file named by
    /// `stylesheet`/`stylesdir` from disk.
    ///
    /// The content is used only when the stylesheet is embedded. Under
    /// `linkcss` (including the `Secure` default, which links) the converter
    /// links to the stylesheet's normalized web path instead and ignores this
    /// value. It is likewise ignored when the document uses the default
    /// stylesheet or unsets the stylesheet entirely.
    ///
    /// # Examples
    ///
    /// ```
    /// use asciidoc_html5::{convert_with, Options, SafeMode};
    ///
    /// let opts = Options::new()
    ///     .standalone(true)
    ///     .safe_mode(SafeMode::Unsafe)
    ///     .attribute("stylesheet", "my-theme.css")
    ///     .stylesheet_content("body { color: #ff0000; }");
    /// let html = convert_with("= Doc\n\nBody.", &opts);
    /// assert!(html.contains("<style>\nbody { color: #ff0000; }\n</style>"));
    /// ```
    pub fn stylesheet_content<S: Into<String>>(mut self, css: S) -> Self {
        self.stylesheet_content = Some(css.into());
        self
    }

    /// Sets the base directory that filesystem-relative resources resolve
    /// against — Asciidoctor's `-B`/`--base-dir` (the `base_dir` API option).
    ///
    /// Such resources are `include::` directives and docinfo files. The base
    /// directory anchors relative include targets and docinfo file lookups and,
    /// under the `safe` and `server` [safe modes](SafeMode), is the boundary
    /// those reads may not cross: a target (or a `docinfodir`) that tries to
    /// climb above it is recovered back inside, matching Asciidoctor. Under
    /// `unsafe` there is no such restriction, and under `secure` includes are
    /// turned into links and docinfo is dropped, without any file being read.
    ///
    /// When left unset, the base directory is derived from the primary file's
    /// directory (see [`input_file`](Self::input_file)); with neither a base
    /// directory nor a primary file, include and docinfo resolution are not
    /// enabled and `include::` directives and docinfo are left unresolved.
    ///
    /// The path should be absolute; relative paths are interpreted against the
    /// process's current directory when files are read.
    pub fn base_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    /// Sets the path of the primary document.
    ///
    /// This names the document in diagnostics, anchors the resolution of its
    /// top-level `include::` directives (a relative include target resolves
    /// against this file's directory), and provides the `docname` from which
    /// *private* docinfo file names are built (`<docname>-docinfo.html`, …).
    /// When [`base_dir`](Self::base_dir) is unset, this file's directory also
    /// becomes the base directory.
    ///
    /// [`convert_file_with`](crate::convert_file_with) sets this automatically
    /// from the path it reads; callers that convert already-read source with
    /// [`convert_with`](crate::convert_with) can set it explicitly so includes
    /// and docinfo resolve as they would for the file on disk.
    pub fn input_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.primary_file = Some(path.into());
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

        // `copycss` needs no seeding here: the parser sets it on by default in
        // every safe mode (document-overridable), and the copy is gated on the
        // safe mode being below `Secure` where it is resolved (see the
        // [`copycss`](crate::copycss) module). `copycss` only governs whether a
        // *linked* stylesheet is also copied next to the output; it never
        // affects the HTML.

        // Matching Asciidoctor: `Server` and above forbid the *document* from
        // controlling docinfo — only the API may (Asciidoctor's SERVER "prevents
        // the document from setting … docinfo"). Re-seed docinfo *silently*
        // locked (`ApiOnly`) at whatever value the API directives resolved to,
        // or unset when the API did not touch it, so any document `:docinfo:` is
        // dropped with no warning and a docinfo file is read only when the API
        // asked for it. This runs after the directive loop, so it wins — and,
        // unlike a plain `mentions` check, it also covers a *soft* default: a
        // soft API value seeds `docinfo` as document-overridable, which would
        // otherwise let a document enable docinfo reads under `Server`. (Under
        // `Secure` the parser drops docinfo resolution outright, so the value
        // locked here is moot there.)
        if mode >= SafeMode::Server {
            let ctx = ModificationContext::ApiOnly;
            parser = match self.last_action("docinfo") {
                Some(Action::Value(value)) => {
                    parser.with_intrinsic_attribute_silent("docinfo", value, ctx)
                }
                Some(Action::Set) => {
                    parser.with_intrinsic_attribute_bool_silent("docinfo", true, ctx)
                }
                // An explicit API unset, or no API mention at all: docinfo off,
                // locked against the document.
                Some(Action::Unset) | None => {
                    parser.with_intrinsic_attribute_bool_silent("docinfo", false, ctx)
                }
            };
        }

        // Surface the input-file attribute family — `docfile`, `docdir`,
        // `docname`, `docfilesuffix` — the way Asciidoctor's loader does,
        // honoring the safe mode. Asciidoctor derives all four from the input
        // path; `asciidoc-parser` does not originate them, so this crate seeds
        // them. `docfile`/`docdir` reveal the host location, so the safe mode
        // sanitizes them: below `Server` they carry the input file's absolute
        // path and directory (Asciidoctor's `File.absolute_path`/`File.dirname`);
        // `Server` and above conceal the host — `docfile` is trimmed to its
        // basename and `docdir` is emptied. `Secure`, being higher than
        // `Server`, inherits the same sanitization (both modes leave `docdir`
        // empty and `docfile` a bare basename, matching Asciidoctor and the
        // AsciiDoc attributes reference). `docname` (the file stem) and
        // `docfilesuffix` (the file extension) expose no more than the concealed
        // `docfile` already does, so they carry no safe-mode nuance and are set
        // the same in every mode. All are locked (`ApiOnly`) so the document
        // cannot reassign them, and seeded silently so re-seeding over a
        // caller-supplied value or the directive loop raises no lock warning.
        let conceal = mode >= SafeMode::Server;

        // The source document's path: a caller-supplied `docfile` value or,
        // failing that, the primary file (made absolute). Absent both — a plain
        // source string with no file — nothing names the document, so none of
        // the four file intrinsics are seeded, matching Asciidoctor (a string
        // carries no `docfile`/`docname`).
        let docfile_source = self.last_value("docfile").map(str::to_owned).or_else(|| {
            self.primary_file
                .as_deref()
                .map(|path| canonicalize_or(path).to_string_lossy().into_owned())
        });
        if let Some(source) = &docfile_source {
            // `docfile` names the source document; `Server` and above trim it to
            // its basename to conceal the host location.
            let docfile = if conceal {
                file_basename(source)
            } else {
                source.clone()
            };
            parser = parser.with_intrinsic_attribute_silent(
                "docfile",
                docfile,
                ModificationContext::ApiOnly,
            );

            // `docfilesuffix` is the file extension (leading dot included, empty
            // when the name has none) and `docname` the basename with that
            // suffix removed — Asciidoctor's `Helpers.extname`/`Helpers.basename`.
            // A caller-supplied value wins over the derived one.
            let docfilesuffix = self
                .last_value("docfilesuffix")
                .map(str::to_owned)
                .unwrap_or_else(|| file_extension(source));
            let docname = self
                .last_value("docname")
                .map(str::to_owned)
                .unwrap_or_else(|| document_name(source, &docfilesuffix));
            parser = parser.with_intrinsic_attribute_silent(
                "docfilesuffix",
                docfilesuffix,
                ModificationContext::ApiOnly,
            );
            parser = parser.with_intrinsic_attribute_silent(
                "docname",
                docname,
                ModificationContext::ApiOnly,
            );
        }

        // `docdir` is the source document's directory. Under concealment it is
        // emptied; otherwise it is a caller-supplied `docdir`, the base
        // directory, or — failing both — the current directory (Asciidoctor's
        // `Dir.pwd` fallback for string input). It is always set, so `{docdir}`
        // resolves to the empty string under `Server`/`Secure` rather than
        // being left unresolved.
        let docdir = if conceal {
            String::new()
        } else {
            self.last_value("docdir")
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    self.effective_base_dir()
                        .or_else(|| std::env::current_dir().ok())
                        .map(|dir| dir.to_string_lossy().into_owned())
                        .unwrap_or_default()
                })
        };
        parser =
            parser.with_intrinsic_attribute_silent("docdir", docdir, ModificationContext::ApiOnly);

        // html5 is the only backend this crate produces, so `backend` is pinned
        // to `html5` in *every* safe mode and locked against the document.
        // Seeding it as a *silent* `ApiOnly` intrinsic drops any document
        // `:backend:` with no warning; running after the directive loop makes it
        // win over an API `backend` directive or a *soft* API default too. This
        // goes further than Asciidoctor — whose SERVER "disallows the document
        // from setting attributes that would affect conversion" (backend among
        // them) and whose SECURE "sets the backend to html5," while lower modes
        // honor a document `:backend:` — precisely because a non-html5 backend
        // is out of scope here: the `{backend}` intrinsic always reflects what
        // is actually rendered.
        parser = parser.with_intrinsic_attribute_silent(
            "backend",
            "html5",
            ModificationContext::ApiOnly,
        );

        // `article` is the only doctype this renderer models, so `doctype` is
        // pinned to `article` in *every* safe mode and locked against the
        // document, mirroring the `backend` pin above. Seeding it as a *silent*
        // `ApiOnly` intrinsic drops any document `:doctype:` (e.g. `book`,
        // `manpage`) with no warning; running after the directive loop makes it
        // win over an API `doctype` directive or a *soft* API default too. This
        // goes further than Asciidoctor — whose SERVER "disallows the document
        // from setting attributes that would affect conversion" (doctype among
        // them) while lower modes honor a document `:doctype:` — precisely
        // because non-`article` doctypes are out of scope here: the `{doctype}`
        // intrinsic (and the `<body class>` it drives) always reflects what is
        // actually rendered.
        parser = parser.with_intrinsic_attribute_silent(
            "doctype",
            "article",
            ModificationContext::ApiOnly,
        );

        // Anchor filesystem-relative resources: `include::` targets and docinfo
        // files. Naming the primary file lets the parser resolve top-level
        // includes against that file's directory and derive the `docname` for
        // private docinfo; supplying a base directory (given directly or derived
        // from the primary file) installs the filesystem include and docinfo
        // handlers, each confined by the safe mode. Under `secure` the parser
        // converts includes to links and drops docinfo without consulting either
        // handler, so installing them there is harmless.
        if let Some(primary) = &self.primary_file {
            parser = parser.with_primary_file_name(canonicalize_or(primary).to_string_lossy());
        }
        if let Some(base) = self.effective_base_dir() {
            parser = parser
                .with_include_file_handler(FsIncludeFileHandler::new(base.clone(), mode))
                .with_docinfo_file_handler(FsDocinfoFileHandler::new(base, mode));
        }

        parser
    }

    /// The base directory that anchors include and docinfo resolution, or
    /// `None` when there is nothing to anchor (neither a base directory nor
    /// a primary file).
    ///
    /// An explicit [`base_dir`](Self::base_dir) wins; otherwise the primary
    /// file's directory is used (an empty directory component — a bare file
    /// name — means the current directory). The result is canonicalized when it
    /// exists on disk, so the handler's jail comparisons and the primary file's
    /// name share one absolute form.
    pub(crate) fn effective_base_dir(&self) -> Option<PathBuf> {
        if let Some(base) = &self.base_dir {
            return Some(canonicalize_or(base));
        }

        let primary = self.primary_file.as_deref()?;
        let dir = primary
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        Some(canonicalize_or(dir))
    }

    /// Whether any recorded directive names `name` (already lowercased). Used
    /// to decide whether the caller has taken control of an attribute that
    /// the safe mode would otherwise default.
    fn mentions(&self, name: &str) -> bool {
        self.attributes.iter().any(|d| d.name == name)
    }

    /// The CSS to embed for a custom stylesheet, if the caller supplied any.
    pub(crate) fn custom_stylesheet(&self) -> Option<&str> {
        self.stylesheet_content.as_deref()
    }

    /// The safe mode conversion runs under, defaulting to [`SafeMode::Secure`]
    /// (Asciidoctor's API default) when the caller left it unset.
    pub(crate) fn safe_mode_or_default(&self) -> SafeMode {
        self.safe_mode.unwrap_or(SafeMode::Secure)
    }

    /// Whether to render a standalone document, resolving the unset default to
    /// *embedded* (`false`) — the string entry points' default. The file entry
    /// points pre-fill `Some(true)` with
    /// [`default_standalone`](Self::default_standalone) before conversion,
    /// so this returns `true` for them unless the caller opted into
    /// embedded output.
    pub(crate) fn is_standalone(&self) -> bool {
        self.standalone.unwrap_or(false)
    }

    /// Fills in *standalone* as the output mode when the caller has not chosen
    /// one — the file entry points' default. A caller who set the mode
    /// explicitly (including [`embedded(true)`](Self::embedded)) keeps their
    /// choice, so `convert_file` stays standalone by default while still
    /// honoring an explicit request for embedded output.
    pub(crate) fn default_standalone(mut self) -> Self {
        self.standalone.get_or_insert(true);
        self
    }

    /// The [`Action`] of the last directive naming `name` (already lowercased),
    /// or `None` when no directive names it. This is the value [`apply`] leaves
    /// in force, since it replays the directives in order and a later one for
    /// the same name wins — regardless of whether it was an override or a soft
    /// default, both of which set the same value (they differ only in the
    /// modification context).
    ///
    /// [`apply`]: Self::apply
    fn last_action(&self, name: &str) -> Option<&Action> {
        self.attributes
            .iter()
            .rev()
            .find(|directive| directive.name == name)
            .map(|directive| &directive.action)
    }

    /// The explicit string value the last directive naming `name` assigns, when
    /// that directive is a value assignment ([`Action::Value`]). A `Set`,
    /// `Unset`, or no directive at all yields `None`. Used to let a
    /// caller-supplied `docfile`/`docdir` seed the intrinsic this crate
    /// otherwise derives from the file paths.
    fn last_value(&self, name: &str) -> Option<&str> {
        match self.last_action(name)? {
            Action::Value(value) => Some(value),
            Action::Set | Action::Unset => None,
        }
    }
}

/// The final path component of `path` — the file's basename — used to conceal a
/// `docfile`'s host location under the `Server` and `Secure` safe modes. Falls
/// back to the whole string when the path has no final component.
fn file_basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned())
}

/// The `docfilesuffix` value for `path`: the file extension including the
/// leading dot, or an empty string when the final path component has no
/// extension. This mirrors Asciidoctor's `Helpers.extname` — the substring from
/// the last `.` to the end, unless that dot lies in a directory component (a
/// path separator follows it) or there is no dot at all.
fn file_extension(path: &str) -> String {
    match path.rfind('.') {
        Some(idx) if !path[idx..].contains(['/', '\\']) => path[idx..].to_string(),
        _ => String::new(),
    }
}

/// The `docname` value for `path`: its basename with the trailing `suffix`
/// (the `docfilesuffix`) removed, mirroring Asciidoctor's `Helpers.basename`.
/// An empty `suffix` (an extensionless name) removes nothing, and a basename
/// that is *entirely* the suffix — a leading-dot name such as `.adoc` — is kept
/// whole rather than reduced to an empty stem.
fn document_name(path: &str, suffix: &str) -> String {
    let base = file_basename(path);
    if !suffix.is_empty() {
        if let Some(stem) = base.strip_suffix(suffix) {
            if !stem.is_empty() {
                return stem.to_owned();
            }
        }
    }
    base
}

/// Canonicalizes `path` to its absolute form, falling back to the path as given
/// when it cannot be canonicalized (for example, when it does not exist on
/// disk). A canonical base directory keeps the include and docinfo handlers'
/// jail comparisons on the same footing as the paths the parser reports.
fn canonicalize_or(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use crate::{Options, SafeMode};

    // These option tests assert the standalone document shell (its stylesheet
    // and web-font links, the header, and the footer), so they render in
    // standalone mode explicitly. The string entry points now default to
    // embedded output, so `convert`/`convert_with` are shadowed here to force
    // `standalone(true)`, keeping these tests focused on attribute and
    // safe-mode behavior.

    /// Converts `source` to a standalone document under the default safe mode —
    /// the standalone counterpart of [`crate::convert`].
    fn convert(source: &str) -> String {
        crate::convert_with(source, &Options::new().standalone(true))
    }

    /// Converts `source` to a standalone document under `options` — the
    /// standalone counterpart of [`crate::convert_with`].
    fn convert_with(source: &str, options: &Options) -> String {
        crate::convert_with(source, &options.clone().standalone(true))
    }

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

    // Docinfo is read from the base directory only when docinfo is enabled and
    // the safe mode permits it: below `Server` a document `:docinfo:` enables it;
    // `Server` and above require an API-set value (a document `:docinfo:` is
    // ignored); `Secure` drops docinfo entirely — matching Asciidoctor. These
    // exercise the wiring in `apply`; the handler's own resolution and jail are
    // covered in `docinfo_handler`.

    /// Creates a fresh temp directory named after `tag`, populated with `files`
    /// (name → content), for a docinfo test to point a base directory at.
    fn docinfo_scratch(tag: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("adoc-opts-docinfo-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        for (name, content) in files {
            std::fs::write(dir.join(name), content).expect("write scratch file");
        }
        dir
    }

    #[test]
    fn docinfo_is_read_from_the_base_directory_below_secure() {
        let dir = docinfo_scratch("below-secure", &[("docinfo.html", "<meta name=\"x\">")]);

        // `Safe` permits a document-set `:docinfo:` (only `Server` and above
        // forbid it) and installs the handler, so the file is read.
        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(dir.clone()),
        );

        assert!(html.contains("<meta name=\"x\">\n</head>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn document_set_docinfo_is_ignored_under_server() {
        // Under `Server`, a document that turns docinfo on itself is ignored —
        // Asciidoctor's SERVER "prevents the document from setting … docinfo".
        // The file exists in the base directory but must not be read.
        let dir = docinfo_scratch("server-doc", &[("docinfo.html", "<meta name=\"x\">")]);

        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .base_dir(dir.clone()),
        );

        assert!(!html.contains("<meta name=\"x\">"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn api_set_docinfo_still_applies_under_server() {
        // The restriction is on the *document*, not the API: an API-set
        // `docinfo` is honored under `Server`, and the document need not (and
        // here does not) mention it.
        let dir = docinfo_scratch("server-api", &[("docinfo.html", "<meta name=\"x\">")]);

        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .attribute("docinfo", "shared")
                .base_dir(dir.clone()),
        );

        assert!(html.contains("<meta name=\"x\">\n</head>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn api_bare_set_docinfo_applies_under_server() {
        // A *bare* API `docinfo` (a set with no value, `Action::Set`) enables
        // docinfo under `Server` just like a valued one. A boolean `docinfo`
        // resolves to *private*, so it reads `<docname>-docinfo.html` given a
        // primary file. This exercises the `Some(Action::Set)` arm, distinct
        // from the valued `attribute("docinfo", …)` path above.
        let dir = docinfo_scratch(
            "server-api-bare",
            &[("guide-docinfo.html", "<meta name=\"x\">")],
        );

        let html = convert_with(
            "= Doc\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .set("docinfo")
                .base_dir(dir.clone())
                .input_file(dir.join("guide.adoc")),
        );

        assert!(html.contains("<meta name=\"x\">\n</head>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn soft_default_docinfo_does_not_let_the_document_enable_it_under_server() {
        // A *soft* API default leaves an attribute document-overridable, so a
        // `mentions`-based guard would skip the safe-mode lock. Under `Server`
        // docinfo must stay API-controlled regardless: a document `:docinfo:`
        // is still ignored even when the API only soft-touched docinfo (here a
        // soft unset, which by itself does not enable docinfo).
        let dir = docinfo_scratch("server-soft", &[("docinfo.html", "<meta name=\"x\">")]);

        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .unset_default("docinfo")
                .base_dir(dir.clone()),
        );

        assert!(!html.contains("<meta name=\"x\">"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn soft_default_docinfo_value_is_honored_but_locked_under_server() {
        // A soft-default docinfo *value* still enables docinfo under `Server`
        // (the API asked for it), but the document cannot turn it off: the
        // document's `:docinfo!:` is ignored and the file is still read.
        let dir = docinfo_scratch("server-soft-val", &[("docinfo.html", "<meta name=\"x\">")]);

        let html = convert_with(
            "= Doc\n:docinfo!:\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .attribute_default("docinfo", "shared")
                .base_dir(dir.clone()),
        );

        assert!(html.contains("<meta name=\"x\">\n</head>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn docinfo_is_disabled_under_the_secure_default() {
        // Secure is the default; docinfo is dropped without any file being read,
        // even with a base directory and the `docinfo` attribute set.
        let dir = docinfo_scratch("secure", &[("docinfo.html", "<meta name=\"x\">")]);

        let html = convert_with(
            "= Doc\n:docinfo: shared\n\nBody.",
            &Options::new().base_dir(dir.clone()),
        );

        assert!(!html.contains("<meta name=\"x\">"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn private_docinfo_requires_a_primary_file() {
        let dir = docinfo_scratch(
            "private",
            &[("guide-docinfo.html", "<meta name=\"private\">")],
        );

        // `Safe` keeps the document's `:docinfo: private` in force (unlike
        // `Server`); with only a base directory the `<docname>` is unknown, so
        // the private file is not resolved.
        let without = convert_with(
            "= Doc\n:docinfo: private\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(dir.clone()),
        );

        assert!(!without.contains("name=\"private\""));

        // Naming the primary file `guide.adoc` supplies the docname, so
        // `guide-docinfo.html` is found and injected.
        let with = convert_with(
            "= Doc\n:docinfo: private\n\nBody.",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(dir.clone())
                .input_file(dir.join("guide.adoc")),
        );

        assert!(with.contains("name=\"private\""));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // `docfile` and `docdir` are intrinsic attributes this crate originates
    // (the parser does not), derived from the primary file and base directory
    // and sanitized by the safe mode: below `Server` they carry the absolute
    // path and directory; `Server` and above trim `docfile` to its basename and
    // empty `docdir`, matching Asciidoctor.

    /// Creates a temp directory named after `tag` containing an empty
    /// `main.adoc`, returning the directory and the file both in the canonical
    /// form `apply` records them in.
    fn docpath_scratch(tag: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("adoc-opts-docpath-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        let file = dir.join("main.adoc");
        std::fs::write(&file, "").expect("write scratch file");
        (super::canonicalize_or(&dir), super::canonicalize_or(&file))
    }

    #[test]
    fn docfile_and_docdir_are_absolute_below_server() {
        let (dir, file) = docpath_scratch("below-server");

        let html = convert_with(
            "= Doc\n\nfile={docfile} dir={docdir}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file(file.clone()),
        );

        assert!(
            html.contains(&format!("file={} dir={}", file.display(), dir.display())),
            "{html}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn server_trims_docfile_and_empties_docdir() {
        let (dir, file) = docpath_scratch("server");

        let html = convert_with(
            "= Doc\n\nfile={docfile} dir={docdir}",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .input_file(file.clone()),
        );

        // `docfile` is trimmed to the basename and `docdir` is empty, so the
        // host directory never appears.
        assert!(html.contains("<p>file=main.adoc dir=</p>"), "{html}");
        assert!(!html.contains(&dir.display().to_string()), "{html}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn secure_conceals_docfile_and_docdir_like_server() {
        let (dir, file) = docpath_scratch("secure");

        // `Secure` is the API default; it inherits `Server`'s sanitization.
        let html = convert_with(
            "= Doc\n\nfile={docfile} dir={docdir}",
            &Options::new().input_file(file.clone()),
        );

        assert!(html.contains("<p>file=main.adoc dir=</p>"), "{html}");
        assert!(!html.contains(&dir.display().to_string()), "{html}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_document_cannot_reassign_docdir_or_docfile() {
        let (dir, file) = docpath_scratch("locked");

        // The intrinsics are locked (`ApiOnly`), so a header assignment is
        // dropped and the derived values stand — with no lock warning, since
        // they are seeded silently.
        let html = convert_with(
            "= Doc\n:docdir: HACKED\n:docfile: HACKED\n\nfile={docfile} dir={docdir}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file(file.clone()),
        );

        assert!(!html.contains("HACKED"), "{html}");
        assert!(
            html.contains(&format!("file={} dir={}", file.display(), dir.display())),
            "{html}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_caller_supplied_docdir_seeds_the_value_below_server() {
        // With no primary file, a caller-supplied `docdir` stands in for the
        // derived directory — used as given (not expanded), matching
        // Asciidoctor's "docdir specified via API is not expanded".
        let html = convert_with(
            "= Doc\n\ndir={docdir}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute("docdir", "virtual/directory"),
        );

        assert!(html.contains("<p>dir=virtual/directory</p>"), "{html}");
    }

    #[test]
    fn string_input_has_docdir_but_no_docfile() {
        // A plain source string names no file, so `docfile` stays unset (its
        // reference is left unresolved), while `docdir` falls back to the
        // current directory below `Server`.
        let cwd = std::env::current_dir().expect("cwd");
        let html = convert_with(
            "= Doc\n\nfile=[{docfile}] dir={docdir}",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );

        assert!(html.contains("file=[{docfile}]"), "{html}");
        assert!(html.contains(&format!("dir={}", cwd.display())), "{html}");
    }

    // `docname` (the file stem) and `docfilesuffix` (the file extension) round
    // out the input-file attribute family. Unlike `docfile`/`docdir` they carry
    // no safe-mode nuance — the stem and extension expose nothing the concealed
    // `docfile` basename does not — so they are set the same in every mode.

    // The path-splitting helpers behind `docname`/`docfilesuffix` are exercised
    // directly for the Asciidoctor `Helpers.extname`/`Helpers.basename` edges
    // that the file-driven tests above do not reach.
    #[test]
    fn file_basename_falls_back_for_a_nameless_path() {
        // A normal path yields its final component; a path with no final
        // component (a bare root) falls back to the whole string.
        assert_eq!(super::file_basename("/docs/guide.adoc"), "guide.adoc");
        assert_eq!(super::file_basename("/"), "/");
    }

    #[test]
    fn file_extension_matches_asciidoctor_extname() {
        // A normal extension, and only the *final* one.
        assert_eq!(super::file_extension("/docs/guide.adoc"), ".adoc");
        assert_eq!(super::file_extension("/tmp/archive.tar.gz"), ".gz");

        // No dot at all → no extension.
        assert_eq!(super::file_extension("/tmp/README"), "");

        // A dot in a *directory* component is not an extension (a path
        // separator follows the last dot).
        assert_eq!(super::file_extension("/etc/rc.d/README"), "");
    }

    #[test]
    fn document_name_matches_asciidoctor_basename() {
        // The suffix is stripped when a non-empty stem remains.
        assert_eq!(super::document_name("/docs/guide.adoc", ".adoc"), "guide");

        // Only the given suffix is removed (multi-extension).
        assert_eq!(
            super::document_name("/tmp/archive.tar.gz", ".gz"),
            "archive.tar"
        );

        // An empty suffix (an extensionless name) removes nothing.
        assert_eq!(super::document_name("/docs/README", ""), "README");

        // A basename that is *entirely* the suffix — a leading-dot name such as
        // `.adoc` — is kept whole rather than reduced to an empty stem.
        assert_eq!(super::document_name("/docs/.adoc", ".adoc"), ".adoc");

        // A suffix that is not actually a suffix of the basename leaves it
        // whole (in practice the suffix is always the path's own extension).
        assert_eq!(
            super::document_name("/docs/guide.adoc", ".xyz"),
            "guide.adoc"
        );
    }

    #[test]
    fn a_bare_set_docfile_or_docdir_is_not_treated_as_a_value() {
        // A bare API `set` (no value) is not a value directive, so `docdir`
        // still derives from the file's directory rather than being treated as
        // caller-supplied. This exercises `last_value`'s non-value arm.
        let (dir, file) = docpath_scratch("bare-set");

        let html = convert_with(
            "= Doc\n\ndir={docdir}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .set("docdir")
                .input_file(file.clone()),
        );

        assert!(html.contains(&format!("dir={}", dir.display())), "{html}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn docname_and_docfilesuffix_are_derived_from_the_file() {
        for mode in [
            SafeMode::Unsafe,
            SafeMode::Safe,
            SafeMode::Server,
            SafeMode::Secure,
        ] {
            let html = convert_with(
                "= Doc\n\nname={docname} suffix={docfilesuffix}",
                &Options::new()
                    .safe_mode(mode)
                    .input_file("/docs/guide/userguide.adoc"),
            );
            assert!(
                html.contains("<p>name=userguide suffix=.adoc</p>"),
                "{mode:?}: {html}"
            );
        }
    }

    #[test]
    fn docfilesuffix_preserves_an_alternate_extension() {
        let html = convert_with(
            "= Doc\n\nname={docname} suffix={docfilesuffix}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file("/docs/notes.asciidoc"),
        );
        assert!(
            html.contains("<p>name=notes suffix=.asciidoc</p>"),
            "{html}"
        );
    }

    #[test]
    fn an_extensionless_name_has_an_empty_docfilesuffix() {
        // With no extension, `docfilesuffix` is empty and `docname` is the whole
        // basename (Asciidoctor's `Helpers.extname` fallback).
        let html = convert_with(
            "= Doc\n\nname={docname} suffix=[{docfilesuffix}]",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file("/docs/README"),
        );
        assert!(html.contains("<p>name=README suffix=[]</p>"), "{html}");
    }

    #[test]
    fn a_document_cannot_reassign_docname_or_docfilesuffix() {
        let html = convert_with(
            "= Doc\n:docname: HACKED\n:docfilesuffix: .HACKED\n\nname={docname} suffix={docfilesuffix}",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .input_file("/docs/guide/userguide.adoc"),
        );
        assert!(!html.contains("HACKED"), "{html}");
        assert!(
            html.contains("<p>name=userguide suffix=.adoc</p>"),
            "{html}"
        );
    }

    #[test]
    fn string_input_has_no_docname_or_docfilesuffix() {
        // Without a file, none of the file-derived intrinsics beyond `docdir`
        // are set, so their references are left unresolved.
        let html = convert_with(
            "= Doc\n\nname=[{docname}] suffix=[{docfilesuffix}]",
            &Options::new().safe_mode(SafeMode::Unsafe),
        );
        assert!(
            html.contains("name=[{docname}] suffix=[{docfilesuffix}]"),
            "{html}"
        );
    }

    // The backend the document sees is reachable through the `{backend}`
    // intrinsic reference. html5 is the only backend this crate produces, so
    // `apply` pins `backend` to `html5` in every safe mode and locks it against
    // the document and the API alike — going further than Asciidoctor, which
    // only restricts the document at `Server`/`Secure`.

    // A helper document that echoes the resolved `backend` intrinsic into the
    // body, where it lands in the rendered output.
    const BACKEND_ECHO: &str = "= Doc\n:backend: docbook\n\nbackend={backend}";

    #[test]
    fn document_set_backend_is_pinned_to_html5_in_every_mode() {
        // A document `:backend:` is dropped and `{backend}` resolves to html5
        // regardless of the safe mode — including the lower modes where
        // Asciidoctor would honor it.
        for mode in [
            SafeMode::Unsafe,
            SafeMode::Safe,
            SafeMode::Server,
            SafeMode::Secure,
        ] {
            let html = convert_with(BACKEND_ECHO, &Options::new().safe_mode(mode));
            assert!(html.contains("backend=html5"), "{mode:?} should pin html5");
            assert!(
                !html.contains("backend=docbook"),
                "{mode:?} dropped docbook"
            );
        }
    }

    #[test]
    fn api_set_backend_is_also_pinned_to_html5() {
        // The pin overrides the API too: an API `backend` directive does not
        // survive in any mode (unlike, e.g., `docinfo`, which the API may set).
        for mode in [SafeMode::Unsafe, SafeMode::Server, SafeMode::Secure] {
            let html = convert_with(
                BACKEND_ECHO,
                &Options::new()
                    .safe_mode(mode)
                    .attribute("backend", "docbook5"),
            );
            assert!(html.contains("backend=html5"), "{mode:?} should pin html5");
            assert!(!html.contains("docbook"), "{mode:?} dropped the API value");
        }
    }

    #[test]
    fn soft_default_backend_does_not_survive() {
        // A *soft* API default leaves an attribute document-overridable, so a
        // `mentions`-based guard would skip the pin. The backend stays html5
        // regardless: neither the document's `:backend:` nor the API's soft
        // default takes effect.
        let html = convert_with(
            BACKEND_ECHO,
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute_default("backend", "docbook5"),
        );
        assert!(html.contains("backend=html5"));
        assert!(!html.contains("docbook"));
    }

    // The doctype the document sees is reachable through the `{doctype}`
    // intrinsic reference (and drives the `<body class>`). `article` is the only
    // doctype this renderer models, so `apply` pins `doctype` to `article` in
    // every safe mode and locks it against the document and the API alike —
    // going further than Asciidoctor, which only restricts the document at
    // `Server`/`Secure`.

    // A helper document that sets a non-`article` doctype and echoes the
    // resolved `doctype` intrinsic into the body, where it lands in the rendered
    // output alongside the `<body class>`.
    const DOCTYPE_ECHO: &str = "= Doc\n:doctype: book\n\ndoctype={doctype}";

    #[test]
    fn document_set_doctype_is_pinned_to_article_in_every_mode() {
        // A document `:doctype:` is dropped and `{doctype}` resolves to article
        // regardless of the safe mode — including the lower modes where
        // Asciidoctor would honor it. The `<body class>` follows suit.
        for mode in [
            SafeMode::Unsafe,
            SafeMode::Safe,
            SafeMode::Server,
            SafeMode::Secure,
        ] {
            let html = convert_with(DOCTYPE_ECHO, &Options::new().safe_mode(mode));
            assert!(
                html.contains("doctype=article"),
                "{mode:?} should pin article"
            );
            assert!(!html.contains("doctype=book"), "{mode:?} dropped book");
            assert!(
                html.contains("<body class=\"article\">"),
                "{mode:?} body class"
            );
        }
    }

    #[test]
    fn api_set_doctype_is_also_pinned_to_article() {
        // The pin overrides the API too: an API `doctype` directive does not
        // survive in any mode (unlike, e.g., `docinfo`, which the API may set).
        for mode in [SafeMode::Unsafe, SafeMode::Server, SafeMode::Secure] {
            let html = convert_with(
                DOCTYPE_ECHO,
                &Options::new().safe_mode(mode).attribute("doctype", "book"),
            );
            assert!(
                html.contains("doctype=article"),
                "{mode:?} should pin article"
            );
            assert!(
                !html.contains("doctype=book"),
                "{mode:?} dropped the API value"
            );
        }
    }

    #[test]
    fn soft_default_doctype_does_not_survive() {
        // A *soft* API default leaves an attribute document-overridable, so a
        // `mentions`-based guard would skip the pin. The doctype stays article
        // regardless: neither the document's `:doctype:` nor the API's soft
        // default takes effect.
        let html = convert_with(
            DOCTYPE_ECHO,
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute_default("doctype", "book"),
        );
        assert!(html.contains("doctype=article"));
        assert!(!html.contains("doctype=book"));
    }
}
