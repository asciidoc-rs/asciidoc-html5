//! Port of Asciidoctor's `preamble_test.rb`.
//!
//! Asciidoctor wraps the content before a document's first section in a
//! `<div id="preamble">` (only when the document has a title *and* at least one
//! section). This crate renders that same structure, so the HTML5 preamble
//! tests port directly, driven through `convert_with(..standalone(true)..)` —
//! the counterpart to the Ruby suite's standalone `convert_string`.
//!
//! Not ported (kept `non_normative!`): the DocBook-backend tests (this crate
//! targets only the `html5` backend), the `book` doctype / `partintro` cases
//! (not yet rendered here), and the `toc` case (TOC rendering is not wired up
//! yet — see <https://github.com/asciidoc-rs/asciidoc-html5/issues/86>).

use crate::{
    convert_with,
    tests::{assert_html::assert_xpath, sdd::*},
    Options,
};

track_file!("ref/asciidoctor/test/preamble_test.rb");

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'Preamble' do
"#
);

#[test]
fn title_and_single_paragraph_preamble_before_section() {
    verifies!(
        r#"
  test 'title and single paragraph preamble before section' do
    input = <<~'EOS'
    = Title

    Preamble paragraph 1.

    == First Section

    Section paragraph 1.
    EOS
    result = convert_string(input)
    assert_xpath '//p', result, 2
    assert_xpath '//*[@id="preamble"]', result, 1
    assert_xpath '//*[@id="preamble"]//p', result, 1
    assert_xpath '//*[@id="preamble"]/following-sibling::*//h2[@id="_first_section"]', result, 1
    assert_xpath '//*[@id="preamble"]/following-sibling::*//p', result, 1
  end

"#
    );

    let input = "= Title\n\nPreamble paragraph 1.\n\n== First Section\n\nSection paragraph 1.\n";
    let html = convert_with(input, &Options::new().standalone(true));
    assert_xpath(&html, "//p", 2);
    assert_xpath(&html, r#"//*[@id="preamble"]"#, 1);
    assert_xpath(&html, r#"//*[@id="preamble"]//p"#, 1);
    assert_xpath(
        &html,
        r#"//*[@id="preamble"]/following-sibling::*//h2[@id="_first_section"]"#,
        1,
    );
    assert_xpath(&html, r#"//*[@id="preamble"]/following-sibling::*//p"#, 1);
}

non_normative!(
    r#"
  test 'title of preface is blank by default in DocBook output' do
    input = <<~'EOS'
    = Document Title
    :doctype: book

    Preface content.

    == First Section

    Section content.
    EOS
    result = convert_string input, backend: :docbook
    assert_xpath '//preface/title', result, 1
    title_node = xmlnodes_at_xpath '//preface/title', result, 1
    assert_equal '', title_node.text
  end

  test 'preface-title attribute is assigned as title of preface in DocBook output' do
    input = <<~'EOS'
    = Document Title
    :doctype: book
    :preface-title: Preface

    Preface content.

    == First Section

    Section content.
    EOS
    result = convert_string input, backend: :docbook
    assert_xpath '//preface/title[text()="Preface"]', result, 1
  end

"#
);

#[test]
fn title_and_multi_paragraph_preamble_before_section() {
    verifies!(
        r#"
  test 'title and multi-paragraph preamble before section' do
    input = <<~'EOS'
    = Title

    Preamble paragraph 1.

    Preamble paragraph 2.

    == First Section

    Section paragraph 1.
    EOS
    result = convert_string(input)
    assert_xpath '//p', result, 3
    assert_xpath '//*[@id="preamble"]', result, 1
    assert_xpath '//*[@id="preamble"]//p', result, 2
    assert_xpath '//*[@id="preamble"]/following-sibling::*//h2[@id="_first_section"]', result, 1
    assert_xpath '//*[@id="preamble"]/following-sibling::*//p', result, 1
  end

"#
    );

    let input =
        "= Title\n\nPreamble paragraph 1.\n\nPreamble paragraph 2.\n\n== First Section\n\nSection paragraph 1.\n";
    let html = convert_with(input, &Options::new().standalone(true));
    assert_xpath(&html, "//p", 3);
    assert_xpath(&html, r#"//*[@id="preamble"]"#, 1);
    assert_xpath(&html, r#"//*[@id="preamble"]//p"#, 2);
    assert_xpath(
        &html,
        r#"//*[@id="preamble"]/following-sibling::*//h2[@id="_first_section"]"#,
        1,
    );
    assert_xpath(&html, r#"//*[@id="preamble"]/following-sibling::*//p"#, 1);
}

#[test]
fn should_not_wrap_content_in_preamble_if_document_has_title_but_no_sections() {
    verifies!(
        r#"
  test 'should not wrap content in preamble if document has title but no sections' do
    input = <<~'EOS'
    = Title

    paragraph
    EOS
    result = convert_string(input)
    assert_xpath '//p', result, 1
    assert_xpath '//*[@id="content"]/*[@class="paragraph"]/p', result, 1
    assert_xpath '//*[@id="content"]/*[@class="paragraph"]/following-sibling::*', result, 0
  end

"#
    );

    let input = "= Title\n\nparagraph\n";
    let html = convert_with(input, &Options::new().standalone(true));
    assert_xpath(&html, "//p", 1);
    assert_xpath(&html, r#"//*[@id="content"]/*[@class="paragraph"]/p"#, 1);
    assert_xpath(
        &html,
        r#"//*[@id="content"]/*[@class="paragraph"]/following-sibling::*"#,
        0,
    );
}

#[test]
fn title_and_section_without_preamble() {
    verifies!(
        r#"
  test 'title and section without preamble' do
    input = <<~'EOS'
    = Title

    == First Section

    Section paragraph 1.
    EOS
    result = convert_string(input)
    assert_xpath '//p', result, 1
    assert_xpath '//*[@id="preamble"]', result, 0
    assert_xpath '//h2[@id="_first_section"]', result, 1
  end

"#
    );

    let input = "= Title\n\n== First Section\n\nSection paragraph 1.\n";
    let html = convert_with(input, &Options::new().standalone(true));
    assert_xpath(&html, "//p", 1);
    assert_xpath(&html, r#"//*[@id="preamble"]"#, 0);
    assert_xpath(&html, r#"//h2[@id="_first_section"]"#, 1);
}

#[test]
fn no_title_with_preamble_and_section() {
    verifies!(
        r#"
  test 'no title with preamble and section' do
    input = <<~'EOS'
    Preamble paragraph 1.

    == First Section

    Section paragraph 1.
    EOS
    result = convert_string(input)
    assert_xpath '//p', result, 2
    assert_xpath '//*[@id="preamble"]', result, 0
    assert_xpath '//h2[@id="_first_section"]/preceding::p', result, 1
  end

"#
    );

    let input = "Preamble paragraph 1.\n\n== First Section\n\nSection paragraph 1.\n";
    let html = convert_with(input, &Options::new().standalone(true));
    assert_xpath(&html, "//p", 2);
    assert_xpath(&html, r#"//*[@id="preamble"]"#, 0);
    assert_xpath(&html, r#"//h2[@id="_first_section"]/preceding::p"#, 1);
}

non_normative!(
    r#"
  test 'preamble in book doctype' do
      input = <<~'EOS'
      = Book
      :doctype: book

      Back then...

      = Chapter One

      [partintro]
      It was a dark and stormy night...

      == Scene One

      Someone's gonna get axed.

      = Chapter Two

      [partintro]
      They couldn't believe their eyes when...

      == Scene One

      The axe came swinging.
      EOS

      d = document_from_string(input)
      assert_equal 'book', d.doctype
      output = d.convert
      assert_xpath '//h1', output, 3
      assert_xpath %{//*[@id="preamble"]//p[text() = "Back then#{decode_char 8230}#{decode_char 8203}"]}, output, 1
  end

  test 'should output table of contents in preamble if toc-placement attribute value is preamble' do
    input = <<~'EOS'
    = Article
    :toc:
    :toc-placement: preamble

    Once upon a time...

    == Section One

    It was a dark and stormy night...

    == Section Two

    They couldn't believe their eyes when...
    EOS

    output = convert_string input
    assert_xpath '//*[@id="preamble"]/*[@id="toc"]', output, 1
  end

  test 'should move abstract in implicit preface to info tag when converting to DocBook' do
    input = <<~'EOS'
    = Document Title

    [abstract]
    This is the abstract.

    == Fin
    EOS

    %w(article book).each do |doctype|
      output = convert_string input, backend: 'docbook', doctype: doctype
      assert_xpath '//abstract', output, 1
      assert_xpath %(/#{doctype}/info/abstract), output, 1
    end
  end

  test 'should move abstract as first section to info tag when converting to DocBook' do
    input = <<~'EOS'
    = Document Title

    [abstract]
    == Abstract

    This is the abstract.

    == Fin
    EOS

    output = convert_string input, backend: 'docbook'
    assert_xpath '//abstract', output, 1
    assert_xpath '/article/info/abstract', output, 1
  end

  test 'should move abstract in preface section to info tag when converting to DocBook' do
    input = <<~'EOS'
    = Document Title
    :doctype: book

    [preface]
    == Preface

    [abstract]
    This is the abstract.

    == Fin
    EOS

    output = convert_string input, backend: 'docbook'
    assert_xpath '//abstract', output, 1
    assert_xpath '/book/info/abstract', output, 1
    assert_xpath '//preface', output, 0
  end
end
"#
);
