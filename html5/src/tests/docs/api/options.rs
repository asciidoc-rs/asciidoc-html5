use crate::{convert, convert_with, tests::sdd::*, Options, SafeMode};

track_file!("docs/modules/api/pages/options.adoc");

// This crate's "API Options" page. It documents the subset of Asciidoctor's API
// options that `asciidoc_html5` supports, mapped onto the `Options` builder:
// `attribute`/`attribute_default`, `standalone`/`embedded`, `safe_mode`, and
// `base_dir`. Each shown Rust snippet is verified here; the surrounding prose,
// the summary table, and the closing "Options without a counterpart" list are
// non-normative.

non_normative!(
    r#"
= API Options
:navtitle: API Options
:description: The options the asciidoc_html5 conversion API accepts, and how they map to Asciidoctor's API options.

Asciidoctor's Ruby API accepts a large table of options. `asciidoc_html5` is a
focused HTML5 library, so it supports the subset of those options that apply to
converting AsciiDoc to HTML5. You supply them through the `Options` builder and
pass it to a `_with` entry point (`convert_with` or `convert_file_with`).

[NOTE]
====
The prose on this page is non-normative documentation. The API invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

== Supported options

[cols="1,2,2"]
|===
|Asciidoctor option |`Options` method |Summary

|`:attributes`
|`attribute` / `set` / `unset` (and the `_default` soft-set variants)
|Supply document attributes from outside the document.

|`:base_dir`
|`base_dir`
|The directory relative `include::` targets and docinfo files resolve against.

|`:safe`
|`safe_mode`
|The xref:ROOT:safe-modes.adoc[safe mode]; defaults to `secure`.

|`:standalone`
|`standalone` / `embedded`
|Whether to emit a full document or body-only output.
|===

== Set document attributes

`attribute` overrides an attribute, so the API value wins over the document:

"#
);

// `attribute` overrides an attribute so the API value wins over the document's.
#[test]
fn attribute_overrides_the_document() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_html5::{convert_with, Options};

let opts = Options::new().attribute("myattr", "from-api");
let html = convert_with("= Doc\n:myattr: from-doc\n\nval={myattr}", &opts);
assert!(html.contains("val=from-api"));
----

"#
    );

    let opts = Options::new().attribute("myattr", "from-api");
    let html = convert_with("= Doc\n:myattr: from-doc\n\nval={myattr}", &opts);
    assert!(html.contains("val=from-api"));
}

non_normative!(
    r#"
Use `attribute_default` for a soft value that a document assignment may override:

"#
);

// `attribute_default` is a soft value the document may override.
#[test]
fn attribute_default_yields_to_the_document() {
    verifies!(
        r#"
[,rust]
----
let opts = Options::new().attribute_default("myattr", "from-api");
let html = convert_with("= Doc\n:myattr: from-doc\n\nval={myattr}", &opts);
assert!(html.contains("val=from-doc"));
----

"#
    );

    let opts = Options::new().attribute_default("myattr", "from-api");
    let html = convert_with("= Doc\n:myattr: from-doc\n\nval={myattr}", &opts);
    assert!(html.contains("val=from-doc"));
}

non_normative!(
    r#"
The `set` / `unset` methods (and `set_default` / `unset_default`) turn an
attribute on or off without giving it a value.

== Choose standalone or embedded output

`standalone(true)` emits the full `<!DOCTYPE html>` document, whereas a string
conversion is body-only by default:

"#
);

// `standalone(true)` emits the full document; a string conversion is embedded
// (body-only) by default.
#[test]
fn standalone_and_the_default_string_output() {
    verifies!(
        r#"
[,rust]
----
let opts = Options::new().standalone(true);
let html = convert_with("= Doc\n\nBody.", &opts);
assert!(html.starts_with("<!DOCTYPE html>"));

// A plain string conversion is embedded (body-only) by default.
let embedded = asciidoc_html5::convert("= Doc\n\nBody.");
assert!(!embedded.starts_with("<!DOCTYPE html>"));
----

"#
    );

    let opts = Options::new().standalone(true);
    let html = convert_with("= Doc\n\nBody.", &opts);
    assert!(html.starts_with("<!DOCTYPE html>"));

    // A plain string conversion is embedded (body-only) by default.
    let embedded = convert("= Doc\n\nBody.");
    assert!(!embedded.starts_with("<!DOCTYPE html>"));
}

non_normative!(
    r#"
`embedded(true)` is the inverse spelling. Converting a file is standalone by
default.

== Set the base directory

`base_dir` anchors relative `include::` targets and docinfo lookups; the safe
mode governs whether those reads are allowed:

"#
);

// `base_dir` anchors relative include targets. The shown snippet uses a
// placeholder path; the test proves the anchoring against a real directory.
#[test]
fn base_dir_anchors_relative_includes() {
    verifies!(
        r#"
[,rust]
----
use asciidoc_html5::{convert_with, Options, SafeMode};

let opts = Options::new()
    .safe_mode(SafeMode::Safe)
    .base_dir("/path/to/docs");
let html = convert_with("= Doc\n\ninclude::part.adoc[]\n", &opts);
----

"#
    );

    // The documented builder call compiles and constructs the options.
    let _shown = Options::new()
        .safe_mode(SafeMode::Safe)
        .base_dir("/path/to/docs");

    // Point `base_dir` at a real directory to show it anchors the include.
    let dir = std::env::temp_dir().join(format!(
        "asciidoc-html5-docs-options-basedir-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create base dir");
    std::fs::write(dir.join("part.adoc"), "Included body text.\n").expect("write include");

    let opts = Options::new()
        .safe_mode(SafeMode::Safe)
        .base_dir(dir.clone());
    let html = convert_with("= Doc\n\ninclude::part.adoc[]\n", &opts);
    assert!(html.contains("Included body text."), "{html}");

    let _ = std::fs::remove_dir_all(&dir);
}

non_normative!(
    r#"
See xref:ROOT:safe-modes.adoc[Safe Modes] for how the safe mode restricts those
reads.

== Options without a counterpart

Many Asciidoctor API options do not apply to this library:

* *Backend and doctype.* `asciidoc_html5` always renders the `html5` backend and
  the `article` doctype, so `:backend` and `:doctype` are fixed.
* *Source mapping.* `:sourcemap` has no toggle because source locations are
  always tracked; see xref:sourcemap.adoc[Source Locations].
* *Ruby and template machinery.* `:converter`, `:eruby`, `:extensions`,
  `:extension_registry`, `:logger`, the `:template_*` options, and `:timings`
  are specific to Asciidoctor's Ruby runtime and template engines.
* *Output writing.* `:to_file`, `:to_dir`, and `:mkdirs` do not apply because
  the library returns a `String`; the xref:cli:options.adoc[`adoc` CLI] handles
  output paths.
* *Not yet implemented.* `:catalog_assets`, `:parse_header_only`, and `:parse`
  (deferred parsing) are tracked for future work.
"#
);
