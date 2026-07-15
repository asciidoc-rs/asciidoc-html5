//! Port of Asciidoctor's `attribute_list_test.rb`.
//!
//! These are unit tests of Asciidoctor's parser-internal `AttributeList` class:
//! each builds an `AttributeList` from a raw attrlist string, calls
//! `parse_into`, and asserts on the resulting positional/named attribute map —
//! no HTML is rendered. This crate is a renderer, so it has no direct analog of
//! `AttributeList`, but it depends on `asciidoc-parser`, whose equivalent parse
//! *is* reachable through the public API: wrapping the raw list in a block
//! attribute line (`[<line>]`), parsing the document, and reading the block's
//! [`Attrlist`](asciidoc_parser::attributes::Attrlist) via
//! [`IsBlock::attrlist`](asciidoc_parser::blocks::IsBlock::attrlist). Each test
//! we port drives that public path and asserts on the parsed attributes.
//!
//! **Modeling differences from Asciidoctor's `AttributeList#parse_into`** (why
//! several tests stay `non_normative!`). Asciidoctor's `parse_into` collects
//! into a single hash that keys *positional* attributes by a running 1-based
//! entry index which counts named entries too, and preserves empty slots as
//! `nil`. `asciidoc-parser` instead models an ordered list where
//! [`nth_attribute`](asciidoc_parser::attributes::Attrlist::nth_attribute)
//! counts only the *unnamed* attributes and empty entries are not retained as
//! placeholders. Two consequences:
//!
//! * A middle empty entry (`,,`) is dropped rather than kept as a `nil` slot,
//!   so later positionals are not index-aligned with Asciidoctor's.
//! * An empty *positional* whose value is `""` / `''` yields no attribute at
//!   all (an empty *named* value like `caption=""` is retained, matching
//!   Asciidoctor).
//!
//! Additionally, the `AttributeList` unit tests exercise the *no-document* code
//! path (they pass no document, or install an `apply_subs` that raises), so
//! substitutions are never applied. Driving the parse through a real document
//! always carries a document, so `asciidoc-parser` applies the normal
//! substitution group and attribute-reference substitution to quoted values —
//! diverging on the tests that assert `apply_subs` is *not* called.
//!
//! Tests turning on any of those differences — empty-quoted positionals, blank
//! positional slots, the `apply_subs`-not-called guarantee, `options` list
//! whitespace, leading-whitespace attrlists, and the `parse_into` rekey /
//! static `rekey` API (which this crate does not expose) — are kept
//! `non_normative!` and annotated inline. Everything else is verified against
//! `asciidoc-parser`'s parsed attributes.

use asciidoc_parser::{blocks::IsBlock as _, Parser};

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/test/attribute_list_test.rb");

/// The positional and named attributes parsed from a raw attrlist string, plus
/// its option list — collected into owned data so the borrowed `Document` can
/// be dropped before the caller asserts.
struct ParsedAttrlist {
    positional: Vec<String>,
    named: Vec<(String, String)>,
    options: Vec<String>,
}

impl ParsedAttrlist {
    /// Value of the (1-based) `n`th *positional* (unnamed) attribute.
    fn nth(&self, n: usize) -> Option<&str> {
        self.positional.get(n.checked_sub(1)?).map(String::as_str)
    }

    /// Value of the named attribute `name`, if present.
    fn named(&self, name: &str) -> Option<&str> {
        self.named
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Parse a raw attrlist string the way Asciidoctor's `AttributeList.new(line)`
/// does, but through `asciidoc-parser`'s public API: wrap it in a block
/// attribute line and read the resulting block's `Attrlist`. See the module
/// docs for the modeling differences this path implies.
fn parse_attrlist(line: &str) -> ParsedAttrlist {
    let src = format!("[{line}]\ntext\n");
    let mut parser = Parser::default();
    let doc = parser.parse(&src);
    let block = doc.nested_blocks().next().expect("one block");
    let attrlist = block.attrlist().expect("block carries an attrlist");

    let mut positional = vec![];
    let mut named = vec![];

    for attr in attrlist.attributes() {
        match attr.name() {
            Some(name) => named.push((name.to_string(), attr.value().to_string())),
            None => positional.push(attr.value().to_string()),
        }
    }

    let options = attrlist.options().iter().map(|o| o.to_string()).collect();

    ParsedAttrlist {
        positional,
        named,
        options,
    }
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'AttributeList' do
"#
);

#[test]
fn collect_unnamed_attribute() {
    verifies!(
        r#"
  test 'collect unnamed attribute' do
    attributes = {}
    line = 'quote'
    expected = { 1 => 'quote' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("quote");
    assert_eq!(a.nth(1), Some("quote"));
    assert!(a.named.is_empty());
}

#[test]
fn collect_unnamed_attribute_double_quoted() {
    verifies!(
        r#"
  test 'collect unnamed attribute double-quoted' do
    attributes = {}
    line = '"quote"'
    expected = { 1 => 'quote' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist(r#""quote""#);
    assert_eq!(a.nth(1), Some("quote"));
    assert!(a.named.is_empty());
}

// `asciidoc-parser` produces no attribute for an empty double-quoted positional
// value, where Asciidoctor's `parse_into` records `{ 1 => '' }`.
non_normative!(
    r#"
  test 'collect empty unnamed attribute double-quoted' do
    attributes = {}
    line = '""'
    expected = { 1 => '' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn collect_unnamed_attribute_double_quoted_containing_escaped_quote() {
    verifies!(
        r#"
  test 'collect unnamed attribute double-quoted containing escaped quote' do
    attributes = {}
    line = '"ba\"zaar"'
    expected = { 1 => 'ba"zaar' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist(r#""ba\"zaar""#);
    assert_eq!(a.nth(1), Some(r#"ba"zaar"#));
    assert!(a.named.is_empty());
}

#[test]
fn collect_unnamed_attribute_single_quoted() {
    verifies!(
        r#"
  test 'collect unnamed attribute single-quoted' do
    attributes = {}
    line = '\'quote\''
    expected = { 1 => 'quote' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("'quote'");
    assert_eq!(a.nth(1), Some("quote"));
    assert!(a.named.is_empty());
}

// `asciidoc-parser` produces no attribute for an empty single-quoted positional
// value, where Asciidoctor's `parse_into` records `{ 1 => '' }`.
non_normative!(
    r#"
  test 'collect empty unnamed attribute single-quoted' do
    attributes = {}
    line = '\'\''
    expected = { 1 => '' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn collect_isolated_single_quote_positional_attribute() {
    verifies!(
        r#"
  test 'collect isolated single quote positional attribute' do
    attributes = {}
    line = '\''
    expected = { 1 => '\'' }
    doc = empty_document
    def doc.apply_subs *args
      raise 'apply_subs should not be called'
    end
    Asciidoctor::AttributeList.new(line, doc).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("'");
    assert_eq!(a.nth(1), Some("'"));
    assert!(a.named.is_empty());
}

#[test]
fn collect_isolated_single_quote_attribute_value() {
    verifies!(
        r#"
  test 'collect isolated single quote attribute value' do
    attributes = {}
    line = 'name=\''
    expected = { 'name' => '\'' }
    doc = empty_document
    def doc.apply_subs *args
      raise 'apply_subs should not be called'
    end
    Asciidoctor::AttributeList.new(line, doc).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("name='");
    assert_eq!(a.named("name"), Some("'"));
    assert!(a.positional.is_empty());
}

// Asciidoctor keeps the value literal here because `parse_into` runs on the
// no-document path (its `apply_subs` is stubbed to raise). Driving the parse
// through a document, `asciidoc-parser` applies attribute-reference
// substitution, so `name='{val}` with `val` defined resolves to `'val` rather
// than the literal `'{val}`.
//
// Likewise, a single-quoted value receives the normal substitution group, so
// `'ba\'zaar'` becomes `ba&#8217;zaar` (typographic apostrophe) rather than the
// literal `ba'zaar` the no-document path yields.
non_normative!(
    r#"
  test 'collect attribute value as is if it has only leading single quote' do
    attributes = {}
    line = 'name=\'{val}'
    expected = { 'name' => '\'{val}' }
    doc = empty_document attributes: { 'val' => 'val' }
    def doc.apply_subs *args
      raise 'apply_subs should not be called'
    end
    Asciidoctor::AttributeList.new(line, doc).parse_into(attributes)
    assert_equal expected, attributes
  end

  test 'collect unnamed attribute single-quoted containing escaped quote' do
    attributes = {}
    line = '\'ba\\\'zaar\''
    expected = { 1 => 'ba\'zaar' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn collect_unnamed_attribute_with_dangling_delimiter() {
    verifies!(
        r#"
  test 'collect unnamed attribute with dangling delimiter' do
    attributes = {}
    line = 'quote , '
    expected = { 1 => 'quote', 2 => nil }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    // The dangling delimiter yields a trailing empty positional; `asciidoc-parser`
    // represents its value as `""` where Asciidoctor records `nil`.
    let a = parse_attrlist("quote , ");
    assert_eq!(a.nth(1), Some("quote"));
    assert_eq!(a.nth(2), Some(""));
}

#[test]
fn collect_unnamed_attribute_in_second_position_after_empty_attribute() {
    verifies!(
        r#"
  test 'collect unnamed attribute in second position after empty attribute' do
    attributes = {}
    line = ', John Smith'
    expected = { 1 => nil, 2 => 'John Smith' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    // The leading delimiter yields an empty first positional (`""` here, `nil` in
    // Asciidoctor), with the name in the second position.
    let a = parse_attrlist(", John Smith");
    assert_eq!(a.nth(1), Some(""));
    assert_eq!(a.nth(2), Some("John Smith"));
}

#[test]
fn collect_unnamed_attributes() {
    verifies!(
        r#"
  test 'collect unnamed attributes' do
    attributes = {}
    line = 'first, second one, third'
    expected = { 1 => 'first', 2 => 'second one', 3 => 'third' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("first, second one, third");
    assert_eq!(a.nth(1), Some("first"));
    assert_eq!(a.nth(2), Some("second one"));
    assert_eq!(a.nth(3), Some("third"));
    assert!(a.named.is_empty());
}

// `asciidoc-parser` drops the middle empty entry (`,,`) rather than keeping it
// as a `nil` slot, so it collects three positionals (`first`, `third`, `''`)
// instead of Asciidoctor's four (`{ 1 => 'first', 2 => nil, 3 => 'third', 4 =>
// nil }`), and the surviving positionals are no longer index-aligned.
non_normative!(
    r#"
  test 'collect blank unnamed attributes' do
    attributes = {}
    line = 'first,,third,'
    expected = { 1 => 'first', 2 => nil, 3 => 'third', 4 => nil }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn collect_unnamed_attribute_enclosed_in_equal_signs() {
    verifies!(
        r#"
  test 'collect unnamed attribute enclosed in equal signs' do
    attributes = {}
    line = '=foo='
    expected = { 1 => '=foo=' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("=foo=");
    assert_eq!(a.nth(1), Some("=foo="));
    assert!(a.named.is_empty());
}

#[test]
fn collect_named_attribute() {
    verifies!(
        r#"
  test 'collect named attribute' do
    attributes = {}
    line = 'foo=bar'
    expected = { 'foo' => 'bar' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("foo=bar");
    assert_eq!(a.named("foo"), Some("bar"));
    assert!(a.positional.is_empty());
}

#[test]
fn collect_named_attribute_double_quoted() {
    verifies!(
        r#"
  test 'collect named attribute double-quoted' do
    attributes = {}
    line = 'foo="bar"'
    expected = { 'foo' => 'bar' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist(r#"foo="bar""#);
    assert_eq!(a.named("foo"), Some("bar"));
    assert!(a.positional.is_empty());
}

#[test]
fn collect_named_attribute_with_double_quoted_empty_value() {
    verifies!(
        r#"
  test 'collect named attribute with double-quoted empty value' do
    attributes = {}
    line = 'height=100,caption="",link="images/octocat.png"'
    expected = { 'height' => '100', 'caption' => '', 'link' => 'images/octocat.png' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist(r#"height=100,caption="",link="images/octocat.png""#);
    assert_eq!(a.named("height"), Some("100"));
    assert_eq!(a.named("caption"), Some(""));
    assert_eq!(a.named("link"), Some("images/octocat.png"));
}

#[test]
fn collect_named_attribute_single_quoted() {
    verifies!(
        r#"
  test 'collect named attribute single-quoted' do
    attributes = {}
    line = 'foo=\'bar\''
    expected = { 'foo' => 'bar' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("foo='bar'");
    assert_eq!(a.named("foo"), Some("bar"));
    assert!(a.positional.is_empty());
}

#[test]
fn collect_named_attribute_with_single_quoted_empty_value() {
    verifies!(
        r#"
  test 'collect named attribute with single-quoted empty value' do
    attributes = {}
    line = %(height=100,caption='',link='images/octocat.png')
    expected = { 'height' => '100', 'caption' => '', 'link' => 'images/octocat.png' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("height=100,caption='',link='images/octocat.png'");
    assert_eq!(a.named("height"), Some("100"));
    assert_eq!(a.named("caption"), Some(""));
    assert_eq!(a.named("link"), Some("images/octocat.png"));
}

#[test]
fn collect_single_named_attribute_with_empty_value() {
    verifies!(
        r#"
  test 'collect single named attribute with empty value' do
    attributes = {}
    line = 'foo='
    expected = { 'foo' => '' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("foo=");
    assert_eq!(a.named("foo"), Some(""));
    assert!(a.positional.is_empty());
}

#[test]
fn collect_single_named_attribute_with_empty_value_when_followed_by_other_attributes() {
    verifies!(
        r#"
  test 'collect single named attribute with empty value when followed by other attributes' do
    attributes = {}
    line = 'foo=,bar=baz'
    expected = { 'foo' => '', 'bar' => 'baz' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("foo=,bar=baz");
    assert_eq!(a.named("foo"), Some(""));
    assert_eq!(a.named("bar"), Some("baz"));
}

#[test]
fn collect_named_attributes_unquoted() {
    verifies!(
        r#"
  test 'collect named attributes unquoted' do
    attributes = {}
    line = 'first=value, second=two, third=3'
    expected = { 'first' => 'value', 'second' => 'two', 'third' => '3' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist("first=value, second=two, third=3");
    assert_eq!(a.named("first"), Some("value"));
    assert_eq!(a.named("second"), Some("two"));
    assert_eq!(a.named("third"), Some("3"));
}

#[test]
fn collect_named_attributes_quoted() {
    verifies!(
        r#"
  test 'collect named attributes quoted' do
    attributes = {}
    line = %(first='value', second="value two", third=three)
    expected = { 'first' => 'value', 'second' => 'value two', 'third' => 'three' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    let a = parse_attrlist(r#"first='value', second="value two", third=three"#);
    assert_eq!(a.named("first"), Some("value"));
    assert_eq!(a.named("second"), Some("value two"));
    assert_eq!(a.named("third"), Some("three"));
}

// A block attribute line with leading whitespace inside the brackets is not
// recognized as an attrlist by `asciidoc-parser`, so this non-semantic-space
// case cannot be driven through the public API.
non_normative!(
    r#"
  test 'collect named attributes quoted containing non-semantic spaces' do
    attributes = {}
    line = %(     first    =     'value', second     ="value two"     , third=       three      )
    expected = { 'first' => 'value', 'second' => 'value two', 'third' => 'three' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn collect_mixed_named_and_unnamed_attributes() {
    verifies!(
        r#"
  test 'collect mixed named and unnamed attributes' do
    attributes = {}
    line = %(first, second="value two", third=three, Sherlock Holmes)
    expected = { 1 => 'first', 'second' => 'value two', 'third' => 'three', 4 => 'Sherlock Holmes' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    // Named and unnamed attributes coexist with the expected values. Asciidoctor
    // keys the trailing positional as index 4 (its running counter includes the
    // two named entries); `asciidoc-parser` counts only unnamed positions, so it
    // is the 2nd positional.
    let a = parse_attrlist(r#"first, second="value two", third=three, Sherlock Holmes"#);
    assert_eq!(a.nth(1), Some("first"));
    assert_eq!(a.named("second"), Some("value two"));
    assert_eq!(a.named("third"), Some("three"));
    assert_eq!(a.nth(2), Some("Sherlock Holmes"));
}

// `asciidoc-parser` drops the blank unnamed entries (`,,`) rather than keeping
// them as `nil` slots, so it does not reproduce the `2 => nil, 4 => nil`
// placeholders and the surviving positional is not index-aligned with
// Asciidoctor's.
//
// The `options` / `opts` tests below also diverge: `asciidoc-parser` does not
// trim whitespace around option tokens, so `'opt1,,opt2 , opt3'` yields
// `opt2 ` and ` opt3` (with spaces) rather than the trimmed `opt2` / `opt3`
// Asciidoctor records as `opt2-option` / `opt3-option`.
non_normative!(
    r#"
  test 'collect mixed empty named and blank unnamed attributes' do
    attributes = {}
    line = 'first,,third=,,fifth=five'
    expected = { 1 => 'first', 2 => nil, 'third' => '', 4 => nil, 'fifth' => 'five' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

  test 'collect options attribute' do
    attributes = {}
    line = %(quote, options='opt1,,opt2 , opt3')
    expected = { 1 => 'quote', 'opt1-option' => '', 'opt2-option' => '', 'opt3-option' => '' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

  test 'collect opts attribute as options' do
    attributes = {}
    line = %(quote, opts='opt1,,opt2 , opt3')
    expected = { 1 => 'quote', 'opt1-option' => '', 'opt2-option' => '', 'opt3-option' => '' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
);

#[test]
fn should_ignore_options_attribute_if_empty() {
    verifies!(
        r#"
  test 'should ignore options attribute if empty' do
    attributes = {}
    line = %(quote, opts=)
    expected = { 1 => 'quote' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes)
    assert_equal expected, attributes
  end

"#
    );

    // An empty `opts=` contributes no options.
    let a = parse_attrlist("quote, opts=");
    assert_eq!(a.nth(1), Some("quote"));
    assert!(a.options.is_empty());
}

// The remaining tests exercise `parse_into`'s positional rekeying (mapping
// positional attributes onto supplied names) and the static
// `AttributeList.rekey` helper. `asciidoc-parser` does not expose either — it
// offers only per-lookup name-or-position resolution
// (`named_or_positional_attribute`), not a rekey that folds positional names
// into the attribute set — and the blank-slot dropping described above further
// shifts the positions these tests depend on. Kept `non_normative!`.
non_normative!(
    r#"
  test 'collect and rekey unnamed attributes' do
    attributes = {}
    line = 'first, second one, third, fourth'
    expected = { 1 => 'first', 2 => 'second one', 3 => 'third', 4 => 'fourth', 'a' => 'first', 'b' => 'second one', 'c' => 'third' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes, ['a', 'b', 'c'])
    assert_equal expected, attributes
  end

  test 'should not assign nil to attribute mapped to missing positional attribute' do
    attributes = {}
    line = 'alt text,,100'
    expected = { 1 => 'alt text', 2 => nil, 3 => '100', 'alt' => 'alt text', 'height' => '100' }
    Asciidoctor::AttributeList.new(line).parse_into(attributes, %w(alt width height))
    assert_equal expected, attributes
  end

  test 'rekey positional attributes' do
    attributes = { 1 => 'source', 2 => 'java' }
    expected = { 1 => 'source', 2 => 'java', 'style' => 'source', 'language' => 'java' }
    Asciidoctor::AttributeList.rekey(attributes, ['style', 'language', 'linenums'])
    assert_equal expected, attributes
  end
end
"#
);
