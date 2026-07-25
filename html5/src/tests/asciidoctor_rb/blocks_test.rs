//! Port of Asciidoctor's `blocks_test.rb` — **front half only** (through the
//! `Open Blocks` context, source lines 1–1748).
//!
//! This crate already renders the block types the front half exercises — layout
//! breaks, comments, sidebar/quote/verse/example/admonition/open blocks, and
//! verbatim (listing/literal/source) blocks — so those contexts port directly,
//! driven through `convert` (embedded) / `convert_with(..standalone(true)..)`.
//!
//! The back half (Passthrough, Math, Images, Media, Admonition icons, Source
//! code, Abstract/Part Intro, Substitutions, References — lines 1749+) hits
//! block types this renderer does not implement yet and is deliberately not
//! reproduced here; it is being sequenced as its own implement-then-port work.
//!
//! What stays `non_normative!` in the front half:
//! - DocBook-backend tests (this crate targets only the `html5` backend);
//! - `asciidoc-parser` parser-model assertions (`document_from_string` +
//!   `blocks[..].numeral`/`content`/`subs`, `find_by`, `block_from_string`) —
//!   only the rendered HTML of such tests is re-expressed here;
//! - the `markdown_syntax` compliance-toggle test (no compliance API here);
//! - deferred features, each tracked by an issue: example captions/counters (<https://github.com/asciidoc-rs/asciidoc-html5/issues/113>),
//!   collapsible examples (#114).
//!
//! Logger assertions (`assert_message @logger, :WARN, …`) are verified against
//! the document's warnings inventory via [`assert_warning`].

use asciidoc_parser::{
    blocks::{FindBlocks, IsBlock},
    warnings::WarningType,
};

use crate::{
    convert, convert_with, load,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options,
};

track_file!("ref/asciidoctor/test/blocks_test.rb");

/// Asserts that loading `input` surfaces exactly one warning matching `pred` on
/// document line `line` — the parse-model counterpart to Asciidoctor's
/// `assert_message @logger, :WARN, '<stdin>: line N: …'` (the parser reflects
/// these diagnostics in the document's warnings inventory).
fn assert_warning(input: &str, line: usize, pred: impl Fn(&WarningType) -> bool) {
    let doc = load(input);
    let count = doc
        .warnings()
        .filter(|w| w.source.line() == line && pred(&w.warning))
        .count();
    assert_eq!(
        count, 1,
        "expected exactly one matching warning at line {line}"
    );
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'Blocks' do
  default_logger = Asciidoctor::LoggerManager.logger

  setup do
    Asciidoctor::LoggerManager.logger = (@logger = Asciidoctor::MemoryLogger.new)
  end

  teardown do
    Asciidoctor::LoggerManager.logger = default_logger
  end

"#
);

mod layout_breaks {
    use super::*;

    non_normative!(
        r#"
  context 'Layout Breaks' do
"#
    );

    #[test]
    fn horizontal_rule() {
        verifies!(
            r#"
    test 'horizontal rule' do
      %w(''' '''' '''''').each do |line|
        output = convert_string_to_embedded line
        assert_includes output, '<hr>'
      end
    end

"#
        );

        for line in ["'''", "''''", "'''''"] {
            let output = convert(line);
            assert!(output.contains("<hr>"), "{line}");
        }
    }

    // Toggling `Asciidoctor::Compliance.markdown_syntax` is not supported here
    // (this crate has no compliance API), so the disabled-syntax half of this
    // test cannot be reproduced. Markdown-style breaks (`---`/`***`/`___`) are
    // always recognized.
    non_normative!(
        r#"
    test 'horizontal rule with markdown syntax disabled' do
      old_markdown_syntax = Asciidoctor::Compliance.markdown_syntax
      begin
        Asciidoctor::Compliance.markdown_syntax = false
        %w(''' '''' '''''').each do |line|
          output = convert_string_to_embedded line
          assert_includes output, '<hr>'
        end
        %w(--- *** ___).each do |line|
          output = convert_string_to_embedded line
          refute_includes output, '<hr>'
        end
      ensure
        Asciidoctor::Compliance.markdown_syntax = old_markdown_syntax
      end
    end

"#
    );

    #[test]
    fn lt_3_chars_does_not_make_horizontal_rule() {
        verifies!(
            r#"
    test '< 3 chars does not make horizontal rule' do
      %w(' '').each do |line|
        output = convert_string_to_embedded line
        refute_includes output, '<hr>'
        assert_includes output, %(<p>#{line}</p>)
      end
    end

"#
        );

        for line in ["'", "''"] {
            let output = convert(line);
            assert!(!output.contains("<hr>"));
            assert!(output.contains(&format!("<p>{line}</p>")));
        }
    }

    #[test]
    fn mixed_chars_does_not_make_horizontal_rule() {
        verifies!(
            r#"
    test 'mixed chars does not make horizontal rule' do
      [%q(''<), %q('''<), %q(' ' ')].each do |line|
        output = convert_string_to_embedded line
        refute_includes output, '<hr>'
        assert_includes output, %(<p>#{line.sub '<', '&lt;'}</p>)
      end
    end

"#
        );

        for line in ["''<", "'''<", "' ' '"] {
            let output = convert(line);
            assert!(!output.contains("<hr>"));
            let expected = line.replacen('<', "&lt;", 1);
            assert!(output.contains(&format!("<p>{expected}</p>")));
        }
    }

    #[test]
    fn horizontal_rule_between_blocks() {
        verifies!(
            r#"
    test 'horizontal rule between blocks' do
      output = convert_string_to_embedded %(Block above\n\n'''\n\nBlock below)
      assert_xpath '/hr', output, 1
      assert_xpath '/hr/preceding-sibling::*', output, 1
      assert_xpath '/hr/following-sibling::*', output, 1
    end

"#
        );

        let output = convert("Block above\n\n'''\n\nBlock below");
        assert_xpath(&output, "/hr", 1);
        assert_xpath(&output, "/hr/preceding-sibling::*", 1);
        assert_xpath(&output, "/hr/following-sibling::*", 1);
    }

    #[test]
    fn page_break() {
        verifies!(
            r#"
    test 'page break' do
      output = convert_string_to_embedded %(page 1\n\n<<<\n\npage 2)
      assert_xpath '/*[translate(@style, ";", "")="page-break-after: always"]', output, 1
      assert_xpath '/*[translate(@style, ";", "")="page-break-after: always"]/preceding-sibling::div/p[text()="page 1"]', output, 1
      assert_xpath '/*[translate(@style, ";", "")="page-break-after: always"]/following-sibling::div/p[text()="page 2"]', output, 1
    end
"#
        );

        // The renderer emits a deterministic `style="page-break-after: always;"`,
        // so the Ruby `translate(@style, ";", "")` normalization is re-expressed
        // as an exact attribute match on that value.
        let output = convert("page 1\n\n<<<\n\npage 2");
        assert_xpath(&output, r#"/*[@style="page-break-after: always;"]"#, 1);
        assert_xpath(
            &output,
            r#"/*[@style="page-break-after: always;"]/preceding-sibling::div/p[text()="page 1"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"/*[@style="page-break-after: always;"]/following-sibling::div/p[text()="page 2"]"#,
            1,
        );
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod comments {
    use super::*;

    non_normative!(
        r#"
  context 'Comments' do
"#
    );

    #[test]
    fn line_comment_between_paragraphs_offset_by_blank_lines() {
        verifies!(
            r#"
    test 'line comment between paragraphs offset by blank lines' do
      input = <<~'EOS'
      first paragraph

      // line comment

      second paragraph
      EOS
      output = convert_string_to_embedded input
      refute_match(/line comment/, output)
      assert_xpath '//p', output, 2
    end

"#
        );

        // A lone `// line comment` survives parsing as an empty paragraph, which
        // the renderer drops (see `renderer::renders_nothing`), so only the two
        // real paragraphs remain.
        let output = convert("first paragraph\n\n// line comment\n\nsecond paragraph\n");
        assert!(!output.contains("line comment"));
        assert_xpath(&output, "//p", 2);
    }

    #[test]
    fn adjacent_line_comment_between_paragraphs() {
        verifies!(
            r#"
    test 'adjacent line comment between paragraphs' do
      input = <<~'EOS'
      first line
      // line comment
      second line
      EOS
      output = convert_string_to_embedded input
      refute_match(/line comment/, output)
      assert_xpath '//p', output, 1
      assert_xpath "//p[1][text()='first line\nsecond line']", output, 1
    end

"#
        );

        let output = convert("first line\n// line comment\nsecond line\n");
        assert!(!output.contains("line comment"));
        assert_xpath(&output, "//p", 1);
        assert_xpath(&output, "//p[1][text()='first line\nsecond line']", 1);
    }

    #[test]
    fn comment_block_between_paragraphs_offset_by_blank_lines() {
        verifies!(
            r#"
    test 'comment block between paragraphs offset by blank lines' do
      input = <<~'EOS'
      first paragraph

      ////
      block comment
      ////

      second paragraph
      EOS
      output = convert_string_to_embedded input
      refute_match(/block comment/, output)
      assert_xpath '//p', output, 2
    end

"#
        );

        let output = convert("first paragraph\n\n////\nblock comment\n////\n\nsecond paragraph\n");
        assert!(!output.contains("block comment"));
        assert_xpath(&output, "//p", 2);
    }

    #[test]
    fn comment_block_between_paragraphs_offset_by_blank_lines_inside_delimited_block() {
        verifies!(
            r#"
    test 'comment block between paragraphs offset by blank lines inside delimited block' do
      input = <<~'EOS'
      ====
      first paragraph

      ////
      block comment
      ////

      second paragraph
      ====
      EOS
      output = convert_string_to_embedded input
      refute_match(/block comment/, output)
      assert_xpath '//p', output, 2
    end

"#
        );

        let output = convert(
            "====\nfirst paragraph\n\n////\nblock comment\n////\n\nsecond paragraph\n====\n",
        );
        assert!(!output.contains("block comment"));
        assert_xpath(&output, "//p", 2);
    }

    #[test]
    fn adjacent_comment_block_between_paragraphs() {
        verifies!(
            r#"
    test 'adjacent comment block between paragraphs' do
      input = <<~'EOS'
      first paragraph
      ////
      block comment
      ////
      second paragraph
      EOS
      output = convert_string_to_embedded input
      refute_match(/block comment/, output)
      assert_xpath '//p', output, 2
    end

"#
        );

        let output = convert("first paragraph\n////\nblock comment\n////\nsecond paragraph\n");
        assert!(!output.contains("block comment"));
        assert_xpath(&output, "//p", 2);
    }

    #[test]
    fn can_convert_with_block_comment_at_end_of_document_with_trailing_newlines() {
        verifies!(
            r#"
    test "can convert with block comment at end of document with trailing newlines" do
      input = <<~'EOS'
      paragraph

      ////
      block comment
      ////


      EOS
      output = convert_string_to_embedded input
      refute_match(/block comment/, output)
    end

"#
        );

        let output = convert("paragraph\n\n////\nblock comment\n////\n\n\n");
        assert!(!output.contains("block comment"));
    }

    #[test]
    fn trailing_newlines_after_block_comment_at_end_of_document_does_not_create_paragraph() {
        verifies!(
            r#"
    test "trailing newlines after block comment at end of document does not create paragraph" do
      input = <<~'EOS'
      paragraph

      ////
      block comment
      ////


      EOS
      d = document_from_string input
      assert_equal 1, d.blocks.size
      assert_xpath '//p', d.convert, 1
    end

"#
        );

        // Inspect the parsed `Document` to confirm the trailing newlines create
        // no spurious empty paragraph. This crate's parser *retains* comment
        // blocks in the model (unlike Asciidoctor, whose `d.blocks.size` is 1),
        // so the document holds exactly two blocks — the paragraph and the
        // comment — and no third, empty paragraph.
        let input = "paragraph\n\n////\nblock comment\n////\n\n\n";
        let contexts: Vec<String> = load(input)
            .child_blocks()
            .map(|b| b.resolved_context().to_string())
            .collect();
        assert_eq!(contexts, ["paragraph", "comment"]);

        assert_xpath(&convert(input), "//p", 1);
    }

    #[test]
    fn line_starting_with_three_slashes_should_not_be_line_comment() {
        verifies!(
            r#"
    test 'line starting with three slashes should not be line comment' do
      input = '/// not a line comment'
      output = convert_string_to_embedded input
      refute_empty output.strip, "Line should be emitted => #{input.rstrip}"
    end

"#
        );

        let output = convert("/// not a line comment");
        assert!(!output.trim().is_empty(), "Line should be emitted");
    }

    #[test]
    fn preprocessor_directives_should_not_be_processed_within_comment_block_within_block_metadata()
    {
        verifies!(
            r#"
    test 'preprocessor directives should not be processed within comment block within block metadata' do
      input = <<~'EOS'
      .sample title
      ////
      ifdef::asciidoctor[////]
      ////
      line should be shown
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p[text()="line should be shown"]', output, 1
    end

"#
        );

        let output =
            convert(".sample title\n////\nifdef::asciidoctor[////]\n////\nline should be shown\n");
        assert_xpath(&output, r#"//p[text()="line should be shown"]"#, 1);
    }

    #[test]
    fn preprocessor_directives_should_not_be_processed_within_comment_block() {
        verifies!(
            r#"
    test 'preprocessor directives should not be processed within comment block' do
      input = <<~'EOS'
      dummy line

      ////
      ifdef::asciidoctor[////]
      ////

      line should be shown
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p[text()="line should be shown"]', output, 1
    end

"#
        );

        let output =
            convert("dummy line\n\n////\nifdef::asciidoctor[////]\n////\n\nline should be shown\n");
        assert_xpath(&output, r#"//p[text()="line should be shown"]"#, 1);
    }

    #[test]
    fn should_warn_if_unterminated_comment_block_is_detected_in_body() {
        verifies!(
            r#"
    test 'should warn if unterminated comment block is detected in body' do
      input = <<~'EOS'
      before comment block

      ////
      content that has been disabled

      supposed to be after comment block, except it got swallowed by block comment
      EOS

      convert_string_to_embedded input
      assert_message @logger, :WARN, '<stdin>: line 3: unterminated comment block', Hash
    end

"#
        );

        let input = "before comment block\n\n////\ncontent that has been disabled\n\nsupposed to be after comment block, except it got swallowed by block comment\n";
        assert_warning(input, 3, |w| {
            matches!(w, WarningType::UnterminatedDelimitedBlock)
        });
    }

    #[test]
    fn should_warn_if_unterminated_comment_block_is_detected_inside_another_block() {
        verifies!(
            r#"
    test 'should warn if unterminated comment block is detected inside another block' do
      input = <<~'EOS'
      before sidebar block

      ****
      ////
      content that has been disabled
      ****

      supposed to be after sidebar block, except it got swallowed by block comment
      EOS

      convert_string_to_embedded input
      assert_message @logger, :WARN, '<stdin>: line 4: unterminated comment block', Hash
    end

"#
        );

        let input = "before sidebar block\n\n****\n////\ncontent that has been disabled\n****\n\nsupposed to be after sidebar block, except it got swallowed by block comment\n";
        assert_warning(input, 4, |w| {
            matches!(w, WarningType::UnterminatedDelimitedBlock)
        });
    }

    #[test]
    fn preprocessor_directives_should_not_be_processed_within_comment_open_block() {
        verifies!(
            r#"
    # WARNING if first line of content is a directive, it will get interpreted before we know it's a comment block
    # it happens because we always look a line ahead...not sure what we can do about it
    test 'preprocessor directives should not be processed within comment open block' do
      input = <<~'EOS'
      [comment]
      --
      first line of comment
      ifdef::asciidoctor[--]
      line should not be shown
      --

      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p', output, 0
    end

"#
        );

        let output = convert(
            "[comment]\n--\nfirst line of comment\nifdef::asciidoctor[--]\nline should not be shown\n--\n\n",
        );
        assert_xpath(&output, "//p", 0);
    }

    #[test]
    fn preprocessor_directives_should_not_be_processed_on_subsequent_lines_of_a_comment_paragraph()
    {
        verifies!(
            r#"
    # WARNING this assertion fails if the directive is the first line of the paragraph instead of the second
    # it happens because we always look a line ahead; not sure what we can do about it
    test 'preprocessor directives should not be processed on subsequent lines of a comment paragraph' do
      input = <<~'EOS'
      [comment]
      first line of content
      ifdef::asciidoctor[////]

      this line should be shown
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p[text()="this line should be shown"]', output, 1
    end

"#
        );

        let output = convert(
            "[comment]\nfirst line of content\nifdef::asciidoctor[////]\n\nthis line should be shown\n",
        );
        assert_xpath(&output, r#"//p[text()="this line should be shown"]"#, 1);
    }

    #[test]
    fn comment_style_on_open_block_should_only_skip_block() {
        verifies!(
            r#"
    test 'comment style on open block should only skip block' do
      input = <<~'EOS'
      [comment]
      --
      skip

      this block
      --

      not this text
      EOS
      result = convert_string_to_embedded input
      assert_xpath '//p', result, 1
      assert_xpath '//p[text()="not this text"]', result, 1
    end

"#
        );

        let result = convert("[comment]\n--\nskip\n\nthis block\n--\n\nnot this text\n");
        assert_xpath(&result, "//p", 1);
        assert_xpath(&result, r#"//p[text()="not this text"]"#, 1);
    }

    #[test]
    fn comment_style_on_paragraph_should_only_skip_paragraph() {
        verifies!(
            r#"
    test 'comment style on paragraph should only skip paragraph' do
      input = <<~'EOS'
      [comment]
      skip
      this paragraph

      not this text
      EOS
      result = convert_string_to_embedded input
      assert_xpath '//p', result, 1
      assert_xpath '//p[text()="not this text"]', result, 1
    end

"#
        );

        let result = convert("[comment]\nskip\nthis paragraph\n\nnot this text\n");
        assert_xpath(&result, "//p", 1);
        assert_xpath(&result, r#"//p[text()="not this text"]"#, 1);
    }

    #[test]
    fn comment_style_on_paragraph_should_not_cause_adjacent_block_to_be_skipped() {
        verifies!(
            r#"
    test 'comment style on paragraph should not cause adjacent block to be skipped' do
      input = <<~'EOS'
      [comment]
      skip
      this paragraph
      [example]
      not this text
      EOS
      result = convert_string_to_embedded input
      assert_xpath '/*[@class="exampleblock"]', result, 1
      assert_xpath '/*[@class="exampleblock"]//*[normalize-space(text())="not this text"]', result, 1
    end

"#
        );

        let result = convert("[comment]\nskip\nthis paragraph\n[example]\nnot this text\n");
        assert_xpath(&result, r#"/*[@class="exampleblock"]"#, 1);
        assert_xpath(
            &result,
            r#"/*[@class="exampleblock"]//*[normalize-space(text())="not this text"]"#,
            1,
        );
    }

    #[test]
    fn should_not_drop_content_that_follows_skipped_content_inside_a_delimited_block() {
        verifies!(
            r#"
    # NOTE this test verifies the nil return value of Parser#next_block
    test 'should not drop content that follows skipped content inside a delimited block' do
      input = <<~'EOS'
      ====
      paragraph

      [comment#idname]
      skip

      paragraph
      ====
      EOS
      result = convert_string_to_embedded input
      assert_xpath '/*[@class="exampleblock"]', result, 1
      assert_xpath '/*[@class="exampleblock"]//*[@class="paragraph"]', result, 2
      assert_xpath '//*[@class="paragraph"][@id="idname"]', result, 0
    end
"#
        );

        let result = convert("====\nparagraph\n\n[comment#idname]\nskip\n\nparagraph\n====\n");
        assert_xpath(&result, r#"/*[@class="exampleblock"]"#, 1);
        assert_xpath(
            &result,
            r#"/*[@class="exampleblock"]//*[@class="paragraph"]"#,
            2,
        );
        assert_xpath(&result, r#"//*[@class="paragraph"][@id="idname"]"#, 0);
    }

    non_normative!(
        r#"
  end

"#
    );
}
mod sidebar_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Sidebar Blocks' do
"#
    );

    #[test]
    fn should_parse_sidebar_block() {
        verifies!(
            r#"
    test 'should parse sidebar block' do
      input = <<~'EOS'
      == Section

      .Sidebar
      ****
      Content goes here
      ****
      EOS
      result = convert_string input
      assert_xpath "//*[@class='sidebarblock']//p", result, 1
    end
  end

"#
        );

        let result = convert_with(
            "== Section\n\n.Sidebar\n****\nContent goes here\n****\n",
            &Options::new().standalone(true),
        );
        assert_xpath(&result, r#"//*[@class="sidebarblock"]//p"#, 1);
    }
}

mod quote_and_verse_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Quote and Verse Blocks' do
"#
    );

    #[test]
    fn quote_block_with_no_attribution() {
        verifies!(
            r#"
    test 'quote block with no attribution' do
      input = <<~'EOS'
      ____
      A famous quote.
      ____
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_css '.quoteblock > .attribution', output, 0
      assert_xpath '//*[@class="quoteblock"]//p[text()="A famous quote."]', output, 1
    end

"#
        );

        let output = convert_with(
            "____\nA famous quote.\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_css(&output, ".quoteblock > .attribution", 0);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]//p[text()="A famous quote."]"#,
            1,
        );
    }

    #[test]
    fn quote_block_with_attribution() {
        verifies!(
            r##"
    test 'quote block with attribution' do
      input = <<~'EOS'
      [quote, Famous Person, Famous Book (1999)]
      ____
      A famous quote.
      ____
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_css '.quoteblock > .attribution', output, 1
      assert_css '.quoteblock > .attribution > cite', output, 1
      assert_css '.quoteblock > .attribution > br + cite', output, 1
      assert_xpath '//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Book (1999)"]', output, 1
      attribution = xmlnodes_at_xpath '//*[@class="quoteblock"]/*[@class="attribution"]', output, 1
      author = attribution.children.first
      assert_equal "#{decode_char 8212} Famous Person", author.text.strip
    end

"##
        );

        let output = convert_with(
            "[quote, Famous Person, Famous Book (1999)]\n____\nA famous quote.\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_css(&output, ".quoteblock > .attribution", 1);
        assert_css(&output, ".quoteblock > .attribution > cite", 1);
        assert_css(&output, ".quoteblock > .attribution > br + cite", 1);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Book (1999)"]"#,
            1,
        );
        // The `attribution.children.first` author-text check is re-expressed as a
        // `contains` on the attribution's own text ("— Famous Person").
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"][contains(text(),"Famous Person")]"#,
            1,
        );
    }

    #[test]
    fn quote_block_with_attribute_and_id_and_role_shorthand() {
        verifies!(
            r#"
    test 'quote block with attribute and id and role shorthand' do
      input = <<~'EOS'
      [quote#justice-to-all.solidarity, Martin Luther King, Jr.]
      ____
      Injustice anywhere is a threat to justice everywhere.
      ____
      EOS

      output = convert_string_to_embedded input
      assert_css '.quoteblock', output, 1
      assert_css '#justice-to-all.quoteblock.solidarity', output, 1
      assert_css '.quoteblock > .attribution', output, 1
    end

"#
        );

        let output = convert(
            "[quote#justice-to-all.solidarity, Martin Luther King, Jr.]\n____\nInjustice anywhere is a threat to justice everywhere.\n____\n",
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, "#justice-to-all.quoteblock.solidarity", 1);
        assert_css(&output, ".quoteblock > .attribution", 1);
    }

    #[test]
    fn setting_id_using_style_shorthand_should_not_reset_block_style() {
        verifies!(
            r#"
    test 'setting ID using style shorthand should not reset block style' do
      input = <<~'EOS'
      [quote]
      [#justice-to-all.solidarity, Martin Luther King, Jr.]
      ____
      Injustice anywhere is a threat to justice everywhere.
      ____
      EOS

      output = convert_string_to_embedded input
      assert_css '.quoteblock', output, 1
      assert_css '#justice-to-all.quoteblock.solidarity', output, 1
      assert_css '.quoteblock > .attribution', output, 1
    end

"#
        );

        let output = convert(
            "[quote]\n[#justice-to-all.solidarity, Martin Luther King, Jr.]\n____\nInjustice anywhere is a threat to justice everywhere.\n____\n",
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, "#justice-to-all.quoteblock.solidarity", 1);
        assert_css(&output, ".quoteblock > .attribution", 1);
    }

    #[test]
    fn quote_block_with_complex_content() {
        verifies!(
            r#"
    test 'quote block with complex content' do
      input = <<~'EOS'
      ____
      A famous quote.

      NOTE: _That_ was inspiring.
      ____
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph', output, 1
      assert_css '.quoteblock > blockquote > .paragraph + .admonitionblock', output, 1
    end

"#
        );

        let output = convert_with(
            "____\nA famous quote.\n\nNOTE: _That_ was inspiring.\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph", 1);
        assert_css(
            &output,
            ".quoteblock > blockquote > .paragraph + .admonitionblock",
            1,
        );
    }

    // DocBook-backend output is out of scope (this crate targets only `html5`).
    non_normative!(
        r#"
    test 'quote block with attribution converted to DocBook' do
      input = <<~'EOS'
      [quote, Famous Person, Famous Book (1999)]
      ____
      A famous quote.
      ____
      EOS
      output = convert_string input, backend: :docbook
      assert_css 'blockquote', output, 1
      assert_css 'blockquote > simpara', output, 1
      assert_css 'blockquote > attribution', output, 1
      assert_css 'blockquote > attribution > citetitle', output, 1
      assert_xpath '//blockquote/attribution/citetitle[text()="Famous Book (1999)"]', output, 1
      attribution = xmlnodes_at_xpath '//blockquote/attribution', output, 1
      author = attribution.children.first
      assert_equal 'Famous Person', author.text.strip
    end

    test 'epigraph quote block with attribution converted to DocBook' do
      input = <<~'EOS'
      [.epigraph, Famous Person, Famous Book (1999)]
      ____
      A famous quote.
      ____
      EOS
      output = convert_string input, backend: :docbook
      assert_css 'epigraph', output, 1
      assert_css 'epigraph > simpara', output, 1
      assert_css 'epigraph > attribution', output, 1
      assert_css 'epigraph > attribution > citetitle', output, 1
      assert_xpath '//epigraph/attribution/citetitle[text()="Famous Book (1999)"]', output, 1
      attribution = xmlnodes_at_xpath '//epigraph/attribution', output, 1
      author = attribution.children.first
      assert_equal 'Famous Person', author.text.strip
    end

"#
    );

    #[test]
    fn markdown_style_quote_block_with_single_paragraph_and_no_attribution() {
        verifies!(
            r#"
    test 'markdown-style quote block with single paragraph and no attribution' do
      input = <<~'EOS'
      > A famous quote.
      > Some more inspiring words.
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_css '.quoteblock > .attribution', output, 0
      assert_xpath %(//*[@class="quoteblock"]//p[text()="A famous quote.\nSome more inspiring words."]), output, 1
    end

"#
        );

        let output = convert_with(
            "> A famous quote.\n> Some more inspiring words.\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_css(&output, ".quoteblock > .attribution", 0);
        assert_xpath(
            &output,
            "//*[@class=\"quoteblock\"]//p[text()=\"A famous quote.\nSome more inspiring words.\"]",
            1,
        );
    }

    #[test]
    fn lazy_markdown_style_quote_block_with_single_paragraph_and_no_attribution() {
        verifies!(
            r#"
    test 'lazy markdown-style quote block with single paragraph and no attribution' do
      input = <<~'EOS'
      > A famous quote.
      Some more inspiring words.
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_css '.quoteblock > .attribution', output, 0
      assert_xpath %(//*[@class="quoteblock"]//p[text()="A famous quote.\nSome more inspiring words."]), output, 1
    end

"#
        );

        let output = convert_with(
            "> A famous quote.\nSome more inspiring words.\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_css(&output, ".quoteblock > .attribution", 0);
        assert_xpath(
            &output,
            "//*[@class=\"quoteblock\"]//p[text()=\"A famous quote.\nSome more inspiring words.\"]",
            1,
        );
    }

    #[test]
    fn markdown_style_quote_block_with_multiple_paragraphs_and_no_attribution() {
        verifies!(
            r#"
    test 'markdown-style quote block with multiple paragraphs and no attribution' do
      input = <<~'EOS'
      > A famous quote.
      >
      > Some more inspiring words.
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 2
      assert_css '.quoteblock > .attribution', output, 0
      assert_xpath %((//*[@class="quoteblock"]//p)[1][text()="A famous quote."]), output, 1
      assert_xpath %((//*[@class="quoteblock"]//p)[2][text()="Some more inspiring words."]), output, 1
    end

"#
        );

        let output = convert_with(
            "> A famous quote.\n>\n> Some more inspiring words.\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 2);
        assert_css(&output, ".quoteblock > .attribution", 0);
        assert_xpath(
            &output,
            r#"(//*[@class="quoteblock"]//p)[1][text()="A famous quote."]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//*[@class="quoteblock"]//p)[2][text()="Some more inspiring words."]"#,
            1,
        );
    }

    #[test]
    fn markdown_style_quote_block_with_multiple_blocks_and_no_attribution() {
        verifies!(
            r#"
    test 'markdown-style quote block with multiple blocks and no attribution' do
      input = <<~'EOS'
      > A famous quote.
      >
      > NOTE: Some more inspiring words.
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_css '.quoteblock > blockquote > .admonitionblock', output, 1
      assert_css '.quoteblock > .attribution', output, 0
      assert_xpath %((//*[@class="quoteblock"]//p)[1][text()="A famous quote."]), output, 1
      assert_xpath %((//*[@class="quoteblock"]//*[@class="admonitionblock note"]//*[@class="content"])[1][normalize-space(text())="Some more inspiring words."]), output, 1
    end

"#
        );

        let output = convert_with(
            "> A famous quote.\n>\n> NOTE: Some more inspiring words.\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_css(&output, ".quoteblock > blockquote > .admonitionblock", 1);
        assert_css(&output, ".quoteblock > .attribution", 0);
        assert_xpath(
            &output,
            r#"(//*[@class="quoteblock"]//p)[1][text()="A famous quote."]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//*[@class="quoteblock"]//*[@class="admonitionblock note"]//*[@class="content"])[1][normalize-space(text())="Some more inspiring words."]"#,
            1,
        );
    }

    #[test]
    fn markdown_style_quote_block_with_single_paragraph_and_attribution() {
        verifies!(
            r##"
    test 'markdown-style quote block with single paragraph and attribution' do
      input = <<~'EOS'
      > A famous quote.
      > Some more inspiring words.
      > -- Famous Person, Famous Source, Volume 1 (1999)
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph > p', output, 1
      assert_xpath %(//*[@class="quoteblock"]//p[text()="A famous quote.\nSome more inspiring words."]), output, 1
      assert_css '.quoteblock > .attribution', output, 1
      assert_css '.quoteblock > .attribution > cite', output, 1
      assert_css '.quoteblock > .attribution > br + cite', output, 1
      assert_xpath '//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Source, Volume 1 (1999)"]', output, 1
      attribution = xmlnodes_at_xpath '//*[@class="quoteblock"]/*[@class="attribution"]', output, 1
      author = attribution.children.first
      assert_equal "#{decode_char 8212} Famous Person", author.text.strip
    end

"##
        );

        let output = convert_with(
            "> A famous quote.\n> Some more inspiring words.\n> -- Famous Person, Famous Source, Volume 1 (1999)\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph > p", 1);
        assert_xpath(
            &output,
            "//*[@class=\"quoteblock\"]//p[text()=\"A famous quote.\nSome more inspiring words.\"]",
            1,
        );
        assert_css(&output, ".quoteblock > .attribution", 1);
        assert_css(&output, ".quoteblock > .attribution > cite", 1);
        assert_css(&output, ".quoteblock > .attribution > br + cite", 1);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Source, Volume 1 (1999)"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"][contains(text(),"Famous Person")]"#,
            1,
        );
    }

    #[test]
    fn markdown_style_quote_block_with_only_attribution() {
        verifies!(
            r#"
    test 'markdown-style quote block with only attribution' do
      input = '> -- Anonymous'
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > *', output, 0
      assert_css '.quoteblock > .attribution', output, 1
      assert_xpath %(//*[@class="quoteblock"]//*[@class="attribution"][contains(text(),"Anonymous")]), output, 1
    end

"#
        );

        let output = convert_with("> -- Anonymous", &Options::new().standalone(true));
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > *", 0);
        assert_css(&output, ".quoteblock > .attribution", 1);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]//*[@class="attribution"][contains(text(),"Anonymous")]"#,
            1,
        );
    }

    #[test]
    fn should_parse_credit_line_in_markdown_style_quote_block_like_positional_block_attributes() {
        verifies!(
            r#"
    test 'should parse credit line in markdown-style quote block like positional block attributes' do
      input = <<~'EOS'
      > I hold it that a little rebellion now and then is a good thing,
      > and as necessary in the political world as storms in the physical.
      -- Thomas Jefferson, https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1[The Papers of Thomas Jefferson, Volume 11]
      EOS

      output = convert_string_to_embedded input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock cite a[href="https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1"]', output, 1
    end

"#
        );

        let output = convert(
            "> I hold it that a little rebellion now and then is a good thing,\n> and as necessary in the political world as storms in the physical.\n-- Thomas Jefferson, https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1[The Papers of Thomas Jefferson, Volume 11]\n",
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(
            &output,
            r#".quoteblock cite a[href="https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1"]"#,
            1,
        );
    }

    #[test]
    fn quoted_paragraph_style_quote_block_with_attribution() {
        verifies!(
            r##"
    test 'quoted paragraph-style quote block with attribution' do
      input = <<~'EOS'
      "A famous quote.
      Some more inspiring words."
      -- Famous Person, Famous Source, Volume 1 (1999)
      EOS
      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_xpath %(//*[@class="quoteblock"]/blockquote[normalize-space(text())="A famous quote. Some more inspiring words."]), output, 1
      assert_css '.quoteblock > .attribution', output, 1
      assert_css '.quoteblock > .attribution > cite', output, 1
      assert_css '.quoteblock > .attribution > br + cite', output, 1
      assert_xpath '//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Source, Volume 1 (1999)"]', output, 1
      attribution = xmlnodes_at_xpath '//*[@class="quoteblock"]/*[@class="attribution"]', output, 1
      author = attribution.children.first
      assert_equal "#{decode_char 8212} Famous Person", author.text.strip
    end

"##
        );

        let output = convert_with(
            "\"A famous quote.\nSome more inspiring words.\"\n-- Famous Person, Famous Source, Volume 1 (1999)\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/blockquote[normalize-space(text())="A famous quote. Some more inspiring words."]"#,
            1,
        );
        assert_css(&output, ".quoteblock > .attribution", 1);
        assert_css(&output, ".quoteblock > .attribution > cite", 1);
        assert_css(&output, ".quoteblock > .attribution > br + cite", 1);
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"]/cite[text()="Famous Source, Volume 1 (1999)"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="quoteblock"]/*[@class="attribution"][contains(text(),"Famous Person")]"#,
            1,
        );
    }

    #[test]
    fn should_parse_credit_line_in_quoted_paragraph_style_quote_block_like_positional_block_attributes(
    ) {
        verifies!(
            r#"
    test 'should parse credit line in quoted paragraph-style quote block like positional block attributes' do
      input = <<~'EOS'
      "I hold it that a little rebellion now and then is a good thing,
      and as necessary in the political world as storms in the physical."
      -- Thomas Jefferson, https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1[The Papers of Thomas Jefferson, Volume 11]
      EOS

      output = convert_string_to_embedded input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock cite a[href="https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1"]', output, 1
    end

"#
        );

        let output = convert(
            "\"I hold it that a little rebellion now and then is a good thing,\nand as necessary in the political world as storms in the physical.\"\n-- Thomas Jefferson, https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1[The Papers of Thomas Jefferson, Volume 11]\n",
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(
            &output,
            r#".quoteblock cite a[href="https://jeffersonpapers.princeton.edu/selected-documents/james-madison-1"]"#,
            1,
        );
    }

    #[test]
    fn single_line_verse_block_without_attribution() {
        verifies!(
            r#"
    test 'single-line verse block without attribution' do
      input = <<~'EOS'
      [verse]
      ____
      A famous verse.
      ____
      EOS
      output = convert_string input
      assert_css '.verseblock', output, 1
      assert_css '.verseblock > pre', output, 1
      assert_css '.verseblock > .attribution', output, 0
      assert_css '.verseblock p', output, 0
      assert_xpath '//*[@class="verseblock"]/pre[normalize-space(text())="A famous verse."]', output, 1
    end

"#
        );

        let output = convert_with(
            "[verse]\n____\nA famous verse.\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".verseblock", 1);
        assert_css(&output, ".verseblock > pre", 1);
        assert_css(&output, ".verseblock > .attribution", 0);
        assert_css(&output, ".verseblock p", 0);
        assert_xpath(
            &output,
            r#"//*[@class="verseblock"]/pre[normalize-space(text())="A famous verse."]"#,
            1,
        );
    }

    #[test]
    fn single_line_verse_block_with_attribution() {
        verifies!(
            r##"
    test 'single-line verse block with attribution' do
      input = <<~'EOS'
      [verse, Famous Poet, Famous Poem]
      ____
      A famous verse.
      ____
      EOS
      output = convert_string input
      assert_css '.verseblock', output, 1
      assert_css '.verseblock p', output, 0
      assert_css '.verseblock > pre', output, 1
      assert_css '.verseblock > .attribution', output, 1
      assert_css '.verseblock > .attribution > cite', output, 1
      assert_css '.verseblock > .attribution > br + cite', output, 1
      assert_xpath '//*[@class="verseblock"]/*[@class="attribution"]/cite[text()="Famous Poem"]', output, 1
      attribution = xmlnodes_at_xpath '//*[@class="verseblock"]/*[@class="attribution"]', output, 1
      author = attribution.children.first
      assert_equal "#{decode_char 8212} Famous Poet", author.text.strip
    end

"##
        );

        let output = convert_with(
            "[verse, Famous Poet, Famous Poem]\n____\nA famous verse.\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".verseblock", 1);
        assert_css(&output, ".verseblock p", 0);
        assert_css(&output, ".verseblock > pre", 1);
        assert_css(&output, ".verseblock > .attribution", 1);
        assert_css(&output, ".verseblock > .attribution > cite", 1);
        assert_css(&output, ".verseblock > .attribution > br + cite", 1);
        assert_xpath(
            &output,
            r#"//*[@class="verseblock"]/*[@class="attribution"]/cite[text()="Famous Poem"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="verseblock"]/*[@class="attribution"][contains(text(),"Famous Poet")]"#,
            1,
        );
    }

    // DocBook-backend output is out of scope (this crate targets only `html5`).
    non_normative!(
        r#"
    test 'single-line verse block with attribution converted to DocBook' do
      input = <<~'EOS'
      [verse, Famous Poet, Famous Poem]
      ____
      A famous verse.
      ____
      EOS
      output = convert_string input, backend: :docbook
      assert_css 'blockquote', output, 1
      assert_css 'blockquote simpara', output, 0
      assert_css 'blockquote > literallayout', output, 1
      assert_css 'blockquote > attribution', output, 1
      assert_css 'blockquote > attribution > citetitle', output, 1
      assert_xpath '//blockquote/attribution/citetitle[text()="Famous Poem"]', output, 1
      attribution = xmlnodes_at_xpath '//blockquote/attribution', output, 1
      author = attribution.children.first
      assert_equal 'Famous Poet', author.text.strip
    end

    test 'single-line epigraph verse block with attribution converted to DocBook' do
      input = <<~'EOS'
      [verse.epigraph, Famous Poet, Famous Poem]
      ____
      A famous verse.
      ____
      EOS
      output = convert_string input, backend: :docbook
      assert_css 'epigraph', output, 1
      assert_css 'epigraph simpara', output, 0
      assert_css 'epigraph > literallayout', output, 1
      assert_css 'epigraph > attribution', output, 1
      assert_css 'epigraph > attribution > citetitle', output, 1
      assert_xpath '//epigraph/attribution/citetitle[text()="Famous Poem"]', output, 1
      attribution = xmlnodes_at_xpath '//epigraph/attribution', output, 1
      author = attribution.children.first
      assert_equal 'Famous Poet', author.text.strip
    end

"#
    );

    #[test]
    fn multi_stanza_verse_block() {
        verifies!(
            r#"
    test 'multi-stanza verse block' do
      input = <<~'EOS'
      [verse]
      ____
      A famous verse.

      Stanza two.
      ____
      EOS
      output = convert_string input
      assert_xpath '//*[@class="verseblock"]', output, 1
      assert_xpath '//*[@class="verseblock"]/pre', output, 1
      assert_xpath '//*[@class="verseblock"]//p', output, 0
      assert_xpath '//*[@class="verseblock"]/pre[contains(text(), "A famous verse.")]', output, 1
      assert_xpath '//*[@class="verseblock"]/pre[contains(text(), "Stanza two.")]', output, 1
    end

"#
        );

        let output = convert_with(
            "[verse]\n____\nA famous verse.\n\nStanza two.\n____\n",
            &Options::new().standalone(true),
        );
        assert_xpath(&output, r#"//*[@class="verseblock"]"#, 1);
        assert_xpath(&output, r#"//*[@class="verseblock"]/pre"#, 1);
        assert_xpath(&output, r#"//*[@class="verseblock"]//p"#, 0);
        assert_xpath(
            &output,
            r#"//*[@class="verseblock"]/pre[contains(text(), "A famous verse.")]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="verseblock"]/pre[contains(text(), "Stanza two.")]"#,
            1,
        );
    }

    #[test]
    fn verse_block_does_not_contain_block_elements() {
        verifies!(
            r#"
    test 'verse block does not contain block elements' do
      input = <<~'EOS'
      [verse]
      ____
      A famous verse.

      ....
      not a literal
      ....
      ____
      EOS
      output = convert_string input
      assert_css '.verseblock', output, 1
      assert_css '.verseblock > pre', output, 1
      assert_css '.verseblock p', output, 0
      assert_css '.verseblock .literalblock', output, 0
    end

"#
        );

        let output = convert_with(
            "[verse]\n____\nA famous verse.\n\n....\nnot a literal\n....\n____\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".verseblock", 1);
        assert_css(&output, ".verseblock > pre", 1);
        assert_css(&output, ".verseblock p", 0);
        assert_css(&output, ".verseblock .literalblock", 0);
    }

    // `verse.subs` is an `asciidoc-parser` model assertion (no rendered form).
    non_normative!(
        r#"
    test 'verse should have normal subs' do
      input = <<~'EOS'
      [verse]
      ____
      A famous verse
      ____
      EOS

      verse = block_from_string input
      assert_equal Asciidoctor::Substitutors::NORMAL_SUBS, verse.subs
    end

"#
    );

    #[test]
    fn should_not_recognize_callouts_in_a_verse() {
        verifies!(
            r#"
    test 'should not recognize callouts in a verse' do
      input = <<~'EOS'
      [verse]
      ____
      La la la <1>
      ____
      <1> Not pointing to a callout
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//pre[text()="La la la <1>"]', output, 1
      assert_message @logger, :WARN, '<stdin>: line 5: no callout found for <1>', Hash
    end

"#
        );

        let input = "[verse]\n____\nLa la la <1>\n____\n<1> Not pointing to a callout\n";
        let output = convert(input);
        assert_xpath(&output, r#"//pre[text()="La la la <1>"]"#, 1);
        assert_warning(input, 5, |w| matches!(w, WarningType::NoCalloutFound(1)));
    }

    #[test]
    fn should_perform_normal_subs_on_a_verse_block() {
        verifies!(
            r##"
    test 'should perform normal subs on a verse block' do
      input = <<~'EOS'
      [verse]
      ____
      _GET /groups/link:#group-id[\{group-id\}]_
      ____
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '<pre class="content"><em>GET /groups/<a href="#group-id">{group-id}</a></em></pre>'
    end
  end

"##
        );

        let output = convert("[verse]\n____\n_GET /groups/link:#group-id[\\{group-id\\}]_\n____\n");
        assert!(output.contains(
            r##"<pre class="content"><em>GET /groups/<a href="#group-id">{group-id}</a></em></pre>"##
        ));
    }
}

mod example_blocks {
    use super::*;

    non_normative!(
        r#"
  context "Example Blocks" do
"#
    );

    #[test]
    fn can_convert_example_block() {
        verifies!(
            r#"
    test "can convert example block" do
      input = <<~'EOS'
      ====
      This is an example of an example block.

      How crazy is that?
      ====
      EOS

      output = convert_string input
      assert_xpath '//*[@class="exampleblock"]//p', output, 2
    end

"#
        );

        let output = convert_with(
            "====\nThis is an example of an example block.\n\nHow crazy is that?\n====\n",
            &Options::new().standalone(true),
        );
        assert_xpath(&output, r#"//*[@class="exampleblock"]//p"#, 2);
    }

    #[test]
    fn assigns_sequential_numbered_caption_to_example_block_with_title() {
        verifies!(
            r#"
    test 'assigns sequential numbered caption to example block with title' do
      input = <<~'EOS'
      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====

      .Writing Docs with DocBook
      ====
      Here's how you write DocBook.

      You futz with XML.
      ====
      EOS

      doc = document_from_string input
      assert_equal 1, doc.blocks[0].numeral
      assert_equal 1, doc.blocks[0].number
      assert_equal 2, doc.blocks[1].numeral
      assert_equal 2, doc.blocks[1].number
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example 1. Writing Docs with AsciiDoc"]', output, 1
      assert_xpath '(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example 2. Writing Docs with DocBook"]', output, 1
      assert_equal 2, doc.attributes['example-number']
    end

"#
        );

        // The `numeral`/`number`/`example-number` checks are `asciidoc-parser`
        // model assertions; only the rendered captions are re-expressed here.
        let output = convert(
            ".Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n\n.Writing Docs with DocBook\n====\nHere's how you write DocBook.\n\nYou futz with XML.\n====\n",
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example 1. Writing Docs with AsciiDoc"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example 2. Writing Docs with DocBook"]"#,
            1,
        );
    }

    // Example captions and counters beyond the default `Example N. ` form —
    // alphabetic/API-seeded numbering, explicit `[caption=…]`, and the
    // `example-caption` toggle — are not implemented yet; tracked in
    // <https://github.com/asciidoc-rs/asciidoc-html5/issues/113>.
    non_normative!(
        r#"
    test 'assigns sequential character caption to example block with title' do
      input = <<~'EOS'
      :example-number: @

      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====

      .Writing Docs with DocBook
      ====
      Here's how you write DocBook.

      You futz with XML.
      ====
      EOS

      doc = document_from_string input
      assert_equal 'A', doc.blocks[0].numeral
      assert_equal 'A', doc.blocks[0].number
      assert_equal 'B', doc.blocks[1].numeral
      assert_equal 'B', doc.blocks[1].number
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example A. Writing Docs with AsciiDoc"]', output, 1
      assert_xpath '(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example B. Writing Docs with DocBook"]', output, 1
      assert_equal 'B', doc.attributes['example-number']
    end

    test 'should increment counter for example even when example-number is locked by the API' do
      input = <<~'EOS'
      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====

      .Writing Docs with DocBook
      ====
      Here's how you write DocBook.

      You futz with XML.
      ====
      EOS

      doc = document_from_string input, attributes: { 'example-number' => '`' }
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example a. Writing Docs with AsciiDoc"]', output, 1
      assert_xpath '(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example b. Writing Docs with DocBook"]', output, 1
      assert_equal 'b', doc.attributes['example-number']
    end

    test 'should use explicit caption if specified' do
      input = <<~'EOS'
      [caption="Look! "]
      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====
      EOS

      doc = document_from_string input
      assert_nil doc.blocks[0].numeral
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]', output, 1
      refute doc.attributes.key? 'example-number'
    end

    test 'automatic caption can be turned off and on and modified' do
      input = <<~'EOS'
      .first example
      ====
      an example
      ====

      :caption:

      .second example
      ====
      another example
      ====

      :caption!:
      :example-caption: Exhibit

      .third example
      ====
      yet another example
      ====
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="exampleblock"]', output, 3
      assert_xpath '(/*[@class="exampleblock"])[1]/*[@class="title"][starts-with(text(), "Example ")]', output, 1
      assert_xpath '(/*[@class="exampleblock"])[2]/*[@class="title"][text()="second example"]', output, 1
      assert_xpath '(/*[@class="exampleblock"])[3]/*[@class="title"][starts-with(text(), "Exhibit ")]', output, 1
    end

    test 'should use explicit caption if specified even if block-specific global caption is disabled' do
      input = <<~'EOS'
      :!example-caption:

      [caption="Look! "]
      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====
      EOS

      doc = document_from_string input
      assert_nil doc.blocks[0].numeral
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]', output, 1
      refute doc.attributes.key? 'example-number'
    end

    test 'should use global caption if specified even if block-specific global caption is disabled' do
      input = <<~'EOS'
      :!example-caption:
      :caption: Look!{sp}

      .Writing Docs with AsciiDoc
      ====
      Here's how you write AsciiDoc.

      You just write.
      ====
      EOS

      doc = document_from_string input
      assert_nil doc.blocks[0].numeral
      output = doc.convert
      assert_xpath '(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]', output, 1
      refute doc.attributes.key? 'example-number'
    end

"#
    );

    #[test]
    fn should_not_process_caption_attribute_on_block_that_does_not_support_a_caption() {
        verifies!(
            r#"
    test 'should not process caption attribute on block that does not support a caption' do
      input = <<~'EOS'
      [caption="Look! "]
      .No caption here
      --
      content
      --
      EOS

      doc = document_from_string input
      assert_nil doc.blocks[0].caption
      assert_equal 'Look! ', (doc.blocks[0].attr 'caption')
      output = doc.convert
      assert_xpath '(//*[@class="openblock"])[1]/*[@class="title"][text()="No caption here"]', output, 1
    end

"#
        );

        // Only the rendered title is re-expressed; `blocks[0].caption`/`attr` are
        // `asciidoc-parser` model assertions.
        let output = convert("[caption=\"Look! \"]\n.No caption here\n--\ncontent\n--\n");
        assert_xpath(
            &output,
            r#"(//*[@class="openblock"])[1]/*[@class="title"][text()="No caption here"]"#,
            1,
        );
    }

    // Collapsible example blocks (`%collapsible` → `<details>/<summary>`) are not
    // implemented yet; tracked in
    // <https://github.com/asciidoc-rs/asciidoc-html5/issues/114>.
    non_normative!(
        r#"
    test 'should create details/summary set if collapsible option is set' do
      input = <<~'EOS'
      .Toggle Me
      [%collapsible]
      ====
      This content is revealed when the user clicks the words "Toggle Me".
      ====
      EOS

      output = convert_string_to_embedded input
      assert_css 'details', output, 1
      assert_css 'details[open]', output, 0
      assert_css 'details > summary.title', output, 1
      assert_xpath '//details/summary[text()="Toggle Me"]', output, 1
      assert_css 'details > summary.title + .content', output, 1
      assert_css 'details > summary.title + .content p', output, 1
    end

    test 'should open details/summary set if collapsible and open options are set' do
      input = <<~'EOS'
      .Toggle Me
      [%collapsible%open]
      ====
      This content is revealed when the user clicks the words "Toggle Me".
      ====
      EOS

      output = convert_string_to_embedded input
      assert_css 'details', output, 1
      assert_css 'details[open]', output, 1
      assert_css 'details > summary.title', output, 1
      assert_xpath '//details/summary[text()="Toggle Me"]', output, 1
    end

    test 'should add default summary element if collapsible option is set and title is not specifed' do
      input = <<~'EOS'
      [%collapsible]
      ====
      This content is revealed when the user clicks the words "Details".
      ====
      EOS

      output = convert_string_to_embedded input
      assert_css 'details', output, 1
      assert_css 'details > summary.title', output, 1
      assert_xpath '//details/summary[text()="Details"]', output, 1
    end

    test 'should not allow collapsible block to increment example number' do
      input = <<~'EOS'
      .Before
      ====
      before
      ====

      .Show Me The Goods
      [%collapsible]
      ====
      This content is revealed when the user clicks the words "Show Me The Goods".
      ====

      .After
      ====
      after
      ====
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//*[@class="title"][text()="Example 1. Before"]', output, 1
      assert_xpath '//*[@class="title"][text()="Example 2. After"]', output, 1
      assert_css 'details', output, 1
      assert_css 'details > summary.title', output, 1
      assert_xpath '//details/summary[text()="Show Me The Goods"]', output, 1
    end

"#
    );

    #[test]
    fn should_warn_if_example_block_is_not_terminated() {
        verifies!(
            r#"
    test 'should warn if example block is not terminated' do
      input = <<~'EOS'
      outside

      ====
      inside

      still inside

      eof
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="exampleblock"]', output, 1
      assert_message @logger, :WARN, '<stdin>: line 3: unterminated example block', Hash
    end
"#
        );

        let input = "outside\n\n====\ninside\n\nstill inside\n\neof\n";
        let output = convert(input);
        assert_xpath(&output, r#"/*[@class="exampleblock"]"#, 1);
        assert_warning(input, 3, |w| {
            matches!(w, WarningType::UnterminatedDelimitedBlock)
        });
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod admonition_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Admonition Blocks' do
"#
    );

    #[test]
    fn caption_block_level_attribute_should_be_used_as_caption() {
        verifies!(
            r#"
    test 'caption block-level attribute should be used as caption' do
      input = <<~'EOS'
      :tip-caption: Pro Tip

      [caption="Pro Tip"]
      TIP: Override the caption of an admonition block using an attribute entry
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Pro Tip"]', output, 1
    end

"#
        );

        let output = convert(
            ":tip-caption: Pro Tip\n\n[caption=\"Pro Tip\"]\nTIP: Override the caption of an admonition block using an attribute entry\n",
        );
        assert_xpath(
            &output,
            r#"/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Pro Tip"]"#,
            1,
        );
    }

    #[test]
    fn can_override_caption_of_admonition_block_using_document_attribute() {
        verifies!(
            r#"
    test 'can override caption of admonition block using document attribute' do
      input = <<~'EOS'
      :tip-caption: Pro Tip

      TIP: Override the caption of an admonition block using an attribute entry
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Pro Tip"]', output, 1
    end

"#
        );

        let output = convert(
            ":tip-caption: Pro Tip\n\nTIP: Override the caption of an admonition block using an attribute entry\n",
        );
        assert_xpath(
            &output,
            r#"/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Pro Tip"]"#,
            1,
        );
    }

    #[test]
    fn blank_caption_document_attribute_should_not_blank_admonition_block_caption() {
        verifies!(
            r#"
    test 'blank caption document attribute should not blank admonition block caption' do
      input = <<~'EOS'
      :caption:

      TIP: Override the caption of an admonition block using an attribute entry
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Tip"]', output, 1
    end
  end

"#
        );

        let output = convert(
            ":caption:\n\nTIP: Override the caption of an admonition block using an attribute entry\n",
        );
        assert_xpath(
            &output,
            r#"/*[@class="admonitionblock tip"]//*[@class="icon"]/*[@class="title"][text()="Tip"]"#,
            1,
        );
    }
}

mod preformatted_blocks {
    use super::*;

    non_normative!(
        r#"
  context "Preformatted Blocks" do
"#
    );

    #[test]
    fn should_separate_adjacent_paragraphs_and_listing_into_blocks() {
        verifies!(
            r#"
    test 'should separate adjacent paragraphs and listing into blocks' do
      input = <<~'EOS'
      paragraph 1
      ----
      listing content
      ----
      paragraph 2
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="paragraph"]/p', output, 2
      assert_xpath '/*[@class="listingblock"]', output, 1
      assert_xpath '(/*[@class="paragraph"]/following-sibling::*)[1][@class="listingblock"]', output, 1
    end

"#
        );

        let output = convert("paragraph 1\n----\nlisting content\n----\nparagraph 2\n");
        assert_xpath(&output, r#"/*[@class="paragraph"]/p"#, 2);
        assert_xpath(&output, r#"/*[@class="listingblock"]"#, 1);
        assert_xpath(
            &output,
            r#"(/*[@class="paragraph"]/following-sibling::*)[1][@class="listingblock"]"#,
            1,
        );
    }

    #[test]
    fn should_warn_if_listing_block_is_not_terminated() {
        verifies!(
            r#"
    test 'should warn if listing block is not terminated' do
      input = <<~'EOS'
      outside

      ----
      inside

      still inside

      eof
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="listingblock"]', output, 1
      assert_message @logger, :WARN, '<stdin>: line 3: unterminated listing block', Hash
    end

"#
        );

        let input = "outside\n\n----\ninside\n\nstill inside\n\neof\n";
        let output = convert(input);
        assert_xpath(&output, r#"/*[@class="listingblock"]"#, 1);
        assert_warning(input, 3, |w| {
            matches!(w, WarningType::UnterminatedDelimitedBlock)
        });
    }

    #[test]
    fn should_not_crash_when_converting_verbatim_block_that_has_no_lines() {
        verifies!(
            r#"
    test 'should not crash when converting verbatim block that has no lines' do
      [%(----\n----), %(....\n....)].each do |input|
        output = convert_string_to_embedded input
        assert_css 'pre', output, 1
        assert_css 'pre:empty', output, 1
      end
    end

"#
        );

        for input in ["----\n----", "....\n...."] {
            let output = convert(input);
            assert_css(&output, "pre", 1);
            assert_css(&output, "pre:empty", 1);
        }
    }

    // `blocks[0].content` is an `asciidoc-parser` model assertion (no rendered
    // form).
    non_normative!(
        r#"
    test 'should return content as empty string for verbatim or raw block that has no lines' do
      [%(----\n----), %(....\n....)].each do |input|
        doc = document_from_string input
        assert_equal '', doc.blocks[0].content
      end
    end

"#
    );

    #[test]
    fn should_preserve_newlines_in_literal_block() {
        verifies!(
            r#"
    test 'should preserve newlines in literal block' do
      input = <<~'EOS'
      ....
      line one

      line two

      line three
      ....
      EOS
      [true, false].each do |standalone|
        output = convert_string input, standalone: standalone
        assert_xpath '//pre', output, 1
        assert_xpath '//pre/text()', output, 1
        text = xmlnodes_at_xpath('//pre/text()', output, 1).text
        lines = text.lines
        assert_equal 5, lines.size
        expected = "line one\n\nline two\n\nline three".lines
        assert_equal expected, lines
        blank_lines = output.scan(/\n[ \t]*\n/).size
        assert blank_lines >= 2
      end
    end

"#
        );

        // `//pre/text()` line-counting is re-expressed as an exact direct-text
        // match on the single `<pre>`.
        let input = "....\nline one\n\nline two\n\nline three\n....\n";
        for output in [
            convert(input),
            convert_with(input, &Options::new().standalone(true)),
        ] {
            assert_xpath(&output, "//pre", 1);
            assert_xpath(
                &output,
                "//pre[text()=\"line one\n\nline two\n\nline three\"]",
                1,
            );
        }
    }

    #[test]
    fn should_preserve_newlines_in_listing_block() {
        verifies!(
            r#"
    test 'should preserve newlines in listing block' do
      input = <<~'EOS'
      ----
      line one

      line two

      line three
      ----
      EOS
      [true, false].each do |standalone|
        output = convert_string input, standalone: standalone
        assert_xpath '//pre', output, 1
        assert_xpath '//pre/text()', output, 1
        text = xmlnodes_at_xpath('//pre/text()', output, 1).text
        lines = text.lines
        assert_equal 5, lines.size
        expected = "line one\n\nline two\n\nline three".lines
        assert_equal expected, lines
        blank_lines = output.scan(/\n[ \t]*\n/).size
        assert blank_lines >= 2
      end
    end

"#
        );

        let input = "----\nline one\n\nline two\n\nline three\n----\n";
        for output in [
            convert(input),
            convert_with(input, &Options::new().standalone(true)),
        ] {
            assert_xpath(&output, "//pre", 1);
            assert_xpath(
                &output,
                "//pre[text()=\"line one\n\nline two\n\nline three\"]",
                1,
            );
        }
    }

    #[test]
    fn should_preserve_newlines_in_verse_block() {
        verifies!(
            r#"
    test 'should preserve newlines in verse block' do
      input = <<~'EOS'
      --
      [verse]
      ____
      line one

      line two

      line three
      ____
      --
      EOS
      [true, false].each do |standalone|
        output = convert_string input, standalone: standalone
        assert_xpath '//*[@class="verseblock"]/pre', output, 1
        assert_xpath '//*[@class="verseblock"]/pre/text()', output, 1
        text = xmlnodes_at_xpath('//*[@class="verseblock"]/pre/text()', output, 1).text
        lines = text.lines
        assert_equal 5, lines.size
        expected = "line one\n\nline two\n\nline three".lines
        assert_equal expected, lines
        blank_lines = output.scan(/\n[ \t]*\n/).size
        assert blank_lines >= 2
      end
    end

"#
        );

        let input = "--\n[verse]\n____\nline one\n\nline two\n\nline three\n____\n--\n";
        for output in [
            convert(input),
            convert_with(input, &Options::new().standalone(true)),
        ] {
            assert_xpath(&output, r#"//*[@class="verseblock"]/pre"#, 1);
            assert_xpath(
                &output,
                "//*[@class=\"verseblock\"]/pre[text()=\"line one\n\nline two\n\nline three\"]",
                1,
            );
        }
    }

    #[test]
    fn should_strip_leading_and_trailing_blank_lines_when_converting_verbatim_block() {
        verifies!(
            r#"
    test 'should strip leading and trailing blank lines when converting verbatim block' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      [subs="attributes"]
      ....


        first line

      last line

      {empty}

      ....
      EOS

      doc = document_from_string input, standalone: false
      block = doc.blocks.first
      assert_equal ['', '', '  first line', '', 'last line', '', '{empty}', ''], block.lines
      result = doc.convert
      assert_xpath %(//pre[text()="  first line\n\nlast line"]), result, 1
    end

"#
        );

        // The `block.lines` assertion inspects the parser's line buffer (verified
        // in `asciidoc-parser`); here we drive the rendered output. The
        // `subs="attributes"` step expands `{empty}` to nothing (the parser's
        // job), leaving a run of trailing blank lines the renderer then trims
        // along with the leading ones.
        let output = convert(
            "[subs=\"attributes\"]\n....\n\n\n  first line\n\nlast line\n\n{empty}\n\n....\n",
        );
        assert_xpath(&output, "//pre[text()=\"  first line\n\nlast line\"]", 1);
    }

    #[test]
    fn should_process_block_with_crlf_line_endings() {
        verifies!(
            r#"
    test 'should process block with CRLF line endings' do
      input = <<~EOS
      ----\r
      source line 1\r
      source line 2\r
      ----\r
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="listingblock"]//pre', output, 1
      assert_xpath %(/*[@class="listingblock"]//pre[text()="source line 1\nsource line 2"]), output, 1
    end

"#
        );

        let output = convert("----\r\nsource line 1\r\nsource line 2\r\n----\r\n");
        assert_xpath(&output, r#"/*[@class="listingblock"]//pre"#, 1);
        assert_xpath(
            &output,
            "/*[@class=\"listingblock\"]//pre[text()=\"source line 1\nsource line 2\"]",
            1,
        );
    }

    #[test]
    fn should_remove_block_indent_if_indent_attribute_is_0() {
        verifies!(
            r#"
    test 'should remove block indent if indent attribute is 0' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      [indent="0"]
      ----
          def names

            @names.split

          end
      ----
      EOS

      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      expected = <<~EOS.chop
      def names

        @names.split

      end
      EOS

      output = convert_string_to_embedded input
      assert_css 'pre', output, 1
      assert_css '.listingblock pre', output, 1
      result = xmlnodes_at_xpath('//pre', output, 1).text
      assert_equal expected, result
    end

"#
        );

        let output =
            convert("[indent=\"0\"]\n----\n    def names\n\n      @names.split\n\n    end\n----\n");
        assert_css(&output, "pre", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_xpath(
            &output,
            "//pre[text()=\"def names\n\n  @names.split\n\nend\"]",
            1,
        );
    }

    #[test]
    fn should_not_remove_block_indent_if_indent_attribute_is_minus_1() {
        verifies!(
            r#"
    test 'should not remove block indent if indent attribute is -1' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      [indent="-1"]
      ----
          def names

            @names.split

          end
      ----
      EOS

      expected = (input.lines.slice 2, 5).join.chop

      output = convert_string_to_embedded input
      assert_css 'pre', output, 1
      assert_css '.listingblock pre', output, 1
      result = xmlnodes_at_xpath('//pre', output, 1).text
      assert_equal expected, result
    end

"#
        );

        let output = convert(
            "[indent=\"-1\"]\n----\n    def names\n\n      @names.split\n\n    end\n----\n",
        );
        assert_css(&output, "pre", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_xpath(
            &output,
            "//pre[text()=\"    def names\n\n      @names.split\n\n    end\"]",
            1,
        );
    }

    // Indent normalization (`indent="1"` / `source-indent`).
    #[test]
    fn should_set_block_indent_to_value_specified_by_indent_attribute() {
        verifies!(
            r#"
    test 'should set block indent to value specified by indent attribute' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      [indent="1"]
      ----
          def names

            @names.split

          end
      ----
      EOS

      expected = (input.lines.slice 2, 5).map {|l| l.sub '    ', ' ' }.join.chop

      output = convert_string_to_embedded input
      assert_css 'pre', output, 1
      assert_css '.listingblock pre', output, 1
      result = xmlnodes_at_xpath('//pre', output, 1).text
      assert_equal expected, result
    end

"#
        );

        // `expected` replaces the four-space block indent with a single space on
        // each of the five content lines: ` def names`, ``, `   @names.split`,
        // ``, ` end`.
        let output =
            convert("[indent=\"1\"]\n----\n    def names\n\n      @names.split\n\n    end\n----\n");
        assert_css(&output, "pre", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_xpath(
            &output,
            "//pre[text()=\" def names\n\n   @names.split\n\n end\"]",
            1,
        );
    }

    #[test]
    fn should_set_block_indent_to_value_specified_by_indent_document_attribute() {
        verifies!(
            r#"
    test 'should set block indent to value specified by indent document attribute' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      :source-indent: 1

      [source,ruby]
      ----
          def names

            @names.split

          end
      ----
      EOS

      expected = (input.lines.slice 4, 5).map {|l| l.sub '    ', ' ' }.join.chop

      output = convert_string_to_embedded input
      assert_css 'pre', output, 1
      assert_css '.listingblock pre', output, 1
      result = xmlnodes_at_xpath('//pre', output, 1).text
      assert_equal expected, result
    end

"#
        );

        // The `source-indent` document attribute supplies the indent for the
        // source block, normalizing the four-space indent to one space.
        let output = convert(
            ":source-indent: 1\n\n[source,ruby]\n----\n    def names\n\n      @names.split\n\n    end\n----\n",
        );
        assert_css(&output, "pre", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_xpath(
            &output,
            "//pre[text()=\" def names\n\n   @names.split\n\n end\"]",
            1,
        );
    }

    #[test]
    fn should_expand_tabs_if_tabsize_attribute_is_positive() {
        verifies!(
            r#"
    test 'should expand tabs if tabsize attribute is positive' do
      input = <<~EOS
      :tabsize: 4

      [indent=0]
      ----
      \tdef names

      \t\t@names.split

      \tend
      ----
      EOS

      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      expected = <<~EOS.chop
      def names

          @names.split

      end
      EOS

      output = convert_string_to_embedded input
      assert_css 'pre', output, 1
      assert_css '.listingblock pre', output, 1
      result = xmlnodes_at_xpath('//pre', output, 1).text
      assert_equal expected, result
    end

"#
        );

        // Each leading tab expands to four spaces on the tab stop, then the
        // `indent=0` normalization removes the resulting four-space block
        // indent, leaving the second line indented four spaces.
        let output = convert(
            ":tabsize: 4\n\n[indent=0]\n----\n\tdef names\n\n\t\t@names.split\n\n\tend\n----\n",
        );
        assert_css(&output, "pre", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_xpath(
            &output,
            "//pre[text()=\"def names\n\n    @names.split\n\nend\"]",
            1,
        );
    }

    #[test]
    fn literal_block_should_honor_nowrap_option() {
        verifies!(
            r#"
    test 'literal block should honor nowrap option' do
      input = <<~'EOS'
      [options="nowrap"]
      ----
      Do not wrap me if I get too long.
      ----
      EOS

      output = convert_string_to_embedded input
      assert_css 'pre.nowrap', output, 1
    end

"#
        );

        let output =
            convert("[options=\"nowrap\"]\n----\nDo not wrap me if I get too long.\n----\n");
        assert_css(&output, "pre.nowrap", 1);
    }

    #[test]
    fn literal_block_should_set_nowrap_class_if_prewrap_document_attribute_is_disabled() {
        verifies!(
            r#"
    test 'literal block should set nowrap class if prewrap document attribute is disabled' do
      input = <<~'EOS'
      :prewrap!:

      ----
      Do not wrap me if I get too long.
      ----
      EOS

      output = convert_string_to_embedded input
      assert_css 'pre.nowrap', output, 1
    end

"#
        );

        let output = convert(":prewrap!:\n\n----\nDo not wrap me if I get too long.\n----\n");
        assert_css(&output, "pre.nowrap", 1);
    }

    #[test]
    fn should_preserve_guard_in_front_of_callout_if_icons_are_not_enabled() {
        verifies!(
            r#"
    test 'should preserve guard in front of callout if icons are not enabled' do
      input = <<~'EOS'
      ----
      puts 'Hello, World!' # <1>
      puts 'Goodbye, World ;(' # <2>
      ----
      EOS

      result = convert_string_to_embedded input
      assert_include ' # <b class="conum">(1)</b>', result
      assert_include ' # <b class="conum">(2)</b>', result
    end

"#
        );

        let result =
            convert("----\nputs 'Hello, World!' # <1>\nputs 'Goodbye, World ;(' # <2>\n----\n");
        assert!(result.contains(r#" # <b class="conum">(1)</b>"#));
        assert!(result.contains(r#" # <b class="conum">(2)</b>"#));
    }

    #[test]
    fn should_preserve_guard_around_callout_if_icons_are_not_enabled() {
        verifies!(
            r#"
    test 'should preserve guard around callout if icons are not enabled' do
      input = <<~'EOS'
      ----
      <parent> <!--1-->
        <child/> <!--2-->
      </parent>
      ----
      EOS

      result = convert_string_to_embedded input
      assert_include ' &lt;!--<b class="conum">(1)</b>--&gt;', result
      assert_include ' &lt;!--<b class="conum">(2)</b>--&gt;', result
    end

"#
        );

        let result = convert("----\n<parent> <!--1-->\n  <child/> <!--2-->\n</parent>\n----\n");
        assert!(result.contains(r#" &lt;!--<b class="conum">(1)</b>--&gt;"#));
        assert!(result.contains(r#" &lt;!--<b class="conum">(2)</b>--&gt;"#));
    }

    #[test]
    fn literal_block_should_honor_explicit_subs_list() {
        verifies!(
            r#"
    test 'literal block should honor explicit subs list' do
      input = <<~'EOS'
      [subs="verbatim,quotes"]
      ----
      Map<String, String> *attributes*; //<1>
      ----
      EOS

      block = block_from_string input
      assert_equal [:specialcharacters, :callouts, :quotes], block.subs
      output = block.convert
      assert_includes output, 'Map&lt;String, String&gt; <strong>attributes</strong>;'
      assert_xpath '//pre/b[text()="(1)"]', output, 1
    end

"#
        );

        // `block.subs` is an `asciidoc-parser` model assertion; only the rendered
        // HTML is re-expressed.
        let output = convert(
            "[subs=\"verbatim,quotes\"]\n----\nMap<String, String> *attributes*; //<1>\n----\n",
        );
        assert!(output.contains("Map&lt;String, String&gt; <strong>attributes</strong>;"));
        assert_xpath(&output, r#"//pre/b[text()="(1)"]"#, 1);
    }

    #[test]
    fn should_be_able_to_disable_callouts_for_literal_block() {
        verifies!(
            r#"
    test 'should be able to disable callouts for literal block' do
      input = <<~'EOS'
      [subs="specialcharacters"]
      ----
      No callout here <1>
      ----
      EOS
      block = block_from_string input
      assert_equal [:specialcharacters], block.subs
      output = block.convert
      assert_xpath '//pre/b[text()="(1)"]', output, 0
    end

"#
        );

        // `block.subs` is an `asciidoc-parser` model assertion; only the rendered
        // HTML is re-expressed.
        let output = convert("[subs=\"specialcharacters\"]\n----\nNo callout here <1>\n----\n");
        assert_xpath(&output, r#"//pre/b[text()="(1)"]"#, 0);
    }

    #[test]
    fn listing_block_should_honor_explicit_subs_list() {
        verifies!(
            r#"
    test 'listing block should honor explicit subs list' do
      input = <<~'EOS'
      [subs="specialcharacters,quotes"]
      ----
      $ *python functional_tests.py*
      Traceback (most recent call last):
        File "functional_tests.py", line 4, in <module>
          assert 'Django' in browser.title
      AssertionError
      ----
      EOS

      output = convert_string_to_embedded input

      assert_css '.listingblock pre', output, 1
      assert_css '.listingblock pre strong', output, 1
      assert_css '.listingblock pre em', output, 0

      input2 = <<~'EOS'
      [subs="specialcharacters,macros"]
      ----
      $ pass:quotes[*python functional_tests.py*]
      Traceback (most recent call last):
        File "functional_tests.py", line 4, in <module>
          assert pass:quotes['Django'] in browser.title
      AssertionError
      ----
      EOS

      output2 = convert_string_to_embedded input2
      # FIXME JRuby is adding extra trailing newlines in the second document,
      # for now, rstrip is necessary
      assert_equal output.rstrip, output2.rstrip
    end

"#
        );

        let output = convert("[subs=\"specialcharacters,quotes\"]\n----\n$ *python functional_tests.py*\nTraceback (most recent call last):\n  File \"functional_tests.py\", line 4, in <module>\n    assert 'Django' in browser.title\nAssertionError\n----\n");
        assert_css(&output, ".listingblock pre", 1);
        assert_css(&output, ".listingblock pre strong", 1);
        assert_css(&output, ".listingblock pre em", 0);

        let output2 = convert("[subs=\"specialcharacters,macros\"]\n----\n$ pass:quotes[*python functional_tests.py*]\nTraceback (most recent call last):\n  File \"functional_tests.py\", line 4, in <module>\n    assert pass:quotes['Django'] in browser.title\nAssertionError\n----\n");
        assert_eq!(output.trim_end(), output2.trim_end());
    }

    #[test]
    fn first_character_of_block_title_may_be_a_period_if_not_followed_by_space() {
        verifies!(
            r#"
    test 'first character of block title may be a period if not followed by space' do
      input = <<~'EOS'
      ..gitignore
      ----
      /.bundle/
      /build/
      /Gemfile.lock
      ----
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//*[@class="title"][text()=".gitignore"]', output
    end

"#
        );

        let output = convert("..gitignore\n----\n/.bundle/\n/build/\n/Gemfile.lock\n----\n");
        assert_xpath(&output, r#"//*[@class="title"][text()=".gitignore"]"#, 1);
    }

    // DocBook-backend output is out of scope (this crate targets only `html5`).
    non_normative!(
        r#"
    test 'listing block without title should generate screen element in docbook' do
      input = <<~'EOS'
      ----
      listing block
      ----
      EOS

      output = convert_string_to_embedded input, backend: 'docbook'
      assert_xpath '/screen[text()="listing block"]', output, 1
    end

    test 'listing block with title should generate screen element inside formalpara element in docbook' do
      input = <<~'EOS'
      .title
      ----
      listing block
      ----
      EOS

      output = convert_string_to_embedded input, backend: 'docbook'
      assert_xpath '/formalpara', output, 1
      assert_xpath '/formalpara/title[text()="title"]', output, 1
      assert_xpath '/formalpara/para/screen[text()="listing block"]', output, 1
    end

"#
    );

    #[test]
    fn should_not_prepend_caption_to_title_of_listing_block_with_title_if_listing_caption_attribute_is_not_set(
    ) {
        verifies!(
            r#"
    test 'should not prepend caption to title of listing block with title if listing-caption attribute is not set' do
      input = <<~'EOS'
      .title
      ----
      listing block content
      ----
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="listingblock"][1]/*[@class="title"][text()="title"]', output, 1
    end

"#
        );

        let output = convert(".title\n----\nlisting block content\n----\n");
        assert_xpath(
            &output,
            r#"/*[@class="listingblock"][1]/*[@class="title"][text()="title"]"#,
            1,
        );
    }

    #[test]
    fn should_prepend_caption_specified_by_listing_caption_attribute_and_number_to_title_of_listing_block_with_title(
    ) {
        verifies!(
            r#"
    test 'should prepend caption specified by listing-caption attribute and number to title of listing block with title' do
      input = <<~'EOS'
      :listing-caption: Listing

      .title
      ----
      listing block content
      ----
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="listingblock"][1]/*[@class="title"][text()="Listing 1. title"]', output, 1
    end

"#
        );

        let output =
            convert(":listing-caption: Listing\n\n.title\n----\nlisting block content\n----\n");
        assert_xpath(
            &output,
            r#"/*[@class="listingblock"][1]/*[@class="title"][text()="Listing 1. title"]"#,
            1,
        );
    }

    #[test]
    fn should_prepend_caption_specified_by_caption_attribute_on_listing_block_even_if_listing_caption_attribute_is_not_set(
    ) {
        verifies!(
            r#"
    test 'should prepend caption specified by caption attribute on listing block even if listing-caption attribute is not set' do
      input = <<~'EOS'
      [caption="Listing {counter:listing-number}. "]
      .Behold!
      ----
      listing block content
      ----
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="listingblock"][1]/*[@class="title"][text()="Listing 1. Behold!"]', output, 1
    end

"#
        );

        let output =
            convert("[caption=\"Listing {counter:listing-number}. \"]\n.Behold!\n----\nlisting block content\n----\n");
        assert_xpath(
            &output,
            r#"/*[@class="listingblock"][1]/*[@class="title"][text()="Listing 1. Behold!"]"#,
            1,
        );
    }

    // The listing/source-promotion tests assert `asciidoc-parser` model state
    // (`find_by`, `style`, `attr 'language'`), and the last two target DocBook.
    // Both are out of scope for a rendered-HTML port here.
    non_normative!(
        r#"
    test 'listing block without an explicit style and with a second positional argument should be promoted to a source block' do
      input = <<~'EOS'
      [,ruby]
      ----
      puts 'Hello, Ruby!'
      ----
      EOS
      matches = (document_from_string input).find_by context: :listing, style: 'source'
      assert_equal 1, matches.length
      assert_equal 'ruby', (matches[0].attr 'language')
    end

    test 'listing block without an explicit style should be promoted to a source block if source-language is set' do
      input = <<~'EOS'
      :source-language: ruby

      ----
      puts 'Hello, Ruby!'
      ----
      EOS
      matches = (document_from_string input).find_by context: :listing, style: 'source'
      assert_equal 1, matches.length
      assert_equal 'ruby', (matches[0].attr 'language')
    end

    test 'listing block with an explicit style and a second positional argument should not be promoted to a source block' do
      input = <<~'EOS'
      [listing,ruby]
      ----
      puts 'Hello, Ruby!'
      ----
      EOS
      matches = (document_from_string input).find_by context: :listing
      assert_equal 1, matches.length
      assert_equal 'listing', matches[0].style
      assert_nil matches[0].attr 'language'
    end

    test 'listing block with an explicit style should not be promoted to a source block if source-language is set' do
      input = <<~'EOS'
      :source-language: ruby

      [listing]
      ----
      puts 'Hello, Ruby!'
      ----
      EOS
      matches = (document_from_string input).find_by context: :listing
      assert_equal 1, matches.length
      assert_equal 'listing', matches[0].style
      assert_nil matches[0].attr 'language'
    end

    test 'source block with no title or language should generate screen element in docbook' do
      input = <<~'EOS'
      [source]
      ----
      source block
      ----
      EOS

      output = convert_string_to_embedded input, backend: 'docbook'
      assert_xpath '/screen[@linenumbering="unnumbered"][text()="source block"]', output, 1
    end

    test 'source block with title and no language should generate screen element inside formalpara element for docbook' do
      input = <<~'EOS'
      [source]
      .title
      ----
      source block
      ----
      EOS

      output = convert_string_to_embedded input, backend: 'docbook'
      assert_xpath '/formalpara', output, 1
      assert_xpath '/formalpara/title[text()="title"]', output, 1
      assert_xpath '/formalpara/para/screen[@linenumbering="unnumbered"][text()="source block"]', output, 1
    end
  end

"#
    );
}

mod open_blocks {
    use super::*;

    non_normative!(
        r#"
  context "Open Blocks" do
"#
    );

    #[test]
    fn can_convert_open_block() {
        verifies!(
            r#"
    test "can convert open block" do
      input = <<~'EOS'
      --
      This is an open block.

      It can span multiple lines.
      --
      EOS

      output = convert_string input
      assert_xpath '//*[@class="openblock"]//p', output, 2
    end

"#
        );

        let output = convert_with(
            "--\nThis is an open block.\n\nIt can span multiple lines.\n--\n",
            &Options::new().standalone(true),
        );
        assert_xpath(&output, r#"//*[@class="openblock"]//p"#, 2);
    }

    #[test]
    fn open_block_can_contain_another_block() {
        verifies!(
            r#"
    test "open block can contain another block" do
      input = <<~'EOS'
      --
      This is an open block.

      It can span multiple lines.

      ____
      It can hold great quotes like this one.
      ____
      --
      EOS

      output = convert_string input
      assert_xpath '//*[@class="openblock"]//p', output, 3
      assert_xpath '//*[@class="openblock"]//*[@class="quoteblock"]', output, 1
    end

"#
        );

        let output = convert_with(
            "--\nThis is an open block.\n\nIt can span multiple lines.\n\n____\nIt can hold great quotes like this one.\n____\n--\n",
            &Options::new().standalone(true),
        );
        assert_xpath(&output, r#"//*[@class="openblock"]//p"#, 3);
        assert_xpath(
            &output,
            r#"//*[@class="openblock"]//*[@class="quoteblock"]"#,
            1,
        );
    }

    // DocBook-backend output is out of scope (this crate targets only `html5`).
    non_normative!(
        r#"
    test 'should transfer id and reftext on open block to DocBook output' do
      input = <<~'EOS'
      Check out that <<open>>!

      [[open,Open Block]]
      --
      This is an open block.

      TIP: An open block can have other blocks inside of it.
      --

      Back to our regularly scheduled programming.
      EOS

      output = convert_string input, backend: :docbook, keep_namespaces: true
      assert_css 'article:root > para[xml|id="open"]', output, 1
      assert_css 'article:root > para[xreflabel="Open Block"]', output, 1
      assert_css 'article:root > simpara', output, 2
      assert_css 'article:root > para', output, 1
      assert_css 'article:root > para > simpara', output, 1
      assert_css 'article:root > para > tip', output, 1
    end

    test 'should transfer id and reftext on open paragraph to DocBook output' do
      input = <<~'EOS'
      [open#openpara,reftext="Open Paragraph"]
      This is an open paragraph.
      EOS

      output = convert_string input, backend: :docbook, keep_namespaces: true
      assert_css 'article:root > simpara', output, 1
      assert_css 'article:root > simpara[xml|id="openpara"]', output, 1
      assert_css 'article:root > simpara[xreflabel="Open Paragraph"]', output, 1
    end

    test 'should transfer title on open block to DocBook output' do
      input = <<~'EOS'
      .Behold the open
      --
      This is an open block with a title.
      --
      EOS

      output = convert_string input, backend: :docbook
      assert_css 'article > formalpara', output, 1
      assert_css 'article > formalpara > *', output, 2
      assert_css 'article > formalpara > title', output, 1
      assert_xpath '/article/formalpara/title[text()="Behold the open"]', output, 1
      assert_css 'article > formalpara > para', output, 1
      assert_css 'article > formalpara > para > simpara', output, 1
    end

    test 'should transfer title on open paragraph to DocBook output' do
      input = <<~'EOS'
      .Behold the open
      This is an open paragraph with a title.
      EOS

      output = convert_string input, backend: :docbook
      assert_css 'article > formalpara', output, 1
      assert_css 'article > formalpara > *', output, 2
      assert_css 'article > formalpara > title', output, 1
      assert_xpath '/article/formalpara/title[text()="Behold the open"]', output, 1
      assert_css 'article > formalpara > para', output, 1
      assert_css 'article > formalpara > para[text()="This is an open paragraph with a title."]', output, 1
    end

    test 'should transfer role on open block to DocBook output' do
      input = <<~'EOS'
      [.container]
      --
      This is an open block.
      It holds stuff.
      --
      EOS

      output = convert_string input, backend: :docbook
      assert_css 'article > para[role=container]', output, 1
      assert_css 'article > para[role=container] > simpara', output, 1
    end

    test 'should transfer role on open paragraph to DocBook output' do
      input = <<~'EOS'
      [.container]
      This is an open block.
      It holds stuff.
      EOS

      output = convert_string input, backend: :docbook
      assert_css 'article > simpara[role=container]', output, 1
    end
  end

"#
    );
}
