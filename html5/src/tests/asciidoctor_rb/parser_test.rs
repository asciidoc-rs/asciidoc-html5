//! Port of Asciidoctor's `parser_test.rb`.
//!
//! `parser_test.rb` almost entirely unit-tests the Ruby `Asciidoctor::Parser`
//! class directly — static helpers (`is_section_title?`,
//! `sanitize_attribute_name`, `store_attribute`, `parse_style_attribute`,
//! `adjust_indentation!`) and `parse_header_metadata`. This crate exposes none
//! of those methods; it renders through `convert` / inspects a loaded
//! [`Document`]. So the port drives the *closest observable behavior*:
//!
//! - **Author and revision metadata** (`parse_header_metadata`) is Document
//!   state, so it is verified through [`load`] — the [`Document::authors`] list
//!   (`name`/`firstname`/`middlename`/`lastname`/`email`/`initials`) and the
//!   resolved `author*` / `rev*` document attributes. An author or revision
//!   line only exists beneath a document title, so [`header`] prepends one.
//! - **`sanitize_attribute_name`** is observed through `convert`: a `{name}`
//!   reference resolves only when the entry name was sanitized to `name`.
//! - **The duplicate inline-anchor warning** is observed through the document
//!   warnings inventory.
//!
//! What stays `non_normative!` here:
//! - the static `Parser` helpers with no document-level analogue
//!   (`is_section_title?`, `store_attribute`, `parse_style_attribute`,
//!   `adjust_indentation!`) — their observable slices (section-title
//!   recognition, block id/role/option rendering, verbatim indentation) are
//!   covered by `sections_test`, `attribute_list_test`, and `blocks_test`;
//! - the raw metadata-hash-shape assertion made without a document.

use asciidoc_parser::{document::InterpretedValue, warnings::WarningType, Document};

use crate::{convert, load, load_with, tests::sdd::*, Options};

track_file!("ref/asciidoctor/test/parser_test.rb");

/// Loads `= T` followed by `header_lines`, so an author or revision line (which
/// only exists beneath a document title) is parsed as document header metadata.
fn header(header_lines: &str) -> Document<'static> {
    load(&format!("= T\n{header_lines}\n"))
}

/// The resolved string value of a document attribute, or `None` when it is
/// unset — the `convert`-level analogue of a missing metadata-hash key.
fn attr(doc: &Document, name: &str) -> Option<String> {
    match doc.attribute_value(name) {
        InterpretedValue::Value(v) => Some(v),
        _ => None,
    }
}

/// Inner HTML of the first rendered `<p>`, used to observe a resolved attribute
/// reference.
fn para(src: &str) -> String {
    let html = convert(src);
    let start = html.find("<p>").expect("rendered a paragraph") + "<p>".len();
    let end = html.find("</p>").expect("rendered a paragraph");
    html[start..end].to_string()
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

"#
);

non_normative!(
    r#"
context "Parser" do
"#
);

// `Parser.is_section_title?` is a static string predicate with no
// document-level analogue here; the setext (underlined) title form it
// checks is out of scope for this project, and ATX section-title
// recognition is covered by `sections_test`.
non_normative!(
    r#"
  test "is_section_title?" do
    assert Asciidoctor::Parser.is_section_title?('AsciiDoc Home Page', '==================')
    assert Asciidoctor::Parser.is_section_title?('=== AsciiDoc Home Page')
  end

"#
);

#[test]
fn sanitize_attribute_name() {
    verifies!(
        r#"
  test 'sanitize attribute name' do
    assert_equal 'foobar', Asciidoctor::Parser.sanitize_attribute_name("Foo Bar")
    assert_equal 'foo', Asciidoctor::Parser.sanitize_attribute_name("foo")
    assert_equal 'foo3-bar', Asciidoctor::Parser.sanitize_attribute_name("Foo 3^ # - Bar[")
  end

"#
    );

    // Asciidoctor's static `Parser.sanitize_attribute_name` has no direct
    // analogue here; the closest observable behavior is that a `{name}`
    // reference resolves only when the entry name was sanitized to `name`.
    assert_eq!(para(":Foo Bar: v\n\n{foobar}"), "v");
    assert_eq!(para(":foo: v\n\n{foo}"), "v");
    assert_eq!(para(":Foo 3^ # - Bar[: v\n\n{foo3-bar}"), "v");
}

// The `store attribute` and `parse style attribute` cases exercise the
// static `Parser.store_attribute` / `Parser.parse_style_attribute`
// helpers and the parser's internal attributes hash (the
// `:attribute_entries` bookkeeping, the `style`/`id`/`role`/`*-option`
// keys), none of which this crate exposes. Block id/role/option
// rendering is covered by `attribute_list_test` and `blocks_test`.
non_normative!(
    r#"
  test 'store attribute with value' do
    attr_name, attr_value = Asciidoctor::Parser.store_attribute 'foo', 'bar'
    assert_equal 'foo', attr_name
    assert_equal 'bar', attr_value
  end

  test 'store attribute with negated value' do
    { 'foo!' => nil, '!foo' => nil, 'foo' => nil }.each do |name, value|
      attr_name, attr_value = Asciidoctor::Parser.store_attribute name, value
      assert_equal name.sub('!', ''), attr_name
      assert_nil attr_value
    end
  end

  test 'store accessible attribute on document with value' do
    doc = empty_document
    doc.set_attribute 'foo', 'baz'
    attrs = {}
    attr_name, attr_value = Asciidoctor::Parser.store_attribute 'foo', 'bar', doc, attrs
    assert_equal 'foo', attr_name
    assert_equal 'bar', attr_value
    assert_equal 'bar', (doc.attr 'foo')
    assert attrs.key?(:attribute_entries)
    assert_equal 1, attrs[:attribute_entries].size
    assert_equal 'foo', attrs[:attribute_entries][0].name
    assert_equal 'bar', attrs[:attribute_entries][0].value
  end

  test 'store accessible attribute on document with value that contains attribute reference' do
    doc = empty_document
    doc.set_attribute 'foo', 'baz'
    doc.set_attribute 'release', 'ultramega'
    attrs = {}
    attr_name, attr_value = Asciidoctor::Parser.store_attribute 'foo', '{release}', doc, attrs
    assert_equal 'foo', attr_name
    assert_equal 'ultramega', attr_value
    assert_equal 'ultramega', (doc.attr 'foo')
    assert attrs.key?(:attribute_entries)
    assert_equal 1, attrs[:attribute_entries].size
    assert_equal 'foo', attrs[:attribute_entries][0].name
    assert_equal 'ultramega', attrs[:attribute_entries][0].value
  end

  test 'store inaccessible attribute on document with value' do
    doc = empty_document attributes: { 'foo' => 'baz' }
    attrs = {}
    attr_name, attr_value = Asciidoctor::Parser.store_attribute 'foo', 'bar', doc, attrs
    assert_equal 'foo', attr_name
    assert_equal 'bar', attr_value
    assert_equal 'baz', (doc.attr 'foo')
    refute attrs.key?(:attribute_entries)
  end

  test 'store accessible attribute on document with negated value' do
    { 'foo!' => nil, '!foo' => nil, 'foo' => nil }.each do |name, value|
      doc = empty_document
      doc.set_attribute 'foo', 'baz'
      attrs = {}
      attr_name, attr_value = Asciidoctor::Parser.store_attribute name, value, doc, attrs
      assert_equal name.sub('!', ''), attr_name
      assert_nil attr_value
      assert attrs.key?(:attribute_entries)
      assert_equal 1, attrs[:attribute_entries].size
      assert_equal 'foo', attrs[:attribute_entries][0].name
      assert_nil attrs[:attribute_entries][0].value
    end
  end

  test 'store inaccessible attribute on document with negated value' do
    { 'foo!' => nil, '!foo' => nil, 'foo' => nil }.each do |name, value|
      doc = empty_document attributes: { 'foo' => 'baz' }
      attrs = {}
      attr_name, attr_value = Asciidoctor::Parser.store_attribute name, value, doc, attrs
      assert_equal name.sub('!', ''), attr_name
      assert_nil attr_value
      refute attrs.key?(:attribute_entries)
    end
  end

  test 'parse style attribute with id and role' do
    attributes = { 1 => 'style#id.role' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_equal 'style', style
    assert_equal 'style', attributes['style']
    assert_equal 'id', attributes['id']
    assert_equal 'role', attributes['role']
    assert_equal 'style#id.role', attributes[1]
  end

  test 'parse style attribute with style, role, id and option' do
    attributes = { 1 => 'style.role#id%fragment' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_equal 'style', style
    assert_equal 'style', attributes['style']
    assert_equal 'id', attributes['id']
    assert_equal 'role', attributes['role']
    assert_equal '', attributes['fragment-option']
    assert_equal 'style.role#id%fragment', attributes[1]
    refute attributes.key? 'options'
  end

  test 'parse style attribute with style, id and multiple roles' do
    attributes = { 1 => 'style#id.role1.role2' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_equal 'style', style
    assert_equal 'style', attributes['style']
    assert_equal 'id', attributes['id']
    assert_equal 'role1 role2', attributes['role']
    assert_equal 'style#id.role1.role2', attributes[1]
  end

  test 'parse style attribute with style, multiple roles and id' do
    attributes = { 1 => 'style.role1.role2#id' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_equal 'style', style
    assert_equal 'style', attributes['style']
    assert_equal 'id', attributes['id']
    assert_equal 'role1 role2', attributes['role']
    assert_equal 'style.role1.role2#id', attributes[1]
  end

  test 'parse style attribute with positional and original style' do
    attributes = { 1 => 'new_style', 'style' => 'original_style' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_equal 'new_style', style
    assert_equal 'new_style', attributes['style']
    assert_equal 'new_style', attributes[1]
  end

  test 'parse style attribute with id and role only' do
    attributes = { 1 => '#id.role' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_nil style
    assert_equal 'id', attributes['id']
    assert_equal 'role', attributes['role']
    assert_equal '#id.role', attributes[1]
  end

  test 'parse empty style attribute' do
    attributes = { 1 => nil }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_nil style
    assert_nil attributes['id']
    assert_nil attributes['role']
    assert_nil attributes[1]
  end

  test 'parse style attribute with option should preserve existing options' do
    attributes = { 1 => '%header', 'footer-option' => '' }
    style = Asciidoctor::Parser.parse_style_attribute(attributes)
    assert_nil style
    assert_equal '', attributes['header-option']
    assert_equal '', attributes['footer-option']
  end

"#
);

#[test]
fn parse_author_first() {
    verifies!(
        r#"
  test "parse author first" do
    metadata, _ = parse_header_metadata 'Stuart'
    assert_equal 5, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stuart', metadata['firstname']
    assert_equal 'S', metadata['authorinitials']
  end

"#
    );

    let doc = header("Stuart");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Stuart");
    assert_eq!(a.firstname(), "Stuart");
    assert_eq!(a.initials(), "S");
    assert_eq!(attr(&doc, "author").as_deref(), Some("Stuart"));
    assert_eq!(attr(&doc, "firstname").as_deref(), Some("Stuart"));
    assert_eq!(attr(&doc, "authorinitials").as_deref(), Some("S"));
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_first_last() {
    verifies!(
        r#"
  test "parse author first last" do
    metadata, _ = parse_header_metadata 'Yukihiro Matsumoto'
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Yukihiro Matsumoto', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Yukihiro', metadata['firstname']
    assert_equal 'Matsumoto', metadata['lastname']
    assert_equal 'YM', metadata['authorinitials']
  end

"#
    );

    let doc = header("Yukihiro Matsumoto");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Yukihiro Matsumoto");
    assert_eq!(a.firstname(), "Yukihiro");
    assert_eq!(a.lastname(), Some("Matsumoto"));
    assert_eq!(a.initials(), "YM");
    assert_eq!(attr(&doc, "author").as_deref(), Some("Yukihiro Matsumoto"));
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_first_middle_last() {
    verifies!(
        r#"
  test "parse author first middle last" do
    metadata, _ = parse_header_metadata 'David Heinemeier Hansson'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'David Heinemeier Hansson', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'David', metadata['firstname']
    assert_equal 'Heinemeier', metadata['middlename']
    assert_equal 'Hansson', metadata['lastname']
    assert_equal 'DHH', metadata['authorinitials']
  end

"#
    );

    let doc = header("David Heinemeier Hansson");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "David Heinemeier Hansson");
    assert_eq!(a.firstname(), "David");
    assert_eq!(a.middlename(), Some("Heinemeier"));
    assert_eq!(a.lastname(), Some("Hansson"));
    assert_eq!(a.initials(), "DHH");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_first_middle_last_email() {
    verifies!(
        r#"
  test "parse author first middle last email" do
    metadata, _ = parse_header_metadata 'David Heinemeier Hansson <rails@ruby-lang.org>'
    assert_equal 8, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'David Heinemeier Hansson', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'David', metadata['firstname']
    assert_equal 'Heinemeier', metadata['middlename']
    assert_equal 'Hansson', metadata['lastname']
    assert_equal 'rails@ruby-lang.org', metadata['email']
    assert_equal 'DHH', metadata['authorinitials']
  end

"#
    );

    let doc = header("David Heinemeier Hansson <rails@ruby-lang.org>");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "David Heinemeier Hansson");
    assert_eq!(a.firstname(), "David");
    assert_eq!(a.middlename(), Some("Heinemeier"));
    assert_eq!(a.lastname(), Some("Hansson"));
    assert_eq!(a.email(), Some("rails@ruby-lang.org"));
    assert_eq!(a.initials(), "DHH");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_first_email() {
    verifies!(
        r#"
  test "parse author first email" do
    metadata, _ = parse_header_metadata 'Stuart <founder@asciidoc.org>'
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stuart', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stuart', metadata['firstname']
    assert_equal 'founder@asciidoc.org', metadata['email']
    assert_equal 'S', metadata['authorinitials']
  end

"#
    );

    let doc = header("Stuart <founder@asciidoc.org>");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Stuart");
    assert_eq!(a.firstname(), "Stuart");
    assert_eq!(a.email(), Some("founder@asciidoc.org"));
    assert_eq!(a.initials(), "S");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_first_last_email() {
    verifies!(
        r#"
  test "parse author first last email" do
    metadata, _ = parse_header_metadata 'Stuart Rackham <founder@asciidoc.org>'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stuart Rackham', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stuart', metadata['firstname']
    assert_equal 'Rackham', metadata['lastname']
    assert_equal 'founder@asciidoc.org', metadata['email']
    assert_equal 'SR', metadata['authorinitials']
  end

"#
    );

    let doc = header("Stuart Rackham <founder@asciidoc.org>");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Stuart Rackham");
    assert_eq!(a.firstname(), "Stuart");
    assert_eq!(a.lastname(), Some("Rackham"));
    assert_eq!(a.email(), Some("founder@asciidoc.org"));
    assert_eq!(a.initials(), "SR");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_with_hyphen() {
    verifies!(
        r#"
  test "parse author with hyphen" do
    metadata, _ = parse_header_metadata 'Tim Berners-Lee <founder@www.org>'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Tim Berners-Lee', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Tim', metadata['firstname']
    assert_equal 'Berners-Lee', metadata['lastname']
    assert_equal 'founder@www.org', metadata['email']
    assert_equal 'TB', metadata['authorinitials']
  end

"#
    );

    let doc = header("Tim Berners-Lee <founder@www.org>");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Tim Berners-Lee");
    assert_eq!(a.firstname(), "Tim");
    assert_eq!(a.lastname(), Some("Berners-Lee"));
    assert_eq!(a.email(), Some("founder@www.org"));
    assert_eq!(a.initials(), "TB");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_with_single_quote() {
    verifies!(
        r#"
  test "parse author with single quote" do
    metadata, _ = parse_header_metadata 'Stephen O\'Grady <founder@redmonk.com>'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stephen O\'Grady', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stephen', metadata['firstname']
    assert_equal 'O\'Grady', metadata['lastname']
    assert_equal 'founder@redmonk.com', metadata['email']
    assert_equal 'SO', metadata['authorinitials']
  end

"#
    );

    let doc = header("Stephen O'Grady <founder@redmonk.com>");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Stephen O'Grady");
    assert_eq!(a.firstname(), "Stephen");
    assert_eq!(a.lastname(), Some("O'Grady"));
    assert_eq!(a.email(), Some("founder@redmonk.com"));
    assert_eq!(a.initials(), "SO");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_with_dotted_initial() {
    verifies!(
        r#"
  test "parse author with dotted initial" do
    metadata, _ = parse_header_metadata 'Heiko W. Rupp <hwr@example.de>'
    assert_equal 8, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Heiko W. Rupp', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Heiko', metadata['firstname']
    assert_equal 'W.', metadata['middlename']
    assert_equal 'Rupp', metadata['lastname']
    assert_equal 'hwr@example.de', metadata['email']
    assert_equal 'HWR', metadata['authorinitials']
  end

"#
    );

    let doc = header("Heiko W. Rupp <hwr@example.de>");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Heiko W. Rupp");
    assert_eq!(a.firstname(), "Heiko");
    assert_eq!(a.middlename(), Some("W."));
    assert_eq!(a.lastname(), Some("Rupp"));
    assert_eq!(a.email(), Some("hwr@example.de"));
    assert_eq!(a.initials(), "HWR");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_with_underscore() {
    verifies!(
        r#"
  test "parse author with underscore" do
    metadata, _ = parse_header_metadata 'Tim_E Fella'
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Tim E Fella', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Tim E', metadata['firstname']
    assert_equal 'Fella', metadata['lastname']
    assert_equal 'TF', metadata['authorinitials']
  end

"#
    );

    let doc = header("Tim_E Fella");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Tim E Fella");
    assert_eq!(a.firstname(), "Tim E");
    assert_eq!(a.lastname(), Some("Fella"));
    assert_eq!(a.initials(), "TF");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_name_with_letters_outside_basic_latin() {
    verifies!(
        r#"
  test 'parse author name with letters outside basic latin' do
    metadata, _ = parse_header_metadata 'Stéphane Brontë'
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stéphane Brontë', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stéphane', metadata['firstname']
    assert_equal 'Brontë', metadata['lastname']
    assert_equal 'SB', metadata['authorinitials']
  end

"#
    );

    let doc = header("St\u{e9}phane Bront\u{eb}");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "St\u{e9}phane Bront\u{eb}");
    assert_eq!(a.firstname(), "St\u{e9}phane");
    assert_eq!(a.lastname(), Some("Bront\u{eb}"));
    assert_eq!(a.initials(), "SB");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_ideographic_author_names() {
    verifies!(
        r#"
  test 'parse ideographic author names' do
    metadata, _ = parse_header_metadata '李 四 <si.li@example.com>'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal '李 四', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal '李', metadata['firstname']
    assert_equal '四', metadata['lastname']
    assert_equal 'si.li@example.com', metadata['email']
    assert_equal '李四', metadata['authorinitials']
  end

"#
    );

    let doc = header("\u{674e} \u{56db} <si.li@example.com>");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "\u{674e} \u{56db}");
    assert_eq!(a.firstname(), "\u{674e}");
    assert_eq!(a.lastname(), Some("\u{56db}"));
    assert_eq!(a.email(), Some("si.li@example.com"));
    assert_eq!(a.initials(), "\u{674e}\u{56db}");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_author_condenses_whitespace() {
    verifies!(
        r#"
  test "parse author condenses whitespace" do
    metadata, _ = parse_header_metadata 'Stuart       Rackham     <founder@asciidoc.org>'
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stuart Rackham', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stuart', metadata['firstname']
    assert_equal 'Rackham', metadata['lastname']
    assert_equal 'founder@asciidoc.org', metadata['email']
    assert_equal 'SR', metadata['authorinitials']
  end

"#
    );

    let doc = header("Stuart       Rackham     <founder@asciidoc.org>");
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Stuart Rackham");
    assert_eq!(a.firstname(), "Stuart");
    assert_eq!(a.lastname(), Some("Rackham"));
    assert_eq!(a.email(), Some("founder@asciidoc.org"));
    assert_eq!(a.initials(), "SR");
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_invalid_author_line_becomes_author() {
    verifies!(
        r#"
  test "parse invalid author line becomes author" do
    metadata, _ = parse_header_metadata '   Stuart       Rackham, founder of AsciiDoc   <founder@asciidoc.org>'
    assert_equal 5, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Stuart Rackham, founder of AsciiDoc <founder@asciidoc.org>', metadata['author']
    assert_equal metadata['author'], metadata['authors']
    assert_equal 'Stuart Rackham, founder of AsciiDoc <founder@asciidoc.org>', metadata['firstname']
    assert_equal 'S', metadata['authorinitials']
  end

"#
    );

    let doc = header("   Stuart       Rackham, founder of AsciiDoc   <founder@asciidoc.org>");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));

    // The Ruby `metadata['author']`/`['firstname']` are the pre-substitution
    // values; the `raw_*` accessors (asciidoc-parser #1081) expose exactly that,
    // so they carry the literal `<founder@asciidoc.org>`. (The escaped
    // `name()`/`firstname()` and the resolved `author`/`firstname` attributes
    // read `&lt;…&gt;`, matching Asciidoctor's public `doc.author` /
    // `doc.attributes`; the raw accessors mirror its internal `metadata` hash.)
    let a = &doc.authors()[0];
    assert_eq!(
        a.raw_name(),
        "Stuart Rackham, founder of AsciiDoc <founder@asciidoc.org>"
    );
    assert_eq!(
        a.raw_firstname(),
        "Stuart Rackham, founder of AsciiDoc <founder@asciidoc.org>"
    );
    assert_eq!(attr(&doc, "authorinitials").as_deref(), Some("S"));
    assert_eq!(attr(&doc, "authors"), attr(&doc, "author"));
}

#[test]
fn parse_multiple_authors() {
    verifies!(
        r#"
  test 'parse multiple authors' do
    metadata, _ = parse_header_metadata 'Doc Writer <doc.writer@asciidoc.org>; John Smith <john.smith@asciidoc.org>'
    assert_equal 2, metadata['authorcount']
    assert_equal 'Doc Writer, John Smith', metadata['authors']
    assert_equal 'Doc Writer', metadata['author']
    assert_equal 'Doc Writer', metadata['author_1']
    assert_equal 'John Smith', metadata['author_2']
  end

"#
    );

    let doc = header("Doc Writer <doc.writer@asciidoc.org>; John Smith <john.smith@asciidoc.org>");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("2"));
    assert_eq!(
        attr(&doc, "authors").as_deref(),
        Some("Doc Writer, John Smith")
    );
    assert_eq!(attr(&doc, "author").as_deref(), Some("Doc Writer"));
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Doc Writer"));
    assert_eq!(attr(&doc, "author_2").as_deref(), Some("John Smith"));
}

#[test]
fn should_not_parse_multiple_authors_if_semi_colon_is_not_followed_by_space() {
    verifies!(
        r#"
  test 'should not parse multiple authors if semi-colon is not followed by space' do
    metadata, _ = parse_header_metadata 'Joe Doe;Smith Johnson'
    assert_equal 1, metadata['authorcount']
  end

"#
    );

    let doc = header("Joe Doe;Smith Johnson");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    assert_eq!(doc.authors().len(), 1);
}

#[test]
fn skips_blank_author_entries_in_implicit_author_line() {
    verifies!(
        r#"
  test 'skips blank author entries in implicit author line' do
    metadata, _ = parse_header_metadata 'Doc Writer; ; John Smith <john.smith@asciidoc.org>;'
    assert_equal 2, metadata['authorcount']
    assert_equal 'Doc Writer', metadata['author_1']
    assert_equal 'John Smith', metadata['author_2']
  end

"#
    );

    let doc = header("Doc Writer; ; John Smith <john.smith@asciidoc.org>;");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("2"));
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Doc Writer"));
    assert_eq!(attr(&doc, "author_2").as_deref(), Some("John Smith"));
}

#[test]
fn parse_name_with_more_than_3_parts_in_author_attribute() {
    verifies!(
        r#"
  test 'parse name with more than 3 parts in author attribute' do
    doc = empty_document
    parse_header_metadata ':author: Leroy  Harold  Scherer,  Jr.', doc
    assert_equal 'Leroy Harold Scherer, Jr.', doc.attributes['author']
    assert_equal 'Leroy', doc.attributes['firstname']
    assert_equal 'Harold', doc.attributes['middlename']
    assert_equal 'Scherer, Jr.', doc.attributes['lastname']
  end

"#
    );

    let doc = header(":author: Leroy  Harold  Scherer,  Jr.");
    assert_eq!(
        attr(&doc, "author").as_deref(),
        Some("Leroy Harold Scherer, Jr.")
    );
    assert_eq!(attr(&doc, "firstname").as_deref(), Some("Leroy"));
    assert_eq!(attr(&doc, "middlename").as_deref(), Some("Harold"));
    assert_eq!(attr(&doc, "lastname").as_deref(), Some("Scherer, Jr."));
}

#[test]
fn use_explicit_authorinitials_if_set_after_implicit_author_line() {
    verifies!(
        r#"
  test 'use explicit authorinitials if set after implicit author line' do
    input = <<~'EOS'
    Jean-Claude Van Damme
    :authorinitials: JCVD
    EOS
    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 'JCVD', doc.attributes['authorinitials']
  end

"#
    );

    let doc = header("Jean-Claude Van Damme\n:authorinitials: JCVD");
    assert_eq!(attr(&doc, "authorinitials").as_deref(), Some("JCVD"));
}

#[test]
fn use_explicit_authorinitials_if_set_after_author_attribute() {
    verifies!(
        r#"
  test 'use explicit authorinitials if set after author attribute' do
    input = <<~'EOS'
    :author: Jean-Claude Van Damme
    :authorinitials: JCVD
    EOS
    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 'JCVD', doc.attributes['authorinitials']
  end

"#
    );

    let doc = header(":author: Jean-Claude Van Damme\n:authorinitials: JCVD");
    assert_eq!(attr(&doc, "authorinitials").as_deref(), Some("JCVD"));
}

#[test]
fn use_implicit_authors_if_value_of_authors_attribute_matches_computed_value() {
    verifies!(
        r#"
  test 'use implicit authors if value of authors attribute matches computed value' do
    input = <<~'EOS'
    Doc Writer; Junior Writer
    :authors: Doc Writer, Junior Writer
    EOS
    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 'Doc Writer, Junior Writer', doc.attributes['authors']
    assert_equal 'Doc Writer', doc.attributes['author_1']
    assert_equal 'Junior Writer', doc.attributes['author_2']
  end

"#
    );

    let doc = header("Doc Writer; Junior Writer\n:authors: Doc Writer, Junior Writer");
    assert_eq!(
        attr(&doc, "authors").as_deref(),
        Some("Doc Writer, Junior Writer")
    );
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Doc Writer"));
    assert_eq!(attr(&doc, "author_2").as_deref(), Some("Junior Writer"));
}

#[test]
fn replace_implicit_authors_if_value_of_authors_attribute_does_not_match_computed_value() {
    verifies!(
        r#"
  test 'replace implicit authors if value of authors attribute does not match computed value' do
    input = <<~'EOS'
    Doc Writer; Junior Writer
    :authors: Stuart Rackham; Dan Allen; Sarah White
    EOS
    doc = empty_document
    metadata, _ = parse_header_metadata input, doc
    assert_equal 3, metadata['authorcount']
    assert_equal 3, doc.attributes['authorcount']
    assert_equal 'Stuart Rackham, Dan Allen, Sarah White', doc.attributes['authors']
    assert_equal 'Stuart Rackham', doc.attributes['author_1']
    assert_equal 'Dan Allen', doc.attributes['author_2']
    assert_equal 'Sarah White', doc.attributes['author_3']
  end

"#
    );

    let doc = header("Doc Writer; Junior Writer\n:authors: Stuart Rackham; Dan Allen; Sarah White");
    // The explicit `:authors:` entry replaces the implicit author line.
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("3"));
    assert_eq!(
        attr(&doc, "authors").as_deref(),
        Some("Stuart Rackham, Dan Allen, Sarah White")
    );
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Stuart Rackham"));
    assert_eq!(attr(&doc, "author_2").as_deref(), Some("Dan Allen"));
    assert_eq!(attr(&doc, "author_3").as_deref(), Some("Sarah White"));
}

#[test]
fn sets_authorcount_to_0_if_document_has_no_authors() {
    verifies!(
        r#"
  test 'sets authorcount to 0 if document has no authors' do
    input = ''
    doc = empty_document
    metadata, _ = parse_header_metadata input, doc
    assert_equal 0, doc.attributes['authorcount']
    assert_equal 0, metadata['authorcount']
  end

"#
    );

    let doc = load("");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("0"));
}

// Asserts the shape of the raw metadata hash returned by
// `parse_header_metadata` with no document, which this crate does not
// expose.
non_normative!(
    r#"
  test 'returns empty hash if document has no authors and invoked without document' do
    metadata, _ = parse_header_metadata ''
    assert_empty metadata
  end

"#
);

#[test]
fn does_not_drop_name_joiner_when_using_multiple_authors() {
    verifies!(
        r#"
  test 'does not drop name joiner when using multiple authors' do
    input = 'Kismet Chameleon; Lazarus het_Draeke'
    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 2, doc.attributes['authorcount']
    assert_equal 'Kismet Chameleon, Lazarus het Draeke', doc.attributes['authors']
    assert_equal 'Kismet Chameleon', doc.attributes['author_1']
    assert_equal 'Lazarus het Draeke', doc.attributes['author_2']
    assert_equal 'het Draeke', doc.attributes['lastname_2']
  end

"#
    );

    let doc = header("Kismet Chameleon; Lazarus het_Draeke");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("2"));
    // The name joiner keeps `het Draeke` intact as the second author's name.
    assert_eq!(
        attr(&doc, "authors").as_deref(),
        Some("Kismet Chameleon, Lazarus het Draeke")
    );
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Kismet Chameleon"));
    assert_eq!(
        attr(&doc, "author_2").as_deref(),
        Some("Lazarus het Draeke")
    );
    assert_eq!(attr(&doc, "lastname_2").as_deref(), Some("het Draeke"));
}

#[test]
fn allows_authors_to_be_overridden_using_explicit_author_attributes() {
    verifies!(
        r#"
  test 'allows authors to be overridden using explicit author attributes' do
    input = <<~'EOS'
    Kismet Chameleon; Johnny Bravo; Lazarus het_Draeke
    :author_2: Danger Mouse
    EOS
    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 3, doc.attributes['authorcount']
    assert_equal 'Kismet Chameleon, Danger Mouse, Lazarus het Draeke', doc.attributes['authors']
    assert_equal 'Kismet Chameleon', doc.attributes['author_1']
    assert_equal 'Danger Mouse', doc.attributes['author_2']
    assert_equal 'Lazarus het Draeke', doc.attributes['author_3']
    assert_equal 'het Draeke', doc.attributes['lastname_3']
  end

"#
    );

    let doc = header("Kismet Chameleon; Johnny Bravo; Lazarus het_Draeke\n:author_2: Danger Mouse");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("3"));
    // The explicit `:author_2:` entry overrides the second implicit author,
    // and the override is reflected in the combined `authors` string.
    assert_eq!(
        attr(&doc, "authors").as_deref(),
        Some("Kismet Chameleon, Danger Mouse, Lazarus het Draeke")
    );
    assert_eq!(attr(&doc, "author_1").as_deref(), Some("Kismet Chameleon"));
    assert_eq!(attr(&doc, "author_2").as_deref(), Some("Danger Mouse"));
    assert_eq!(
        attr(&doc, "author_3").as_deref(),
        Some("Lazarus het Draeke")
    );
    assert_eq!(attr(&doc, "lastname_3").as_deref(), Some("het Draeke"));
}

#[test]
fn removes_formatting_before_partitioning_author_defined_using_author_attribute() {
    verifies!(
        r#"
  test 'removes formatting before partitioning author defined using author attribute' do
    input = ':author: pass:n[http://example.org/community/team.html[Ze_**Project** team]]'

    doc = empty_document
    parse_header_metadata input, doc
    assert_equal 1, doc.attributes['authorcount']
    assert_equal '<a href="http://example.org/community/team.html">Ze <strong>Project</strong> team</a>', doc.attributes['authors']
    assert_equal 'Ze Project', doc.attributes['firstname']
    assert_equal 'team', doc.attributes['lastname']
  end

"#
    );

    let doc =
        header(":author: pass:n[http://example.org/community/team.html[Ze_**Project** team]]");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    // The `pass:[]` macro and inline formatting are resolved before the
    // name is partitioned, and `Ze_Project` becomes `Ze Project`.
    assert_eq!(
            attr(&doc, "authors").as_deref(),
            Some("<a href=\"http://example.org/community/team.html\">Ze <strong>Project</strong> team</a>")
        );
    assert_eq!(attr(&doc, "firstname").as_deref(), Some("Ze Project"));
    assert_eq!(attr(&doc, "lastname").as_deref(), Some("team"));
}

#[test]
fn parse_rev_number_date_remark() {
    verifies!(
        r#"
  test "parse rev number date remark" do
    input = <<~'EOS'
    Ryan Waldron
    v0.0.7, 2013-12-18: The first release you can stand on
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 9, metadata.size
    assert_equal '0.0.7', metadata['revnumber']
    assert_equal '2013-12-18', metadata['revdate']
    assert_equal 'The first release you can stand on', metadata['revremark']
  end

"#
    );

    let doc = header("Ryan Waldron\nv0.0.7, 2013-12-18: The first release you can stand on");
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("0.0.7"));
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("2013-12-18"));
    assert_eq!(
        attr(&doc, "revremark").as_deref(),
        Some("The first release you can stand on")
    );
}

#[test]
fn parse_rev_number_data_and_remark_as_attribute_references() {
    verifies!(
        r#"
  test 'parse rev number, data, and remark as attribute references' do
    input = <<~'EOS'
    Author Name
    v{project-version}, {release-date}: {release-summary}
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 9, metadata.size
    assert_equal '{project-version}', metadata['revnumber']
    assert_equal '{release-date}', metadata['revdate']
    assert_equal '{release-summary}', metadata['revremark']
  end

"#
    );

    let doc = header("Author Name\nv{project-version}, {release-date}: {release-summary}");
    assert_eq!(
        attr(&doc, "revnumber").as_deref(),
        Some("{project-version}")
    );
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("{release-date}"));
    assert_eq!(
        attr(&doc, "revremark").as_deref(),
        Some("{release-summary}")
    );
}

#[test]
fn should_resolve_attribute_references_in_rev_number_data_and_remark() {
    verifies!(
        r#"
  test 'should resolve attribute references in rev number, data, and remark' do
    input = <<~'EOS'
    = Document Title
    Author Name
    {project-version}, {release-date}: {release-summary}
    EOS
    doc = document_from_string input, attributes: {
      'project-version' => '1.0.1',
      'release-date' => '2018-05-15',
      'release-summary' => 'The one you can count on!',
    }
    assert_equal '1.0.1', (doc.attr 'revnumber')
    assert_equal '2018-05-15', (doc.attr 'revdate')
    assert_equal 'The one you can count on!', (doc.attr 'revremark')
  end

"#
    );

    let opts = Options::new()
        .attribute("project-version", "1.0.1")
        .attribute("release-date", "2018-05-15")
        .attribute("release-summary", "The one you can count on!");
    let doc = load_with(
        "= Document Title\nAuthor Name\n{project-version}, {release-date}: {release-summary}\n",
        &opts,
    );
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("1.0.1"));
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("2018-05-15"));
    assert_eq!(
        attr(&doc, "revremark").as_deref(),
        Some("The one you can count on!")
    );
}

#[test]
fn parse_rev_date() {
    verifies!(
        r#"
  test "parse rev date" do
    input = <<~'EOS'
    Ryan Waldron
    2013-12-18
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 7, metadata.size
    assert_equal '2013-12-18', metadata['revdate']
  end

"#
    );

    let doc = header("Ryan Waldron\n2013-12-18");
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("2013-12-18"));
    assert_eq!(attr(&doc, "revnumber"), None);
    assert_eq!(attr(&doc, "revremark"), None);
}

#[test]
fn parse_rev_number_with_trailing_comma() {
    verifies!(
        r#"
  test 'parse rev number with trailing comma' do
    input = <<~'EOS'
    Stuart Rackham
    v8.6.8,
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 7, metadata.size
    assert_equal '8.6.8', metadata['revnumber']
    refute metadata.key?('revdate')
  end

"#
    );

    let doc = header("Stuart Rackham\nv8.6.8,");
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("8.6.8"));
    assert_eq!(attr(&doc, "revremark"), None);
    // Unlike Asciidoctor (which leaves `revdate` unset), this parser records
    // an empty `revdate` for a revision line carrying only a number.
    assert!(attr(&doc, "revdate").is_none_or(|v| v.is_empty()));
}

#[test]
fn parse_rev_number() {
    verifies!(
        r#"
  # Asciidoctor recognizes a standalone revision without a trailing comma
  test 'parse rev number' do
    input = <<~'EOS'
    Stuart Rackham
    v8.6.8
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 7, metadata.size
    assert_equal '8.6.8', metadata['revnumber']
    refute metadata.key?('revdate')
  end

"#
    );

    let doc = header("Stuart Rackham\nv8.6.8");
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("8.6.8"));
    assert_eq!(attr(&doc, "revremark"), None);
    assert!(attr(&doc, "revdate").is_none_or(|v| v.is_empty()));
}

#[test]
fn treats_arbitrary_text_on_rev_line_as_revdate() {
    verifies!(
        r#"
  # while compliant w/ AsciiDoc, this is just sloppy parsing
  test "treats arbitrary text on rev line as revdate" do
    input = <<~'EOS'
    Ryan Waldron
    foobar
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 7, metadata.size
    assert_equal 'foobar', metadata['revdate']
  end

"#
    );

    let doc = header("Ryan Waldron\nfoobar");
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("foobar"));
}

#[test]
fn parse_rev_date_remark() {
    verifies!(
        r#"
  test "parse rev date remark" do
    input = <<~'EOS'
    Ryan Waldron
    2013-12-18:  The first release you can stand on
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 8, metadata.size
    assert_equal '2013-12-18', metadata['revdate']
    assert_equal 'The first release you can stand on', metadata['revremark']
  end

"#
    );

    let doc = header("Ryan Waldron\n2013-12-18:  The first release you can stand on");
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("2013-12-18"));
    assert_eq!(
        attr(&doc, "revremark").as_deref(),
        Some("The first release you can stand on")
    );
}

#[test]
fn should_not_mistake_attribute_entry_as_rev_remark() {
    verifies!(
        r#"
  test "should not mistake attribute entry as rev remark" do
    input = <<~'EOS'
    Joe Cool
    :page-layout: post
    EOS
    metadata, _ = parse_header_metadata input
    refute_equal 'page-layout: post', metadata['revremark']
    refute metadata.key?('revdate')
  end

"#
    );

    let doc = header("Joe Cool\n:page-layout: post");
    assert!(doc.header().revision_line().is_none());
    assert_ne!(
        attr(&doc, "revremark").as_deref(),
        Some("page-layout: post")
    );
    assert_eq!(attr(&doc, "revdate"), None);
}

#[test]
fn parse_rev_remark_only() {
    verifies!(
        r#"
  test "parse rev remark only" do
    # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
    input = <<~EOS
    Joe Cool
     :Must start revremark-only line with space
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 'Must start revremark-only line with space', metadata['revremark']
    refute metadata.key?('revdate')
  end

"#
    );

    let doc = header("Joe Cool\n :Must start revremark-only line with space");
    assert_eq!(
        attr(&doc, "revremark").as_deref(),
        Some("Must start revremark-only line with space")
    );
    assert_eq!(attr(&doc, "revnumber"), None);
}

#[test]
fn skip_line_comments_before_author() {
    verifies!(
        r#"
  test "skip line comments before author" do
    input = <<~'EOS'
    // Asciidoctor
    // release artist
    Ryan Waldron
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Ryan Waldron', metadata['author']
    assert_equal 'Ryan', metadata['firstname']
    assert_equal 'Waldron', metadata['lastname']
    assert_equal 'RW', metadata['authorinitials']
  end

"#
    );

    let doc = header("// Asciidoctor\n// release artist\nRyan Waldron");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Ryan Waldron");
    assert_eq!(a.firstname(), "Ryan");
    assert_eq!(a.lastname(), Some("Waldron"));
    assert_eq!(a.initials(), "RW");
}

#[test]
fn skip_block_comment_before_author() {
    verifies!(
        r#"
  test "skip block comment before author" do
    input = <<~'EOS'
    ////
    Asciidoctor
    release artist
    ////
    Ryan Waldron
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 6, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Ryan Waldron', metadata['author']
    assert_equal 'Ryan', metadata['firstname']
    assert_equal 'Waldron', metadata['lastname']
    assert_equal 'RW', metadata['authorinitials']
  end

"#
    );

    let doc = header("////\nAsciidoctor\nrelease artist\n////\nRyan Waldron");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    let a = &doc.authors()[0];
    assert_eq!(a.name(), "Ryan Waldron");
    assert_eq!(a.firstname(), "Ryan");
    assert_eq!(a.lastname(), Some("Waldron"));
    assert_eq!(a.initials(), "RW");
}

#[test]
fn skip_block_comment_before_rev() {
    verifies!(
        r#"
  test "skip block comment before rev" do
    input = <<~'EOS'
    Ryan Waldron
    ////
    Asciidoctor
    release info
    ////
    v0.0.7, 2013-12-18
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 8, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Ryan Waldron', metadata['author']
    assert_equal '0.0.7', metadata['revnumber']
    assert_equal '2013-12-18', metadata['revdate']
  end

"#
    );

    let doc = header("Ryan Waldron\n////\nAsciidoctor\nrelease info\n////\nv0.0.7, 2013-12-18");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    assert_eq!(doc.authors()[0].name(), "Ryan Waldron");
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("0.0.7"));
    assert_eq!(attr(&doc, "revdate").as_deref(), Some("2013-12-18"));
}

#[test]
fn break_header_at_line_with_three_forward_slashes() {
    verifies!(
        r#"
  test 'break header at line with three forward slashes' do
    input = <<~'EOS'
    Joe Cool
    v1.0
    ///
    stuff
    EOS
    metadata, _ = parse_header_metadata input
    assert_equal 7, metadata.size
    assert_equal 1, metadata['authorcount']
    assert_equal 'Joe Cool', metadata['author']
    assert_equal '1.0', metadata['revnumber']
  end

"#
    );

    let doc = header("Joe Cool\nv1.0\n///\nstuff");
    assert_eq!(attr(&doc, "authorcount").as_deref(), Some("1"));
    assert_eq!(doc.authors()[0].name(), "Joe Cool");
    assert_eq!(attr(&doc, "revnumber").as_deref(), Some("1.0"));
}

#[test]
fn attribute_entry_overrides_generated_author_initials() {
    verifies!(
        r#"
  test 'attribute entry overrides generated author initials' do
    doc = empty_document
    metadata, _ = parse_header_metadata %(Stuart Rackham <founder@asciidoc.org>\n:Author Initials: SJR), doc
    assert_equal 'SR', metadata['authorinitials']
    assert_equal 'SJR', doc.attributes['authorinitials']
  end

"#
    );

    let doc = header("Stuart Rackham <founder@asciidoc.org>\n:Author Initials: SJR");
    // The name still partitions to the computed initials ...
    assert_eq!(doc.authors()[0].initials(), "SR");
    // ... but the explicit attribute entry wins for the document attribute.
    assert_eq!(attr(&doc, "authorinitials").as_deref(), Some("SJR"));
}

// The `adjust indentation` cases exercise the static
// `Parser.adjust_indentation!` over raw line arrays with explicit
// indent / tab-size arguments; this crate exposes no such knob.
// Verbatim-block indentation normalization is covered by `blocks_test`.
non_normative!(
    r#"
  test 'adjust indentation to 0' do
    input = <<~EOS
    \x20   def names

    \x20     @name.split

    \x20   end
    EOS

    # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
    expected = <<~EOS.chop
    def names

      @name.split

    end
    EOS

    lines = input.split ?\n
    Asciidoctor::Parser.adjust_indentation! lines
    assert_equal expected, (lines * ?\n)
  end

  test 'adjust indentation mixed with tabs and spaces to 0' do
    input = <<~EOS
        def names

    \t  @name.split

        end
    EOS

    expected = <<~EOS.chop
    def names

      @name.split

    end
    EOS

    lines = input.split ?\n
    Asciidoctor::Parser.adjust_indentation! lines, 0, 4
    assert_equal expected, (lines * ?\n)
  end

  test 'expands tabs to spaces' do
    input = <<~'EOS'
    Filesystem				Size	Used	Avail	Use%	Mounted on
    Filesystem              Size    Used    Avail   Use%    Mounted on
    devtmpfs				3.9G	   0	 3.9G	  0%	/dev
    /dev/mapper/fedora-root	 48G	 18G	  29G	 39%	/
    EOS

    expected = <<~'EOS'.chop
    Filesystem              Size    Used    Avail   Use%    Mounted on
    Filesystem              Size    Used    Avail   Use%    Mounted on
    devtmpfs                3.9G       0     3.9G     0%    /dev
    /dev/mapper/fedora-root  48G     18G      29G    39%    /
    EOS

    lines = input.split ?\n
    Asciidoctor::Parser.adjust_indentation! lines, 0, 4
    assert_equal expected, (lines * ?\n)
  end

  test 'adjust indentation to non-zero' do
    input = <<~EOS
    \x20   def names

    \x20     @name.split

    \x20   end
    EOS

    expected = <<~EOS.chop
    \x20 def names

    \x20   @name.split

    \x20 end
    EOS

    lines = input.split ?\n
    Asciidoctor::Parser.adjust_indentation! lines, 2
    assert_equal expected, (lines * ?\n)
  end

  test 'preserve block indent if indent is -1' do
    input = <<~EOS
    \x20   def names

    \x20     @name.split

    \x20   end
    EOS

    expected = input

    lines = input.lines
    Asciidoctor::Parser.adjust_indentation! lines, -1
    assert_equal expected, lines.join
  end

  test 'adjust indentation handles empty lines gracefully' do
    input = []
    expected = input

    lines = input.dup
    Asciidoctor::Parser.adjust_indentation! lines
    assert_equal expected, lines
  end

"#
);

#[test]
fn should_warn_if_inline_anchor_is_already_in_use() {
    verifies!(
        r#"
  test 'should warn if inline anchor is already in use' do
    input = <<~'EOS'
    [#in-use]
    A paragraph with an id.

    Another paragraph
    [[in-use]]that uses an id
    which is already in use.
    EOS

    using_memory_logger do |logger|
      document_from_string input
      assert_message logger, :WARN, '<stdin>: line 5: id assigned to anchor already in use: in-use', Hash
    end
  end
"#
    );

    let input = concat!(
        "[#in-use]\n",
        "A paragraph with an id.\n\n",
        "Another paragraph\n",
        "[[in-use]]that uses an id\n",
        "which is already in use.\n"
    );
    // Asciidoctor logs a WARN; this parser surfaces the same diagnostic in
    // the document's warnings inventory (its reported line is the start of
    // the offending paragraph rather than the anchor line).
    let n = load(input)
        .warnings()
        .filter(|w| matches!(&w.warning, WarningType::DuplicateId(id) if id == "in-use"))
        .count();
    assert_eq!(n, 1);
}

non_normative!(
    r#"
end
"#
);
