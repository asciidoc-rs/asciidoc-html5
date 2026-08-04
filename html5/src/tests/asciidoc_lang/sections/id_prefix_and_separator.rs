//! Coverage of the AsciiDoc language description's *Change the ID Prefix and
//! Separator* page.
//!
//! The page's verifiable rules are that `idprefix` sets (or, when empty,
//! removes) the leading text of an auto-generated section ID, and that
//! `idseparator` sets (or, when empty, removes) the word separator. Each is
//! verified through `convert` by driving the attribute entry the page shows.
//! The introduction, the WARNING about an empty `idprefix`, and the NOTE about
//! GitHub's settings are advisory prose and tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/sections/pages/id-prefix-and-separator.adoc");

// Title and introductory description of the two attributes: descriptive framing
// with no distinct HTML rule to drive.
non_normative!(
    r#"
= Change the ID Prefix and Separator

When an AsciiDoc processor xref:auto-ids.adoc[auto-generates section IDs], it begins the value with an underscore and uses a hyphen between each word.
These characters can be customized with the `idprefix` and `idseparator` attributes.

"#
);

// Section heading (with its `[#prefix]` anchor) plus the advisory prose about
// why the default underscore prefix can cause xref-formatting problems and the
// `\{empty}` workaround: guidance with no distinct HTML rule of its own.
non_normative!(
    r#"
[#prefix]
== Change the ID prefix

By default, the AsciiDoc processor begins an auto-generated section ID with an underscore (`+_+`).
This default can cause problems when referencing the ID in an xref (either within the same file or a deep link to another file).
The leading underscore may get paired with an underscore somewhere else in the paragraph, thus resulting in unexpected text formatting.
One workaround is to disrupt the match by prefixing the ID with `\{empty}` (e.g., `\{empty}_section_title`) or using an attribute to refer to the target.
Instead, we strongly encourage you to customize the ID prefix.

"#
);

// Setting `idprefix` to a new value changes the leading text of a generated
// section ID, so `:idprefix: id_` turns `_two_words` into `id_two_words`.
#[test]
fn idprefix_changes_the_prefix() {
    verifies!(
        r#"
You can change this prefix by setting the `idprefix` attribute and assigning it a new value.
The value of `idprefix` must begin with a valid ID start character and can have any number of additional valid ID characters.

[source]
----
:idprefix: id_
----

"#
    );

    let html = convert("= D\n:idprefix: id_\n\n== Two Words");

    assert!(
        html.contains(r#"<h2 id="id_two_words">Two Words</h2>"#),
        "{html}"
    );
}

// Setting `idprefix` to an empty value removes the prefix, so the generated ID
// begins directly with the first word (`two_words`, with no leading `_`).
#[test]
fn empty_idprefix_removes_the_prefix() {
    verifies!(
        r#"
If you want to remove the prefix, set the attribute to an empty value.

[source]
----
:idprefix:
----

"#
    );

    let html = convert("= D\n:idprefix:\n\n== Two Words");

    assert!(
        html.contains(r#"<h2 id="two_words">Two Words</h2>"#),
        "{html}"
    );
}

// The WARNING that an empty `idprefix` can yield IDs invalid in DocBook or that
// collide with a built-in HTML id: advisory prose, not a distinct HTML rule.
non_normative!(
    r#"
WARNING: If you set the `idprefix` to empty, you could end up generating IDs that are invalid in DocBook output (e.g., an ID that begins with a number) or that match a built-in ID in the HTML output (e.g., `header`).
In this case, we recommend either using a non-empty value of `idprefix` or assigning xref:custom-ids.adoc[explicit IDs to your sections].

"#
);

// Section heading (with its `[#separator]` anchor) only.
non_normative!(
    r#"
[#separator]
== Change the ID word separator

"#
);

// Setting `idseparator` changes the character placed between words in a
// generated ID, so `:idseparator: -` turns `_two_words` into `_two-words`.
#[test]
fn idseparator_changes_the_separator() {
    verifies!(
        r#"
The default section ID word separator is an underscore (`+_+`).
You can change the separator with the `idseparator` attribute.
Unless empty, the value of the `idseparator` must be _exactly one valid ID character_.

[source]
----
:idseparator: -
----

"#
    );

    let html = convert("= D\n:idseparator: -\n\n== Two Words");

    assert!(
        html.contains(r#"<h2 id="_two-words">Two Words</h2>"#),
        "{html}"
    );
}

// Setting `idseparator` to an empty value removes the separator, so the words
// run together with no character between them (`_twowords`).
#[test]
fn empty_idseparator_removes_the_separator() {
    verifies!(
        r#"
If you don't want to use a separator, set the attribute to an empty value.

[source]
----
:idseparator:
----

"#
    );

    let html = convert("= D\n:idseparator:\n\n== Two Words");

    assert!(
        html.contains(r#"<h2 id="_twowords">Two Words</h2>"#),
        "{html}"
    );
}

// The NOTE that GitHub renders with an empty `idprefix` and a `-` separator to
// match Asciidoctor's IDs: advisory prose describing another tool's settings.
non_normative!(
    r#"
NOTE: When a document is rendered on GitHub, the `idprefix` is set to an empty value and the `idseparator` is set to `-`.
These settings are used to ensure that the IDs generated by GitHub match the IDs generated by Asciidoctor.
"#
);
