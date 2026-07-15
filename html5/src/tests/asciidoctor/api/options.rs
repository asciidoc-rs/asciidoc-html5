use crate::{convert, convert_file, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("ref/asciidoctor/docs/modules/api/pages/options.adoc");

// Asciidoctor's "API Options" page: the reference table of options accepted by
// the Ruby `Asciidoctor` API. Most rows describe Ruby-only machinery with no
// analog in a Rust text-to-text library and are tracked as non-normative; four
// rows map directly onto this crate's `Options` builder and are verified:
//
// * `:attributes` -> `Options::attribute` / `attribute_default` / `set` /
//   `unset` / `set_default` / `unset_default` (override vs. soft-set
//   precedence).
// * `:base_dir` -> `Options::base_dir` (anchors relative includes / docinfo).
// * `:safe` -> `Options::safe_mode`.
// * `:standalone` -> `Options::standalone` / `embedded`.
//
// A few rows have a fixed counterpart, so only their *default value* is
// verified:
//
// * `:backend` is pinned to `html5` and `:doctype` to `article` -- the only
//   backend and doctype this crate models (the workspace README's compatibility
//   target). Selecting another converter/doctype is out of scope, so the
//   "Allowed values" cells are non-normative.
// * `:sourcemap` is effectively always on here: `asciidoc-parser` tracks every
//   block's source location unconditionally, so there is no toggle. That is
//   covered on the dedicated sourcemap page (see `api::sourcemap`), so the row
//   is non-normative here.
//
// The remaining rows are tracked as non-normative, in three groups:
//
// * Ruby- and template-engine-specific options (`:converter`, `:eruby`,
//   `:extensions`, `:extension_registry`, `:logger`, `:template_*`, `:timings`)
//   have no place in this library.
// * Parser behaviors this crate does not yet expose are tracked by GitHub
//   issues, linked at their rows: `:catalog_assets` (#95), `:parse_header_only`
//   (#96), and `:parse` / deferred parsing (#97).
// * Output-writing options (`:to_file`, `:to_dir`, `:mkdirs`) do not apply to
//   the library, which converts text to text (`convert_file` returns a
//   `String`). The `adoc` CLI already covers output naming via `-o` and
//   `-D`/`--destination-dir`, and `AssetWriter` handles writing non-primary
//   output assets alongside the converted document.
//
// The page is purely about the API, so -- like the other `api::` pages -- it is
// tracked only from this crate.

non_normative!(
    r#"
= API Options

[cols="~,~,15%,15%"]
|===
|Name |Description |Default value |Allowed values

"#
);

// `:attributes` maps to this crate's attribute directives. An override
// (`attribute`) wins over a same-named document assignment; a soft-set default
// (`attribute_default`) yields to it -- the "override ... unless soft set"
// rule.
#[test]
fn attributes_override_the_document_unless_soft_set() {
    verifies!(
        r#"
|`:attributes`
|Sets document attributes, which override equivalently-named attributes defined in the document unless soft set.
"#
    );

    // An override supplied through the API wins over the document header.
    let overridden = convert_with(
        "= Doc\n:myattr: from-doc\n\nval={myattr}",
        &Options::new().attribute("myattr", "from-api"),
    );
    assert!(overridden.contains("val=from-api"), "{overridden}");
    assert!(!overridden.contains("from-doc"), "{overridden}");

    // A soft-set default yields to the document header ("unless soft set").
    let soft = convert_with(
        "= Doc\n:myattr: from-doc\n\nval={myattr}",
        &Options::new().attribute_default("myattr", "from-api"),
    );
    assert!(soft.contains("val=from-doc"), "{soft}");
}

// The remainder of the `:attributes` row documents the Ruby Hash/Array/String
// argument forms and their `nil`/`false` unset semantics -- Ruby-call-shape
// details with no counterpart in this crate's typed builder, whose
// override/soft-set behavior is verified above.
non_normative!(
    r#"
No substitutions are applied to the value of these attributes.
In the Hash format, the name *must* be a String, not a Symbol (e.g., `name: 'value'` is *invalid*).
In this format, a `nil` value hard unsets the attribute and a `false` value soft unsets the attribute.
In the String format, entries are separated by a space.
To include a space in the value, place a backslash in front of it.
|_not set_
a|xref:asciidoc:attributes:document-attributes.adoc[Document attributes] in the following formats:

*Hash* +
`{ 'name' \=> 'value' }`

*Array* +
`[ 'name=value' ]`

*String* +
`'name=value'`

"#
);

non_normative!(
    r#"
|`:backend`
|Selects converter to use.
"#
);

// html5 is the only backend this crate produces, so `{backend}` is pinned to
// `html5` -- the documented default. Selecting another converter (`docbook5`,
// `manpage`, ...) is out of scope; those "Allowed values" are non-normative.
#[test]
fn backend_defaults_to_and_is_pinned_to_html5() {
    verifies!(
        r#"
|`html5`
"#
    );

    let html = convert("= Doc\n\nbackend={backend}");
    assert!(html.contains("backend=html5"), "{html}");
}

non_normative!(
    r#"
|`html5`, `docbook5`, `manpage`, or a backend mapped to an available converter

"#
);

// `:base_dir` maps to `Options::base_dir`: the anchor that filesystem-relative
// resources (here, an `include::` target) resolve against. When left unset, the
// directory of the source file stands in, matching Asciidoctor's default.
#[test]
fn base_dir_sets_the_directory_relative_resources_resolve_against() {
    verifies!(
        r#"
|`:base_dir`
|Sets the base (aka working) directory containing the document and resources.
|Directory of the source file, or the working directory if the source is read from a stream.
|file path

"#
    );

    let dir = std::env::temp_dir().join(format!("adoc-api-options-basedir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create base dir");
    std::fs::write(dir.join("part.adoc"), "Included body text.\n").expect("write include");

    // An explicit base directory anchors the relative include target. (`Safe`
    // permits includes; `Secure`, the default, would turn them into links.)
    let with_base = convert_with(
        "= Doc\n\ninclude::part.adoc[]\n",
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.clone()),
    );
    assert!(with_base.contains("Included body text."), "{with_base}");

    // Default: with no base directory, the source file's own directory stands
    // in -- naming a file in `dir` resolves the same relative include.
    let from_file_dir = convert_with(
        "= Doc\n\ninclude::part.adoc[]\n",
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .input_file(dir.join("main.adoc")),
    );
    assert!(
        from_file_dir.contains("Included body text."),
        "{from_file_dir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// `:catalog_assets` -- capturing images and links in the reference table -- has
// no counterpart: `asciidoc-parser` does not expose an asset-cataloging toggle.
// Tracked in https://github.com/asciidoc-rs/asciidoc-html5/issues/95.
non_normative!(
    r#"
|`:catalog_assets`
|If `true`, the parser captures images and links in the reference table.
(Normally only IDs, footnotes and indexterms are included).
The reference table is available via the `references` property on the `document` AST object.
//NOTE: This is still a primitive and experimental feature.
//It is intended for early adopters to address special use cases.
_(Experimental)._
|`false`
|_Boolean_

"#
);

// `:converter` selects a user-supplied Ruby converter class or instance. This
// crate has a single built-in HTML5 converter and no pluggable-converter API,
// so the option is out of scope.
non_normative!(
    r#"
|`:converter`
|Specifies a user-supplied converter class or instance, used in place of the converter that is automatically resolved from the `backend` value.
|_not set_
|`Asciidoctor::Converter` class or instance

"#
);

non_normative!(
    r#"
|`:doctype`
|Sets the document type.
"#
);

// `article` is the only doctype this renderer models, so `{doctype}` is pinned
// to `article` -- the documented default. Other doctypes (`book`, `manpage`,
// `inline`) are out of scope, so those "Allowed values" are non-normative.
#[test]
fn doctype_defaults_to_and_is_pinned_to_article() {
    verifies!(
        r#"
|`article`
"#
    );

    let html = convert("= Doc\n\ndoctype={doctype}");
    assert!(html.contains("doctype=article"), "{html}");
}

non_normative!(
    r#"
|`article`, `book`, `manpage`, `inline`

"#
);

// The `:eruby`, `:extensions`, `:extension_registry`, and `:logger` options are
// all Ruby-runtime machinery -- an ERB engine selector, a Ruby extensions
// block, a Ruby extension-registry instance, and the global Ruby
// `LoggerManager`. None has an analog in this library, so all four rows are
// non-normative.
non_normative!(
    r#"
|`:eruby`
|Specifies the eRuby implementation to use for executing the converter templates written in ERB.
|`erb`
|`erb`, `erubis`

|`:extensions`
|A Ruby block that registers (and possibly defines) xref:extensions:register.adoc[Asciidoctor extensions] for this instance of the processor.
|_not set_
|A Ruby block that conforms to the Asciidoctor extensions API (the same code that would be passed to the `Extensions.register` method).

|`:extension_registry`
|Overrides the extensions registry instance.
Instead of providing a Ruby block containing extensions to register, this option lets you replace the extension registry itself, giving you complete control over how extensions are registered for this processor.
|_not set_
|`Extensions::Registry` instance

|`:logger`
|Shorthand to assign a new value to the global `LoggerManager.logger`.
This is persistent change, so you either have to reset the value afterwards or pass the option each time you call the API.
If value is falsy, it assigns a null logger, effectively turning off logging.
|_not set_
|`Logger` instance

"#
);

// `:parse_header_only` -- stopping the parser after the header -- has no
// counterpart: `load`/`load_file` always parse the whole document. Tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/96.
non_normative!(
    r#"
|`:parse_header_only`
|If `true`, the parser stops after reading the header.
|`false`
|_Boolean_

"#
);

// `:standalone` maps to `Options::standalone` / `embedded`. `standalone(true)`
// emits the full document shell; the unset default varies by entry point --
// embedded for a string, standalone for a file -- matching Asciidoctor.
#[test]
fn standalone_controls_the_document_shell_and_defaults_by_entry_point() {
    verifies!(
        r#"
|`:standalone`
|If `true`, generates a standalone output document (which includes the shell around the body content, such as the header and footer).
When converting to a file, the default value is `true`.
Otherwise, the default value is `false`.
"#
    );

    // `standalone(true)` produces the full `<!DOCTYPE html>` shell.
    let full = convert_with("= Doc\n\nBody.", &Options::new().standalone(true));
    assert!(full.starts_with("<!DOCTYPE html>"), "{full}");

    // Default: a string conversion is embedded (body-only) output.
    let embedded = convert("= Doc\n\nBody.");
    assert!(!embedded.starts_with("<!DOCTYPE html>"), "{embedded}");
    assert!(embedded.contains("<p>Body.</p>"), "{embedded}");

    // Default: converting to a file is standalone.
    let path = std::env::temp_dir().join(format!(
        "adoc-api-options-standalone-{}.adoc",
        std::process::id()
    ));
    std::fs::write(&path, "= Doc\n\nBody.").expect("write input");
    let from_file = convert_file(&path).expect("convert_file");
    let _ = std::fs::remove_file(&path);
    assert!(from_file.starts_with("<!DOCTYPE html>"), "{from_file}");
}

// The `:header_footer` deprecated alias, the note on the option's default being
// opposite the CLI's, and the `_Varies_` default are all descriptive framing
// around the behavior verified above.
non_normative!(
    r#"
The deprecated alias for this option is `:header_footer`.
The default value for this option is opposite of the default value for the CLI.
|_Varies_
|_Boolean_

"#
);

// `:mkdirs` -- creating output directories -- is an output-writing concern that
// does not apply to the library (`convert_file` returns a `String`); it is a
// CLI matter. See the `AssetWriter` trait for how non-primary output assets are
// written, and the `adoc` CLI for output-file handling.
non_normative!(
    r#"
|`:mkdirs`
|If `true`, the processor creates the necessary output directories if they don't yet exist.
|`false`
|_Boolean_

"#
);

// `:parse` toggles eager vs. deferred parsing. This crate's `load`/`load_file`
// are eager only; `asciidoc-parser` exposes `parse_deferred`, so a deferred
// variant is feasible but unimplemented. Tracked in
// https://github.com/asciidoc-rs/asciidoc-html5/issues/97.
non_normative!(
    r#"
|`:parse`
|If `true`, the source is parsed eagerly (i.e., as soon as the source is passed to the `load` or `load_file` API).
If `false`, parsing is deferred until the `parse` method is explicitly invoked.
|`true`
|_Boolean_

"#
);

// `:safe` maps to `Options::safe_mode`. The default is `Secure` (matching
// Asciidoctor's API), and all four modes are selectable.
#[test]
fn safe_sets_the_safe_mode_defaulting_to_secure() {
    verifies!(
        r#"
|`:safe`
|Sets the xref:ROOT:safe-modes.adoc[safe mode].
|`:secure`
|`:unsafe`, `:safe`, `:server`, `:secure`

"#
    );

    // The default safe mode is Secure.
    let default = convert("= Doc\n\nsafe={safe-mode-name}");
    assert!(default.contains("safe=secure"), "{default}");

    // Each of the four modes is selectable and reported via `safe-mode-name`.
    for (mode, name) in [
        (SafeMode::Unsafe, "unsafe"),
        (SafeMode::Safe, "safe"),
        (SafeMode::Server, "server"),
        (SafeMode::Secure, "secure"),
    ] {
        let html = convert_with(
            "= Doc\n\nsafe={safe-mode-name}",
            &Options::new().safe_mode(mode),
        );
        assert!(html.contains(&format!("safe={name}")), "{mode:?}: {html}");
    }
}

// `:sourcemap` toggles source-location tracking. This crate tracks every
// block's location unconditionally (via `asciidoc-parser`), so there is no
// toggle; the capability is covered on the dedicated sourcemap page.
non_normative!(
    r#"
|`:sourcemap`
|Tracks the file and line number for each parsed block.
Useful for tooling applications where the association between the converted output and the source file is important.
|`false`
|_Boolean_

"#
);

// The `:template_*` options configure Tilt-compatible custom converter
// templates (a cache toggle, template directories, the template engine, and
// low-level engine options), and `:timings` captures internal timing data. All
// are Ruby/template-engine machinery with no counterpart here, so every row --
// including the commented-out deprecated `:template_dir` -- is non-normative.
non_normative!(
    r#"
|`:template_cache`
|Enables the built-in cache used by the template converter when reading the source of template files.
Only relevant if `:template_dirs` is specified.
|`true`
|_Boolean_

//|`:template_dir`
//|Specifies a directory of Tilt-compatible templates to be used instead of the default built-in templates.
//*Deprecated. Use `:template_dirs` instead.*
//|_not set_
//|file path

|`:template_dirs`
|Array of directories containing Tilt-compatible converter templates to be used instead of the default built-in templates.
|_not set_
|Array of file paths

|`:template_engine`
|Template engine to use for the custom converter templates.
The gem with the same name as the engine will be loaded automatically.
This name is also used to build the full path to the custom converter templates.
|_auto_ +
(Set based on the file extension of the custom converter templates found).
|Template engine name (e.g., `slim`, `haml`, `erb`, etc.)

|`:template_engine_options`
|Low-level options passed directly to the template engine.
//(You can see an example in the Bespoke.js converter at https://github.com/asciidoctor/asciidoctor-bespoke/blob/v1.0.0.alpha.1/lib/asciidoctor-bespoke/converter.rb#L24-L28).
|_not set_
|Nested Hash of options with the template engine name as the top-level key and the option name as the second-level key.

|`:timings`
|Capture time taken to read, parse, and convert document.
*Internal use only.*
|_not set_
|`Asciidoctor::Timings` instance

"#
);

// `:to_file` and `:to_dir` name where the converted output is written. The
// library converts text to text (`convert_file` returns a `String`), so these
// do not apply to the API; the `adoc` CLI covers them via `-o` and
// `-D`/`--destination-dir`, and `AssetWriter` writes non-primary output assets.
non_normative!(
    r#"
|`:to_file`
|Name of the output file to write, or `true` to use the default output file (`docname` + `outfilesuffix`).
|_not set_
|`true`, file path

|`:to_dir`
|Destination directory for output file(s), relative to `base_dir`.
|Directory containing source file, or working directory if source is read from a stream.
|File path
|===
"#
);
