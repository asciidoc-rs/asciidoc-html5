//! Port of Asciidoctor's `paragraphs_test.rb`.
//!
//! This crate renders normal, literal, listing, source, open, quote, verse, and
//! admonition paragraphs, so those tests port directly, driven through
//! `convert` (embedded, the counterpart to `convert_string_to_embedded`) or
//! `convert_with(..standalone(true)..)` (the counterpart to `convert_string`).
//!
//! Kept `non_normative!` are the tests this crate's stack cannot satisfy: the
//! DocBook-backend tests (this crate targets only the `html5` backend); the
//! verse escaped-brace subs test (`\{` is not unescaped by `asciidoc-parser`
//! yet); the preprocessor-conditional test (it needs `ifdef` handling and the
//! `asciidoctor-version` attribute); the inline doctype; and the custom-style
//! logging tests (this crate has no logger). The `[source]` parser-model
//! assertions (`block_from_string`) test `asciidoc-parser` internals; only the
//! rendered HTML of those tests is re-expressed here.

use crate::{
    convert, convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoctor/test/paragraphs_test.rb");

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'Paragraphs' do
"#
);

mod normal {
    use super::*;

    non_normative!(
        r#"
  context 'Normal' do
"#
    );

    #[test]
    fn should_treat_plain_text_separated_by_blank_lines_as_paragraphs() {
        verifies!(
            r#"
    test 'should treat plain text separated by blank lines as paragraphs' do
      input = <<~'EOS'
      Plain text for the win!

      Yep. Text. Plain and simple.
      EOS
      output = convert_string_to_embedded input
      assert_css 'p', output, 2
      assert_xpath '(//p)[1][text() = "Plain text for the win!"]', output, 1
      assert_xpath '(//p)[2][text() = "Yep. Text. Plain and simple."]', output, 1
    end

"#
        );

        let html = convert("Plain text for the win!\n\nYep. Text. Plain and simple.\n");
        assert_css(&html, "p", 2);
        assert_xpath(&html, r#"(//p)[1][text() = "Plain text for the win!"]"#, 1);
        assert_xpath(
            &html,
            r#"(//p)[2][text() = "Yep. Text. Plain and simple."]"#,
            1,
        );
    }

    #[test]
    fn should_associate_block_title_with_paragraph() {
        verifies!(
            r#"
    test 'should associate block title with paragraph' do
      input = <<~'EOS'
      .Titled
      Paragraph.

      Winning.
      EOS
      output = convert_string_to_embedded input

      assert_css 'p', output, 2
      assert_xpath '(//p)[1]/preceding-sibling::*[@class = "title"]', output, 1
      assert_xpath '(//p)[1]/preceding-sibling::*[@class = "title"][text() = "Titled"]', output, 1
      assert_xpath '(//p)[2]/preceding-sibling::*[@class = "title"]', output, 0
    end

"#
        );

        let html = convert(".Titled\nParagraph.\n\nWinning.\n");
        assert_css(&html, "p", 2);
        assert_xpath(
            &html,
            r#"(//p)[1]/preceding-sibling::*[@class = "title"]"#,
            1,
        );
        assert_xpath(
            &html,
            r#"(//p)[1]/preceding-sibling::*[@class = "title"][text() = "Titled"]"#,
            1,
        );
        assert_xpath(
            &html,
            r#"(//p)[2]/preceding-sibling::*[@class = "title"]"#,
            0,
        );
    }

    #[test]
    fn no_duplicate_block_before_next_section() {
        verifies!(
            r#"
    test 'no duplicate block before next section' do
      input = <<~'EOS'
      = Title

      Preamble

      == First Section

      Paragraph 1

      Paragraph 2

      == Second Section

      Last words
      EOS

      output = convert_string input
      assert_xpath '//p[text() = "Paragraph 2"]', output, 1
    end

"#
        );

        let input = "= Title\n\nPreamble\n\n== First Section\n\nParagraph 1\n\nParagraph 2\n\n== Second Section\n\nLast words\n";
        let html = convert_with(input, &Options::new().standalone(true));
        assert_xpath(&html, r#"//p[text() = "Paragraph 2"]"#, 1);
    }

    #[test]
    fn does_not_treat_wrapped_line_as_a_list_item() {
        verifies!(
            r#"
    test 'does not treat wrapped line as a list item' do
      input = <<~'EOS'
      paragraph
      . wrapped line
      EOS

      output = convert_string_to_embedded input
      assert_css 'p', output, 1
      assert_xpath %(//p[text()="paragraph\n. wrapped line"]), output, 1
    end

"#
        );

        let html = convert("paragraph\n. wrapped line\n");
        assert_css(&html, "p", 1);
        assert_xpath(&html, "//p[text()=\"paragraph\n. wrapped line\"]", 1);
    }

    #[test]
    fn does_not_treat_wrapped_line_as_a_block_title() {
        verifies!(
            r#"
    test 'does not treat wrapped line as a block title' do
      input = <<~'EOS'
      paragraph
      .wrapped line
      EOS

      output = convert_string_to_embedded input
      assert_css 'p', output, 1
      assert_xpath %(//p[text()="paragraph\n.wrapped line"]), output, 1
    end

"#
        );

        let html = convert("paragraph\n.wrapped line\n");
        assert_css(&html, "p", 1);
        assert_xpath(&html, "//p[text()=\"paragraph\n.wrapped line\"]", 1);
    }

    #[test]
    fn interprets_normal_paragraph_style_as_normal_paragraph() {
        verifies!(
            r#"
    test 'interprets normal paragraph style as normal paragraph' do
      input = <<~'EOS'
      [normal]
      Normal paragraph.
      Nothing special.
      EOS

      output = convert_string_to_embedded input
      assert_css 'p', output, 1
    end

"#
        );

        let html = convert("[normal]\nNormal paragraph.\nNothing special.\n");
        assert_css(&html, "p", 1);
    }

    #[test]
    fn removes_indentation_from_literal_paragraph_marked_as_normal() {
        verifies!(
            r#"
    test 'removes indentation from literal paragraph marked as normal' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      [normal]
        Normal paragraph.
          Nothing special.
        Last line.
      EOS

      output = convert_string_to_embedded input
      assert_css 'p', output, 1
      assert_xpath %(//p[text()="Normal paragraph.\n  Nothing special.\nLast line."]), output, 1
    end

"#
        );

        let html = convert("[normal]\n  Normal paragraph.\n    Nothing special.\n  Last line.\n");
        assert_css(&html, "p", 1);
        assert_xpath(
            &html,
            "//p[text()=\"Normal paragraph.\n  Nothing special.\nLast line.\"]",
            1,
        );
    }

    #[test]
    fn normal_paragraph_terminates_at_block_attribute_list() {
        verifies!(
            r#"
    test 'normal paragraph terminates at block attribute list' do
      input = <<~'EOS'
      normal text
      [literal]
      literal text
      EOS
      output = convert_string_to_embedded input
      assert_css '.paragraph:root', output, 1
      assert_css '.literalblock:root', output, 1
    end

"#
        );

        let html = convert("normal text\n[literal]\nliteral text\n");
        assert_css(&html, ".paragraph:root", 1);
        assert_css(&html, ".literalblock:root", 1);
    }

    #[test]
    fn normal_paragraph_terminates_at_block_delimiter() {
        verifies!(
            r#"
    test 'normal paragraph terminates at block delimiter' do
      input = <<~'EOS'
      normal text
      --
      text in open block
      --
      EOS
      output = convert_string_to_embedded input
      assert_css '.paragraph:root', output, 1
      assert_css '.openblock:root', output, 1
    end

"#
        );

        let html = convert("normal text\n--\ntext in open block\n--\n");
        assert_css(&html, ".paragraph:root", 1);
        assert_css(&html, ".openblock:root", 1);
    }

    #[test]
    fn normal_paragraph_terminates_at_list_continuation() {
        verifies!(
            r#"
    test 'normal paragraph terminates at list continuation' do
      input = <<~'EOS'
      normal text
      +
      EOS
      output = convert_string_to_embedded input
      assert_css '.paragraph:root', output, 2
      assert_xpath %((/*[@class="paragraph"])[1]/p[text() = "normal text"]), output, 1
      assert_xpath %((/*[@class="paragraph"])[2]/p[text() = "+"]), output, 1
    end

"#
        );

        let html = convert("normal text\n+\n");
        assert_css(&html, ".paragraph:root", 2);
        assert_xpath(
            &html,
            r#"(/*[@class="paragraph"])[1]/p[text() = "normal text"]"#,
            1,
        );
        assert_xpath(&html, r#"(/*[@class="paragraph"])[2]/p[text() = "+"]"#, 1);
    }

    #[test]
    fn normal_style_turns_literal_paragraph_into_normal_paragraph() {
        verifies!(
            r#"
    test 'normal style turns literal paragraph into normal paragraph' do
      input = <<~'EOS'
      [normal]
       normal paragraph,
       despite the leading indent
      EOS

      output = convert_string_to_embedded input
      assert_css '.paragraph:root > p', output, 1
    end

"#
        );

        let html = convert("[normal]\n normal paragraph,\n despite the leading indent\n");
        assert_css(&html, ".paragraph:root > p", 1);
    }

    // The index-term promotion tests assert on DocBook output, which this crate
    // does not produce.
    non_normative!(
        r#"
    test 'automatically promotes index terms in DocBook output if indexterm-promotion-option is set' do
      input = <<~'EOS'
      Here is an index entry for ((tigers)).
      indexterm:[Big cats,Tigers,Siberian Tiger]
      Here is an index entry for indexterm2:[Linux].
      (((Operating Systems,Linux)))
      Note that multi-entry terms generate separate index entries.
      EOS

      output = convert_string_to_embedded input, backend: 'docbook', attributes: { 'indexterm-promotion-option' => '' }
      assert_xpath '/simpara', output, 1
      term1 = xmlnodes_at_xpath '(//indexterm)[1]', output, 1
      assert_equal %(<indexterm>\n<primary>tigers</primary>\n</indexterm>), term1.to_s
      assert term1.next.content.start_with?('tigers')

      term2 = xmlnodes_at_xpath '(//indexterm)[2]', output, 1
      term2_elements = term2.elements
      assert_equal 3, term2_elements.size
      assert_equal '<primary>Big cats</primary>', term2_elements[0].to_s
      assert_equal '<secondary>Tigers</secondary>', term2_elements[1].to_s
      assert_equal '<tertiary>Siberian Tiger</tertiary>', term2_elements[2].to_s

      term3 = xmlnodes_at_xpath '(//indexterm)[3]', output, 1
      term3_elements = term3.elements
      assert_equal 2, term3_elements.size
      assert_equal '<primary>Tigers</primary>', term3_elements[0].to_s
      assert_equal '<secondary>Siberian Tiger</secondary>', term3_elements[1].to_s

      term4 = xmlnodes_at_xpath '(//indexterm)[4]', output, 1
      term4_elements = term4.elements
      assert_equal 1, term4_elements.size
      assert_equal '<primary>Siberian Tiger</primary>', term4_elements[0].to_s

      term5 = xmlnodes_at_xpath '(//indexterm)[5]', output, 1
      assert_equal %(<indexterm>\n<primary>Linux</primary>\n</indexterm>), term5.to_s
      assert term5.next.content.start_with?('Linux')

      assert_xpath '(//indexterm)[6]/*', output, 2
      assert_xpath '(//indexterm)[7]/*', output, 1
    end

    test 'does not automatically promote index terms in DocBook output if indexterm-promotion-option is not set' do
      input = <<~'EOS'
      The Siberian Tiger is one of the biggest living cats.
      indexterm:[Big cats,Tigers,Siberian Tiger]
      Note that multi-entry terms generate separate index entries.
      (((Operating Systems,Linux)))
      EOS

      output = convert_string_to_embedded input, backend: 'docbook'

      assert_css 'indexterm', output, 2

      terms = xmlnodes_at_css 'indexterm', output, 2
      term1 = terms[0]
      term1_elements = term1.elements
      assert_equal 3, term1_elements.size
      assert_equal '<primary>Big cats</primary>', term1_elements[0].to_s
      assert_equal '<secondary>Tigers</secondary>', term1_elements[1].to_s
      assert_equal '<tertiary>Siberian Tiger</tertiary>', term1_elements[2].to_s
      term2 = terms[1]
      term2_elements = term2.elements
      assert_equal 2, term2_elements.size
      assert_equal '<primary>Operating Systems</primary>', term2_elements[0].to_s
      assert_equal '<secondary>Linux</secondary>', term2_elements[1].to_s
    end

"#
    );

    #[test]
    fn normal_paragraph_should_honor_explicit_subs_list() {
        verifies!(
            r#"
    test 'normal paragraph should honor explicit subs list' do
      input = <<~'EOS'
      [subs="specialcharacters"]
      *<Hey Jude>*
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '*&lt;Hey Jude&gt;*'
    end

"#
        );

        let html = convert("[subs=\"specialcharacters\"]\n*<Hey Jude>*\n");
        assert!(html.contains("*&lt;Hey Jude&gt;*"));
    }

    #[test]
    fn normal_paragraph_should_honor_specialchars_shorthand() {
        verifies!(
            r#"
    test 'normal paragraph should honor specialchars shorthand' do
      input = <<~'EOS'
      [subs="specialchars"]
      *<Hey Jude>*
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '*&lt;Hey Jude&gt;*'
    end

"#
        );

        let html = convert("[subs=\"specialchars\"]\n*<Hey Jude>*\n");
        assert!(html.contains("*&lt;Hey Jude&gt;*"));
    }

    #[test]
    fn should_add_a_hardbreak_at_end_of_each_line_when_hardbreaks_option_is_set() {
        verifies!(
            r#"
    test 'should add a hardbreak at end of each line when hardbreaks option is set' do
      input = <<~'EOS'
      [%hardbreaks]
      read
      my
      lips
      EOS

      output = convert_string_to_embedded input
      assert_css 'br', output, 2
      assert_xpath '//p', output, 1
      assert_includes output, "<p>read<br>\nmy<br>\nlips</p>"
    end

"#
        );

        let html = convert("[%hardbreaks]\nread\nmy\nlips\n");
        assert_css(&html, "br", 2);
        assert_xpath(&html, "//p", 1);
        assert!(html.contains("<p>read<br>\nmy<br>\nlips</p>"));
    }

    #[test]
    fn should_be_able_to_toggle_hardbreaks_by_setting_hardbreaks_option_on_document() {
        verifies!(
            r#"
    test 'should be able to toggle hardbreaks by setting hardbreaks-option on document' do
      input = <<~'EOS'
      :hardbreaks-option:

      make
      it
      so

      :!hardbreaks:

      roll it back
      EOS

      output = convert_string_to_embedded input
      assert_xpath '(//p)[1]/br', output, 2
      assert_xpath '(//p)[2]/br', output, 0
    end
"#
        );

        let html =
            convert(":hardbreaks-option:\n\nmake\nit\nso\n\n:!hardbreaks:\n\nroll it back\n");
        assert_xpath(&html, "(//p)[1]/br", 2);
        assert_xpath(&html, "(//p)[2]/br", 0);
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod literal {
    use super::*;

    non_normative!(
        r#"
  context 'Literal' do
"#
    );

    #[test]
    fn single_line_literal_paragraphs() {
        verifies!(
            r#"
    test 'single-line literal paragraphs' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      you know what?

       LITERALS

       ARE LITERALLY

       AWESOME!
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//pre', output, 3
    end

"#
        );

        let html = convert("you know what?\n\n LITERALS\n\n ARE LITERALLY\n\n AWESOME!\n");
        assert_xpath(&html, "//pre", 3);
    }

    #[test]
    fn multi_line_literal_paragraph() {
        verifies!(
            r#"
    test 'multi-line literal paragraph' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      Install instructions:

       yum install ruby rubygems
       gem install asciidoctor

      You're good to go!
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//pre', output, 1
      # indentation should be trimmed from literal block
      assert_xpath %(//pre[text() = "yum install ruby rubygems\ngem install asciidoctor"]), output, 1
    end

"#
        );

        let html =
            convert("Install instructions:\n\n yum install ruby rubygems\n gem install asciidoctor\n\nYou're good to go!\n");
        assert_xpath(&html, "//pre", 1);
        assert_xpath(
            &html,
            "//pre[text() = \"yum install ruby rubygems\ngem install asciidoctor\"]",
            1,
        );
    }

    #[test]
    fn literal_paragraph() {
        verifies!(
            r#"
    test 'literal paragraph' do
      input = <<~'EOS'
      [literal]
      this text is literally literal
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="literalblock"]//pre[text()="this text is literally literal"]), output, 1
    end

"#
        );

        let html = convert("[literal]\nthis text is literally literal\n");
        assert_xpath(
            &html,
            r#"/*[@class="literalblock"]//pre[text()="this text is literally literal"]"#,
            1,
        );
    }

    #[test]
    fn should_read_content_below_literal_style_verbatim() {
        verifies!(
            r#"
    test 'should read content below literal style verbatim' do
      input = <<~'EOS'
      [literal]
      image::not-an-image-block[]
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="literalblock"]//pre[text()="image::not-an-image-block[]"]), output, 1
      assert_css 'img', output, 0
    end

"#
        );

        let html = convert("[literal]\nimage::not-an-image-block[]\n");
        assert_xpath(
            &html,
            r#"/*[@class="literalblock"]//pre[text()="image::not-an-image-block[]"]"#,
            1,
        );
        assert_css(&html, "img", 0);
    }

    #[test]
    fn listing_paragraph() {
        verifies!(
            r#"
    test 'listing paragraph' do
      input = <<~'EOS'
      [listing]
      this text is a listing
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="listingblock"]//pre[text()="this text is a listing"]), output, 1
    end

"#
        );

        let html = convert("[listing]\nthis text is a listing\n");
        assert_xpath(
            &html,
            r#"/*[@class="listingblock"]//pre[text()="this text is a listing"]"#,
            1,
        );
    }

    #[test]
    fn source_paragraph() {
        verifies!(
            r#"
    test 'source paragraph' do
      input = <<~'EOS'
      [source]
      use the source, luke!
      EOS
      block = block_from_string input
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :paragraph, (block.attr 'cloaked-context')
      assert_nil (block.attr 'language')
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="listingblock"]//pre[@class="highlight"]/code[text()="use the source, luke!"]), output, 1
    end

"#
        );

        // The `block_from_string` parser-model assertions test `asciidoc-parser`
        // internals, not this crate; only the HTML output is re-expressed here.
        let html = convert("[source]\nuse the source, luke!\n");
        assert_xpath(
            &html,
            r#"/*[@class="listingblock"]//pre[@class="highlight"]/code[text()="use the source, luke!"]"#,
            1,
        );
    }

    #[test]
    fn source_code_paragraph_with_language() {
        verifies!(
            r#"
    test 'source code paragraph with language' do
      input = <<~'EOS'
      [source, perl]
      die 'zomg perl is tough';
      EOS
      block = block_from_string input
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :paragraph, (block.attr 'cloaked-context')
      assert_equal 'perl', (block.attr 'language')
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="listingblock"]//pre[@class="highlight"]/code[@class="language-perl"][@data-lang="perl"][text()="die 'zomg perl is tough';"]), output, 1
    end

"#
        );

        // As above, only the rendered HTML is checked, not the parser-model
        // attributes the Ruby test also asserts.
        let html = convert("[source, perl]\ndie 'zomg perl is tough';\n");
        assert_xpath(
            &html,
            r#"/*[@class="listingblock"]//pre[@class="highlight"]/code[@class="language-perl"][@data-lang="perl"][text()="die 'zomg perl is tough';"]"#,
            1,
        );
    }

    #[test]
    fn literal_paragraph_terminates_at_block_attribute_list() {
        verifies!(
            r#"
    test 'literal paragraph terminates at block attribute list' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
       literal text
      [normal]
      normal text
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="literalblock"]), output, 1
      assert_xpath %(/*[@class="paragraph"]), output, 1
    end

"#
        );

        let html = convert(" literal text\n[normal]\nnormal text\n");
        assert_xpath(&html, r#"/*[@class="literalblock"]"#, 1);
        assert_xpath(&html, r#"/*[@class="paragraph"]"#, 1);
    }

    #[test]
    fn literal_paragraph_terminates_at_block_delimiter() {
        verifies!(
            r#"
    test 'literal paragraph terminates at block delimiter' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
       literal text
      --
      normal text
      --
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="literalblock"]), output, 1
      assert_xpath %(/*[@class="openblock"]), output, 1
    end

"#
        );

        let html = convert(" literal text\n--\nnormal text\n--\n");
        assert_xpath(&html, r#"/*[@class="literalblock"]"#, 1);
        assert_xpath(&html, r#"/*[@class="openblock"]"#, 1);
    }

    #[test]
    fn literal_paragraph_terminates_at_list_continuation() {
        verifies!(
            r#"
    test 'literal paragraph terminates at list continuation' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
       literal text
      +
      EOS
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="literalblock"]), output, 1
      assert_xpath %(/*[@class="literalblock"]//pre[text() = "literal text"]), output, 1
      assert_xpath %(/*[@class="paragraph"]), output, 1
      assert_xpath %(/*[@class="paragraph"]/p[text() = "+"]), output, 1
    end
"#
        );

        let html = convert(" literal text\n+\n");
        assert_xpath(&html, r#"/*[@class="literalblock"]"#, 1);
        assert_xpath(
            &html,
            r#"/*[@class="literalblock"]//pre[text() = "literal text"]"#,
            1,
        );
        assert_xpath(&html, r#"/*[@class="paragraph"]"#, 1);
        assert_xpath(&html, r#"/*[@class="paragraph"]/p[text() = "+"]"#, 1);
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod quote {
    use super::*;

    non_normative!(
        r#"
  context 'Quote' do
"#
    );

    #[test]
    fn single_line_quote_paragraph() {
        verifies!(
            r#"
    test "single-line quote paragraph" do
      input = <<~'EOS'
      [quote]
      Famous quote.
      EOS
      output = convert_string input
      assert_xpath '//*[@class = "quoteblock"]', output, 1
      assert_xpath '//*[@class = "quoteblock"]//p', output, 0
      assert_xpath '//*[@class = "quoteblock"]//*[contains(text(), "Famous quote.")]', output, 1
    end

"#
        );

        let html = convert_with("[quote]\nFamous quote.\n", &Options::new().standalone(true));
        assert_xpath(&html, r#"//*[@class = "quoteblock"]"#, 1);
        assert_xpath(&html, r#"//*[@class = "quoteblock"]//p"#, 0);
        assert_xpath(
            &html,
            r#"//*[@class = "quoteblock"]//*[contains(text(), "Famous quote.")]"#,
            1,
        );
    }

    #[test]
    fn quote_paragraph_terminates_at_list_continuation() {
        verifies!(
            r#"
    test 'quote paragraph terminates at list continuation' do
      input = <<~'EOS'
      [quote]
      A famouse quote.
      +
      EOS
      output = convert_string_to_embedded input
      assert_css '.quoteblock:root', output, 1
      assert_css '.paragraph:root', output, 1
      assert_xpath %(/*[@class="paragraph"]/p[text() = "+"]), output, 1
    end

"#
        );

        let html = convert("[quote]\nA famouse quote.\n+\n");
        assert_css(&html, ".quoteblock:root", 1);
        assert_css(&html, ".paragraph:root", 1);
        assert_xpath(&html, r#"/*[@class="paragraph"]/p[text() = "+"]"#, 1);
    }

    #[test]
    fn verse_paragraph() {
        verifies!(
            r#"
    test "verse paragraph" do
      output = convert_string("[verse]\nFamous verse.")
      assert_xpath '//*[@class = "verseblock"]', output, 1
      assert_xpath '//*[@class = "verseblock"]/pre', output, 1
      assert_xpath '//*[@class = "verseblock"]//p', output, 0
      assert_xpath '//*[@class = "verseblock"]/pre[normalize-space(text()) = "Famous verse."]', output, 1
    end

"#
        );

        let html = convert_with("[verse]\nFamous verse.", &Options::new().standalone(true));
        assert_xpath(&html, r#"//*[@class = "verseblock"]"#, 1);
        assert_xpath(&html, r#"//*[@class = "verseblock"]/pre"#, 1);
        assert_xpath(&html, r#"//*[@class = "verseblock"]//p"#, 0);
        assert_xpath(
            &html,
            r#"//*[@class = "verseblock"]/pre[normalize-space(text()) = "Famous verse."]"#,
            1,
        );
    }

    // `\{group-id\}` should render as `{group-id}`, but `asciidoc-parser` does
    // not yet unescape `\{`, so the expected substitution output differs.
    non_normative!(
        r##"
    test 'should perform normal subs on a verse paragraph' do
      input = <<~'EOS'
      [verse]
      _GET /groups/link:#group-id[\{group-id\}]_
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '<pre class="content"><em>GET /groups/<a href="#group-id">{group-id}</a></em></pre>'
    end

"##
    );

    #[test]
    fn quote_paragraph_should_honor_explicit_subs_list() {
        verifies!(
            r#"
    test 'quote paragraph should honor explicit subs list' do
      input = <<~'EOS'
      [subs="specialcharacters"]
      [quote]
      *Hey Jude*
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '*Hey Jude*'
    end
"#
        );

        let html = convert("[subs=\"specialcharacters\"]\n[quote]\n*Hey Jude*\n");
        assert!(html.contains("*Hey Jude*"));
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod special {
    use super::*;

    // Asciidoctor::ADMONITION_STYLES, as (style, css-name) pairs.
    const ADMONITION_STYLES: [(&str, &str); 5] = [
        ("NOTE", "note"),
        ("TIP", "tip"),
        ("IMPORTANT", "important"),
        ("WARNING", "warning"),
        ("CAUTION", "caution"),
    ];

    non_normative!(
        r#"
  context "special" do
"#
    );

    #[test]
    fn note_multiline_syntax() {
        verifies!(
            r#"
    test "note multiline syntax" do
      Asciidoctor::ADMONITION_STYLES.each do |style|
        assert_xpath "//div[@class='admonitionblock #{style.downcase}']", convert_string("[#{style}]\nThis is a winner.")
      end
    end

"#
        );

        for (style, name) in ADMONITION_STYLES {
            let html = convert_with(
                &format!("[{style}]\nThis is a winner."),
                &Options::new().standalone(true),
            );
            assert_xpath(&html, &format!("//div[@class='admonitionblock {name}']"), 1);
        }
    }

    #[test]
    fn note_block_syntax() {
        verifies!(
            r#"
    test "note block syntax" do
      Asciidoctor::ADMONITION_STYLES.each do |style|
        assert_xpath "//div[@class='admonitionblock #{style.downcase}']", convert_string("[#{style}]\n====\nThis is a winner.\n====")
      end
    end

"#
        );

        for (style, name) in ADMONITION_STYLES {
            let html = convert_with(
                &format!("[{style}]\n====\nThis is a winner.\n===="),
                &Options::new().standalone(true),
            );
            assert_xpath(&html, &format!("//div[@class='admonitionblock {name}']"), 1);
        }
    }

    #[test]
    fn note_inline_syntax() {
        verifies!(
            r##"
    test "note inline syntax" do
      Asciidoctor::ADMONITION_STYLES.each do |style|
        assert_xpath "//div[@class='admonitionblock #{style.downcase}']", convert_string("#{style}: This is important, fool!")
      end
    end

"##
        );

        for (style, name) in ADMONITION_STYLES {
            let html = convert_with(
                &format!("{style}: This is important, fool!"),
                &Options::new().standalone(true),
            );
            assert_xpath(&html, &format!("//div[@class='admonitionblock {name}']"), 1);
        }
    }

    // Requires `ifdef` preprocessing and the `asciidoctor-version` attribute,
    // neither of which this stack provides.
    non_normative!(
        r#"
    test 'should process preprocessor conditional in paragraph content' do
      input = <<~'EOS'
      ifdef::asciidoctor-version[]
      [sidebar]
      First line of sidebar.
      ifdef::backend[The backend is {backend}.]
      Last line of sidebar.
      endif::[]
      EOS

      expected = <<~'EOS'.chop
      <div class="sidebarblock">
      <div class="content">
      First line of sidebar.
      The backend is html5.
      Last line of sidebar.
      </div>
      </div>
      EOS

      result = convert_string_to_embedded input
      assert_equal expected, result
    end

"#
    );

    mod styled_paragraphs {
        use super::*;

        // DocBook output is out of scope.
        non_normative!(
            r#"
    context 'Styled Paragraphs' do
      test 'should wrap text in simpara for styled paragraphs when converted to DocBook' do
        input = <<~'EOS'
        = Book
        :doctype: book

        [preface]
        = About this book

        [abstract]
        An abstract for the book.

        = Part 1

        [partintro]
        An intro to this part.

        == Chapter 1

        [sidebar]
        Just a side note.

        [example]
        As you can see here.

        [quote]
        Wise words from a wise person.

        [open]
        Make it what you want.
        EOS

        output = convert_string input, backend: 'docbook'
        assert_css 'abstract > simpara', output, 1
        assert_css 'partintro > simpara', output, 1
        assert_css 'sidebar > simpara', output, 1
        assert_css 'informalexample > simpara', output, 1
        assert_css 'blockquote > simpara', output, 1
        assert_css 'chapter > simpara', output, 1
      end

"#
        );

        #[test]
        fn should_convert_open_paragraph_to_open_block() {
            verifies!(
                r#"
      test 'should convert open paragraph to open block' do
        input = <<~'EOS'
        [open]
        Make it what you want.
        EOS

        output = convert_string_to_embedded input
        assert_css '.openblock', output, 1
        assert_css '.openblock p', output, 0
      end

"#
            );

            let html = convert("[open]\nMake it what you want.\n");
            assert_css(&html, ".openblock", 1);
            assert_css(&html, ".openblock p", 0);
        }

        // DocBook output is out of scope.
        non_normative!(
            r#"
      test 'should wrap text in simpara for styled paragraphs with title when converted to DocBook' do
        input = <<~'EOS'
        = Book
        :doctype: book

        [preface]
        = About this book

        [abstract]
        .Abstract title
        An abstract for the book.

        = Part 1

        [partintro]
        .Part intro title
        An intro to this part.

        == Chapter 1

        [sidebar]
        .Sidebar title
        Just a side note.

        [example]
        .Example title
        As you can see here.

        [quote]
        .Quote title
        Wise words from a wise person.
        EOS

        output = convert_string input, backend: 'docbook'
        assert_css 'abstract > title', output, 1
        assert_xpath '//abstract/title[text() = "Abstract title"]', output, 1
        assert_css 'abstract > title + simpara', output, 1
        assert_css 'partintro > title', output, 1
        assert_xpath '//partintro/title[text() = "Part intro title"]', output, 1
        assert_css 'partintro > title + simpara', output, 1
        assert_css 'sidebar > title', output, 1
        assert_xpath '//sidebar/title[text() = "Sidebar title"]', output, 1
        assert_css 'sidebar > title + simpara', output, 1
        assert_css 'example > title', output, 1
        assert_xpath '//example/title[text() = "Example title"]', output, 1
        assert_css 'example > title + simpara', output, 1
        assert_css 'blockquote > title', output, 1
        assert_xpath '//blockquote/title[text() = "Quote title"]', output, 1
        assert_css 'blockquote > title + simpara', output, 1
      end
"#
        );

        non_normative!(
            r#"
    end

"#
        );
    }

    // The inline doctype is not supported.
    mod inline_doctype {
        use super::*;

        non_normative!(
            r#"
    context 'Inline doctype' do
      test 'should only format and output text in first paragraph when doctype is inline' do
        input = "http://asciidoc.org[AsciiDoc] is a _lightweight_ markup language...\n\nignored"
        output = convert_string input, doctype: 'inline'
        assert_equal '<a href="http://asciidoc.org">AsciiDoc</a> is a <em>lightweight</em> markup language&#8230;&#8203;', output
      end

      test 'should output nil and warn if first block is not a paragraph' do
        input = '* bullet'
        using_memory_logger do |logger|
          output = convert_string input, doctype: 'inline'
          assert_nil output
          assert_message logger, :WARN, '~no inline candidate'
        end
      end
    end
"#
        );
    }

    non_normative!(
        r#"
  end

"#
    );
}

// The custom-style tests assert on logger messages; this crate has no logger.
mod custom {
    use super::*;

    non_normative!(
        r#"
  context 'Custom' do
    test 'should not warn if paragraph style is unregisted' do
      input = <<~'EOS'
      [foo]
      bar
      EOS
      using_memory_logger do |logger|
        convert_string_to_embedded input
        assert_empty logger.messages
      end
    end

    test 'should log debug message if paragraph style is unknown and debug level is enabled' do
      input = <<~'EOS'
      [foo]
      bar
      EOS
      using_memory_logger Logger::Severity::DEBUG do |logger|
        convert_string_to_embedded input
        assert_message logger, :DEBUG, '<stdin>: line 2: unknown style for paragraph: foo', Hash
      end
    end
  end
"#
    );
}

non_normative!(
    r#"
end
"#
);
