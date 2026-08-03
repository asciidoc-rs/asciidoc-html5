//! Coverage of the AsciiDoc language description's *Validate Cross References*
//! page.
//!
//! An AsciiDoc processor provides limited validation of internal cross
//! references. Asciidoctor gates this behind a verbose/pedantic flag; this
//! crate always collects the condition, surfacing an unresolved same-document
//! reference as a `WarningType::PossibleInvalidReference` on the loaded
//! [`Document`]. The Ruby CLI/API mechanisms for enabling pedantic mode and the
//! exact logger message format are non-normative.

use asciidoc_parser::warnings::WarningType;

use crate::{load, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/xref-validate.adoc");

// The intro and the Ruby-specific ways to enable validation (the `-v` CLI flag,
// `$VERBOSE`, the logger level). This crate always collects the warning rather
// than gating it on a verbose mode, so these mechanisms do not apply.
non_normative!(
    r#"
= Validate Cross References

An AsciiDoc processor is only required to provide limited support for validating internal cross references.
Validation occurs when a cross reference is first visited.
Since there are still some references aren't stored in the parse tree (such as an anchor in the middle of a paragraph), which can lead to false positives, these validations are hidden behind a flag.

When using Asciidoctor, you can enable validation of cross references in several ways:

* when using the CLI, passing the `-v` CLI option
* when using the API, setting the global variable `$VERBOSE` to the value `true`
* when using the API, setting the level on the global logger to INFO (i.e., `Asciidoctor::LoggerManager.logger.level = :info`)

"#
);

// A reference to an undefined anchor is reported as a
// possible-invalid-reference warning on the loaded document.
#[test]
fn reports_an_invalid_reference() {
    verifies!(
        r#"
All of these adjustments put the processor into pedantic mode.
In this mode, the parser will immediately validate cross references, issuing a warning message if the reference is not valid.
If you set the global variable `$VERBOSE` to `true`, it will also enable warnings in Ruby, which may not be what you want.

Consider the following example:

----
See <<foobar>>.

[#foobaz]
== Foobaz
----

"#
    );

    // The reference `<<foobar>>` does not resolve to any anchor in the document
    // (the defined anchor is `foobaz`), so loading it records a
    // `PossibleInvalidReference` warning for `foobar`. This crate always collects
    // the warning; Asciidoctor gates it on verbose mode.
    let doc = load("See <<foobar>>.\n\n[#foobaz]\n== Foobaz\n");
    assert!(doc
        .warnings()
        .any(|w| w.warning == WarningType::PossibleInvalidReference("foobar".to_string())));
}

// Asciidoctor's exact logger message format and the same-document scope of
// validation are descriptive.
non_normative!(
    r#"
If you run Asciidoctor in verbose/pedantic mode on this document (`-v`), it will send the following warning message to the logger.

....
asciidoctor: WARNING: invalid reference: foobar
....

An AsciiDoc processor is only required to validate references within the same document (after any includes are resolved).
"#
);
