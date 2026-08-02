//! Coverage of the AsciiDoc language description's `directives` module pages —
//! the preprocessor directives: the conditionals (`ifdef`, `ifndef`, `ifeval`)
//! and the `include` directive with its options.
//!
//! These are *preprocessor* directives: they add or remove lines before the
//! document structure is parsed. This renderer delegates that preprocessing to
//! `asciidoc-parser`, so each page's behavior is verified end to end through
//! `convert` — a conditional's content appears (or doesn't) in the rendered
//! HTML, and an include's target content is merged in and rendered. The prose
//! that motivates each rule, and the illustrative syntax listings the pages
//! display verbatim (many of which use an escaped `\include::`/`\ifdef::` or an
//! unresolvable Antora `example$` reference), are tracked as non-normative.

use std::path::PathBuf;

use asciidoc_parser::warnings::WarningType;

use crate::{convert_with, load_with, Options, SafeMode};

mod conditionals;
mod ifdef_ifndef;
mod ifeval;
mod include;
mod include_lines;
mod include_list_item_content;
mod include_multiple_times_in_same_document;
mod include_tagged_regions;
mod include_uri;
mod include_with_indent;
mod include_with_leveloffset;

/// The base directory local `include::` targets resolve against: the directives
/// module's own fixtures tree, so a relative `<name>` target matches a file
/// this suite controls (the counterpart to Asciidoctor's `base_dir`).
fn fixtures_base_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/asciidoc_lang/directives/fixtures")
}

/// Converts `src` (embedded) with `attrs` seeded as document attributes — the
/// counterpart to setting attributes from the API. Used to exercise the
/// conditional directives, whose condition rests on the presence or value of a
/// document attribute.
fn convert_with_attrs(src: &str, attrs: &[(&str, &str)]) -> String {
    let mut options = Options::new();
    for (name, value) in attrs {
        options = options.attribute(*name, *value);
    }

    convert_with(src, &options)
}

/// Converts `src` (embedded) under the `Safe` safe mode with `base_dir`
/// anchored at the directives fixtures tree, so relative `include::` targets
/// resolve and are read (only `Secure` turns an include into a link).
fn convert_including(src: &str) -> String {
    convert_with(
        src,
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(fixtures_base_dir()),
    )
}

/// Converts `src` (embedded) under the `Secure` safe mode with `base_dir`
/// anchored at the directives fixtures tree. Under `Secure`, an `include::`
/// directive is disabled and converted to a link instead of being read.
fn convert_secure(src: &str) -> String {
    convert_with(
        src,
        &Options::new()
            .safe_mode(SafeMode::Secure)
            .base_dir(fixtures_base_dir()),
    )
}

/// The typed warnings raised while loading `src` under the `Safe` safe mode
/// with the fixtures `base_dir`, as `(WarningType, line)` pairs in source
/// order — used to assert the warning an unresolved include emits.
fn include_warnings(src: &str) -> Vec<(WarningType, usize)> {
    let doc = load_with(
        src,
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(fixtures_base_dir()),
    );

    doc.warnings()
        .map(|w| (w.warning.clone(), w.source.line()))
        .collect()
}
