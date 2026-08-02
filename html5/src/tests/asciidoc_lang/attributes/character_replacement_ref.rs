//! Coverage of the AsciiDoc language description's *Character Replacement
//! Attributes Reference* page.
//!
//! The page catalogs the built-in document attributes the processor populates
//! for character replacement. Each row's promise — a `{name}` reference expands
//! to the listed replacement text — is verified here by converting a reference
//! to that attribute and checking the substituted value in the rendered output.
//! Only the introduction, the table's footnotes, and the closing prose are
//! tracked as non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/attributes/pages/character-replacement-ref.adoc");

/// Converts `X{name}X` and returns the rendered content of the resulting
/// paragraph. The `X` guards anchor the value so significant leading/trailing
/// whitespace (as for `sp` and `nbsp`) and empty replacements survive
/// extraction. The result is compared as a raw string rather than through the
/// HTML assertions because several replacements (`amp`, `lt`, `gt`, …) are bare
/// metacharacters that a DOM parser would not round-trip.
fn rendered_reference(name: &str) -> String {
    let output = convert(&format!("X{{{name}}}X"));
    output
        .split_once("<p>")
        .and_then(|(_, rest)| rest.split_once("</p>"))
        .map(|(inner, _)| inner.to_string())
        .unwrap_or_default()
}

non_normative!(
    r#"
= Character Replacement Attributes Reference
:page-aliases: character-replacement-reference.adoc

This page identifies built-in document attributes populated by the AsciiDoc processor that are geared towards character replacement.

This category of attributes provide portable replacements for common typographical marks (e.g., smart quotes and symbols) and non-visible characters (e.g., empty and no break space), an escaping mechanism for characters which have special meaning in AsciiDoc (e.g., plus and colon), and passthroughs for characters which get encoded by default (e.g., less than and greater than).
Like all document attributes, you can insert the value of any one of these attributes in your content using an xref:reference-attributes.adoc#reference-built-in[attribute reference] (e.g., `\{nbsp}`).

NOTE: The AsciiDoc processor does not prevent you from reassigning these predefined attributes.
However, you're encouraged to treat them as read-only.
Only a converter should override these attributes if the output format requires the use of a different encoding scheme.

"#
);

#[test]
fn built_in_character_replacement_attributes() {
    verifies!(
        r#"
.Built-in document attributes for character replacement
[%autowidth,cols="^~m,^~l,^~"]
|===
|Attribute name |Replacement text |Appearance

d|``blank``^[1]^
e|nothing
|{empty}

|empty
e|nothing
|{empty}

|sp
e|space
|{sp}

|nbsp
|&#160;
|{nbsp}

d|``zwsp``^[2]^
|&#8203;
|{zwsp}

d|``wj``^[3]^
|&#8288;
|{wj}

|apos
|&#39;
|{apos}

|quot
|&#34;
|{quot}

|lsquo
|&#8216;
|{lsquo}

|rsquo
|&#8217;
|{rsquo}

|ldquo
|&#8220;
|{ldquo}

|rdquo
|&#8221;
|{rdquo}

|deg
|&#176;
|{deg}

|plus
|&#43;
|{plus}

|brvbar
|&#166;
|&#166;

|vbar
|\|
|{vbar}

|amp
|&
|&

|lt
|<
|<

|gt
|>
|>

|startsb
|[
|[

|endsb
|]
|]

|caret
|^
|^

|asterisk
|*
|*

|tilde
|~
|~

|backslash
|\
|\

|backtick
|`
|`

|two-colons
|::
|::

|two-semicolons
|;;
|;;

|cpp (deprecated)
|C++
|C++

|cxx
|C++
|C++

|pp
|&#43;&#43;
|&#43;&#43;
|===

"#
    );

    // Referencing each attribute expands it to its documented replacement.
    // Note that the page's "Replacement text" column is illustrative: the value
    // substituted for `cpp`/`cxx` is `C&#43;&#43;` (so the `+` characters are
    // not subject to further substitution), even though the column renders it as
    // `C++`.
    let cases: &[(&str, &str)] = &[
        ("blank", ""),
        ("empty", ""),
        ("sp", " "),
        ("nbsp", "&#160;"),
        ("zwsp", "&#8203;"),
        ("wj", "&#8288;"),
        ("apos", "&#39;"),
        ("quot", "&#34;"),
        ("lsquo", "&#8216;"),
        ("rsquo", "&#8217;"),
        ("ldquo", "&#8220;"),
        ("rdquo", "&#8221;"),
        ("deg", "&#176;"),
        ("plus", "&#43;"),
        ("brvbar", "&#166;"),
        ("vbar", "|"),
        ("amp", "&"),
        ("lt", "<"),
        ("gt", ">"),
        ("startsb", "["),
        ("endsb", "]"),
        ("caret", "^"),
        ("asterisk", "*"),
        ("tilde", "~"),
        ("backslash", "\\"),
        ("backtick", "`"),
        ("two-colons", "::"),
        ("two-semicolons", ";;"),
        ("cpp", "C&#43;&#43;"),
        ("cxx", "C&#43;&#43;"),
        ("pp", "&#43;&#43;"),
    ];

    for (name, expected) in cases {
        assert_eq!(
            rendered_reference(name),
            format!("X{expected}X"),
            "built-in character replacement attribute `{name}` should render as its documented value",
        );
    }
}

#[test]
fn reference_a_character_replacement_attribute() {
    // An attribute reference is replaced with the attribute's value, so `{deg}`
    // renders as the numeric character reference `&#176;` (and the apostrophe is
    // turned into a right single quotation mark by the replacements step).
    let output = convert("Wolpertingers don't like temperatures above 100{deg}C.");

    assert_eq!(
        output
            .split_once("<p>")
            .and_then(|(_, rest)| rest.split_once("</p>"))
            .map(|(inner, _)| inner),
        Some("Wolpertingers don&#8217;t like temperatures above 100&#176;C."),
    );
}

non_normative!(
    r#"
^[1]^ An alias for the attribute `empty`, for those who find this terminology clearer.

^[2]^ The Zero Width Space (ZWSP) is a code point in Unicode that shows where a long word can be split if necessary.

^[3]^ The word joiner (WJ) is a code point in Unicode that prevents a line break at its position.

Notice that some replacement values are Unicode characters, whereas others are numeric character references (e.g., \&#34;).
The numeric character reference is when the Unicode character could interfere with the AsciiDoc syntax.
In this case, it's the responsibility of the converter to transform that numeric character reference into a format that is compatible with the output format.
For example, in the man page converter, each character reference is replaced with a troff macro.

Thus, the abstraction of using AsciiDoc attributes for character replacements not only gives the author control over how the document is interpreted, it also helps decouple content and presentation.
In other words, it's more portable to use an attribute reference in the content rather than hardcode a numeric character reference.
"#
);
