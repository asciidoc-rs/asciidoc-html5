//! Coverage of the AsciiDoc language description's *Document Attributes
//! Reference* page.
//!
//! This page is a catalog of every built-in (reserved) document attribute. Most
//! rows describe an attribute whose *rendered* effect is verified on the
//! dedicated page for that feature (sections, toc, document, verbatim, macros,
//! role, id, …) or that targets a backend/converter this renderer does not
//! implement (DocBook, PDF, manpage, CodeRay/Pygments/Rouge/prettify, MathJax);
//! those rows are tracked as non-normative here with a reason. The one thing
//! this page is the *primary* reference for — the intrinsic attributes the
//! processor computes and the compliance defaults — is verified directly by
//! resolving an attribute reference and inspecting the substituted value.

use std::path::PathBuf;

use crate::{convert, convert_with, load_with, tests::sdd::*, Options, ReferenceTime, SafeMode};

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/document-attributes-ref.adoc");

/// Renders `source` and returns the inner HTML of the first paragraph. The
/// result is compared as a raw string (not through the DOM helpers) because an
/// unresolved reference such as `{safe-mode-safe}` renders as the literal text
/// `{safe-mode-safe}`, which a DOM parser would not round-trip.
fn para(source: &str) -> String {
    let output = convert(source);
    para_of(&output)
}

fn para_with(source: &str, options: &Options) -> String {
    let output = convert_with(source, options);
    para_of(&output)
}

fn para_of(output: &str) -> String {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner.to_string())
        .expect("output should contain a paragraph")
}

/// Creates a scratch source file so the `doc*` path attributes resolve against
/// a real, canonicalizable path. Returns the canonical directory and file.
fn scratch_file(tag: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("adoc-docattr-ref-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    let file = dir.join("userguide.adoc");
    std::fs::write(&file, "").expect("write scratch file");
    (
        std::fs::canonicalize(&dir).expect("canonicalize dir"),
        std::fs::canonicalize(&file).expect("canonicalize file"),
    )
}

non_normative!(
    r#"
= Document Attributes Reference
:page-aliases: document-attributes-reference.adoc
// TODO use icons or emoji for y and n
:y: Yes
:n: No
:endash: &#8211;
:url-epoch: https://reproducible-builds.org/specs/source-date-epoch/

Document attributes are used either to configure behavior in the processor or to relay information about the document and its environment.
This page catalogs all the built-in (i.e., reserved) document attributes in AsciiDoc.

Unless otherwise marked, these attributes can be modified (set or unset) from the API using the `:attributes` option, from the CLI using the `-a` option, or in the document (often in the document header) using an attribute entry.

Use the follow legend to understand the columns and values in the tables on this page.

[horizontal]
Set By Default:: The attribute is automatically set and assigned a default value by the AsciiDoc processor.
The default value is indicated in *bold* (e.g., `*skip*`).

Allowable Values:: Allowable values for the attribute.
Numeric values and values shown in _italic_ are instructional and indicate a value type (e.g., _any_, _empty_, _number_, 1{endash}5, etc.).
+
* _any_ -- Any value is accepted.
* _empty_ -- Indicates the attribute doesn't require an explicit value.
The attribute is simply turned on by being set.
* _empty_[=`effective`] -- In some cases, an empty value is interpreted by the processor as one of the allowable non-empty values.
This effective value is prefixed with an equals sign and enclosed in square brackets (e.g., [=`auto`]).
An attribute reference will resolve to an empty value rather than the effective value.
* (`implied`) -- Built-in attributes that are not set may have an implied value.
The implied value is enclosed in parentheses (e.g., (`attributes`)).
An implied value can't be resolved using an attribute reference.

+
If the attribute doesn't accept _any_ or _empty_, than you must only assign one of the allowable values or specified value type.

Header Only:: The attribute must be set or unset by the end of the document header (i.e., set by the API, CLI, or in the document header).
Otherwise, the assignment won't have any effect on the document.
If an attribute is not marked as _Header Only_, it can be set anywhere in the document, assuming the attribute is not locked by the API or CLI.
However, changing an attribute only affects behavior for content that follows the assignment (in document order).

== Intrinsic attributes

Intrinsic attributes are set automatically by the processor.
These attributes provide information about the document being processed (e.g., `docfile`), the security mode under which the processor is running (e.g., `safe-mode-name`), and information about the user's environment (e.g., `user-home`).

Many of these attributes are read only, which means they can't be modified (with some exceptions).
Attributes which are not are marked as modifiable.
Attributes marked as both modifiable and API/CLI Only can only be set from the API or CLI.
All other attributes marked as modifiable must be set by the end of the header (i.e., Header Only).

"#
);

#[test]
fn intrinsic_backend_family_metadata_and_convenience_attributes() {
    verifies!(
        r#"
[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Modifiable .>|API/CLI Only .>|Notes

|backend
|_any_ +
_ex._ html5
|{y}
|{n}
|The backend used to select and activate the converter that creates the output file.
Usually named according to the output format (e.g., `html5`)

|backend-<backend>
|_empty_
|{n}
|n/a
|A convenience attribute for checking which backend is selected.
<backend> is the value of the `backend` attribute (e.g., `backend-html5`).
Only one such attribute is set at time.

//|backend-<backend>-doctype-<doctype>
//|_empty_
//|{n}
//|n/a
//|A convenience attribute for checking the current backend/doctype combination.
//<backend> is the value of the `backend` attribute and <doctype> is the value of the `doctype` attribute (e.g., `backend-html5-doctype-article`).
//Only one such attribute is set at time.

|basebackend
|_any_ +
_ex._ html
|{n}
|n/a
|The generic backend on which the backend is based.
Typically derived from the backend value minus trailing numbers (e.g., the basebackend for `docbook5` is `docbook`).
May also indicate the internal backend employed by the converter (e.g., the basebackend for `pdf` is `html`).

|basebackend-<basebackend>
|_empty_
|{n}
|n/a
|A convenience attribute for checking which basebackend is active.
<basebackend> is the value of the `basebackend` attribute (e.g., `basebackend-html`).
Only one such attribute is set at time.

//|basebackend-<basebackend>-doctype-<doctype>
//|_empty_
//|{n}
//|n/a
//|A convenience attribute for checking the current basebackend/doctype combination.
//<basebackend> is the value of the `basebackend` attribute and <doctype> is the value of the `doctype` attribute (e.g., `basebackend-html-doctype-article`).
//Only one such attribute is set at time.

|docdate
|_date (ISO)_ +
_ex._ 2019-01-04
|{y}
|{n}
|Last modified date of the source document.^[<<note-docdatetime,1>>,<<note-sourcedateepoch,2>>]^

|docdatetime
|_datetime (ISO)_ +
_ex._ 2019-01-04 19:26:06 UTC
|{y}
|{n}
|Last modified date and time of the source document.^[<<note-docdatetime,1>>,<<note-sourcedateepoch,2>>]^

|docdir
|_directory path_ +
_ex._ /home/user/docs
|If input is a string
|{y}
|Full path of the directory that contains the source document.
Empty if the safe mode is SERVER or SECURE (to conceal the file's location).

|docfile
|_file path_ +
_ex._ /home/user/docs/userguide.adoc
|If input is a string
|{y}
|Full path of the source document.
Truncated to the basename if the safe mode is SERVER or SECURE (to conceal the file's location).

|docfilesuffix
|_file extension_ +
_ex._ .adoc
|If input is a string
|{y}
|File extension of the source document, including the leading period.

|docname
|_file stem basename_ +
_ex._ userguide
|If input is a string
|{y}
|Root name of the source document (no leading path or file extension).

|doctime
|_time (ISO)_ +
_ex._ 19:26:06 UTC
|{y}
|{n}
|Last modified time of the source document.^[<<note-docdatetime,1>>,<<note-sourcedateepoch,2>>]^

|doctype-<doctype>
|_empty_
|{n}
|n/a
|A convenience attribute for checking the doctype of the document.
<doctype> is the value of the `doctype` attribute (e.g., `doctype-book`).
Only one such attribute is set at time.

|docyear
|_integer_ +
_ex._ {docyear}
|{y}
|{n}
|Year that the document was last modified.^[<<note-docdatetime,1>>,<<note-sourcedateepoch,2>>]^

|embedded
|_empty_
|{n}
|n/a
|Only set if content is being converted to an embedded document (i.e., body of document only).

|filetype
|_any_ +
_ex._ html
|If input is a string
|{y}
|File extension of the output file name (without leading period).

|filetype-<filetype>
|_empty_
|{n}
|n/a
|A convenience attribute for checking the filetype of the output.
<filetype> is the value of the `filetype` attribute (e.g., `filetype-html`).
Only one such attribute is set at time.

"#
    );

    // `backend`, `basebackend`, and `filetype` resolve from the active html5
    // backend.
    assert_eq!(
        para("{backend}|{basebackend}|{filetype}"),
        "html5|html|html"
    );

    // For each of those families a convenience attribute `<family>-<value>` is
    // set (to an empty value) for the *active* value, so a reference resolves to
    // the empty string; a reference to an inactive one is left untouched
    // (`attribute-missing` defaults to `skip`). `doctype-<doctype>` behaves the
    // same way for the (pinned) `article` doctype.
    assert_eq!(
        para("[{backend-html5}][{basebackend-html}][{filetype-html}][{doctype-article}]"),
        "[][][][]",
    );
    assert_eq!(para("[{backend-docbook5}]"), "[{backend-docbook5}]");
    assert_eq!(para("[{doctype-book}]"), "[{doctype-book}]");

    // The `doc*` date/time family reflects the source's modification time.
    // Pinning the reference time (which the `doc*` family follows when no input
    // mtime is set) makes the values deterministic.
    let pinned =
        Options::new().reference_time(ReferenceTime::from_local(2019, 1, 2, 3, 4, 5, 6 * 3600));
    assert_eq!(
        para_with("{docdate}|{docyear}|{doctime}|{docdatetime}", &pinned),
        "2019-01-02|2019|03:04:05 +0600|2019-01-02 03:04:05 +0600",
    );

    // The `doc*` path family is derived from the input file. Under Unsafe the
    // full path is exposed, so `docdir` and `docfile` resolve to the scratch
    // directory and file, while `docname` (stem) and `docfilesuffix` (extension)
    // carry no location.
    let (dir, file) = scratch_file("v1");
    let with_file = Options::new()
        .safe_mode(SafeMode::Unsafe)
        .input_file(file.clone());
    assert_eq!(
        para_with("{docname}|{docfilesuffix}", &with_file),
        "userguide|.adoc"
    );
    assert_eq!(
        para_with("{docdir}|{docfile}", &with_file),
        format!("{}|{}", dir.display(), file.display()),
    );
    let _ = std::fs::remove_dir_all(&dir);

    // `embedded` is only ever set when converting an embedded document, so it is
    // unset for this standalone conversion and the reference is left untouched.
    // (This renderer does not populate the attribute even in embedded mode, so
    // only the unset case is checked.)
    assert_eq!(para("[{embedded}]"), "[{embedded}]");
}

// `htmlsyntax` is not modeled: this renderer always emits HTML syntax, so the
// attribute is never populated and has no distinct rendered effect to verify.
non_normative!(
    r#"
|htmlsyntax
|`html` +
`xml`
|{n}
|n/a
|Syntax used when generating the HTML output.
Controlled by and derived from the backend name (html=html or xhtml=html).

"#
);

#[test]
fn intrinsic_local_date_and_time_family() {
    verifies!(
        r#"
|localdate
|_date (ISO)_ +
_ex._ 2019-02-17
|{y}
|{n}
|Date when the document was converted.^[<<note-sourcedateepoch,2>>]^

|localdatetime
|_datetime (ISO)_ +
_ex._ 2019-02-17 19:31:05 UTC
|{y}
|{n}
|Date and time when the document was converted.^[<<note-sourcedateepoch,2>>]^

|localtime
|_time (ISO)_ +
_ex._ 19:31:05 UTC
|{y}
|{n}
|Time when the document was converted.^[<<note-sourcedateepoch,2>>]^

|localyear
|_integer_ +
_ex._ {localyear}
|{y}
|{n}
|Year when the document was converted.^[<<note-sourcedateepoch,2>>]^

"#
    );

    // The `local*` family records when the document was converted ("now"), which
    // a pinned reference time makes deterministic. A zero UTC offset is reported
    // as `UTC`.
    let pinned =
        Options::new().reference_time(ReferenceTime::from_local(2019, 2, 17, 19, 31, 5, 0));
    assert_eq!(
        para_with(
            "{localdate}|{localyear}|{localtime}|{localdatetime}",
            &pinned
        ),
        "2019-02-17|2019|19:31:05 UTC|2019-02-17 19:31:05 UTC",
    );
}

// `outdir` and `outfile` are, by the page's own note, unavailable to attribute
// references (they only exist on the API once the document is written), so
// there is no substituted value to verify from rendered output.
non_normative!(
    r#"
|outdir
|_directory path_ +
_ex._ /home/user/docs/dist
|{n}
|n/a
|Full path of the output directory.
(Cannot be referenced in the content.
Only available to the API once the document is converted).

|outfile
|_file path_ +
_ex._ /home/user/docs/dist/userguide.html
|{n}
|n/a
|Full path of the output file.
(Cannot be referenced in the content.
Only available to the API once the document is converted).

"#
);

#[test]
fn intrinsic_outfilesuffix_safe_mode_and_user_home() {
    verifies!(
        r#"
|outfilesuffix
|_file extension_ +
_ex._ .html
|{y}
|{n}
|File extension of the output file (starting with a period) as determined by the backend (`.html` for `html`, `.xml` for `docbook`, etc.).

|safe-mode-level
|`0` +
`1` +
`10` +
`20`
|{n}
|n/a
|Numeric value of the safe mode setting.
(0=UNSAFE, 1=SAFE, 10=SERVER, 20=SECURE).

|safe-mode-name
|`UNSAFE` +
`SAFE` +
`SERVER` +
`SECURE`
|{n}
|n/a
|Textual value of the safe mode setting.

|safe-mode-unsafe
|_empty_
|{n}
|n/a
|Set if the safe mode is UNSAFE.

|safe-mode-safe
|_empty_
|{n}
|n/a
|Set if the safe mode is SAFE.

|safe-mode-server
|_empty_
|{n}
|n/a
|Set if the safe mode is SERVER.

|safe-mode-secure
|_empty_
|{n}
|n/a
|Set if the safe mode is SECURE.

|user-home
|_directory path_ +
_ex._ /home/user
|{n}
|n/a
|Full path of the home directory for the current user.
Masked as `.` if the safe mode is SERVER or SECURE.
|===
"#
    );

    // `outfilesuffix` reflects the html5 backend.
    assert_eq!(para("{outfilesuffix}"), ".html");

    // `safe-mode-name` and `safe-mode-level` report the active safe mode. (The
    // name is lower case; the page's upper-case values name the mode.)
    let mode = |m: SafeMode| {
        para_with(
            "{safe-mode-name}|{safe-mode-level}",
            &Options::new().safe_mode(m),
        )
    };
    assert_eq!(mode(SafeMode::Unsafe), "unsafe|0");
    assert_eq!(mode(SafeMode::Safe), "safe|1");
    assert_eq!(mode(SafeMode::Server), "server|10");
    assert_eq!(mode(SafeMode::Secure), "secure|20");

    // Exactly the `safe-mode-<name>` convenience attribute for the active mode is
    // set; the others are left unresolved.
    assert_eq!(
        para_with(
            "[{safe-mode-unsafe}][{safe-mode-secure}]",
            &Options::new().safe_mode(SafeMode::Unsafe),
        ),
        "[][{safe-mode-secure}]",
    );
    assert_eq!(
        para_with(
            "[{safe-mode-secure}][{safe-mode-unsafe}]",
            &Options::new().safe_mode(SafeMode::Secure),
        ),
        "[][{safe-mode-unsafe}]",
    );

    // `user-home` is masked to `.` under SERVER and SECURE.
    assert_eq!(
        para_with("{user-home}", &Options::new().safe_mode(SafeMode::Server)),
        ".",
    );
    assert_eq!(
        para_with("{user-home}", &Options::new().safe_mode(SafeMode::Secure)),
        ".",
    );
}

// The intrinsic-table footnotes and the compliance-section heading are
// descriptive prose.
non_normative!(
    r#"
[[note-docdatetime]]^[1]^ Only reflects the last modified time of the source document file.
It does not consider the last modified time of files which are included.

[[note-sourcedateepoch]]^[2]^ If the SOURCE_DATE_EPOCH environment variable is set, the value assigned to this attribute is built from a UTC date object that corresponds to the timestamp (as an integer) stored in that environment variable.
This override offers one way to make the conversion reproducible.
See the {url-epoch}[source date epoch specification] for more information about the SOURCE_DATE_EPOCH environment variable.
Otherwise, the date is expressed in the local time zone, which is reported as a time zone offset (e.g., `-0600`) or UTC if the time zone offset is 0).
To force the use of UTC, set the `TZ=UTC` environment variable when invoking Asciidoctor.

== Compliance attributes

"#
);

#[test]
fn compliance_attribute_missing_and_undefined_defaults() {
    verifies!(
        r#"
[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|attribute-missing
|`drop` +
`drop-line` +
`*skip*` +
`warn`
|{y}
|{n}
|Controls how xref:unresolved-references.adoc#missing[missing attribute references] are handled.

|attribute-undefined
|`drop` +
`*drop-line*`
|{y}
|{n}
|Controls how xref:unresolved-references.adoc#undefined[attribute unassignments] are handled.

"#
    );

    // The two compliance attributes that control unresolved-reference handling
    // are set to their documented defaults (shown in bold on the page).
    assert_eq!(
        para("{attribute-missing}|{attribute-undefined}"),
        "skip|drop-line",
    );
}

// `compat-mode` and `experimental` configure parser/loader features that are
// exercised by `asciidoc-parser`; neither has a distinct rendered effect to
// resolve here.
non_normative!(
    r#"
|compat-mode
|_empty_
|{n}
|{n}
|Controls when the legacy parsing mode is used to parse the document.

|experimental
|_empty_
|{n}
|{y}
|Enables xref:macros:ui-macros.adoc[] and the xref:macros:keyboard-macro.adoc[].

"#
);

#[test]
fn reproducible_and_skip_front_matter_take_effect() {
    verifies!(
        r#"
|reproducible
|_empty_
|{n}
|{y}
|Prevents last-updated date from being added to HTML footer or DocBook info element.
Useful for storing the output in a source code control system as it prevents spurious changes every time you convert the document.
Alternately, you can use the SOURCE_DATE_EPOCH environment variable, which sets the epoch of all source documents and the local datetime to a fixed value.

|skip-front-matter
|_empty_
|{n}
|{y}
|Consume YAML-style frontmatter at top of document and store it in `front-matter` attribute.
//<<front-matter-added-for-static-site-generators>>
|===
"#
    );

    // `reproducible` prevents the last-updated date – and the equally
    // build-volatile `generator` meta – from being added to the HTML output. A
    // pinned reference time keeps the comparison deterministic and standalone
    // output exposes the head and footer: without the attribute the footer
    // stamps "Last updated …" and the head carries the `generator` meta;
    // setting it removes both.
    let standalone = || {
        Options::new()
            .standalone(true)
            .reference_time(ReferenceTime::from_unix_timestamp(0))
    };

    let default_html = convert_with("= Doc\n\nBody.", &standalone());
    assert!(default_html.contains("Last updated "));
    assert!(default_html.contains("name=\"generator\""));

    let reproducible_html = convert_with("= Doc\n:reproducible:\n\nBody.", &standalone());
    assert!(!reproducible_html.contains("Last updated"));
    assert!(!reproducible_html.contains("name=\"generator\""));

    // `skip-front-matter` (an API/CLI-only attribute) consumes a `---`-fenced
    // YAML-style block at the very top of the document. With it set, the block
    // is removed, so the following `= Real Title` line is recognized as the
    // document title; without it, the `---` and YAML lines keep their ordinary
    // meaning as body content and no title is found.
    let src = "---\nlayout: post\ntitle: FM Title\n---\n= Real Title\n\nBody.\n";

    let skipped = convert_with(src, &standalone().set("skip-front-matter"));
    assert!(skipped.contains("<h1>Real Title</h1>"));
    assert!(!skipped.contains("layout: post"));

    let kept = convert_with(src, &standalone());
    assert!(!kept.contains("<h1>Real Title</h1>"));
    assert!(kept.contains("layout: post"));

    // The consumed front matter is stored verbatim (delimiters excluded, lines
    // joined by LF) in the `front-matter` attribute.
    assert_eq!(
        load_with(src, &Options::new().set("skip-front-matter")).attribute_value("front-matter"),
        asciidoc_parser::document::InterpretedValue::Value(
            "layout: post\ntitle: FM Title".to_string()
        ),
    );
}

#[test]
fn i18n_caption_label_and_signifier_defaults() {
    verifies!(
        r#"

[#builtin-attributes-i18n]
== Localization and numbering attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|appendix-caption
|_any_ +
`*Appendix*`
|{y}
|{n}
|Label added before an xref:sections:appendix.adoc[appendix title].

|appendix-number
|_character_ +
(`@`)
|{n}
|{n}
|Sets the seed value for the appendix number sequence.^[<<note-number,1>>]^

|appendix-refsig
|_any_ +
`*Appendix*`
|{y}
|{n}
|Signifier added to Appendix title cross references.

|caution-caption
|_any_ +
`*Caution*`
|{y}
|{n}
|Text used to label CAUTION admonitions when icons aren't enabled.

|chapter-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the chapter number sequence.^[<<note-number,1>>]^
_Book doctype only_.

|chapter-refsig
|_any_ +
`*Chapter*`
|{y}
|{n}
|Signifier added to Chapter titles in cross references.
_Book doctype only_.

|chapter-signifier
|_any_
|{n}
|{n}
|xref:sections:chapters.adoc[Label added to level 1 section titles (chapters)].
_Book doctype only_.

|example-caption
|_any_ +
`*Example*`
|{y}
|{n}
|Text used to label example blocks.

|example-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the example number sequence.^[<<note-number,1>>]^

|figure-caption
|_any_ +
`*Figure*`
|{y}
|{n}
|Text used to label images and figures.

|figure-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the figure number sequence.^[<<note-number,1>>]^

|footnote-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the footnote number sequence.^[<<note-number,1>>]^

|important-caption
|_any_ +
`*Important*`
|{y}
|{n}
|Text used to label IMPORTANT admonitions when icons are not enabled.

|lang
|_BCP 47 language tag_ +
(`en`)
|{n}
|{y}
|Language tag specified on document element of the output document.
Refer to https://html.spec.whatwg.org/#the-lang-and-xml:lang-attributes[the lang and xml:lang attributes section^] of the HTML specification to learn about the acceptable values for this attribute.

|last-update-label
|_any_ +
`*Last updated*`
|{y}
|{y}
|Text used for “Last updated” label in footer.

|listing-caption
|_any_
|{n}
|{n}
|Text used to label listing blocks.

|listing-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the listing number sequence.^[<<note-number,1>>]^

|manname-title
|_any_ +
(`Name`)
|{n}
|{y}
|Label for program name section in the man page.

|nolang
|_empty_
|{n}
|{y}
|Prevents `lang` attribute from being added to root element of the output document.

|note-caption
|_any_ +
`*Note*`
|{y}
|{n}
|Text used to label NOTE admonitions when icons aren't enabled.

|part-refsig
|_any_ +
`*Part*`
|{y}
|{n}
|Signifier added to Part titles in cross references.
_Book doctype only_.

|part-signifier
|_any_
|{n}
|{n}
|xref:sections:chapters.adoc[Label added to level 0 section titles (parts)].
_Book doctype only_.

|preface-title
|_any_
|{n}
|{n}
|Title text for an anonymous preface when `doctype` is `book`.

|section-refsig
|_any_ +
`*Section*`
|{y}
|{n}
|Signifier added to title of numbered sections in cross reference text.

|table-caption
|_any_ +
`*Table*`
|{y}
|{n}
|Text of label prefixed to table titles.

|table-number
|_number_ +
(`0`)
|{n}
|{n}
|Sets the seed value for the table number sequence.^[<<note-number,1>>]^

|tip-caption
|_any_ +
`*Tip*`
|{y}
|{n}
|Text used to label TIP admonitions when icons aren't enabled.

|toc-title
|_any_ +
`*Table of Contents*`
|{y}
|{y}
|xref:toc:title.adoc[Title for table of contents].

|untitled-label
|_any_ +
`*Untitled*`
|{y}
|{y}
|Default document title if document doesn't have a document title.

|version-label
|_any_ +
`*Version*`
|{y}
|{y}
|See xref:document:version-label.adoc[].

|warning-caption
|_any_ +
`*Warning*`
|{y}
|{n}
|Text used to label WARNING admonitions when icons aren't enabled.
|===

"#
    );

    // The caption, label, and refsig attributes the table marks "Set By Default
    // = Yes" resolve to their documented default value (shown in bold on the
    // page). `chapter-refsig` and `part-refsig` carry a value even in the
    // default (article) doctype; their "book doctype only" note governs where
    // the signifier is *used*, not whether the attribute is set.
    assert_eq!(
        para(
            "[{appendix-caption}][{appendix-refsig}][{caution-caption}][{chapter-refsig}]\
             [{example-caption}][{figure-caption}][{important-caption}][{last-update-label}]\
             [{note-caption}][{part-refsig}][{section-refsig}][{table-caption}][{tip-caption}]\
             [{toc-title}][{untitled-label}][{version-label}][{warning-caption}]"
        ),
        "[Appendix][Appendix][Caution][Chapter][Example][Figure][Important][Last updated]\
         [Note][Part][Section][Table][Tip][Table of Contents][Untitled][Version][Warning]",
    );

    // Every row marked "Set By Default = No" is genuinely unset for a default
    // (article-doctype) conversion, so a reference to it is left untouched
    // (`attribute-missing` defaults to `skip`). This covers the numbering seeds
    // (`*-number`), the caption/title/signifier rows that opt out by default
    // (`listing-caption`, `chapter-signifier`, `part-signifier`, `preface-title`),
    // and the localization toggles whose rendered effect lives elsewhere (`lang`,
    // `nolang`, and the manpage-only `manname-title`).
    assert_eq!(
        para(
            "[{appendix-number}][{chapter-number}][{chapter-signifier}][{example-number}]\
             [{figure-number}][{footnote-number}][{lang}][{listing-caption}][{listing-number}]\
             [{manname-title}][{nolang}][{part-signifier}][{preface-title}][{table-number}]"
        ),
        "[{appendix-number}][{chapter-number}][{chapter-signifier}][{example-number}]\
         [{figure-number}][{footnote-number}][{lang}][{listing-caption}][{listing-number}]\
         [{manname-title}][{nolang}][{part-signifier}][{preface-title}][{table-number}]",
    );
}

#[test]
fn document_metadata_header_attributes() {
    verifies!(
        r#"
== Document metadata attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|app-name
|_any_
|{n}
|{y}
|Adds `application-name` meta element for mobile devices inside HTML document head.

|author
|_any_
|Extracted from author info line
|{y}
|Can be set automatically via the author info line or explicitly.
See xref:document:author-information.adoc[].

|authorinitials
|_any_
|Extracted from `author` attribute
|{y}
|Derived from the author attribute by default.
See xref:document:author-information.adoc[].

|authors
|_any_
|Extracted from author info line
|{y}
|Can be set automatically via the author info line or explicitly as a comma-separated value list.
See xref:document:author-information.adoc[].

|copyright
|_any_
|{n}
|{y}
|Adds `copyright` meta element in HTML document head.

|doctitle
|_any_
|Yes, if document has a doctitle
|{y}
|See xref:document:title.adoc#reference-doctitle[doctitle attribute].

|description
|_any_
|{n}
|{y}
|Adds xref:document:metadata.adoc#description[description] meta element in HTML document head.

|email
|_any_
|Extracted from author info line
|{y}
|Can be any inline macro, such as a URL.
See xref:document:author-information.adoc[].

|firstname
|_any_
|Extracted from author info line
|{y}
|See xref:document:author-information.adoc[].

|front-matter
|_any_
|Yes, if front matter is captured
|n/a
|If `skip-front-matter` is set via the API or CLI, any YAML-style frontmatter skimmed from the top of the document is stored in this attribute.

|keywords
|_any_
|{n}
|{y}
|Adds xref:document:metadata.adoc#keywords[keywords] meta element in HTML document head.

|lastname
|_any_
|Extracted from author info line
|{y}
|See xref:document:author-information.adoc[].

|middlename
|_any_
|Extracted from author info line
|{y}
|See xref:document:author-information.adoc[].

"#
    );

    // The head-meta attributes render as `<meta>` elements in a standalone
    // document's `<head>`, in Asciidoctor's order: `app-name` as
    // `application-name`, then `description`, `keywords`, the joined `author`,
    // and `copyright`. `doctitle` drives the `<title>` element.
    let head = convert_with(
        "= Doc Title\nKismet R. Lee <kismet@example.com>\n:app-name: Widgets\n:description: A guide.\n:keywords: alpha, beta\n:copyright: ACME 2020\n\nbody",
        &Options::new().standalone(true),
    );

    for meta in [
        r#"<meta name="application-name" content="Widgets">"#,
        r#"<meta name="description" content="A guide.">"#,
        r#"<meta name="keywords" content="alpha, beta">"#,
        r#"<meta name="author" content="Kismet R. Lee">"#,
        r#"<meta name="copyright" content="ACME 2020">"#,
        r#"<title>Doc Title</title>"#,
    ] {
        assert!(head.contains(meta), "head should contain {meta}:\n{head}");
    }

    // `author`, `authors`, the name-part attributes (`authorinitials`,
    // `firstname`, `middlename`, `lastname`), and `doctitle` are extracted from
    // the author info line and the document title.
    assert_eq!(
        para(
            "= Doc Title\nKismet R. Lee <kismet@example.com>\n\n\
             [{doctitle}|{author}|{authors}|{authorinitials}|{firstname}|{middlename}|{lastname}]"
        ),
        "[Doc Title|Kismet R. Lee|Kismet R. Lee|KRL|Kismet|R.|Lee]",
    );

    // `email` may be any inline macro, so an email address renders as a mailto
    // link.
    assert!(para("= Doc\nKismet R. Lee <kismet@example.com>\n\n{email}")
        .contains(r#"<a href="mailto:kismet@example.com">"#));

    // `front-matter` is populated only when `skip-front-matter` consumes a
    // leading YAML block (its capture is verified in full on the compliance
    // table above).
    assert!(load_with(
        "---\nkey: val\n---\n= T\n\nbody\n",
        &Options::new().set("skip-front-matter")
    )
    .is_attribute_set("front-matter"));
}

// `orgname` targets the DocBook `<info>` element; this renderer emits html5,
// which has no equivalent, so the attribute has no rendered effect to verify
// here.
non_normative!(
    r#"
|orgname
|_any_
|{n}
|{y}
|Adds `<orgname>` element value to DocBook info element.

"#
);

#[test]
fn document_metadata_revision_and_title_attributes() {
    verifies!(
        r#"
|revdate
|_any_
|Extracted from revision info line
|{y}
|See xref:document:revision-information.adoc[].

|revremark
|_any_
|Extracted from revision info line
|{y}
|See xref:document:revision-information.adoc[].

|revnumber
|_any_
|Extracted from revision info line
|{y}
|See xref:document:revision-information.adoc[].

|title
|_any_
|{n}
|{y}
|Value of `<title>` element in HTML `<head>` or main DocBook `<info>` of output document.
Used as a fallback when the document title is not specified.
See xref:document:title.adoc#title-attr[title attribute].
|===

"#
    );

    // The revision attributes are extracted from the revision info line.
    assert_eq!(
        para("= Doc\nAuthor Name\nv2.0, 2020-01-15: First release\n\n[{revnumber}|{revdate}|{revremark}]"),
        "[2.0|2020-01-15|First release]",
    );

    // `title` supplies the `<title>` element's text; a pinned standalone
    // conversion with no document title shows it filling in as the fallback.
    let titled = convert_with(":title: Fallback\n\nbody", &Options::new().standalone(true));
    assert!(titled.contains("<title>Fallback</title>"), "{titled}");
}

#[test]
fn section_id_and_leveloffset_attributes() {
    verifies!(
        r#"
== Section title and table of contents attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|idprefix
|_valid XML ID start character_ +
`*_*`
|{y}
|{n}
|Prefix of auto-generated section IDs.
See xref:sections:id-prefix-and-separator.adoc#prefix[Change the ID prefix].

|idseparator
|_valid XML ID character_ +
`*_*`
|{y}
|{n}
|Word separator used in auto-generated section IDs.
See xref:sections:id-prefix-and-separator.adoc#separator[Change the ID word separator].

|leveloffset
|{startsb}+-{endsb}0{endash}5
|{n}
|{n}
|Increases or decreases level of headings below assignment.
A leading + or - makes the value relative.
//<<include-partitioning>>

"#
    );

    // Section IDs are auto-generated from the title as a lowercased slug: each
    // run of non-word characters becomes the `idseparator` (default `_`) and
    // the whole thing sits behind `idprefix` (default `_`).
    assert!(convert("= Doc\n\n== Hello World").contains(r#"<h2 id="_hello_world">"#));

    // Overriding both attributes changes the prefix and the word separator.
    assert!(
        convert("= Doc\n:idprefix: sect-\n:idseparator: -\n\n== Hello World")
            .contains(r#"<h2 id="sect-hello-world">"#)
    );

    // `leveloffset` shifts the level of the headings that follow it; `+1` turns
    // a level-1 section (`==`, normally an `<h2>`) into an `<h3>`.
    assert!(convert("= Doc\n\n:leveloffset: +1\n== Shifted")
        .contains(r#"<h3 id="_shifted">Shifted</h3>"#));
}

// `partnums` numbers book parts, which only exist in the book doctype this
// renderer does not yet lay out (html5#188), so there is no article-doctype
// effect to resolve here.
non_normative!(
    r#"
|partnums
|_empty_
|{n}
|{n}
|Enables numbering of parts.
See xref:sections:part-numbers-and-labels.adoc#partnums[Number book parts].
_Book doctype only_.

"#
);

#[test]
fn section_anchor_link_and_numbering_attributes() {
    verifies!(
        r#"
|sectanchors
|_empty_
|{n}
|{n}
|xref:sections:title-links.adoc#anchor[Adds anchor in front of section title] on mouse cursor hover.

|sectids
|*_empty_*
|{y}
|{n}
|Generates and assigns an ID to any section that does not have an ID.
See xref:sections:auto-ids.adoc#disable[Disable automatic ID generation].

|sectlinks
|_empty_
|{n}
|{n}
|xref:sections:title-links.adoc[Turns section titles into self-referencing links].

|sectnums
|_empty_ +
`all`
|{n}
|{n}
|xref:sections:numbers.adoc[Numbers sections] to depth specified by `sectnumlevels`.

|sectnumlevels
|0{endash}5 +
(`3`)
|{n}
|{n}
|xref:sections:numbers.adoc#numlevels[Controls depth of section numbering].

"#
    );

    // `sectanchors` prepends a hover anchor to each section title.
    assert!(convert("= Doc\n:sectanchors:\n\n== Hello")
        .contains(r##"<h2 id="_hello"><a class="anchor" href="#_hello"></a>Hello</h2>"##));

    // `sectids` is set by default, so a section gets an auto-generated ID;
    // unsetting it suppresses ID generation.
    assert!(convert("= Doc\n\n== Hello").contains(r#"<h2 id="_hello">"#));
    assert!(convert("= Doc\n:sectids!:\n\n== Hello").contains("<h2>Hello</h2>"));

    // `sectlinks` turns each section title into a self-referencing link.
    assert!(convert("= Doc\n:sectlinks:\n\n== Hello")
        .contains(r##"<h2 id="_hello"><a class="link" href="#_hello">Hello</a></h2>"##));

    // `sectnums` numbers section titles; `sectnumlevels` (default 3) caps how
    // deep the numbering goes, so at level 1 the nested section is left plain.
    let numbered = convert("= Doc\n:sectnums:\n\n== First\n\n=== Sub");
    assert!(numbered.contains(r#"<h2 id="_first">1. First</h2>"#));
    assert!(numbered.contains(r#"<h3 id="_sub">1.1. Sub</h3>"#));
    assert!(
        convert("= Doc\n:sectnums:\n:sectnumlevels: 1\n\n== First\n\n=== Sub")
            .contains(r#"<h3 id="_sub">Sub</h3>"#)
    );
}

// `title-separator` splits a document title into title and subtitle. This
// renderer has no subtitle rendering (and, like Asciidoctor, leaves the
// article-doctype `<h1>` unsplit), so the attribute has no distinct effect to
// verify here.
non_normative!(
    r#"
|title-separator
|_any_
|{n}
|{y}
|Character used to xref:document:subtitle.adoc[separate document title and subtitle].

"#
);

#[test]
fn table_of_contents_attributes() {
    verifies!(
        r#"
|toc
|_empty_[=`auto`] +
`auto` +
`left` +
`right` +
`macro` +
`preamble`
|{n}
|{y}
|Turns on xref:toc:index.adoc[table of contents] and specifies xref:toc:position.adoc[its location].

|toclevels
|0{endash}5 +
(`2`)
|{n}
|{y}
|xref:toc:levels.adoc[Maximum section level to display].

"#
    );

    // `toc` turns on the table of contents; `toclevels` (default 2) lists
    // sections down to the given level.
    let toc = convert_with(
        "= Doc\n:toc:\n\n== First\n\n=== Sub",
        &Options::new().standalone(true),
    );
    assert!(toc.contains(r##"<a href="#_first">First</a>"##));
    assert!(toc.contains(r##"<a href="#_sub">Sub</a>"##));

    // Capping `toclevels` at 1 drops the nested section from the contents.
    let toc1 = convert_with(
        "= Doc\n:toc:\n:toclevels: 1\n\n== First\n\n=== Sub",
        &Options::new().standalone(true),
    );
    assert!(toc1.contains(r##"<a href="#_first">First</a>"##));
    assert!(!toc1.contains(r##"<a href="#_sub">Sub</a>"##));
}

// `fragment` is a parser flag that relaxes section-nesting enforcement; it has
// no distinct rendered effect and is exercised by `asciidoc-parser`.
non_normative!(
    r#"
|fragment
|_empty_
|{n}
|{y}
|Informs parser that document is a fragment and that it shouldn't enforce proper section nesting.
|===

"#
);

// General content and formatting: these configure features verified on
// their own pages (e.g., `doctype`, `docinfo*`, `hide-uri-scheme`, `stem`,
// `table-*`, `xrefstyle`) or target other converters (PDF `media`/
// `show-link-uri`, DocBook `pagewidth`, MathJax `eqnums`), so there is
// nothing distinct to resolve from this catalog row.
non_normative!(
    r#"
== General content and formatting attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|asset-uri-scheme
|_empty_ +
`http` +
(`https`)
|{n}
|{y}
|Controls protocol used for assets hosted on a CDN.

|cache-uri
|_empty_
|{n}
|{y}
|Cache content read from URIs.
//<<caching-uri-content>>

|data-uri
|_empty_
|{n}
|{y}
|Embed graphics as data-uri elements in HTML elements so file is completely self-contained.
//<<managing-images>>

|docinfo
|_empty_[=`private`] +
`shared` +
`private` +
`shared-head` +
`private-head` +
`shared-footer` +
`private-footer`
|{n}
|{y}
|Read input from one or more DocBook info files.
//<<naming-docinfo-files>>

|docinfodir
|_directory path_
|{n}
|{y}
|Location of docinfo files.
Defaults to directory of source file if not specified.
//<<locating-docinfo-files>>

|docinfosubs
|_comma-separated substitution names_ +
(`attributes`)
|{n}
|{y}
|AsciiDoc substitutions that are applied to docinfo content.
//<<attribute-substitution-in-docinfo-files>>

|doctype
|`*article*` +
`book` +
`inline` +
`manpage`
|{y}
|{y}
|Output document type.
//<<document-types>>

|eqnums
|_empty_[=`AMS`] +
`AMS` +
`all` +
`none`
|{n}
|{y}
|Controls automatic equation numbering on LaTeX blocks in HTML output (MathJax).
If the value is AMS, only LaTeX content enclosed in an `+\begin{equation}...\end{equation}+` container will be numbered.
If the value is all, then all LaTeX blocks will be numbered.
See https://docs.mathjax.org/en/v2.5-latest/tex.html#automatic-equation-numbering[equation numbering in MathJax].

"#
);

#[test]
fn hardbreaks_option_preserves_hard_line_breaks() {
    verifies!(
        r#"
|hardbreaks-option
|_empty_
|{n}
|{n}
|xref:blocks:hard-line-breaks.adoc#per-document[Preserve hard line breaks].

"#
    );

    // Set on the document, `hardbreaks-option` turns every newline inside a
    // paragraph into a hard line break (`<br>`), matching Asciidoctor.
    assert_eq!(
        para(":hardbreaks-option:\n\nline one\nline two"),
        "line one<br>\nline two",
    );

    // Without it, the same source keeps the newline as ordinary whitespace.
    assert_eq!(para("line one\nline two"), "line one\nline two");
}

// The remaining general content and formatting attributes configure features
// verified on their own pages or targeting other converters, as noted above.
non_normative!(
    r#"
|hide-uri-scheme
|_empty_
|{n}
|{n}
|xref:macros:links.adoc#hide-uri-scheme[Hides URI scheme] for raw links.

|media
|`prepress` +
`print` +
(`screen`)
|{n}
|{y}
|Specifies media type of output and enables behavior specific to that media type.
_PDF converter only_.

"#
);

#[test]
fn footer_footnotes_header_and_title_toggle_attributes() {
    verifies!(
        r#"
|nofooter
|_empty_
|{n}
|{y}
|Turns off footer.
//<<footer-docinfo-files>>

|nofootnotes
|_empty_
|{n}
|{y}
|Turns off footnotes.
//<<user-footnotes>>

|noheader
|_empty_
|{n}
|{y}
|Turns off header.
//<<doc-header>>

|notitle
|_empty_
|{n}
|{y}
|xref:document:title.adoc#hide-or-show[Hides the doctitle in an embedded document].
Mutually exclusive with the `showtitle` attribute.

|outfilesuffix
|*_file extension_* +
_ex._ .html
|{y}
|{y}
|File extension of output file, including dot (`.`), such as `.html`.
// <<navigating-between-source-files>>

"#
    );

    // Each `no*` toggle removes the corresponding piece of a standalone
    // document: `nofooter` the footer, `nofootnotes` the footnotes block,
    // `noheader` the header, `notitle` the doctitle heading.
    let opts = || Options::new().standalone(true);
    let full = convert_with(
        "= The Title\nAuthor Name\n\nSee this.footnote:[a note]",
        &opts(),
    );
    assert!(full.contains(r#"<div id="header">"#));
    assert!(full.contains("<h1>The Title</h1>"));
    assert!(full.contains(r#"<div id="footnotes">"#));
    assert!(full.contains(r#"<div id="footer">"#));

    assert!(
        !convert_with("= The Title\n:noheader:\nAuthor Name\n\nbody", &opts())
            .contains(r#"<div id="header">"#)
    );
    assert!(
        !convert_with("= The Title\n:notitle:\nAuthor Name\n\nbody", &opts())
            .contains("<h1>The Title</h1>")
    );
    assert!(!convert_with(
        "= The Title\n:nofootnotes:\nAuthor Name\n\nx.footnote:[n]",
        &opts()
    )
    .contains(r#"<div id="footnotes">"#));
    assert!(
        !convert_with("= The Title\n:nofooter:\nAuthor Name\n\nbody", &opts())
            .contains(r#"<div id="footer">"#)
    );

    // `outfilesuffix` is the output file extension (default `.html`); it is also
    // what a relative inter-document xref uses for its target by default.
    assert_eq!(para("{outfilesuffix}"), ".html");
    assert!(convert("= T\n\nxref:other.adoc[go]").contains(r#"<a href="other.html">go</a>"#));
}

// `pagewidth` sizes tables in DocBook output, a backend this renderer does not
// implement.
non_normative!(
    r#"
|pagewidth
|_integer_ +
(`425`)
|{n}
|{y}
|Page width used to calculate the absolute width of tables in the DocBook output.

"#
);

#[test]
fn relative_xref_path_attributes() {
    verifies!(
        r#"
|relfileprefix
|_empty_ +
_path segment_
|{n}
|{n}
|The path prefix to add to relative xrefs.
//<<navigating-between-source-files>>

|relfilesuffix
|*_file extension_* +
_path segment_ +
_ex._ .html
|{y}
|{n}
|The path suffix (e.g., file extension) to add to relative xrefs.
Defaults to the value of the `outfilesuffix` attribute.
(Preferred over modifying outfilesuffix).
//|<<navigating-between-source-files>>

"#
    );

    // `relfilesuffix` (defaulting to `outfilesuffix`) and `relfileprefix` set
    // the suffix and prefix of a relative inter-document xref's target.
    assert!(convert("= T\n:relfilesuffix: .htm\n\nxref:other.adoc[go]")
        .contains(r#"<a href="other.htm">go</a>"#));
    assert!(convert("= T\n:relfileprefix: ../\n\nxref:other.adoc[go]")
        .contains(r#"<a href="../other.html">go</a>"#));

    // Because `relfilesuffix` defaults to `outfilesuffix`, changing the latter
    // changes the xref suffix too.
    assert!(
        convert("= T\n:outfilesuffix: .xhtml\n\nxref:other.adoc[go]")
            .contains(r#"<a href="other.xhtml">go</a>"#)
    );
}

// `show-link-uri` appends the link URI after the link text in the PDF
// converter, which this renderer does not implement.
non_normative!(
    r#"
|show-link-uri
|_empty_
|{n}
|{n}
|Prints the URI of a link after the link text.
_PDF converter only_.

"#
);

#[test]
fn showtitle_attribute() {
    verifies!(
        r#"
|showtitle
|_empty_
|{n}
|{y}
|xref:document:title.adoc#hide-or-show[Displays the doctitle in an embedded document].
Mutually exclusive with the `notitle` attribute.

"#
    );

    // `showtitle` shows the doctitle heading in embedded output (hidden there by
    // default); it is the inverse of `notitle`.
    assert!(convert("= The Title\n:showtitle:\n\nbody").contains("<h1>The Title</h1>"));
    assert!(!convert("= The Title\n\nbody").contains("<h1>The Title</h1>"));
}

// `stem` enables math processing. This renderer emits the MathJax delimiters
// for stem content (e.g. `stem:[x^2]` -> `\$x^2\$`, matching Asciidoctor) but
// does not yet emit the MathJax loader/config docinfo that renders them in the
// browser (html5#250), so the end-to-end feature is tracked there.
non_normative!(
    r#"
|stem
|`_empty_`[=`asciimath`] +
`asciimath` +
`latexmath`
|{n}
|{y}
|Enables xref:stem:index.adoc[mathematics processing and interpreter].

"#
);

#[test]
fn table_defaults_tabsize_webfonts_and_xrefstyle_attributes() {
    verifies!(
        r#"
|table-frame
|(`all`) +
`ends` +
`sides` +
`none`
|{n}
|{n}
|Controls default value for `frame` attribute on tables.

|table-grid
|(`all`) +
`cols` +
`rows` +
`none`
|{n}
|{n}
|Controls default value for `grid` attribute on tables.

|table-stripes
|(`none`) +
`even` +
`odd` +
`hover` +
`all`
|{n}
|{n}
|Controls default value for `stripes` attribute on tables.

|tabsize
|_integer_ (≥ 0)
|{n}
|{n}
|Converts tabs to spaces in verbatim content blocks (e.g., listing, literal).

|webfonts
|*_empty_*
|{y}
|{y}
|Control whether webfonts are loaded when using the default stylesheet.
When set to empty, uses the default font collection from Google Fonts.
A non-empty value replaces the `family` query string parameter in the Google Fonts URL.

|xrefstyle
|`full` +
`short` +
`basic`
|{n}
|{n}
|xref:macros:xref-text-and-style.adoc#cross-reference-styles[Formatting style to apply to cross reference text].
|===

"#
    );

    // The `table-*` attributes set the document-wide default for a table's
    // `frame`, `grid`, and `stripes`, reflected in the table's CSS classes.
    assert!(convert(
        "= T\n:table-frame: sides\n:table-grid: cols\n:table-stripes: even\n\n|===\n|a\n|==="
    )
    .contains(r#"class="tableblock frame-sides grid-cols stripes-even stretch""#));

    // `tabsize` expands a leading tab in verbatim content to that many spaces.
    assert!(convert("= T\n:tabsize: 2\n\n....\n\tX\n....").contains("<pre>  X</pre>"));

    // `webfonts` controls the Google Fonts `<link>` in a standalone document:
    // set (the default) links the default families, a non-empty value replaces
    // the family list, and unsetting it drops the link.
    let wf = |src: &str| convert_with(src, &Options::new().standalone(true));
    assert!(wf("= T\n\nbody").contains(
        r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Open+Sans"#
    ));
    assert!(wf("= T\n:webfonts: Roboto\n\nbody").contains(
        r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Roboto">"#
    ));
    assert!(!wf("= T\n:webfonts!:\n\nbody")
        .contains(r#"<link rel="stylesheet" href="https://fonts.googleapis.com"#));

    // `xrefstyle` selects the auto-generated cross-reference text: `full` gives
    // the number and title, `short` the number, `basic` the title.
    let xr = |style: &str| {
        convert(&format!(
            "= T\n:sectnums:\n:xrefstyle: {style}\n\n[#tgt]\n== Target Section\n\n== Two\n\nSee <<tgt>>."
        ))
    };
    assert!(xr("full").contains(r##"<a href="#tgt">Section 1, &#8220;Target Section&#8221;</a>"##));
    assert!(xr("short").contains(r##"<a href="#tgt">Section 1</a>"##));
    assert!(xr("basic").contains(r##"<a href="#tgt">Target Section</a>"##));
}

// The section heading and table scaffold carry no rendered behavior of their
// own; the individual attribute rows below are verified.
non_normative!(
    r#"
== Image and icon attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

"#
);

#[test]
fn iconfont_stylesheet_link_attributes() {
    verifies!(
        r#"
|iconfont-cdn
|_url_ +
(default CDN URL)
|{n}
|{y}
|If not specified, uses the cdnjs.com service.
Overrides CDN used to link to the Font Awesome stylesheet.

|iconfont-name
|_any_ +
(`font-awesome`)
|{n}
|{y}
|Overrides the name of the icon font stylesheet.
//<<icons>>

|iconfont-remote
|*_empty_*
|{y}
|{y}
|Allows use of a CDN for resolving the icon font.
Only relevant used when value of `icons` attribute is `font`.

"#
    );

    // The Font Awesome `<link>` lives in the standalone document's `<head>`.
    // These examples enable icons from the document header (`:icons: font`),
    // which this crate's default `Secure` safe mode drops (matching Asciidoctor –
    // see #50), so they convert under `Server`, a mode that still "allows icons,"
    // to reproduce the icon-permitting output Asciidoctor's CLI generates.
    let standalone = |source: &str| {
        convert_with(
            source,
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        )
    };

    // `iconfont-remote` is set (empty) by default, so `:icons: font` links the
    // Font Awesome stylesheet from a CDN. `iconfont-cdn`, when unset, defaults
    // to the cdnjs.com service at the pinned Font Awesome version.
    assert!(standalone("= T\n:icons: font\n\n[NOTE]\n====\nhi\n====").contains(
        r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css">"#
    ));

    // `iconfont-cdn` overrides that CDN URL outright.
    assert!(
        standalone(
            "= T\n:icons: font\n:iconfont-cdn: https://cdn.example.org/fa.css\n\n[NOTE]\n====\nhi\n===="
        )
        .contains(r#"<link rel="stylesheet" href="https://cdn.example.org/fa.css">"#)
    );

    // With `iconfont-remote` unset, no CDN is used: the stylesheet is a local
    // `{iconfont-name}.css`, whose name defaults to `font-awesome`.
    assert!(
        standalone("= T\n:icons: font\n:iconfont-remote!:\n\n[NOTE]\n====\nhi\n====")
            .contains(r#"<link rel="stylesheet" href="./font-awesome.css">"#)
    );

    // `iconfont-name` overrides that local stylesheet's name.
    assert!(standalone(
        "= T\n:icons: font\n:iconfont-remote!:\n:iconfont-name: my-icons\n\n[NOTE]\n====\nhi\n===="
    )
    .contains(r#"<link rel="stylesheet" href="./my-icons.css">"#));
}

#[test]
fn icons_and_image_location_attributes() {
    verifies!(
        r#"
|icons
|_empty_[=`image`] +
`image` +
`font`
|{n}
|{y}
|Chooses xref:macros:icons.adoc#icons-attribute[images or font icons] instead of text for admonitions.
Any other value is assumed to be an icontype and sets the value to empty (image-based icons).

|iconsdir
|_directory path_ +
_url_ +
_ex._ ./images/icons
|{y}
|{n}
|Location of non-font-based image icons.
Defaults to the _icons_ folder under `imagesdir` if `imagesdir` is specified and `iconsdir` is not specified.

|icontype
|`jpg` +
(`png`) +
`gif` +
`svg`
|{n}
|{n}
|File type for image icons.
Only relevant when using image-based icons.

|imagesdir
|*_empty_* +
_directory path_ +
_url_
|{y}
|{n}
|Location of image files.
|===

"#
    );

    // The `icons` rows enable icons from the document header, which this crate's
    // default `Secure` safe mode drops (matching Asciidoctor – see #50), so these
    // examples convert under `Server`, a mode that still "allows icons," to
    // reproduce the icon-permitting output Asciidoctor's CLI generates. (The
    // `imagesdir`-only example below sets no `icons`, so it needs no such mode.)
    let icons = |source: &str| convert_with(source, &Options::new().safe_mode(SafeMode::Server));

    // `icons` chooses font or image icons for admonitions instead of the text
    // label: `font` emits a Font Awesome `<i>`, while the empty/`image` value
    // uses an `<img>` from `iconsdir` (default `./images/icons`) with the
    // `icontype` extension (default `png`).
    assert!(icons("= T\n:icons: font\n\n[NOTE]\n====\nhi\n====")
        .contains(r#"<i class="fa icon-note" title="Note">"#));
    assert!(icons("= T\n:icons:\n\n[NOTE]\n====\nhi\n====")
        .contains(r#"<img src="./images/icons/note.png" alt="Note""#));

    // `iconsdir` and `icontype` override the directory and extension of the
    // image icons.
    assert!(
        icons("= T\n:icons:\n:iconsdir: /myicons\n:icontype: svg\n\n[NOTE]\n====\nhi\n====")
            .contains(r#"<img src="/myicons/note.svg""#)
    );

    // `imagesdir` prefixes image targets; when it is set and `iconsdir` is not,
    // the image icons default to the `icons` folder beneath it.
    assert!(convert("= T\n:imagesdir: assets\n\nimage::foo.png[Foo]")
        .contains(r#"<img src="assets/foo.png""#));
    assert!(
        icons("= T\n:icons:\n:imagesdir: assets\n\n[NOTE]\n====\nhi\n====")
            .contains(r#"<img src="assets/icons/note.png" alt="Note""#)
    );
}

// Source highlighting and formatting: `source-highlighter`/`source-language`
// are verified on the `verbatim` module pages; the per-highlighter tuning
// (CodeRay/Pygments/Rouge/prettify/highlight.js) configures the server-side
// highlighters this renderer does not run (html5#223), so those rows have no
// distinct rendered value here.
non_normative!(
    r#"
== Source highlighting and formatting attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|coderay-css
|(`class`) +
`style`
|{n}
|{y}
|Controls whether CodeRay uses CSS classes or inline styles.

|coderay-linenums-mode
|`inline` +
(`table`)
|{n}
|{n}
|Sets how CodeRay inserts line numbers into source listings.

|coderay-unavailable
|_empty_
|{n}
|{y}
|Instructs processor not to load CodeRay.
Also set if processor fails to load CodeRay.

|highlightjsdir
|_directory path_ +
_url_ +
(default CDN URL)
|{n}
|{y}
|Location of the highlight.js source code highlighter library.

|highlightjs-theme
|_highlight.js style name_ +
(`github`)
|{n}
|{y}
|Name of theme used by highlight.js.

|prettifydir
|_directory path_ +
_url_ +
(default CDN URL)
|{n}
|{y}
|Location of non-CDN prettify source code highlighter library.

|prettify-theme
|_prettify style name_ +
(`prettify`)
|{n}
|{y}
|Name of theme used by prettify.

|prewrap
|*_empty_*
|{y}
|{n}
|xref:asciidoctor:html-backend:verbatim-line-wrap.adoc[Wrap wide code listings].

|pygments-css
|(`class`) +
`style`
|{n}
|{y}
|Controls whether Pygments uses CSS classes or inline styles.

|pygments-linenums-mode
|(`table`) +
`inline`
|{n}
|{n}
|Sets how Pygments inserts line numbers into source listings.

|pygments-style
|_Pygments style name_ +
(`default`)
|{n}
|{y}
|Name of style used by Pygments.

|pygments-unavailable
|_empty_
|{n}
|{y}
|Instructs processor not to load Pygments.
Also set if processor fails to load Pygments.

|rouge-css
|(`class`) +
`style`
|{n}
|{y}
|Controls whether Rouge uses CSS classes or inline styles.

|rouge-linenums-mode
|`inline` +
(`table`)
|{n}
|{n}
|Sets how Rouge inserts line numbers into source listings.
_`inline` not yet supported by Asciidoctor._
See https://github.com/asciidoctor/asciidoctor/issues/3641[asciidoctor#3641].

|rouge-style
|_Rouge style name_ +
(`github`)
|{n}
|{y}
|Name of style used by Rouge.

|rouge-unavailable
|_empty_
|{n}
|{y}
|Instructs processor not to load Rouge.
Also set if processor fails to load Rouge.

|source-highlighter
|`coderay` +
`highlight.js` +
`pygments` +
`rouge`
|{n}
|{y}
|xref:verbatim:source-highlighter.adoc[Specifies source code highlighter].
Any other value is permitted, but must be supported by a custom syntax highlighter adapter.

|source-indent
|_integer_
|{n}
|{n}
|Normalize block indentation in source code listings.
//<<normalize-block-indentation>>

|source-language
|_source code language name_
|{n}
|{n}
|xref:verbatim:source-highlighter.adoc[Default language for source code blocks].

|source-linenums-option
|_empty_
|{n}
|{n}
|Turns on line numbers for source code listings.
|===

"#
);

// `copycss` copies the linked stylesheet into the output directory — a file
// side-effect verified in `custom_stylesheet.rs`, not observable in this
// string API — and `css-signature` (a `<body>` id) is not implemented.
non_normative!(
    r#"
== HTML styling attributes

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|copycss
|*_empty_* +
_file path_
|{y}
|{y}
|Copy CSS files to output.
Only relevant when the `linkcss` attribute is set and the output is a standalone HTML file.
//<<applying-a-theme>>

|css-signature
|_valid XML ID_
|{n}
|{y}
|Assign value to `id` attribute of HTML `<body>` element.
*Preferred approach is to assign an ID to document title*.

"#
);

#[test]
fn linkcss_attribute() {
    verifies!(
        r#"
|linkcss
|_empty_
|{n}
|{y}
|Links to stylesheet instead of embedding it.
Can't be unset in SECURE mode.
//<<styling-the-html-with-css>>

"#
    );

    // `linkcss` links the stylesheet from the head instead of embedding it in a
    // `<style>` element.
    let linked = convert_with("= T\n:linkcss:\n\nbody", &Options::new().standalone(true));
    assert!(linked.contains(r#"<link rel="stylesheet" href="./asciidoctor.css">"#));
    assert!(!linked.contains("<style>"));
}

#[test]
fn max_width_attribute() {
    verifies!(
        r#"
|max-width
|CSS length (e.g. 55em, 12cm, etc)
|{n}
|{y}
|Constrains maximum width of document body.
*Not recommended.
Use CSS stylesheet instead.*

"#
    );

    // `max-width` adds an inline `style="max-width: …;"` to each of the
    // standalone layout's container divs — `#header`, `#content`, `#footnotes`,
    // and `#footer` — constraining the document body width (html5#280).
    let constrained = convert_with(
        "= Doc Title\nAuthor Name\n:max-width: 55em\n\nHello.footnote:[A note.]\n",
        &Options::new().standalone(true),
    );
    assert!(constrained.contains(r#"<div id="header" style="max-width: 55em;">"#));
    assert!(constrained.contains(r#"<div id="content" style="max-width: 55em;">"#));
    assert!(constrained.contains(r#"<div id="footnotes" style="max-width: 55em;">"#));
    assert!(constrained.contains(r#"<div id="footer" style="max-width: 55em;">"#));

    // A bare `:max-width:` (set with no value) still counts as present, so the
    // style is emitted with an empty length — matching Asciidoctor's
    // `node.attr? 'max-width'` gate.
    let empty = convert_with(
        "= Doc Title\nAuthor Name\n:max-width:\n\nHello.footnote:[A note.]\n",
        &Options::new().standalone(true),
    );
    assert!(empty.contains(r#"<div id="header" style="max-width: ;">"#));
    assert!(empty.contains(r#"<div id="content" style="max-width: ;">"#));
    assert!(empty.contains(r#"<div id="footnotes" style="max-width: ;">"#));
    assert!(empty.contains(r#"<div id="footer" style="max-width: ;">"#));

    // Without the attribute, the same container divs carry no inline style.
    let plain = convert_with(
        "= Doc Title\nAuthor Name\n\nHello.footnote:[A note.]\n",
        &Options::new().standalone(true),
    );
    assert!(plain.contains(r#"<div id="header">"#));
    assert!(plain.contains(r#"<div id="content">"#));
    assert!(plain.contains(r#"<div id="footnotes">"#));
    assert!(plain.contains(r#"<div id="footer">"#));
}

#[test]
fn stylesheet_location_and_toc_class_attributes() {
    verifies!(
        r#"
|stylesdir
|_directory path_ +
_url_ +
`*.*`
|{y}
|{y}
|Location of CSS stylesheets.
//<<creating-a-theme>>

|stylesheet
|*_empty_* +
_file path_
|{y}
|{y}
|CSS stylesheet file name.
An empty value tells the converter to use the default stylesheet.
//<<applying-a-theme>>

|toc-class
|_valid CSS class name_ +
(`*toc*`) or (`*toc2*`) if toc=left
|{n}
|{y}
|CSS class on the table of contents container.
//<<user-toc>>
|===

"#
    );

    // `stylesdir` (default `.`) and `stylesheet` set the directory and file name
    // of the linked stylesheet.
    let custom = convert_with(
        "= T\n:linkcss:\n:stylesdir: css\n:stylesheet: custom.css\n\nbody",
        &Options::new().standalone(true),
    );
    assert!(custom.contains(r#"<link rel="stylesheet" href="./css/custom.css">"#));

    // `toc-class` sets the CSS class on the table-of-contents container.
    assert!(convert_with(
        "= T\n:toc:\n:toc-class: mytoc\n\n== S\n\ntext",
        &Options::new().standalone(true)
    )
    .contains(r#"<div id="toc" class="mytoc">"#));
}

// Manpage attributes are only relevant to the manpage doctype/backend, which
// this renderer does not implement.
non_normative!(
    r#"
== Manpage attributes

The attribute in this section are only relevant when using the manpage doctype and/or backend.

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|Header Only .>|Notes

|mantitle
|_any_
|Based on content.
|{y}
|Metadata for man page output.
//<<man-pages>>

|manvolnum
|_any_
|Based on content.
|{y}
|Metadata for man page output.
//<<man-pages>>

|manname
|_any_
|Based on content.
|{y}
|Metadata for man page output.
//<<man-pages>>

|manpurpose
|_any_
|Based on content
|{y}
|Metadata for man page output.
//<<man-pages>>

|man-linkstyle
|_link format pattern_ +
(`blue R <>`)
|{n}
|{y}
|Link style in man page output.
//<<man-pages>>

|mansource
|_any_
|{n}
|{y}
|Source (e.g., application and version) the man page describes.
//<<man-pages>>

|manmanual
|_any_
|{n}
|{y}
|Manual name displayed in the man page footer.
//<<man-pages>>
|===

"#
);

// `allow-uri-read` enables reading data from URLs, which neither client does —
// remote fetch is a deliberate non-goal — so its row stays non_normative, but
// the attribute is still settable via the API (see
// `allow_uri_read_is_settable_via_api`).
non_normative!(
    r#"
== Security attributes

Since these attributes deal with security, they can only be set from the API or CLI.

[cols="30m,20,^10,^10,30"]
|===
.>|Name .>|Allowable Values .>|Set By Default .>|API/CLI Only .>|Notes

|allow-uri-read
|_empty_
|{n}
|{y}
|Allows data to be read from URLs.
//<<include-uri>>

"#
);

#[test]
fn security_max_attribute_value_size_limits_resolved_values() {
    verifies!(
        r#"
|max-attribute-value-size
|_integer_ (≥ 0) +
`*4096*`
|If safe mode is SECURE
|{y}
|Limits maximum size (in bytes) of a resolved attribute value.
Default value is only set in SECURE mode.
Since attributes can reference attributes, it's possible to create an output document disproportionately larger than the input document without this limit in place.

"#
    );

    // Set via the API, `max-attribute-value-size` caps the size (in bytes) of a
    // resolved attribute value: a reference to a longer value yields only its
    // first N bytes.
    let limited = Options::new().attribute("max-attribute-value-size", "5");
    assert_eq!(
        para_with("= T\n:long: abcdefghijklmnop\n\n[{long}]", &limited),
        "[abcde]"
    );

    // Without a tighter limit the value resolves in full (the SECURE-mode
    // default of 4096 is far larger than this value).
    assert_eq!(
        para("= T\n:long: abcdefghijklmnop\n\n[{long}]"),
        "[abcdefghijklmnop]"
    );
}

#[test]
fn security_max_include_depth_curtails_nested_includes() {
    verifies!(
        r#"
|max-include-depth
|_integer_ (≥ 0) +
`*64*`
|{y}
|{y}
|Curtail infinite include loops and to limit the opportunity to exploit nested includes to compound the size of the output document.
//<<include-directive>>
|===

"#
    );

    // Set via the API, `max-include-depth` caps how deeply include directives
    // nest. With a chain main -> a -> b and a depth of 1, the second-level
    // include is refused: its content is dropped and a warning is recorded.
    let dir = std::env::temp_dir().join(format!("adoc-maxinc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create include dir");
    std::fs::write(dir.join("a.adoc"), "a-content\n\ninclude::b.adoc[]\n").expect("write a");
    std::fs::write(dir.join("b.adoc"), "b-content\n").expect("write b");
    let main = "= T\n\ninclude::a.adoc[]\n";

    // Includes are only processed below SECURE safe mode; SAFE resolves them
    // while confining reads to the base directory.
    let opts = Options::new()
        .safe_mode(SafeMode::Safe)
        .base_dir(&dir)
        .attribute("max-include-depth", "1");

    let html = convert_with(main, &opts);
    assert!(html.contains("a-content"), "{html}");
    assert!(!html.contains("b-content"), "{html}");

    let doc = load_with(main, &opts);
    assert!(doc.warnings().any(|w| matches!(
        w.warning,
        asciidoc_parser::warnings::WarningType::MaxIncludeDepthExceeded(_)
    )));

    let _ = std::fs::remove_dir_all(&dir);
}

// `allow-uri-read` is a deliberate non-goal (remote fetch), but the attribute
// itself is accepted when set through the API.
#[test]
fn allow_uri_read_is_settable_via_api() {
    let doc = load_with("body", &Options::new().set("allow-uri-read"));
    assert!(doc.is_attribute_set("allow-uri-read"));
}

// The closing `-number` footnote is descriptive prose.
non_normative!(
    r#"
[[note-number]]^[1]^ The `-number` attributes are created and managed automatically by the AsciiDoc processor for numbered blocks.
They are only used if the corresponding `-caption` attribute is set (e.g., `listing-caption`) and the block has a title.
In Asciidoctor, setting the `-number` attributes will influence the next number used for subsequent numbered blocks of that type.
However, you should not rely on this behavior as it is subject to change in future revisions of the language.
// end::table[]
"#
);
