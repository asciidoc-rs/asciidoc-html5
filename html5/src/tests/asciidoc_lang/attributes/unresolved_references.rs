//! Coverage of the AsciiDoc language description's *Handle Unresolved
//! References* page.
//!
//! The `attribute-missing` attribute controls what happens when a reference
//! cannot be resolved, and this renderer honors all four settings: `skip`
//! leaves the reference in place, `drop` removes the reference but keeps the
//! line, `drop-line` discards the whole line, and `warn` leaves the line alone
//! but emits a warning. Each is verified through `convert` (and, for `warn`,
//! `load().warnings()`). The "Forcing failure" section is likewise verified:
//! its CLI `--failure-level=WARN` decides *when* to exit, but the library half
//! it builds on — the whole document still converts while a `Warning`-severity
//! diagnostic is surfaced for the caller to consult — is exercised through
//! `convert_with`/`load_with` with an `Options`-supplied `attribute-missing`.
//!
//! The "Undefined attribute" section describes the `{set:name!}` inline
//! undefine expression, which this renderer (like `asciidoc-parser`) does not
//! implement — the expression is passed through as literal text — so that whole
//! section, along with the not-strictly-honored include/ifeval cases, is
//! tracked as non-normative.

use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/unresolved-references.adoc");

/// Returns the rendered content of the first paragraph in `output`.
fn first_paragraph(output: &str) -> &str {
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner)
        .unwrap_or_default()
}

non_normative!(
    r#"
= Handle Unresolved References

When you reference a missing attribute (e.g., `+{does-not-exist}+`), the AsciiDoc processor will leave the attribute reference behind.
If you undefine an attribute on the same line as other text (e.g., `+{set:attribute-no-more!}+`), the processor will drop the whole line.
You can tailor these behaviors using the `attribute-missing` and `attribute-undefined` attributes.
You'll want to think about how you want the processor to handle these situations and configure the processor accordingly.

"#
);

mod missing {
    use asciidoc_parser::warnings::{WarningSeverity, WarningType};

    use super::*;
    use crate::{convert, convert_with, load, load_with, Options};

    non_normative!(
        r#"
[#missing]
== Missing attribute

The `attribute-missing` attribute controls how missing references are handled.
By default, missing references are left behind so the integrity of the document is preserved and it's easy for the author to track down.

This attribute has four possible values:

`skip`:: leave the reference in place (default setting)
`drop`:: drop the reference, but not the line
`drop-line`:: drop the line on which the reference occurs (matches behavior of AsciiDoc.py)
`warn`:: print a warning about the missing attribute

The setting you might find of most interest is `warn`, which gives you a warning whenever the processor encounters an attribute reference that cannot be resolved, but otherwise leaves the line alone.

Consider the following line:

"#
    );

    #[test]
    fn missing_attribute_results() {
        verifies!(
            r#"
[source]
Hello, {name}!

Here's how the line is handled in each case, assuming the `name` attribute is not defined:

[%autowidth]
|===
|`attribute-missing` value |Result

|`skip` |Hello, \{name}!

|`drop` |Hello, !

|`drop-line` |{empty}

|`warn` |`asciidoctor: WARNING: skipping reference to missing attribute: XYZ`
|===

"#
        );

        // `skip` (the default) leaves the unresolved reference in place.
        assert_eq!(
            first_paragraph(&convert(":attribute-missing: skip\n\nHello, {name}!")),
            "Hello, {name}!",
        );

        // `drop` removes the reference but keeps the rest of the line.
        assert_eq!(
            first_paragraph(&convert(":attribute-missing: drop\n\nHello, {name}!")),
            "Hello, !",
        );

        // `drop-line` discards the whole line, leaving an empty paragraph.
        assert_eq!(
            first_paragraph(&convert(":attribute-missing: drop-line\n\nHello, {name}!")),
            "",
        );

        // `warn` leaves the line alone but emits a warning about the missing
        // attribute.
        let doc = load(":attribute-missing: warn\n\nHello, {name}!");
        assert_eq!(
            first_paragraph(&convert(":attribute-missing: warn\n\nHello, {name}!")),
            "Hello, {name}!",
        );
        assert!(
            doc.warnings().any(|w| w.warning
                == WarningType::SkippingReferenceToMissingAttribute("name".to_string())),
            "the `warn` setting should emit a missing-attribute warning",
        );
    }

    non_normative!(
        r#"
.History
NOTE: AsciiDoc.py always drops the line that contains a reference to a missing attribute (effectively `attribute-missing=drop-line`).
This "`feature`" was a side effect of how the processor was implemented and not designed with the writer in mind.
The behavior is frustrating for the writer because it's hard to detect where its occurring and can result in loss of important content.
That's why Asciidoctor uses a different default behavior and, further, allows the behavior to be customized.

There are a few cases where the `attribute-missing` attribute is not strictly honored.
One of those cases is the include directive.
If a missing attribute is found in the target of an include directive, the processor will issue a warning about the missing attribute and leave behind the same warning message in the converted document.

Another case is the `ifeval` directive.
A missing attribute reference can safely be used in the clause of the `ifeval` directive without any side effects (i.e., `drop`) since the purpose of that statement is to determine whether an attribute resolves to a value.

"#
    );

    #[test]
    fn forcing_failure() {
        verifies!(
            r#"
=== Forcing failure

If you want the processor to fail when the document contains a missing attribute, set the `attribute-missing` attribute to `warn` and pass the `--failure-level=WARN` option to the CLI.

 $ asciidoctor -a attribute-missing=warn --failure-level=WARN doc.adoc

The processor will convert the entire document, but the application will complete with a non-zero exit status.

When using the API, you can consult the logger for the max severity of all messages reported or look for specific messages in the stack.
It's up to the application code to decide how and when to terminate the application.

"#
        );

        // The library equivalent of the CLI's `-a attribute-missing=warn`: the
        // `--failure-level=WARN` half is the CLI deciding *when to exit*, which
        // this library leaves to the caller. What the library provides is what
        // the caller consults — the whole document still converts, and a
        // `Warning`-severity diagnostic is reported for the missing reference.
        let src = "Hello, {name}!\n\nThe rest of the document is converted regardless.";
        let options = Options::new().attribute("attribute-missing", "warn");

        // The processor converts the entire document.
        let output = convert_with(src, &options);
        assert!(
            output.contains("The rest of the document is converted regardless."),
            "the entire document should still be converted",
        );

        // The application can consult the reported warnings for the max
        // severity; the missing reference is surfaced at `Warning` severity —
        // the level the CLI's `--failure-level=WARN` would act on.
        let doc = load_with(src, &options);
        assert!(
            doc.warnings()
                .any(|w| w.warning.severity() == WarningSeverity::Warning
                    && w.warning
                        == WarningType::SkippingReferenceToMissingAttribute("name".to_string())),
            "a Warning-severity missing-attribute diagnostic should be reported",
        );
    }
}

mod undefined {
    use super::*;

    // The `{set:name!}` inline undefine expression is not implemented by this
    // renderer (nor by `asciidoc-parser`); it is passed through as literal text,
    // so the `attribute-undefined` setting has no observable effect and this
    // whole section is tracked as non-normative.
    non_normative!(
        r#"
[#undefined]
== Undefined attribute

The `attribute-undefined` attribute controls how an expression that undefines an attribute (e.g., `+{set:name!}+`) are handled.
By default, the line containing the expression is dropped since the expression is intended to be a statement, not a content reference.

This attribute has two possible values:

`drop`:: substitute the expression with an empty string after processing it
`drop-line`:: drop the line that contains this expression (default setting; matches behavior of AsciiDoc.py)

The option `skip` doesn't make sense here since the statement is not intended to produce content.

Consider the following declaration:

[source]
----
{set:name!}
----

Depending on whether `attribute-undefined` is `drop` or `drop-line`, either the statement or the line that contains it will be discarded.
It's reasonable to stick with the compliant behavior, drop-line, in this case.

TIP: We recommend putting any statement that undefines an attribute on a line by itself.
"#
    );
}
