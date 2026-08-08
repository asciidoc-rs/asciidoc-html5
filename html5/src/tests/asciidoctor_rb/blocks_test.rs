//! Port of Asciidoctor's `blocks_test.rb` — the front half plus the
//! `Passthrough Blocks`, `Math blocks`, `Custom Blocks`, `Metadata`, and
//! `Images` contexts (through source line 3069).
//!
//! This crate already renders the block types these contexts exercise — layout
//! breaks, comments, sidebar/quote/verse/example/admonition/open blocks,
//! verbatim (listing/literal/source) blocks, passthrough blocks, STEM
//! (`stem`/`latexmath`/`asciimath`) blocks, and block images (including SVG
//! `interactive`/`inline` modes and `data-uri` embedding) — so they port
//! directly, driven through `convert` (embedded) /
//! `convert_with(..standalone(true)..)`.
//!
//! The remaining back half (Media, Admonition icons, Source code,
//! Abstract/Part Intro, Substitutions, References — lines 3070+) is being
//! sequenced as its own implement-then-port work.
//!
//! In the `Images` context, what stays `non_normative!` is the DocBook backend,
//! remote fetch (`allow-uri-read`/`cache-uri` — a non-goal), the JRuby
//! classloader embed, and the Ruby-only integer-`set_attr` crash guard; a few
//! render-time image/SVG read *warnings* Asciidoctor logs are noted where the
//! observable rendering is verified but the crate performs the read silently.
//!
//! What stays `non_normative!` through the ported back-half contexts: only the
//! DocBook-backend equation tests in `Math blocks` — the four
//! `eqnums`/`autoNumber` tests there are verified directly, since the MathJax
//! docinfo they assert now ships.
//!
//! What stays `non_normative!` in the front half:
//! - DocBook-backend tests (this crate targets only the `html5` backend);
//! - `asciidoc-parser` parser-model assertions (`document_from_string` +
//!   `blocks[..].numeral`/`content`/`subs`, `find_by`, `block_from_string`) —
//!   only the rendered HTML of such tests is re-expressed here;
//! - the `markdown_syntax` compliance-toggle test (no compliance API here).
//!
//! Logger assertions (`assert_message @logger, :WARN, …`) are verified against
//! the document's warnings inventory via [`assert_warning`].

use asciidoc_parser::{
    blocks::{FindBlocks, IsBlock},
    warnings::{WarningSeverity, WarningType},
};

use crate::{
    convert, convert_with, load,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
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

    #[test]
    fn assigns_sequential_character_caption_to_example_block_with_title() {
        verifies!(
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

"#
        );

        // The `numeral`/`number`/`example-number` checks are `asciidoc-parser`
        // model assertions; only the rendered captions are re-expressed here. An
        // alphabetic counter seed (`:example-number: @`, one before `A`) advances
        // the caption numeral through the letters.
        let output = convert(
            ":example-number: @\n\n.Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n\n.Writing Docs with DocBook\n====\nHere's how you write DocBook.\n\nYou futz with XML.\n====\n",
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example A. Writing Docs with AsciiDoc"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example B. Writing Docs with DocBook"]"#,
            1,
        );
    }

    #[test]
    fn should_increment_counter_for_example_even_when_example_number_is_locked_by_the_api() {
        verifies!(
            r#"
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

"#
        );

        // An API-supplied attribute is locked (override precedence), yet a caption
        // counter still advances it: seeded one before `a` (`` ` ``), the two
        // examples number `a` and `b`. The `example-number` model assertion is an
        // `asciidoc-parser` check; only the rendered captions are re-expressed.
        let output = convert_with(
            ".Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n\n.Writing Docs with DocBook\n====\nHere's how you write DocBook.\n\nYou futz with XML.\n====\n",
            &Options::new().attribute("example-number", "`"),
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Example a. Writing Docs with AsciiDoc"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[2]/*[@class="title"][text()="Example b. Writing Docs with DocBook"]"#,
            1,
        );
    }

    #[test]
    fn should_use_explicit_caption_if_specified() {
        verifies!(
            r#"
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

"#
        );

        // An explicit `[caption=]` override is used verbatim and consumes no
        // counter. The `numeral`/`example-number` checks are `asciidoc-parser`
        // model assertions; only the rendered caption is re-expressed here.
        let output = convert(
            "[caption=\"Look! \"]\n.Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n",
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]"#,
            1,
        );
    }

    #[test]
    fn automatic_caption_can_be_turned_off_and_on_and_modified() {
        verifies!(
            r#"
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

"#
        );

        // A set-but-empty document `caption` (`:caption:`) suppresses the caption
        // and consumes no counter, so the second example shows only its title and
        // the third — after `:caption!:` restores auto-numbering and
        // `:example-caption: Exhibit` relabels it — is "Exhibit 2".
        let output = convert(
            ".first example\n====\nan example\n====\n\n:caption:\n\n.second example\n====\nanother example\n====\n\n:caption!:\n:example-caption: Exhibit\n\n.third example\n====\nyet another example\n====\n",
        );
        assert_xpath(&output, r#"/*[@class="exampleblock"]"#, 3);
        assert_xpath(
            &output,
            r#"(/*[@class="exampleblock"])[1]/*[@class="title"][starts-with(text(), "Example ")]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(/*[@class="exampleblock"])[2]/*[@class="title"][text()="second example"]"#,
            1,
        );
        // Pinned to the exact numeral (stronger than the reference's
        // `starts-with(…, "Exhibit ")`): the empty `:caption:` above must consume
        // no counter, so the third example is "Exhibit 2", not "Exhibit 3".
        assert_xpath(
            &output,
            r#"(/*[@class="exampleblock"])[3]/*[@class="title"][text()="Exhibit 2. third example"]"#,
            1,
        );
    }

    #[test]
    fn should_use_explicit_caption_if_specified_even_if_block_specific_global_caption_is_disabled()
    {
        verifies!(
            r#"
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

"#
        );

        // An explicit `[caption=]` override still applies even when the
        // block-specific global caption is disabled (`:!example-caption:`). The
        // `numeral`/`example-number` checks are `asciidoc-parser` model
        // assertions; only the rendered caption is re-expressed here.
        let output = convert(
            ":!example-caption:\n\n[caption=\"Look! \"]\n.Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n",
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]"#,
            1,
        );
    }

    #[test]
    fn should_use_global_caption_if_specified_even_if_block_specific_global_caption_is_disabled() {
        verifies!(
            r#"
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

        // A document-wide `caption` attribute supplies a verbatim, unnumbered
        // label even when the block-specific global caption is disabled
        // (`:!example-caption:`); `{sp}` expands to the trailing space. The
        // `numeral`/`example-number` checks are `asciidoc-parser` model
        // assertions; only the rendered caption is re-expressed here.
        let output = convert(
            ":!example-caption:\n:caption: Look!{sp}\n\n.Writing Docs with AsciiDoc\n====\nHere's how you write AsciiDoc.\n\nYou just write.\n====\n",
        );
        assert_xpath(
            &output,
            r#"(//*[@class="exampleblock"])[1]/*[@class="title"][text()="Look! Writing Docs with AsciiDoc"]"#,
            1,
        );
    }

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

    #[test]
    fn should_create_details_summary_set_if_collapsible_option_is_set() {
        verifies!(
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

"#
        );

        let output = convert(
            ".Toggle Me\n[%collapsible]\n====\n\
             This content is revealed when the user clicks the words \"Toggle Me\".\n====\n",
        );
        assert_css(&output, "details", 1);
        assert_css(&output, "details[open]", 0);
        assert_css(&output, "details > summary.title", 1);
        assert_xpath(&output, r#"//details/summary[text()="Toggle Me"]"#, 1);
        assert_css(&output, "details > summary.title + .content", 1);
        assert_css(&output, "details > summary.title + .content p", 1);
    }

    #[test]
    fn should_open_details_summary_set_if_collapsible_and_open_options_are_set() {
        verifies!(
            r#"
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

"#
        );

        let output = convert(
            ".Toggle Me\n[%collapsible%open]\n====\n\
             This content is revealed when the user clicks the words \"Toggle Me\".\n====\n",
        );
        assert_css(&output, "details", 1);
        assert_css(&output, "details[open]", 1);
        assert_css(&output, "details > summary.title", 1);
        assert_xpath(&output, r#"//details/summary[text()="Toggle Me"]"#, 1);
    }

    #[test]
    fn should_add_default_summary_element_if_collapsible_option_is_set_and_title_is_not_specifed() {
        verifies!(
            r#"
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

"#
        );

        let output = convert(
            "[%collapsible]\n====\n\
             This content is revealed when the user clicks the words \"Details\".\n====\n",
        );
        assert_css(&output, "details", 1);
        assert_css(&output, "details > summary.title", 1);
        assert_xpath(&output, r#"//details/summary[text()="Details"]"#, 1);
    }

    #[test]
    fn should_not_allow_collapsible_block_to_increment_example_number() {
        verifies!(
            r#"
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

        let output = convert(
            ".Before\n====\nbefore\n====\n\n\
             .Show Me The Goods\n[%collapsible]\n====\n\
             This content is revealed when the user clicks the words \"Show Me The Goods\".\n====\n\n\
             .After\n====\nafter\n====\n",
        );
        assert_xpath(
            &output,
            r#"//*[@class="title"][text()="Example 1. Before"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="title"][text()="Example 2. After"]"#,
            1,
        );
        assert_css(&output, "details", 1);
        assert_css(&output, "details > summary.title", 1);
        assert_xpath(
            &output,
            r#"//details/summary[text()="Show Me The Goods"]"#,
            1,
        );
    }

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
        // The Ruby assertion reads the `<pre>`'s full descendant text; a source
        // block wraps its code in `<code>`, so the reindented content is that
        // element's text (this crate renders a delimited `[source]` block as
        // `<pre class="highlight"><code …>`, matching Asciidoctor — unlike a
        // plain `[listing]`/`----` block, which keeps its text directly on the
        // `<pre>`).
        assert_xpath(
            &output,
            "//pre/code[text()=\" def names\n\n   @names.split\n\n end\"]",
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

mod passthrough_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Passthrough Blocks' do
"#
    );

    // `block_from_string` + `block.lines`/`block.source` inspect the parser's
    // block model, which `asciidoc-parser` verifies; this crate has no rendered
    // output to re-express for it.
    non_normative!(
        r#"
    test 'can parse a passthrough block' do
      input = <<~'EOS'
      ++++
      This is a passthrough block.
      ++++
      EOS

      block = block_from_string input
      refute_nil block
      assert_equal 1, block.lines.size
      assert_equal 'This is a passthrough block.', block.source
    end

"#
    );

    #[test]
    fn does_not_perform_subs_on_a_passthrough_block_by_default() {
        verifies!(
            r#"
    test 'does not perform subs on a passthrough block by default' do
      input = <<~'EOS'
      :type: passthrough

      ++++
      This is a '{type}' block.
      http://asciidoc.org
      image:tiger.png[]
      ++++
      EOS

      expected = %(This is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[])
      output = convert_string_to_embedded input
      assert_equal expected, output.strip
    end

"#
        );

        let input =
            ":type: passthrough\n\n++++\nThis is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[]\n++++\n";
        let expected = "This is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[]";

        // The crate's embedded output carries a single trailing newline, which
        // Ruby's `output.strip` removes; `trim` mirrors that here.
        assert_eq!(convert(input).trim(), expected);
    }

    #[test]
    fn does_not_perform_subs_on_a_passthrough_block_with_pass_style_by_default() {
        verifies!(
            r#"
    test 'does not perform subs on a passthrough block with pass style by default' do
      input = <<~'EOS'
      :type: passthrough

      [pass]
      ++++
      This is a '{type}' block.
      http://asciidoc.org
      image:tiger.png[]
      ++++
      EOS

      expected = %(This is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[])
      output = convert_string_to_embedded input
      assert_equal expected, output.strip
    end

"#
        );

        let input =
            ":type: passthrough\n\n[pass]\n++++\nThis is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[]\n++++\n";
        let expected = "This is a '{type}' block.\nhttp://asciidoc.org\nimage:tiger.png[]";

        assert_eq!(convert(input).trim(), expected);
    }

    #[test]
    fn passthrough_block_honors_explicit_subs_list() {
        verifies!(
            r#"
    test 'passthrough block honors explicit subs list' do
      input = <<~'EOS'
      :type: passthrough

      [subs="attributes,quotes,macros"]
      ++++
      This is a _{type}_ block.
      http://asciidoc.org
      ++++
      EOS

      expected = %(This is a <em>passthrough</em> block.\n<a href="http://asciidoc.org" class="bare">http://asciidoc.org</a>)
      output = convert_string_to_embedded input
      assert_equal expected, output.strip
    end

"#
        );

        let input =
            ":type: passthrough\n\n[subs=\"attributes,quotes,macros\"]\n++++\nThis is a _{type}_ block.\nhttp://asciidoc.org\n++++\n";
        let expected =
            "This is a <em>passthrough</em> block.\n<a href=\"http://asciidoc.org\" class=\"bare\">http://asciidoc.org</a>";

        assert_eq!(convert(input).trim(), expected);
    }

    #[test]
    fn should_strip_leading_and_trailing_blank_lines_when_converting_raw_block() {
        verifies!(
            r#"
    test 'should strip leading and trailing blank lines when converting raw block' do
      # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
      input = <<~EOS
      ++++
      line above
      ++++

      ++++


        first line

      last line


      ++++

      ++++
      line below
      ++++
      EOS

      doc = document_from_string input, standalone: false
      block = doc.blocks[1]
      assert_equal ['', '', '  first line', '', 'last line', '', ''], block.lines
      result = doc.convert
      assert_equal "line above\n  first line\n\nlast line\nline below", result, 1
    end
  end

"#
        );

        // The `block.lines` assertion inspects the parser's line buffer (verified
        // in `asciidoc-parser`); here we drive the rendered output. Each raw
        // block trims its own leading and trailing blank lines, so the three
        // passthrough blocks emit only their non-blank content.
        let input = "++++\nline above\n++++\n\n++++\n\n\n  first line\n\nlast line\n\n\n++++\n\n++++\nline below\n++++\n";

        // Ruby's `result` has no trailing newline; the crate's embedded output —
        // matching the `asciidoctor` CLI — carries the usual single trailing one.
        let expected = "line above\n  first line\n\nlast line\nline below\n";

        assert_eq!(convert(input), expected);
    }
}

mod math_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Math blocks' do
"#
    );

    #[test]
    fn should_not_crash_when_converting_stem_block_that_has_no_lines() {
        verifies!(
            r#"
    test 'should not crash when converting stem block that has no lines' do
      input = <<~'EOS'
      [stem]
      ++++
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
    end

"#
        );

        let html = convert("[stem]\n++++\n++++\n");
        assert_css(&html, ".stemblock", 1);
    }

    #[test]
    fn should_return_content_as_empty_string_for_stem_or_pass_block_that_has_no_lines() {
        verifies!(
            r#"
    test 'should return content as empty string for stem or pass block that has no lines' do
      [%(++++\n++++), %([stem]\n++++\n++++)].each do |input|
        doc = document_from_string input
        assert_equal '', doc.blocks[0].content
      end
    end

"#
        );

        // `doc.blocks[0].content` maps to the first top-level block's
        // post-substitution content, reached through `load`. An empty raw block —
        // both the bare `++++` (pass) and the `[stem]` form — has an empty
        // content string.
        for input in ["++++\n++++", "[stem]\n++++\n++++"] {
            let doc = load(input);
            let block = doc.child_blocks().next().expect("one top-level block");
            assert_eq!(block.rendered_content(), Some(""), "{input}");
        }
    }

    #[test]
    fn should_add_latex_math_delimiters_around_latexmath_block_content() {
        verifies!(
            r#"
    test 'should add LaTeX math delimiters around latexmath block content' do
      input = <<~'EOS'
      [latexmath]
      ++++
      \sqrt{3x-1}+(1+x)^2 < y
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
      assert_equal '\[\sqrt{3x-1}+(1+x)^2 &lt; y\]', nodes.first.to_s.strip
    end

"#
        );

        let html = convert("[latexmath]\n++++\n\\sqrt{3x-1}+(1+x)^2 < y\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains("<div class=\"content\">\n\\[\\sqrt{3x-1}+(1+x)^2 &lt; y\\]\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn should_not_add_latex_math_delimiters_around_latexmath_block_content_if_already_present() {
        verifies!(
            r#"
    test 'should not add LaTeX math delimiters around latexmath block content if already present' do
      input = <<~'EOS'
      [latexmath]
      ++++
      \[\sqrt{3x-1}+(1+x)^2 < y\]
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
      assert_equal '\[\sqrt{3x-1}+(1+x)^2 &lt; y\]', nodes.first.to_s.strip
    end

"#
        );

        let html = convert("[latexmath]\n++++\n\\[\\sqrt{3x-1}+(1+x)^2 < y\\]\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains("<div class=\"content\">\n\\[\\sqrt{3x-1}+(1+x)^2 &lt; y\\]\n</div>"),
            "{html}"
        );
    }

    // The DocBook backend emits an `<informalequation>` with the raw equation in
    // a CDATA `<alt>`/`<mathphrase>`; this crate targets only the `html5`
    // backend, so there is no DocBook output to assert.
    non_normative!(
        r#"
    test 'should display latexmath block in alt of equation in DocBook backend' do
      input = <<~'EOS'
      [latexmath]
      ++++
      \sqrt{3x-1}+(1+x)^2 < y
      ++++
      EOS

      expect = <<~'EOS'
      <informalequation>
      <alt><![CDATA[\sqrt{3x-1}+(1+x)^2 < y]]></alt>
      <mathphrase><![CDATA[\sqrt{3x-1}+(1+x)^2 < y]]></mathphrase>
      </informalequation>
      EOS

      output = convert_string_to_embedded input, backend: :docbook
      assert_equal expect.strip, output.strip
    end

"#
    );

    #[test]
    fn should_set_auto_number_option_for_latexmath_to_none_by_default() {
        verifies!(
            r#"
    test 'should set autoNumber option for latexmath to none by default' do
      input = <<~'EOS'
      :stem: latexmath

      [stem]
      ++++
      y = x^2
      ++++
      EOS

      output = convert_string input
      assert_includes output, 'TeX: { equationNumbers: { autoNumber: "none" } }'
    end

"#
        );

        let output = convert_with(
            ":stem: latexmath\n\n[stem]\n++++\ny = x^2\n++++\n",
            &Options::new().standalone(true),
        );
        assert!(output.contains(r#"TeX: { equationNumbers: { autoNumber: "none" } }"#));
    }

    #[test]
    fn should_set_auto_number_option_for_latexmath_to_none_if_eqnums_is_set_to_none() {
        verifies!(
            r#"
    test 'should set autoNumber option for latexmath to none if eqnums is set to none' do
      input = <<~'EOS'
      :stem: latexmath
      :eqnums: none

      [stem]
      ++++
      y = x^2
      ++++
      EOS

      output = convert_string input
      assert_includes output, 'TeX: { equationNumbers: { autoNumber: "none" } }'
    end

"#
        );

        let output = convert_with(
            ":stem: latexmath\n:eqnums: none\n\n[stem]\n++++\ny = x^2\n++++\n",
            &Options::new().standalone(true),
        );
        assert!(output.contains(r#"TeX: { equationNumbers: { autoNumber: "none" } }"#));
    }

    #[test]
    fn should_set_auto_number_option_for_latexmath_to_ams_if_eqnums_is_set() {
        verifies!(
            r#"
    test 'should set autoNumber option for latexmath to AMS if eqnums is set' do
      input = <<~'EOS'
      :stem: latexmath
      :eqnums:

      [stem]
      ++++
      \begin{equation}
      y = x^2
      \end{equation}
      ++++
      EOS

      output = convert_string input
      assert_includes output, 'TeX: { equationNumbers: { autoNumber: "AMS" } }'
    end

"#
        );

        let output = convert_with(
            ":stem: latexmath\n:eqnums:\n\n[stem]\n++++\n\\begin{equation}\ny = x^2\n\\end{equation}\n++++\n",
            &Options::new().standalone(true),
        );
        assert!(output.contains(r#"TeX: { equationNumbers: { autoNumber: "AMS" } }"#));
    }

    #[test]
    fn should_set_auto_number_option_for_latexmath_to_all_if_eqnums_is_set_to_all() {
        verifies!(
            r#"
    test 'should set autoNumber option for latexmath to all if eqnums is set to all' do
      input = <<~'EOS'
      :stem: latexmath
      :eqnums: all

      [stem]
      ++++
      y = x^2
      ++++
      EOS

      output = convert_string input
      assert_includes output, 'TeX: { equationNumbers: { autoNumber: "all" } }'
    end

"#
        );

        let output = convert_with(
            ":stem: latexmath\n:eqnums: all\n\n[stem]\n++++\ny = x^2\n++++\n",
            &Options::new().standalone(true),
        );
        assert!(output.contains(r#"TeX: { equationNumbers: { autoNumber: "all" } }"#));
    }

    #[test]
    fn should_not_split_equation_in_asciimath_block_at_single_newline() {
        verifies!(
            r#"
    test 'should not split equation in AsciiMath block at single newline' do
      input = <<~'EOS'
      [asciimath]
      ++++
      f: bbb"N" -> bbb"N"
      f: x |-> x + 1
      ++++
      EOS
      expected = <<~'EOS'.chop
      \$f: bbb"N" -&gt; bbb"N"
      f: x |-&gt; x + 1\$
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]', output
      assert_equal expected, nodes.first.inner_html.strip
    end

"#
        );

        let html = convert("[asciimath]\n++++\nf: bbb\"N\" -> bbb\"N\"\nf: x |-> x + 1\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains(
                "<div class=\"content\">\n\\$f: bbb\"N\" -&gt; bbb\"N\"\nf: x |-&gt; x + 1\\$\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn should_split_equation_in_asciimath_block_at_escaped_newline() {
        verifies!(
            r#"
    test 'should split equation in AsciiMath block at escaped newline' do
      input = <<~'EOS'
      [asciimath]
      ++++
      f: bbb"N" -> bbb"N" \
      f: x |-> x + 1
      ++++
      EOS
      expected = <<~'EOS'.chop
      \$f: bbb"N" -&gt; bbb"N"\$
      \$f: x |-&gt; x + 1\$
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]', output
      assert_equal expected, nodes.first.inner_html.strip
    end

"#
        );

        let html = convert("[asciimath]\n++++\nf: bbb\"N\" -> bbb\"N\" \\\nf: x |-> x + 1\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains(
                "<div class=\"content\">\n\\$f: bbb\"N\" -&gt; bbb\"N\"\\$\n\\$f: x |-&gt; x + 1\\$\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn should_split_equation_in_asciimath_block_at_sequence_of_escaped_newlines() {
        verifies!(
            r#"
    test 'should split equation in AsciiMath block at sequence of escaped newlines' do
      input = <<~'EOS'
      [asciimath]
      ++++
      f: bbb"N" -> bbb"N" \
      \
      f: x |-> x + 1
      ++++
      EOS
      expected = <<~'EOS'.chop
      \$f: bbb"N" -&gt; bbb"N"\$
      <br>
      \$f: x |-&gt; x + 1\$
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]', output
      assert_equal expected, nodes.first.inner_html.strip
    end

"#
        );

        let html =
            convert("[asciimath]\n++++\nf: bbb\"N\" -> bbb\"N\" \\\n\\\nf: x |-> x + 1\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains(
                "<div class=\"content\">\n\\$f: bbb\"N\" -&gt; bbb\"N\"\\$\n<br>\n\\$f: x |-&gt; x + 1\\$\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn should_split_equation_in_asciimath_block_at_newline_sequence_and_preserve_breaks() {
        verifies!(
            r#"
    test 'should split equation in AsciiMath block at newline sequence and preserve breaks' do
      input = <<~'EOS'
      [asciimath]
      ++++
      f: bbb"N" -> bbb"N"


      f: x |-> x + 1
      ++++
      EOS
      expected = <<~'EOS'.chop
      \$f: bbb"N" -&gt; bbb"N"\$
      <br>
      <br>
      \$f: x |-&gt; x + 1\$
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]', output
      assert_equal expected, nodes.first.inner_html.strip
    end

"#
        );

        let html =
            convert("[asciimath]\n++++\nf: bbb\"N\" -> bbb\"N\"\n\n\nf: x |-> x + 1\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains(
                "<div class=\"content\">\n\\$f: bbb\"N\" -&gt; bbb\"N\"\\$\n<br>\n<br>\n\\$f: x |-&gt; x + 1\\$\n</div>"
            ),
            "{html}"
        );
    }

    #[test]
    fn should_add_asciimath_delimiters_around_asciimath_block_content() {
        verifies!(
            r#"
    test 'should add AsciiMath delimiters around asciimath block content' do
      input = <<~'EOS'
      [asciimath]
      ++++
      sqrt(3x-1)+(1+x)^2 < y
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
      assert_equal '\$sqrt(3x-1)+(1+x)^2 &lt; y\$', nodes.first.to_s.strip
    end

"#
        );

        let html = convert("[asciimath]\n++++\nsqrt(3x-1)+(1+x)^2 < y\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains("<div class=\"content\">\n\\$sqrt(3x-1)+(1+x)^2 &lt; y\\$\n</div>"),
            "{html}"
        );
    }

    #[test]
    fn should_not_add_asciimath_delimiters_around_asciimath_block_content_if_already_present() {
        verifies!(
            r#"
    test 'should not add AsciiMath delimiters around asciimath block content if already present' do
      input = <<~'EOS'
      [asciimath]
      ++++
      \$sqrt(3x-1)+(1+x)^2 < y\$
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
      assert_equal '\$sqrt(3x-1)+(1+x)^2 &lt; y\$', nodes.first.to_s.strip
    end

"#
        );

        let html = convert("[asciimath]\n++++\n\\$sqrt(3x-1)+(1+x)^2 < y\\$\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains("<div class=\"content\">\n\\$sqrt(3x-1)+(1+x)^2 &lt; y\\$\n</div>"),
            "{html}"
        );
    }

    // MathML conversion of AsciiMath is a DocBook-backend feature gated on the
    // optional `asciimath` Ruby gem; this crate targets only the `html5`
    // backend and has no such conversion to assert.
    non_normative!(
        r#"
    test 'should convert contents of asciimath block to MathML in DocBook output if asciimath gem is available' do
      asciimath_available = !(Asciidoctor::Helpers.require_library 'asciimath', true, :ignore).nil?
      input = <<~'EOS'
      [asciimath]
      ++++
      x+b/(2a)<+-sqrt((b^2)/(4a^2)-c/a)
      ++++

      [asciimath]
      ++++
      ++++
      EOS

      expect = <<~'EOS'.chop
      <informalequation>
      <mml:math xmlns:mml="http://www.w3.org/1998/Math/MathML"><mml:mi>x</mml:mi><mml:mo>+</mml:mo><mml:mfrac><mml:mi>b</mml:mi><mml:mrow><mml:mn>2</mml:mn><mml:mi>a</mml:mi></mml:mrow></mml:mfrac><mml:mo>&lt;</mml:mo><mml:mo>&#xB1;</mml:mo><mml:msqrt><mml:mrow><mml:mfrac><mml:msup><mml:mi>b</mml:mi><mml:mn>2</mml:mn></mml:msup><mml:mrow><mml:mn>4</mml:mn><mml:msup><mml:mi>a</mml:mi><mml:mn>2</mml:mn></mml:msup></mml:mrow></mml:mfrac><mml:mo>&#x2212;</mml:mo><mml:mfrac><mml:mi>c</mml:mi><mml:mi>a</mml:mi></mml:mfrac></mml:mrow></mml:msqrt></mml:math>
      </informalequation>
      <informalequation>
      <mml:math xmlns:mml="http://www.w3.org/1998/Math/MathML"></mml:math>
      </informalequation>
      EOS

      using_memory_logger do |logger|
        doc = document_from_string input, backend: :docbook, standalone: false
        actual = doc.convert
        if asciimath_available
          assert_equal expect, actual.strip
          assert_equal :loaded, doc.converter.instance_variable_get(:@asciimath_status)
        else
          assert_message logger, :WARN, 'optional gem \'asciimath\' is not available. Functionality disabled.'
          assert_equal :unavailable, doc.converter.instance_variable_get(:@asciimath_status)
        end
      end
    end

"#
    );

    #[test]
    fn should_output_title_for_latexmath_block_if_defined() {
        verifies!(
            r#"
    test 'should output title for latexmath block if defined' do
      input = <<~'EOS'
      .The Lorenz Equations
      [latexmath]
      ++++
      \begin{aligned}
      \dot{x} & = \sigma(y-x) \\
      \dot{y} & = \rho x - y - xz \\
      \dot{z} & = -\beta z + xy
      \end{aligned}
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      assert_css '.stemblock .title', output, 1
      assert_xpath '//*[@class="title"][text()="The Lorenz Equations"]', output, 1
    end

"#
        );

        let html = convert(
            ".The Lorenz Equations\n[latexmath]\n++++\n\\begin{aligned}\n\\dot{x} & = \\sigma(y-x) \\\\\n\\dot{y} & = \\rho x - y - xz \\\\\n\\dot{z} & = -\\beta z + xy\n\\end{aligned}\n++++\n",
        );
        assert_css(&html, ".stemblock", 1);
        assert_css(&html, ".stemblock .title", 1);
        assert_xpath(
            &html,
            r#"//*[@class="title"][text()="The Lorenz Equations"]"#,
            1,
        );
    }

    #[test]
    fn should_output_title_for_asciimath_block_if_defined() {
        verifies!(
            r#"
    test 'should output title for asciimath block if defined' do
      input = <<~'EOS'
      .Simple fraction
      [asciimath]
      ++++
      a//b
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_css '.stemblock', output, 1
      assert_css '.stemblock .title', output, 1
      assert_xpath '//*[@class="title"][text()="Simple fraction"]', output, 1
    end

"#
        );

        let html = convert(".Simple fraction\n[asciimath]\n++++\na//b\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert_css(&html, ".stemblock .title", 1);
        assert_xpath(&html, r#"//*[@class="title"][text()="Simple fraction"]"#, 1);
    }

    #[test]
    fn should_add_asciimath_delimiters_around_stem_block_content_for_asciimath_empty_or_unset() {
        verifies!(
            r#"
    test 'should add AsciiMath delimiters around stem block content if stem attribute is asciimath, empty, or not set' do
      input = <<~'EOS'
      [stem]
      ++++
      sqrt(3x-1)+(1+x)^2 < y
      ++++
      EOS

      [
        {},
        { 'stem' => '' },
        { 'stem' => 'asciimath' },
        { 'stem' => 'bogus' },
      ].each do |attributes|
        output = convert_string_to_embedded input, attributes: attributes
        assert_css '.stemblock', output, 1
        nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
        assert_equal '\$sqrt(3x-1)+(1+x)^2 &lt; y\$', nodes.first.to_s.strip
      end
    end

"#
        );

        let input = "[stem]\n++++\nsqrt(3x-1)+(1+x)^2 < y\n++++\n";

        // A bare `[stem]` with the `stem` document attribute unset, empty,
        // `asciimath`, or an unrecognized value (`bogus`) all render with the
        // AsciiMath delimiter pair.
        for html in [
            convert(input),
            convert_with(input, &Options::new().attribute("stem", "")),
            convert_with(input, &Options::new().attribute("stem", "asciimath")),
            convert_with(input, &Options::new().attribute("stem", "bogus")),
        ] {
            assert_css(&html, ".stemblock", 1);
            assert!(
                html.contains("<div class=\"content\">\n\\$sqrt(3x-1)+(1+x)^2 &lt; y\\$\n</div>"),
                "{html}"
            );
        }
    }

    #[test]
    fn should_add_latex_math_delimiters_around_stem_block_content_for_latexmath_latex_or_tex() {
        verifies!(
            r#"
    test 'should add LaTeX math delimiters around stem block content if stem attribute is latexmath, latex, or tex' do
      input = <<~'EOS'
      [stem]
      ++++
      \sqrt{3x-1}+(1+x)^2 < y
      ++++
      EOS

      [
        { 'stem' => 'latexmath' },
        { 'stem' => 'latex' },
        { 'stem' => 'tex' },
      ].each do |attributes|
        output = convert_string_to_embedded input, attributes: attributes
        assert_css '.stemblock', output, 1
        nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
        assert_equal '\[\sqrt{3x-1}+(1+x)^2 &lt; y\]', nodes.first.to_s.strip
      end
    end

"#
        );

        let input = "[stem]\n++++\n\\sqrt{3x-1}+(1+x)^2 < y\n++++\n";

        // The `latexmath`, `latex`, and `tex` aliases all select LaTeX notation.
        for html in [
            convert_with(input, &Options::new().attribute("stem", "latexmath")),
            convert_with(input, &Options::new().attribute("stem", "latex")),
            convert_with(input, &Options::new().attribute("stem", "tex")),
        ] {
            assert_css(&html, ".stemblock", 1);
            assert!(
                html.contains("<div class=\"content\">\n\\[\\sqrt{3x-1}+(1+x)^2 &lt; y\\]\n</div>"),
                "{html}"
            );
        }
    }

    #[test]
    fn should_allow_stem_style_to_be_set_using_second_positional_argument() {
        verifies!(
            r#"
    test 'should allow stem style to be set using second positional argument of block attributes' do
      input = <<~'EOS'
      :stem: latexmath

      [stem,asciimath]
      ++++
      sqrt(3x-1)+(1+x)^2 < y
      ++++
      EOS

      doc = document_from_string input
      stemblock = doc.blocks[0]
      assert_equal :stem, stemblock.context
      assert_equal 'asciimath', stemblock.attributes['style']
      output = doc.convert standalone: false
      assert_css '.stemblock', output, 1
      nodes = xmlnodes_at_xpath '//*[@class="content"]/child::text()', output
      assert_equal '\$sqrt(3x-1)+(1+x)^2 &lt; y\$', nodes.first.to_s.strip
    end
  end

"#
        );

        // The `stemblock.context`/`attributes['style']` assertions are parser
        // model (verified in `asciidoc-parser`); here we drive the rendered
        // output. With `:stem: latexmath`, the block's second positional
        // (`[stem,asciimath]`) still overrides the document default, so the
        // equation renders with the AsciiMath delimiter pair.
        let html =
            convert(":stem: latexmath\n\n[stem,asciimath]\n++++\nsqrt(3x-1)+(1+x)^2 < y\n++++\n");
        assert_css(&html, ".stemblock", 1);
        assert!(
            html.contains("<div class=\"content\">\n\\$sqrt(3x-1)+(1+x)^2 &lt; y\\$\n</div>"),
            "{html}"
        );
    }
}

mod custom_blocks {
    use super::*;

    non_normative!(
        r#"
  context 'Custom Blocks' do
"#
    );

    #[test]
    fn should_not_warn_if_block_style_is_unknown() {
        verifies!(
            r#"
    test 'should not warn if block style is unknown' do
      input = <<~'EOS'
      [foo]
      --
      bar
      --
      EOS
      convert_string_to_embedded input
      assert_empty @logger.messages
    end

"#
        );

        // An unknown block style falls back to a plain open block. The only
        // diagnostic it raises is the low-severity `UnknownBlockStyle` recorded
        // by the debug-message test below — which Asciidoctor logs at DEBUG,
        // below the default WARN threshold that `assert_empty @logger.messages`
        // checks. So the WARN-level counterpart is "no `Warning`-severity
        // diagnostics" (`WarningType::severity()` is public as of
        // asciidoc-parser 0.29.6).
        let input = "[foo]\n--\nbar\n--\n";
        let warn_level = load(input)
            .warnings()
            .filter(|w| w.warning.severity() == WarningSeverity::Warning)
            .count();

        assert_eq!(warn_level, 0);
    }

    #[test]
    fn should_log_debug_message_if_block_style_is_unknown_and_debug_level_is_enabled() {
        verifies!(
            r#"
    test 'should log debug message if block style is unknown and debug level is enabled' do
      input = <<~'EOS'
      [foo]
      --
      bar
      --
      EOS
      using_memory_logger Logger::Severity::DEBUG do |logger|
        convert_string_to_embedded input
        assert_message logger, :DEBUG, '<stdin>: line 2: unknown style for open block: foo', Hash
      end
    end
  end

"#
        );

        // Asciidoctor logs this at DEBUG; `asciidoc-parser` records it as a
        // (Debug-severity) `UnknownBlockStyle` diagnostic carrying the block
        // context ("open") and the unknown style ("foo"), anchored to the block
        // delimiter line (line 2) as Asciidoctor does (0.29.6,
        // asciidoc-rs/asciidoc-parser#1043).
        assert_warning("[foo]\n--\nbar\n--\n", 2, |w| {
            matches!(
                w,
                WarningType::UnknownBlockStyle(context, style)
                    if context == "open" && style == "foo"
            )
        });
    }
}

mod metadata {
    use super::*;

    non_normative!(
        r#"
  context 'Metadata' do
"#
    );

    #[test]
    fn block_title_above_section_gets_carried_over_to_first_block_in_section() {
        verifies!(
            r#"
    test 'block title above section gets carried over to first block in section' do
      input = <<~'EOS'
      .Title
      == Section

      paragraph
      EOS
      output = convert_string input
      assert_xpath '//*[@class="paragraph"]', output, 1
      assert_xpath '//*[@class="paragraph"]/*[@class="title"][text()="Title"]', output, 1
      assert_xpath '//*[@class="paragraph"]/p[text()="paragraph"]', output, 1
    end

"#
        );

        let html = convert(".Title\n== Section\n\nparagraph\n");
        assert_xpath(&html, r#"//*[@class="paragraph"]"#, 1);
        assert_xpath(
            &html,
            r#"//*[@class="paragraph"]/*[@class="title"][text()="Title"]"#,
            1,
        );
        assert_xpath(&html, r#"//*[@class="paragraph"]/p[text()="paragraph"]"#, 1);
    }

    #[test]
    fn block_title_above_document_title_demotes_document_title_to_a_section_title() {
        verifies!(
            r#"
    test 'block title above document title demotes document title to a section title' do
      input = <<~'EOS'
      .Block title
      = Section Title

      section paragraph
      EOS
      output = convert_string input
      assert_xpath '//*[@id="header"]/*', output, 0
      assert_xpath '//*[@id="preamble"]/*', output, 0
      assert_xpath '//*[@id="content"]/h1[text()="Section Title"]', output, 1
      assert_xpath '//*[@class="paragraph"]', output, 1
      assert_xpath '//*[@class="paragraph"]/*[@class="title"][text()="Block title"]', output, 1
      assert_message @logger, :ERROR, '<stdin>: line 2: level 0 sections can only be used when doctype is book', Hash
    end

"#
        );

        // As of `asciidoc-parser` 0.29.6, a block title above the document title
        // demotes it to a level-0 section: no document header, the title becomes
        // an `<h1 class="sect0">` in the content, and the block title carries
        // onto the following paragraph. The `:ERROR:` log maps to the parser's
        // `Level0SectionHeadingNotSupported` warning at line 2.
        let input = ".Block title\n= Section Title\n\nsection paragraph\n";
        let html = convert_with(input, &Options::new().standalone(true));

        assert_xpath(&html, r#"//*[@id="header"]/*"#, 0);
        assert_xpath(&html, r#"//*[@id="preamble"]/*"#, 0);
        assert_xpath(&html, r#"//*[@id="content"]/h1[text()="Section Title"]"#, 1);
        assert_xpath(&html, r#"//*[@class="paragraph"]"#, 1);
        assert_xpath(
            &html,
            r#"//*[@class="paragraph"]/*[@class="title"][text()="Block title"]"#,
            1,
        );

        assert_warning(input, 2, |w| {
            matches!(w, WarningType::Level0SectionHeadingNotSupported)
        });
    }

    #[test]
    fn block_title_above_document_title_carries_over_to_first_section_first_block() {
        verifies!(
            r#"
    test 'block title above document title gets carried over to first block in first section if no preamble' do
      input = <<~'EOS'
      :doctype: book
      .Block title
      = Document Title

      == First Section

      paragraph
      EOS
      doc = document_from_string input
      # NOTE block title demotes document title to level-0 section
      refute doc.header?
      output = doc.convert
      assert_xpath '//*[@class="sect1"]//*[@class="paragraph"]/*[@class="title"][text()="Block title"]', output, 1
    end

"#
        );

        // The block title demotes the document title to a level-0 section, so
        // there is no document title (`refute doc.header?`), and the block title
        // carries onto the first block of the first section.
        let input =
            ":doctype: book\n.Block title\n= Document Title\n\n== First Section\n\nparagraph\n";
        assert!(load(input).doctitle().is_none());

        let html = convert_with(input, &Options::new().standalone(true));
        assert_xpath(
            &html,
            r#"//*[@class="sect1"]//*[@class="paragraph"]/*[@class="title"][text()="Block title"]"#,
            1,
        );
    }

    #[test]
    fn should_apply_substitutions_to_a_block_title_in_normal_order() {
        verifies!(
            r##"
    test 'should apply substitutions to a block title in normal order' do
      input = <<~'EOS'
      .{link-url}[{link-text}]{tm}
      The one and only!
      EOS

      output = convert_string_to_embedded input, attributes: {
        'link-url' => 'https://acme.com',
        'link-text' => 'ACME',
        'tm' => '(TM)',
      }
      assert_css '.title', output, 1
      assert_css '.title a[href="https://acme.com"]', output, 1
      assert_xpath %(//*[@class="title"][contains(text(),"#{decode_char 8482}")]), output, 1
    end

"##
        );

        let html = convert_with(
            ".{link-url}[{link-text}]{tm}\nThe one and only!\n",
            &Options::new()
                .attribute("link-url", "https://acme.com")
                .attribute("link-text", "ACME")
                .attribute("tm", "(TM)"),
        );
        assert_css(&html, ".title", 1);
        assert_css(&html, ".title a[href=\"https://acme.com\"]", 1);

        // `decode_char 8482` is the trademark sign (™); the `(TM)` replacement
        // substitution produces it as a trailing text node of the title.
        assert_xpath(
            &html,
            "//*[@class=\"title\"][contains(text(), \"\u{2122}\")]",
            1,
        );
    }

    #[test]
    fn empty_attribute_list_should_not_appear_in_output() {
        verifies!(
            r#"
    test 'empty attribute list should not appear in output' do
      input = <<~'EOS'
      []
      --
      Block content
      --
      EOS

      output = convert_string_to_embedded input
      assert_includes output, 'Block content'
      refute_includes output, '[]'
    end

"#
        );

        let html = convert("[]\n--\nBlock content\n--\n");
        assert!(html.contains("Block content"), "{html}");
        assert!(!html.contains("[]"), "{html}");
    }

    #[test]
    fn empty_block_anchor_should_not_appear_in_output() {
        verifies!(
            r#"
    test 'empty block anchor should not appear in output' do
      input = <<~'EOS'
      [[]]
      --
      Block content
      --
      EOS

      output = convert_string_to_embedded input
      assert_includes output, 'Block content'
      refute_includes output, '[[]]'
    end
  end

"#
        );

        // `asciidoc-parser` 0.29.3 ignores an empty block anchor
        // (asciidoc-rs/asciidoc-parser#1023), so `[[]]` no longer leaks into the
        // rendered open block.
        let html = convert("[[]]\n--\nBlock content\n--\n");
        assert!(html.contains("Block content"), "{html}");
        assert!(!html.contains("[[]]"), "{html}");
    }
}

mod images {
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    non_normative!(
        r#"
  context 'Images' do
"#
    );

    /// The `data:` URI Asciidoctor embeds for the 1x1 `fixtures/dot.gif` under
    /// `data-uri` (the exact base64 of [`DOT_GIF`]).
    const DOT_GIF_DATA_URI: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=";

    /// The 35-byte 1x1 GIF at `ref/asciidoctor/test/fixtures/dot.gif`.
    const DOT_GIF: &[u8] = &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x05, 0x04,
        0x04, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02,
        0x02, 0x44, 0x01, 0x00, 0x3b,
    ];

    /// The real `ref/asciidoctor/test/fixtures/circle.svg`: an XML preamble,
    /// DOCTYPE, and comment, then an `<svg>` carrying `width`/`height`/`style`
    /// (all of which inline embedding strips when an explicit width is given).
    const CIRCLE_SVG: &str = r#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- An SVG of a black circle -->
<svg
viewBox="0 0 120 120" version="1.1"
xmlns="http://www.w3.org/2000/svg" style="width:500px;height:500px"
width="500px" height="500px">
  <circle cx="60" cy="60" r="50"/>
</svg>
"#;

    /// [`CIRCLE_SVG`] embedded inline at width 100: XML
    /// preamble/DOCTYPE/comment dropped, the file's own
    /// `width`/`height`/`style` removed, and the explicit `width="100"`
    /// appended to the opening tag.
    const CIRCLE_SVG_INLINE_100: &str = concat!(
        "<svg\n",
        "viewBox=\"0 0 120 120\" version=\"1.1\"\n",
        "xmlns=\"http://www.w3.org/2000/svg\" width=\"100\">\n",
        "  <circle cx=\"60\" cy=\"60\" r=\"50\"/>\n",
        "</svg>",
    );

    /// [`CIRCLE_SVG`] embedded inline at width `50%`.
    const CIRCLE_SVG_INLINE_50PCT: &str = concat!(
        "<svg\n",
        "viewBox=\"0 0 120 120\" version=\"1.1\"\n",
        "xmlns=\"http://www.w3.org/2000/svg\" width=\"50%\">\n",
        "  <circle cx=\"60\" cy=\"60\" r=\"50\"/>\n",
        "</svg>",
    );

    /// A throwaway on-disk project directory holding the image fixtures the
    /// `data-uri` and inline-SVG tests read (a file-backed stand-in for
    /// Asciidoctor's `test/fixtures`, reached in Ruby via `docdir: testdir`).
    /// The directory is removed on drop. Point `base_dir` (this crate's
    /// `docdir`) at [`Fixtures::dir`] and resolve targets under
    /// `imagesdir=fixtures`.
    struct Fixtures {
        dir: PathBuf,
    }

    impl Fixtures {
        #[track_caller]
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("adoc-images-{}-{unique}", std::process::id()));

            let fixtures = dir.join("fixtures");
            std::fs::create_dir_all(&fixtures).expect("create fixtures dir");
            std::fs::write(fixtures.join("dot.gif"), DOT_GIF).expect("write dot.gif");
            std::fs::write(fixtures.join("circle.svg"), CIRCLE_SVG).expect("write circle.svg");
            std::fs::write(fixtures.join("empty.svg"), "").expect("write empty.svg");
            std::fs::write(fixtures.join("incomplete.svg"), "<svg\n")
                .expect("write incomplete.svg");

            // A target with no extension embeds as `application/octet-stream`;
            // its bytes are irrelevant to that test, so reuse the GIF's.
            std::fs::write(fixtures.join("dot"), DOT_GIF).expect("write dot");

            Self { dir }
        }

        fn dir(&self) -> &Path {
            &self.dir
        }
    }

    impl Drop for Fixtures {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn t01_block_image_alt_in_macro() {
        verifies!(
            r#"
    test 'can convert block image with alt text defined in macro' do
      input = 'image::images/tiger.png[Tiger]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert("image::images/tiger.png[Tiger]");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t02_svg_uses_img_by_default() {
        verifies!(
            r#"
    test 'converts SVG image using img element by default' do
      input = 'image::tiger.svg[Tiger]'
      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '/*[@class="imageblock"]//img[@src="tiger.svg"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert_with(
            "image::tiger.svg[Tiger]",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="tiger.svg"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t03_interactive_svg_uses_object() {
        verifies!(
            r#"
    test 'converts interactive SVG image with alt text using object element' do
      input = <<~'EOS'
      :imagesdir: images

      [%interactive]
      image::tiger.svg[Tiger,100]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="images/tiger.svg"][@width="100"]/span[@class="alt"][text()="Tiger"]', output, 1
    end

"#
        );

        let html = convert_with(
            ":imagesdir: images\n\n[%interactive]\nimage::tiger.svg[Tiger,100]\n",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="images/tiger.svg"][@width="100"]/span[@class="alt"][text()="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t04_svg_uses_img_when_secure() {
        verifies!(
            r#"
    test 'converts SVG image with alt text using img element when safe mode is secure' do
      input = <<~'EOS'
      [%interactive]
      image::images/tiger.svg[Tiger,100]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.svg"][@alt="Tiger"]', output, 1
    end

"#
        );

        // The API default safe mode is `Secure`, which disables the interactive
        // `<object>` referencing, so the SVG falls through to a plain `<img>`.
        let html = convert("[%interactive]\nimage::images/tiger.svg[Tiger,100]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.svg"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t05_interactive_svg_fallback_image() {
        verifies!(
            r#"
    test 'inserts fallback image for SVG inside object element using same dimensions' do
      input = <<~'EOS'
      :imagesdir: images

      [%interactive]
      image::tiger.svg[Tiger,100,fallback=tiger.png]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="images/tiger.svg"][@width="100"]/img[@src="images/tiger.png"][@width="100"]', output, 1
    end

"#
        );

        let html = convert_with(
            ":imagesdir: images\n\n[%interactive]\nimage::tiger.svg[Tiger,100,fallback=tiger.png]\n",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="images/tiger.svg"][@width="100"]/img[@src="images/tiger.png"][@width="100"]"#,
            1,
        );
    }

    #[test]
    fn t06_svg_uri_with_query_string() {
        verifies!(
            r#"
    test 'detects SVG image URI that contains a query string' do
      input = <<~'EOS'
      :imagesdir: images

      [%interactive]
      image::http://example.org/tiger.svg?foo=bar[Tiger,100]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="http://example.org/tiger.svg?foo=bar"][@width="100"]/span[@class="alt"][text()="Tiger"]', output, 1
    end

"#
        );

        let html = convert_with(
            ":imagesdir: images\n\n[%interactive]\nimage::http://example.org/tiger.svg?foo=bar[Tiger,100]\n",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="http://example.org/tiger.svg?foo=bar"][@width="100"]/span[@class="alt"][text()="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t07_svg_when_format_is_svg() {
        verifies!(
            r#"
    test 'detects SVG image when format attribute is svg' do
      input = <<~'EOS'
      :imagesdir: images

      [%interactive]
      image::http://example.org/tiger-svg[Tiger,100,format=svg]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="http://example.org/tiger-svg"][@width="100"]/span[@class="alt"][text()="Tiger"]', output, 1
    end

"#
        );

        let html = convert_with(
            ":imagesdir: images\n\n[%interactive]\nimage::http://example.org/tiger-svg[Tiger,100,format=svg]\n",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//object[@type="image/svg+xml"][@data="http://example.org/tiger-svg"][@width="100"]/span[@class="alt"][text()="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t08_inline_svg_when_inline_option() {
        verifies!(
            r#"
    test 'converts to inline SVG image when inline option is set on block' do
      input = <<~'EOS'
      :imagesdir: fixtures

      [%inline]
      image::circle.svg[Tiger,100]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => testdir }
      assert_match(/<svg\s[^>]*width="100"[^>]*>/, output, 1)
      refute_match(/<svg\s[^>]*width="500"[^>]*>/, output)
      refute_match(/<svg\s[^>]*height="500"[^>]*>/, output)
      refute_match(/<svg\s[^>]*style="[^>]*>/, output)
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":imagesdir: fixtures\n\n[%inline]\nimage::circle.svg[Tiger,100]\n",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .base_dir(fx.dir()),
        );
        // The `<svg>` file contents are embedded with the explicit `100` width applied
        // and the file's own `width`/`height`/`style` stripped.
        assert!(html.contains(CIRCLE_SVG_INLINE_100), "{html}");
        assert!(!html.contains(r#"width="500""#), "{html}");
        assert!(!html.contains(r#"height="500""#), "{html}");
        assert!(!html.contains("style="), "{html}");
    }

    #[test]
    fn t09_inline_svg_percentage_width() {
        verifies!(
            r#"
    test 'should honor percentage width for SVG image with inline option' do
      input = <<~'EOS'
      :imagesdir: fixtures

      image::circle.svg[Circle,50%,opts=inline]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => testdir }
      assert_match(/<svg\s[^>]*width="50%"[^>]*>/, output, 1)
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":imagesdir: fixtures\n\nimage::circle.svg[Circle,50%,opts=inline]\n",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .base_dir(fx.dir()),
        );
        assert!(html.contains(CIRCLE_SVG_INLINE_50PCT), "{html}");
    }

    // Ruby-specific: the test builds the block, then `set_attr 'width', 50`
    // stores an *Integer* width to prove the inline-SVG path does not crash on a
    // non-string width. This crate has no post-parse `set_attr` API, and its
    // attribute values are always strings, so the integer-coercion hazard cannot
    // arise; the string-width inline path is covered by the tests above.
    non_normative!(
        r#"
    test 'should not crash if explicit width on SVG image block is an integer' do
      input = <<~'EOS'
      :imagesdir: fixtures

      image::circle.svg[Circle,opts=inline]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => testdir }
      doc.blocks[0].set_attr 'width', 50
      output = doc.convert
      assert_match %r/<svg\s[^>]*width="50"[^>]*>/, output, 1
    end

"#
    );

    #[test]
    fn t11_inline_svg_with_data_uri() {
        verifies!(
            r#"
    test 'converts to inline SVG image when inline option is set on block and data-uri is set on document' do
      input = <<~'EOS'
      :imagesdir: fixtures
      :data-uri:

      [%inline]
      image::circle.svg[Tiger,100]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => testdir }
      assert_match(/<svg\s[^>]*width="100">/, output, 1)
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":imagesdir: fixtures\n:data-uri:\n\n[%inline]\nimage::circle.svg[Tiger,100]\n",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .base_dir(fx.dir()),
        );
        // `data-uri` does not disturb inline SVG: the file contents are embedded
        // directly (there is no `src` to turn into a `data:` URI).
        assert!(html.contains(CIRCLE_SVG_INLINE_100), "{html}");
    }

    #[test]
    fn t12_inline_svg_empty() {
        verifies!(
            r#"
    test 'should not throw exception if SVG to inline is empty' do
      input = 'image::empty.svg[nada,opts=inline]'
      output = convert_string_to_embedded input, safe: :safe, attributes: { 'docdir' => testdir, 'imagesdir' => 'fixtures' }
      assert_xpath '//svg', output, 0
      assert_xpath '//span[@class="alt"][text()="nada"]', output, 1
"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            "image::empty.svg[nada,opts=inline]",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(fx.dir())
                .attribute("imagesdir", "fixtures"),
        );
        assert_xpath(&html, "//svg", 0);
        assert_xpath(&html, r#"//span[@class="alt"][text()="nada"]"#, 1);

        // The `assert_message` WARN ("contents of SVG is empty") is a render-time
        // diagnostic that this crate performs silently while falling back to the alt
        // text; the observable fallback rendering above is what is verified.
        non_normative!(
            r#"
      assert_message @logger, :WARN, '~contents of SVG is empty:'
    end

"#
        );
    }

    #[test]
    fn t13_inline_svg_incomplete_start_tag() {
        verifies!(
            r#"
    test 'should not throw exception if SVG to inline contains an incomplete start tag and explicit width is specified' do
      input = 'image::incomplete.svg[,200,opts=inline]'
      output = convert_string_to_embedded input, safe: :safe, attributes: { 'docdir' => testdir, 'imagesdir' => 'fixtures' }
      assert_xpath '//svg', output, 1
      assert_xpath '//span[@class="alt"]', output, 0
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            "image::incomplete.svg[,200,opts=inline]",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(fx.dir())
                .attribute("imagesdir", "fixtures"),
        );
        assert_xpath(&html, "//svg", 1);
        assert_xpath(&html, r#"//span[@class="alt"]"#, 0);
    }

    // Remote fetch is a non-goal for this crate — it performs no network reads
    // (see the crate README), so embedding a remote SVG via `allow-uri-read`, and
    // the `cache-uri` caching of one, are out of scope.
    non_normative!(
        r#"
    test 'embeds remote SVG to inline when inline option is set on block and allow-uri-read is set on document' do
      input = %(image::http://#{resolve_localhost}:9876/fixtures/circle.svg[Circle,100,100,opts=inline])
      output = using_test_webserver do
        convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
      end

      assert_css 'svg', output, 1
      assert_css 'svg[style]', output, 0
      assert_css 'svg[width="100"]', output, 1
      assert_css 'svg[height="100"]', output, 1
      assert_css 'svg circle', output, 1
    end

    test 'should cache remote SVG when allow-uri-read, cache-uri, and inline option are set' do
      begin
        if OpenURI.respond_to? :cache_open_uri
          OpenURI.singleton_class.send :remove_method, :open_uri
          OpenURI.singleton_class.send :alias_method, :open_uri, :cache_open_uri
        end
        using_test_webserver do |base_url, thr|
          image_url = %(#{base_url}/fixtures/circle.svg)
          attributes = { 'allow-uri-read' => '', 'cache-uri' => '' }
          input = %(image::#{image_url}[Circle,100,100,opts=inline])
          output = convert_string_to_embedded input, safe: :safe, attributes: attributes
          assert defined? OpenURI::Cache
          assert_css 'svg circle', output, 1
          # NOTE we can't assert here since this is using the system-wide cache
          #assert_equal thr[:requests].size, 1
          #assert_equal thr[:requests][0], image_url
          thr[:requests].clear
          Dir.mktmpdir do |cache_path|
            original_cache_path = OpenURI::Cache.cache_path
            begin
              OpenURI::Cache.cache_path = cache_path
              assert_nil OpenURI::Cache.get image_url
              2.times do
                output = convert_string_to_embedded input, safe: :safe, attributes: attributes
                refute_nil OpenURI::Cache.get image_url
                assert_css 'svg circle', output, 1
              end
              assert_equal 1, thr[:requests].size
              assert_match %r/ \/fixtures\/circle\.svg /, thr[:requests][0], 1
            ensure
              OpenURI::Cache.cache_path = original_cache_path
            end
          end
        end
      ensure
        OpenURI.singleton_class.send :alias_method, :cache_open_uri, :open_uri
        OpenURI.singleton_class.send :remove_method, :open_uri
        OpenURI.singleton_class.send :alias_method, :open_uri, :original_open_uri
      end
    end

"#
    );

    #[test]
    fn t16_inline_svg_unreadable_falls_back_to_alt() {
        verifies!(
            r#"
    test 'converts to alt text for SVG with inline option set if SVG cannot be read' do
      input = <<~'EOS'
      [%inline]
      image::no-such-image.svg[Alt Text]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '//span[@class="alt"][text()="Alt Text"]', output, 1
"#
        );

        let html = convert_with(
            "[%inline]\nimage::no-such-image.svg[Alt Text]\n",
            &Options::new().safe_mode(SafeMode::Server),
        );
        assert_xpath(&html, r#"//span[@class="alt"][text()="Alt Text"]"#, 1);

        // The `assert_message` WARN ("SVG does not exist or cannot be read") is a
        // render-time diagnostic this crate performs silently while falling back to
        // the alt text; the observable fallback rendering above is what is verified.
        non_normative!(
            r#"
      assert_message @logger, :WARN, '~SVG does not exist or cannot be read'
    end

"#
        );
    }

    #[test]
    fn t17_block_image_alt_with_square_bracket() {
        verifies!(
            r#"
    test 'can convert block image with alt text defined in macro containing square bracket' do
      input = 'image::images/tiger.png[A [Bengal] Tiger]'
      output = convert_string input
      img = xmlnodes_at_xpath '//img', output, 1
      assert_equal 'A [Bengal] Tiger', img.attr('alt')
    end

"#
        );

        // Ruby uses `xmlnodes_at_xpath` to pull the single `<img>` and read its
        // `alt`; the equivalent assertion is a predicate on the decoded attribute.
        let html = convert("image::images/tiger.png[A [Bengal] Tiger]");
        assert_xpath(&html, r#"//img[@alt="A [Bengal] Tiger"]"#, 1);
    }

    #[test]
    fn t18_block_image_target_with_spaces() {
        verifies!(
            r#"
    test 'can convert block image with target containing spaces' do
      input = 'image::images/big tiger.png[A Big Tiger]'
      output = convert_string input
      img = xmlnodes_at_xpath '//img', output, 1
      assert_equal 'images/big%20tiger.png', img.attr('src')
      assert_equal 'A Big Tiger', img.attr('alt')
    end

"#
        );

        let html = convert("image::images/big tiger.png[A Big Tiger]");
        assert_xpath(
            &html,
            r#"//img[@src="images/big%20tiger.png"][@alt="A Big Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t19_block_image_target_leading_trailing_spaces() {
        verifies!(
            r#"
    test 'should not recognize block image if target has leading or trailing spaces' do
      [' tiger.png', 'tiger.png '].each do |target|
        input = %(image::#{target}[Tiger])

        output = convert_string_to_embedded input
        assert_xpath '//img', output, 0
      end
    end

"#
        );

        // A leading- or trailing-space target no longer satisfies the block-image
        // macro (asciidoc-parser 0.29.11): the first line becomes a description
        // list, the second a paragraph — matching Asciidoctor 2.0.26.
        for target in [" tiger.png", "tiger.png "] {
            let html = convert(&format!("image::{target}[Tiger]"));
            assert_xpath(&html, "//img", 0);
        }
    }

    #[test]
    fn t20_block_image_alt_above_macro() {
        verifies!(
            r#"
    test 'can convert block image with alt text defined in block attribute above macro' do
      input = <<~'EOS'
      [Tiger]
      image::images/tiger.png[]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert("[Tiger]\nimage::images/tiger.png[]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t21_alt_in_macro_overrides_above() {
        verifies!(
            r#"
    test 'alt text in macro overrides alt text above macro' do
      input = <<~'EOS'
      [Alt Text]
      image::images/tiger.png[Tiger]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert("[Alt Text]\nimage::images/tiger.png[Tiger]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t22_attribute_references_in_alt() {
        verifies!(
            r#"
    test 'should substitute attribute references in alt text defined in image block macro' do
      input = <<~'EOS'
      :alt-text: Tiger

      image::images/tiger.png[{alt-text}]
      EOS
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert(":alt-text: Tiger\n\nimage::images/tiger.png[{alt-text}]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t23_float_direction_class() {
        verifies!(
            r#"
    test 'should set direction CSS class on image if float attribute is set' do
      input = <<~'EOS'
      [float=left]
      image::images/tiger.png[Tiger]
      EOS

      output = convert_string_to_embedded input
      assert_css '.imageblock.left', output, 1
      assert_css '.imageblock[style]', output, 0
    end

"#
        );

        let html = convert("[float=left]\nimage::images/tiger.png[Tiger]\n");
        assert_css(&html, ".imageblock.left", 1);
        assert_css(&html, ".imageblock[style]", 0);
    }

    #[test]
    fn t24_align_text_class() {
        verifies!(
            r#"
    test 'should set text alignment CSS class on image if align attribute is set' do
      input = <<~'EOS'
      [align=center]
      image::images/tiger.png[Tiger]
      EOS

      output = convert_string_to_embedded input
      assert_css '.imageblock.text-center', output, 1
      assert_css '.imageblock[style]', output, 0
    end

"#
        );

        let html = convert("[align=center]\nimage::images/tiger.png[Tiger]\n");
        assert_css(&html, ".imageblock.text-center", 1);
        assert_css(&html, ".imageblock[style]", 0);
    }

    #[test]
    fn t25_style_attribute_dropped() {
        verifies!(
            r#"
    test 'style attribute is dropped from image macro' do
      input = <<~'EOS'
      [style=value]
      image::images/tiger.png[Tiger]
      EOS

      doc = document_from_string input
      img = doc.blocks[0]
      refute(img.attributes.key? 'style')
      assert_nil img.style
    end

"#
        );

        // Ruby inspects the parsed block: a named `style=value` on an image is
        // dropped (`img.style` nil, no `style` attribute). Observably, the image
        // renders as a plain `imageblock` — no `value` class, no `style` attribute.
        let html = convert("[style=value]\nimage::images/tiger.png[Tiger]\n");
        assert_css(&html, ".imageblock", 1);
        assert_css(&html, ".imageblock.value", 0);
        assert_css(&html, ".imageblock[style]", 0);
    }

    #[test]
    fn t26_specialchars_and_replacements_in_alt() {
        verifies!(
            r##"
    test 'should apply specialcharacters and replacement substitutions to alt text' do
      input = 'A tiger\'s "roar" is < a bear\'s "growl"'
      expected = 'A tiger&#8217;s &quot;roar&quot; is &lt; a bear&#8217;s &quot;growl&quot;'
      result = convert_string_to_embedded %(image::images/tiger-roar.png[#{input}])
      assert_includes result, %(alt="#{expected}")
    end

"##
        );

        let result =
            convert(r#"image::images/tiger-roar.png[A tiger's "roar" is < a bear's "growl"]"#);
        assert!(
            result.contains(
                r#"alt="A tiger&#8217;s &quot;roar&quot; is &lt; a bear&#8217;s &quot;growl&quot;""#
            ),
            "{result}"
        );
    }

    // The DocBook backend is out of scope: this crate targets only the `html5`
    // backend.
    non_normative!(
        r#"
    test 'should not encode double quotes in alt text when converting to DocBook' do
      input = 'Select "File > Open"'
      expected = 'Select "File &gt; Open"'
      result = convert_string_to_embedded %(image::images/open.png[#{input}]), backend: :docbook
      assert_includes result, %(<phrase>#{expected}</phrase>)
    end

"#
    );

    #[test]
    fn t28_auto_generated_alt() {
        verifies!(
            r#"
    test 'should auto-generate alt text for block image if alt text is not specified' do
      input = 'image::images/lions-and-tigers.png[]'
      image = block_from_string input
      assert_equal 'lions and tigers', (image.attr 'alt')
      assert_equal 'lions and tigers', (image.attr 'default-alt')
      output = image.convert
      assert_xpath '/*[@class="imageblock"]//img[@src="images/lions-and-tigers.png"][@alt="lions and tigers"]', output, 1
    end

"#
        );

        // Ruby reads `default-alt` off the parsed block; observably, the auto-alt
        // (target basename with `_`/`-` turned to spaces) lands in the `<img>`.
        let html = convert("image::images/lions-and-tigers.png[]");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/lions-and-tigers.png"][@alt="lions and tigers"]"#,
            1,
        );
    }

    #[test]
    fn t29_block_image_alt_height_width() {
        verifies!(
            r#"
    test "can convert block image with alt text and height and width" do
      input = 'image::images/tiger.png[Tiger, 200, 300]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"][@width="200"][@height="300"]', output, 1
    end

"#
        );

        let html = convert("image::images/tiger.png[Tiger, 200, 300]");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"][@width="200"][@height="300"]"#,
            1,
        );
    }

    #[test]
    fn t30_no_empty_width_attribute() {
        verifies!(
            r#"
    test 'should not output empty width attribute if positional width attribute is empty' do
      input = 'image::images/tiger.png[Tiger,]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"]', output, 1
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@width]', output, 0
    end

"#
        );

        let html = convert("image::images/tiger.png[Tiger,]");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"]"#,
            1,
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@width]"#,
            0,
        );
    }

    #[test]
    fn t31_block_image_with_link() {
        verifies!(
            r#"
    test "can convert block image with link" do
      input = <<~'EOS'
      image::images/tiger.png[Tiger, link='http://en.wikipedia.org/wiki/Tiger']
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"]/img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html =
            convert("image::images/tiger.png[Tiger, link='http://en.wikipedia.org/wiki/Tiger']\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"]/img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t32_rel_noopener_for_blank_window() {
        verifies!(
            r#"
    test 'adds rel=noopener attribute to block image with link that targets _blank window' do
      input = 'image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,window=_blank]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@target="_blank"][@rel="noopener"]/img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert(
            "image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,window=_blank]",
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@target="_blank"][@rel="noopener"]/img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t33_rel_noopener_for_name_window_noopener_opt() {
        verifies!(
            r#"
    test 'adds rel=noopener attribute to block image with link that targets name window when the noopener option is set' do
      input = 'image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,window=name,opts=noopener]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@target="name"][@rel="noopener"]/img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert(
            "image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,window=name,opts=noopener]",
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@target="name"][@rel="noopener"]/img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t34_rel_nofollow_for_nofollow_opt() {
        verifies!(
            r#"
    test 'adds rel=nofollow attribute to block image with a link when the nofollow option is set' do
      input = 'image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,opts=nofollow]'
      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@rel="nofollow"]/img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert(
            "image::images/tiger.png[Tiger,link=http://en.wikipedia.org/wiki/Tiger,opts=nofollow]",
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//a[@class="image"][@href="http://en.wikipedia.org/wiki/Tiger"][@rel="nofollow"]/img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t35_block_image_with_caption() {
        verifies!(
            r#"
    test 'can convert block image with caption' do
      input = <<~'EOS'
      .The AsciiDoc Tiger
      image::images/tiger.png[Tiger]
      EOS

      doc = document_from_string input
      assert_equal 1, doc.blocks[0].numeral
      output = doc.convert
      assert_xpath '//*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
      assert_xpath '//*[@class="imageblock"]/*[@class="title"][text()="Figure 1. The AsciiDoc Tiger"]', output, 1
      assert_equal 1, doc.attributes['figure-number']
    end

"#
        );

        // Ruby also checks the parsed block's `numeral`/`figure-number`; observably
        // the implicit caption renders as "Figure 1. " before the title.
        let html = convert(".The AsciiDoc Tiger\nimage::images/tiger.png[Tiger]\n");
        assert_xpath(
            &html,
            r#"//*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
        assert_xpath(
            &html,
            r#"//*[@class="imageblock"]/*[@class="title"][text()="Figure 1. The AsciiDoc Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t36_block_image_with_explicit_caption() {
        verifies!(
            r#"
    test 'can convert block image with explicit caption' do
      input = <<~'EOS'
      [caption="Voila! "]
      .The AsciiDoc Tiger
      image::images/tiger.png[Tiger]
      EOS

      doc = document_from_string input
      assert_nil doc.blocks[0].numeral
      output = doc.convert
      assert_xpath '//*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
      assert_xpath '//*[@class="imageblock"]/*[@class="title"][text()="Voila! The AsciiDoc Tiger"]', output, 1
      refute doc.attributes.key?('figure-number')
    end

"#
        );

        // An explicit `caption=` overrides the implicit "Figure N." label (and, in
        // Ruby, leaves the block un-numbered); the caption prefixes the title.
        let html =
            convert("[caption=\"Voila! \"]\n.The AsciiDoc Tiger\nimage::images/tiger.png[Tiger]\n");
        assert_xpath(
            &html,
            r#"//*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
        assert_xpath(
            &html,
            r#"//*[@class="imageblock"]/*[@class="title"][text()="Voila! The AsciiDoc Tiger"]"#,
            1,
        );
    }

    // The DocBook backend is out of scope: this crate targets only the `html5`
    // backend, so its `imagedata` align/scale/width/depth attributes are not
    // emitted.
    non_normative!(
        r#"
    test 'can align image in DocBook backend' do
      input = 'image::images/sunset.jpg[Sunset,align=right]'
      output = convert_string_to_embedded input, backend: :docbook
      assert_xpath '//imagedata', output, 1
      assert_xpath '//imagedata[@align="right"]', output, 1
    end

    test 'should set content width and depth in DocBook backend if no scaling' do
      input = 'image::images/sunset.jpg[Sunset,500,332]'
      output = convert_string_to_embedded input, backend: :docbook
      assert_xpath '//imagedata', output, 1
      assert_xpath '//imagedata[@contentwidth="500"]', output, 1
      assert_xpath '//imagedata[@contentdepth="332"]', output, 1
      assert_xpath '//imagedata[@width]', output, 0
      assert_xpath '//imagedata[@depth]', output, 0
    end

    test 'can scale image in DocBook backend' do
      input = 'image::images/sunset.jpg[Sunset,500,332,scale=200]'
      output = convert_string_to_embedded input, backend: :docbook
      assert_xpath '//imagedata', output, 1
      assert_xpath '//imagedata[@scale="200"]', output, 1
      assert_xpath '//imagedata[@width]', output, 0
      assert_xpath '//imagedata[@depth]', output, 0
      assert_xpath '//imagedata[@contentwidth]', output, 0
      assert_xpath '//imagedata[@contentdepth]', output, 0
    end

    test 'scale image width in DocBook backend' do
      input = 'image::images/sunset.jpg[Sunset,500,332,scaledwidth=25%]'
      output = convert_string_to_embedded input, backend: :docbook
      assert_xpath '//imagedata', output, 1
      assert_xpath '//imagedata[@width="25%"]', output, 1
      assert_xpath '//imagedata[@depth]', output, 0
      assert_xpath '//imagedata[@contentwidth]', output, 0
      assert_xpath '//imagedata[@contentdepth]', output, 0
    end

    test 'adds % to scaled width if no units given in DocBook backend ' do
      input = 'image::images/sunset.jpg[Sunset,scaledwidth=25]'
      output = convert_string_to_embedded input, backend: :docbook
      assert_xpath '//imagedata', output, 1
      assert_xpath '//imagedata[@width="25%"]', output, 1
    end

"#
    );

    #[test]
    fn t42_attribute_missing_skip() {
        verifies!(
            r#"
    test 'keeps attribute reference unprocessed if image target is missing attribute reference and attribute-missing is skip' do
      input = <<~'EOS'
      :attribute-missing: skip

      image::{bogus}[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'img[src="{bogus}"]', output, 1
      assert_empty @logger
    end

"#
        );

        let input = ":attribute-missing: skip\n\nimage::{bogus}[]\n";
        let html = convert(input);
        assert_css(&html, r#"img[src="{bogus}"]"#, 1);
        // `assert_empty @logger`: no diagnostics are raised.
        assert_eq!(load(input).warnings().count(), 0);
    }

    #[test]
    fn t43_attribute_missing_drop() {
        verifies!(
            r#"
    test 'should not drop line if image target is missing attribute reference and attribute-missing is drop' do
      input = <<~'EOS'
      :attribute-missing: drop

      image::{bogus}/photo.jpg[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'img[src="/photo.jpg"]', output, 1
      assert_empty @logger
    end

"#
        );

        let input = ":attribute-missing: drop\n\nimage::{bogus}/photo.jpg[]\n";
        let html = convert(input);
        assert_css(&html, r#"img[src="/photo.jpg"]"#, 1);
        assert_eq!(load(input).warnings().count(), 0);
    }

    #[test]
    fn t44_attribute_missing_drop_line() {
        verifies!(
            r#"
    test 'drops line if image target is missing attribute reference and attribute-missing is drop-line' do
      input = <<~'EOS'
      :attribute-missing: drop-line

      image::{bogus}[]
      EOS

      output = convert_string_to_embedded input
      assert_empty output.strip
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: bogus'
    end

"#
        );

        let input = ":attribute-missing: drop-line\n\nimage::{bogus}[]\n";
        let html = convert(input);
        assert!(html.trim().is_empty(), "{html}");
        // Asciidoctor logs an INFO "dropping line ... missing attribute: bogus";
        // asciidoc-parser surfaces the same as a SkippingReferenceToMissingAttribute.
        assert!(load(input)
            .warnings()
            .any(|w| w.warning
                == WarningType::SkippingReferenceToMissingAttribute("bogus".to_string())));
    }

    #[test]
    fn t45_attribute_missing_drop_line_blank_resolves() {
        verifies!(
            r#"
    test 'should not drop line if image target resolves to blank and attribute-missing is drop-line' do
      input = <<~'EOS'
      :attribute-missing: drop-line

      image::{blank}[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'img[src=""]', output, 1
      assert_empty @logger
    end

"#
        );

        let input = ":attribute-missing: drop-line\n\nimage::{blank}[]\n";
        let html = convert(input);
        assert_css(&html, r#"img[src=""]"#, 1);
        assert_eq!(load(input).warnings().count(), 0);
    }

    #[test]
    fn t46_dropped_image_does_not_break_section() {
        verifies!(
            r#"
    test 'dropped image does not break processing of following section and attribute-missing is drop-line' do
      input = <<~'EOS'
      :attribute-missing: drop-line

      image::{bogus}[]

      == Section Title
      EOS

      output = convert_string_to_embedded input
      assert_css 'img', output, 0
      assert_css 'h2', output, 1
      refute_includes output, '== Section Title'
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: bogus'
    end

"#
        );

        let input = ":attribute-missing: drop-line\n\nimage::{bogus}[]\n\n== Section Title\n";
        let html = convert(input);
        assert_css(&html, "img", 0);
        assert_css(&html, "h2", 1);
        assert!(!html.contains("== Section Title"), "{html}");
        assert!(load(input)
            .warnings()
            .any(|w| w.warning
                == WarningType::SkippingReferenceToMissingAttribute("bogus".to_string())));
    }

    #[test]
    fn t47_pass_through_uri_target() {
        verifies!(
            r#"
    test 'should pass through image that references uri' do
      input = <<~'EOS'
      :imagesdir: images

      image::http://asciidoc.org/images/tiger.png[Tiger]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="http://asciidoc.org/images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html =
            convert(":imagesdir: images\n\nimage::http://asciidoc.org/images/tiger.png[Tiger]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="http://asciidoc.org/images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t48_encode_spaces_in_uri_target() {
        verifies!(
            r#"
    test 'should encode spaces in image target if value is a URI' do
      input = 'image::http://example.org/svg?digraph=digraph G { a -> b; }[diagram]'
      output = convert_string_to_embedded input
      assert_xpath %(/*[@class="imageblock"]//img[@src="http://example.org/svg?digraph=digraph%20G%20{%20a%20-#{decode_char 62}%20b;%20}"]), output, 1
    end

"#
        );

        // `decode_char 62` is `>`: the XPath compares against the DOM-decoded `src`,
        // where the emitted `&gt;` reads back as `>`. Spaces become `%20`; `{`, `}`,
        // and `>` stay literal.
        let html = convert("image::http://example.org/svg?digraph=digraph G { a -> b; }[diagram]");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="http://example.org/svg?digraph=digraph%20G%20{%20a%20->%20b;%20}"]"#,
            1,
        );
    }

    #[test]
    fn t49_resolve_relative_to_imagesdir() {
        verifies!(
            r#"
    test 'can resolve image relative to imagesdir' do
      input = <<~'EOS'
      :imagesdir: images

      image::tiger.png[Tiger]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]', output, 1
    end

"#
        );

        let html = convert(":imagesdir: images\n\nimage::tiger.png[Tiger]\n");
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="images/tiger.png"][@alt="Tiger"]"#,
            1,
        );
    }

    #[test]
    fn t50_embeds_data_uri_for_image() {
        verifies!(
            r#"
    test 'embeds base64-encoded data uri for image when data-uri attribute is set' do
      input = <<~'EOS'
      :data-uri:
      :imagesdir: fixtures

      image::dot.gif[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_equal 'fixtures', doc.attributes['imagesdir']
      output = doc.convert
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":data-uri:\n:imagesdir: fixtures\n\nimage::dot.gif[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe).base_dir(fx.dir()),
        );
        assert_xpath(
            &html,
            &format!(r#"//img[@src="{DOT_GIF_DATA_URI}"][@alt="Dot"]"#),
            1,
        );
    }

    // JRuby-only (`if: jruby?`): embedding an image read from a classloader URI
    // (`uri:classloader:/…` inside a `.jar`) has no analog in this crate.
    non_normative!(
        r#"
    test 'embeds base64-encoded data uri for image in classloader when data-uri attribute is set', if: jruby? do
      require fixture_path 'assets.jar'
      input = <<~'EOS'
      :data-uri:
      :imagesdir: uri:classloader:/images-in-jar

      image::dot.gif[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::UNSAFE, attributes: { 'docdir' => testdir }
      assert_equal 'uri:classloader:/images-in-jar', doc.attributes['imagesdir']
      output = doc.convert
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

"#
    );

    #[test]
    fn t52_embeds_svg_with_svg_mimetype() {
        verifies!(
            r#"
    test 'embeds SVG image with image/svg+xml mimetype when file extension is .svg' do
      input = <<~'EOS'
      :imagesdir: fixtures
      :data-uri:

      image::circle.svg[Tiger,100]
      EOS

      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => testdir }
      assert_xpath '//img[starts-with(@src,"data:image/svg+xml;base64,")]', output, 1
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":imagesdir: fixtures\n:data-uri:\n\nimage::circle.svg[Tiger,100]\n",
            &Options::new()
                .safe_mode(SafeMode::Server)
                .base_dir(fx.dir()),
        );
        // `starts-with(@src, …)`: an `.svg` embeds with the `image/svg+xml` mimetype.
        assert!(
            html.contains(r#"src="data:image/svg+xml;base64,"#),
            "{html}"
        );
    }

    #[test]
    fn t53_embeds_empty_data_uri_for_unreadable() {
        verifies!(
            r#"
    test 'embeds empty base64-encoded data uri for unreadable image when data-uri attribute is set' do
      input = <<~'EOS'
      :data-uri:
      :imagesdir: fixtures

      image::unreadable.gif[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_equal 'fixtures', doc.attributes['imagesdir']
      output = doc.convert
      assert_xpath '//img[@src="data:image/gif;base64,"]', output, 1
"#
        );

        let fx = Fixtures::new();
        // `unreadable.gif` is never created, so the read fails and the embed is empty.
        let html = convert_with(
            ":data-uri:\n:imagesdir: fixtures\n\nimage::unreadable.gif[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe).base_dir(fx.dir()),
        );
        assert_xpath(&html, r#"//img[@src="data:image/gif;base64,"]"#, 1);

        // The `assert_message` WARN ("image to embed not found or not readable") is a
        // render-time diagnostic this crate performs silently while emitting the empty
        // `data:` URI above.
        non_normative!(
            r#"
      assert_message @logger, :WARN, '~image to embed not found or not readable'
    end

"#
        );
    }

    #[test]
    fn t54_embeds_octet_stream_when_extension_missing() {
        verifies!(
            r#"
    test 'embeds base64-encoded data uri with application/octet-stream mimetype when file extension is missing' do
      input = <<~'EOS'
      :data-uri:
      :imagesdir: fixtures

      image::dot[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_equal 'fixtures', doc.attributes['imagesdir']
      output = doc.convert
      assert_xpath '//img[starts-with(@src,"data:application/octet-stream;base64,")]', output, 1
    end

"#
        );

        let fx = Fixtures::new();
        let html = convert_with(
            ":data-uri:\n:imagesdir: fixtures\n\nimage::dot[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe).base_dir(fx.dir()),
        );
        // `starts-with(@src, …)`: a target with no extension embeds as
        // `application/octet-stream`.
        assert!(
            html.contains(r#"src="data:application/octet-stream;base64,"#),
            "{html}"
        );
    }

    // Remote fetch is a non-goal for this crate — it performs no network reads —
    // so `data-uri` embedding of a remote image (directly, via a URI `imagesdir`,
    // with `cache-uri`, or falling back when the fetch fails) is out of scope.
    non_normative!(
        r##"
    test 'embeds base64-encoded data uri for remote image when data-uri attribute is set' do
      input = <<~EOS
      :data-uri:

      image::http://#{resolve_localhost}:9876/fixtures/dot.gif[Dot]
      EOS

      output = using_test_webserver do
        convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
      end

      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

    test 'embeds base64-encoded data uri for remote image when imagesdir is a URI and data-uri attribute is set' do
      input = <<~EOS
      :data-uri:
      :imagesdir: http://#{resolve_localhost}:9876/fixtures

      image::dot.gif[Dot]
      EOS

      output = using_test_webserver do
        convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
      end

      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

    test 'should cache remote image when allow-uri-read, cache-uri, and data-uri are set' do
      begin
        if OpenURI.respond_to? :cache_open_uri
          OpenURI.singleton_class.send :remove_method, :open_uri
          OpenURI.singleton_class.send :alias_method, :open_uri, :cache_open_uri
        end
        using_test_webserver do |base_url, thr|
          image_url = %(#{base_url}/fixtures/dot.gif)
          image_data_uri = 'data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs='
          attributes = { 'allow-uri-read' => '', 'cache-uri' => '', 'data-uri' => '' }
          input = %(image::#{image_url}[Dot])
          output = convert_string_to_embedded input, safe: :safe, attributes: attributes
          assert defined? OpenURI::Cache
          assert_xpath %(//img[@src="#{image_data_uri}"][@alt="Dot"]), output, 1
          # NOTE we can't assert here since this is using the system-wide cache
          #assert_equal thr[:requests].size, 1
          #assert_equal thr[:requests][0], image_url
          thr[:requests].clear
          Dir.mktmpdir do |cache_path|
            original_cache_path = OpenURI::Cache.cache_path
            begin
              OpenURI::Cache.cache_path = cache_path
              assert_nil OpenURI::Cache.get image_url
              2.times do
                output = convert_string_to_embedded input, safe: :safe, attributes: attributes
                refute_nil OpenURI::Cache.get image_url
                assert_xpath %(//img[@src="#{image_data_uri}"][@alt="Dot"]), output, 1
              end
              assert_equal 1, thr[:requests].size
              assert_match %r/ \/fixtures\/dot\.gif /, thr[:requests][0], 1
            ensure
              OpenURI::Cache.cache_path = original_cache_path
            end
          end
        end
      ensure
        OpenURI.singleton_class.send :alias_method, :cache_open_uri, :open_uri
        OpenURI.singleton_class.send :remove_method, :open_uri
        OpenURI.singleton_class.send :alias_method, :open_uri, :original_open_uri
      end
    end

    test 'uses remote image uri when data-uri attribute is set and image cannot be retrieved' do
      image_uri = "http://#{resolve_localhost}:9876/fixtures/missing-image.gif"
      input = <<~EOS
      :data-uri:

      image::#{image_uri}[Missing image]
      EOS

      output = using_test_webserver do
        convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
      end

      assert_xpath %(/*[@class="imageblock"]//img[@src="#{image_uri}"][@alt="Missing image"]), output, 1
      assert_message @logger, :WARN, '~could not retrieve image data from URI'
    end

"##
    );

    #[test]
    fn t59_remote_uri_when_allow_uri_read_unset() {
        verifies!(
            r##"
    test 'uses remote image uri when data-uri attribute is set and allow-uri-read is not set' do
      image_uri = "http://#{resolve_localhost}:9876/fixtures/dot.gif"
      input = <<~EOS
      :data-uri:

      image::#{image_uri}[Dot]
      EOS

      output = using_test_webserver do
        convert_string_to_embedded input, safe: :safe
      end

      assert_xpath %(/*[@class="imageblock"]//img[@src="#{image_uri}"][@alt="Dot"]), output, 1
    end

"##
        );

        // `data-uri` is set but the target is a remote URI and `allow-uri-read` is
        // not, so — this crate never fetches over the network (remote reads are a
        // non-goal) — the URI passes through as a plain linked image, matching
        // Asciidoctor's fallback.
        let html = convert_with(
            ":data-uri:\n\nimage::http://example.org/fixtures/dot.gif[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe),
        );
        assert_xpath(
            &html,
            r#"/*[@class="imageblock"]//img[@src="http://example.org/fixtures/dot.gif"][@alt="Dot"]"#,
            1,
        );
    }

    #[test]
    fn t60_embedded_data_uri_images() {
        verifies!(
            r#"
    test 'can handle embedded data uri images' do
      input = 'image::data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=[Dot]'
      output = convert_string_to_embedded input
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

"#
        );

        let html = convert(
            "image::data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=[Dot]",
        );
        assert_xpath(
            &html,
            r#"//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]"#,
            1,
        );
    }

    #[test]
    fn t61_embedded_data_uri_images_with_data_uri() {
        verifies!(
            r#"
    test 'can handle embedded data uri images when data-uri attribute is set' do
      input = <<~'EOS'
      :data-uri:

      image::data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=[Dot]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
    end

"#
        );

        // An already-embedded `data:` URI target passes through unchanged even when
        // `data-uri` is set.
        let html = convert(
            ":data-uri:\n\nimage::data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=[Dot]\n",
        );
        assert_xpath(
            &html,
            r#"//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]"#,
            1,
        );
    }

    #[test]
    fn t62_cleans_ancestor_refs_in_imagesdir() {
        verifies!(
            r##"
    test 'cleans reference to ancestor directories in imagesdir before reading image if safe mode level is at least SAFE' do
      input = <<~'EOS'
      :data-uri:
      :imagesdir: ../..//fixtures/./../../fixtures

      image::dot.gif[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_equal '../..//fixtures/./../../fixtures', doc.attributes['imagesdir']
      output = doc.convert
      # image target resolves to fixtures/dot.gif relative to docdir (which is explicitly set to the directory of this file)
      # the reference cannot fall outside of the document directory in safe mode
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
"##
        );

        let fx = Fixtures::new();
        // `imagesdir` climbs above the jail with `../..`; under `Safe` the read is
        // recovered back inside, resolving to `fixtures/dot.gif`.
        let html = convert_with(
            ":data-uri:\n:imagesdir: ../..//fixtures/./../../fixtures\n\nimage::dot.gif[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe).base_dir(fx.dir()),
        );
        assert_xpath(
            &html,
            &format!(r#"//img[@src="{DOT_GIF_DATA_URI}"][@alt="Dot"]"#),
            1,
        );

        // The `assert_message` WARN ("image has illegal reference to ancestor of
        // jail; recovering automatically") is a render-time diagnostic this crate
        // performs silently while recovering the read back inside the jail.
        non_normative!(
            r##"
      assert_message @logger, :WARN, 'image has illegal reference to ancestor of jail; recovering automatically'
    end

"##
        );
    }

    #[test]
    fn t63_cleans_ancestor_refs_in_target() {
        verifies!(
            r##"
    test 'cleans reference to ancestor directories in target before reading image if safe mode level is at least SAFE' do
      input = <<~'EOS'
      :data-uri:
      :imagesdir: ./

      image::../..//fixtures/./../../fixtures/dot.gif[Dot]
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_equal './', doc.attributes['imagesdir']
      output = doc.convert
      # image target resolves to fixtures/dot.gif relative to docdir (which is explicitly set to the directory of this file)
      # the reference cannot fall outside of the document directory in safe mode
      assert_xpath '//img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Dot"]', output, 1
"##
        );

        let fx = Fixtures::new();
        // The `target` (not `imagesdir`) climbs above the jail; the read is recovered
        // back inside, again resolving to `fixtures/dot.gif`.
        let html = convert_with(
            ":data-uri:\n:imagesdir: ./\n\nimage::../..//fixtures/./../../fixtures/dot.gif[Dot]\n",
            &Options::new().safe_mode(SafeMode::Safe).base_dir(fx.dir()),
        );
        assert_xpath(
            &html,
            &format!(r#"//img[@src="{DOT_GIF_DATA_URI}"][@alt="Dot"]"#),
            1,
        );

        // As above, the "illegal reference to ancestor of jail" WARN is a render-time
        // diagnostic this crate performs silently.
        non_normative!(
            r##"
      assert_message @logger, :WARN, 'image has illegal reference to ancestor of jail; recovering automatically'
    end
  end

"##
        );
    }
}

mod media {
    use super::*;

    non_normative!(
        r#"
  context 'Media' do
"#
    );

    #[test]
    fn should_detect_and_convert_video_macro() {
        verifies!(
            r#"
    test 'should detect and convert video macro' do
      input = 'video::cats-vs-dogs.avi[]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="cats-vs-dogs.avi"]', output, 1
    end

"#
        );

        let output = convert("video::cats-vs-dogs.avi[]");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="cats-vs-dogs.avi"]"#, 1);
    }

    #[test]
    fn should_detect_and_convert_video_macro_with_positional_attributes() {
        verifies!(
            r#"
    test 'should detect and convert video macro with positional attributes for poster and dimensions' do
      input = 'video::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="cats-vs-dogs.avi"]', output, 1
      assert_css 'video[poster="cats-and-dogs.png"]', output, 1
      assert_css 'video[width="200"]', output, 1
      assert_css 'video[height="300"]', output, 1
    end

"#
        );

        let output = convert("video::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="cats-vs-dogs.avi"]"#, 1);
        assert_css(&output, r#"video[poster="cats-and-dogs.png"]"#, 1);
        assert_css(&output, r#"video[width="200"]"#, 1);
        assert_css(&output, r#"video[height="300"]"#, 1);
    }

    #[test]
    fn should_set_direction_css_class_on_video_block_if_float_attribute_is_set() {
        verifies!(
            r#"
    test 'should set direction CSS class on video block if float attribute is set' do
      input = 'video::cats-vs-dogs.avi[cats-and-dogs.png,float=right]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="cats-vs-dogs.avi"]', output, 1
      assert_css '.videoblock.right', output, 1
    end

"#
        );

        let output = convert("video::cats-vs-dogs.avi[cats-and-dogs.png,float=right]");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="cats-vs-dogs.avi"]"#, 1);
        assert_css(&output, ".videoblock.right", 1);
    }

    #[test]
    fn should_set_text_alignment_css_class_on_video_block_if_align_attribute_is_set() {
        verifies!(
            r#"
    test 'should set text alignment CSS class on video block if align attribute is set' do
      input = 'video::cats-vs-dogs.avi[cats-and-dogs.png,align=center]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="cats-vs-dogs.avi"]', output, 1
      assert_css '.videoblock.text-center', output, 1
    end

"#
        );

        let output = convert("video::cats-vs-dogs.avi[cats-and-dogs.png,align=center]");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="cats-vs-dogs.avi"]"#, 1);
        assert_css(&output, ".videoblock.text-center", 1);
    }

    #[test]
    fn video_macro_should_honor_all_options() {
        verifies!(
            r#"
    test 'video macro should honor all options' do
      input = 'video::cats-vs-dogs.avi[options="autoplay,muted,nocontrols,loop",preload="metadata"]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[autoplay]', output, 1
      assert_css 'video[muted]', output, 1
      assert_css 'video:not([controls])', output, 1
      assert_css 'video[loop]', output, 1
      assert_css 'video[preload=metadata]', output, 1
    end

"#
        );

        let output = convert(
            r#"video::cats-vs-dogs.avi[options="autoplay,muted,nocontrols,loop",preload="metadata"]"#,
        );
        assert_css(&output, "video", 1);
        assert_css(&output, "video[autoplay]", 1);
        assert_css(&output, "video[muted]", 1);
        assert_css(&output, "video:not([controls])", 1);
        assert_css(&output, "video[loop]", 1);
        assert_css(&output, "video[preload=metadata]", 1);
    }

    #[test]
    fn video_macro_should_add_time_range_anchor_with_start_time() {
        verifies!(
            r#"
    test 'video macro should add time range anchor with start time if start attribute is set' do
      input = 'video::cats-vs-dogs.avi[start="30"]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_xpath '//video[@src="cats-vs-dogs.avi#t=30"]', output, 1
    end

"#
        );

        let output = convert(r#"video::cats-vs-dogs.avi[start="30"]"#);
        assert_css(&output, "video", 1);
        assert_xpath(&output, r#"//video[@src="cats-vs-dogs.avi#t=30"]"#, 1);
    }

    #[test]
    fn video_macro_should_add_time_range_anchor_with_end_time() {
        verifies!(
            r#"
    test 'video macro should add time range anchor with end time if end attribute is set' do
      input = 'video::cats-vs-dogs.avi[end="30"]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_xpath '//video[@src="cats-vs-dogs.avi#t=,30"]', output, 1
    end

"#
        );

        let output = convert(r#"video::cats-vs-dogs.avi[end="30"]"#);
        assert_css(&output, "video", 1);
        assert_xpath(&output, r#"//video[@src="cats-vs-dogs.avi#t=,30"]"#, 1);
    }

    #[test]
    fn video_macro_should_add_time_range_anchor_with_start_and_end_time() {
        verifies!(
            r#"
    test 'video macro should add time range anchor with start and end time if start and end attributes are set' do
      input = 'video::cats-vs-dogs.avi[start="30",end="60"]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_xpath '//video[@src="cats-vs-dogs.avi#t=30,60"]', output, 1
    end

"#
        );

        let output = convert(r#"video::cats-vs-dogs.avi[start="30",end="60"]"#);
        assert_css(&output, "video", 1);
        assert_xpath(&output, r#"//video[@src="cats-vs-dogs.avi#t=30,60"]"#, 1);
    }

    #[test]
    fn video_macro_should_use_imagesdir_attribute_to_resolve_target_and_poster() {
        verifies!(
            r#"
    test 'video macro should use imagesdir attribute to resolve target and poster' do
      input = <<~'EOS'
      :imagesdir: assets

      video::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]
      EOS

      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="assets/cats-vs-dogs.avi"]', output, 1
      assert_css 'video[poster="assets/cats-and-dogs.png"]', output, 1
      assert_css 'video[width="200"]', output, 1
      assert_css 'video[height="300"]', output, 1
    end

"#
        );

        let output =
            convert(":imagesdir: assets\n\nvideo::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]\n");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="assets/cats-vs-dogs.avi"]"#, 1);
        assert_css(&output, r#"video[poster="assets/cats-and-dogs.png"]"#, 1);
        assert_css(&output, r#"video[width="200"]"#, 1);
        assert_css(&output, r#"video[height="300"]"#, 1);
    }

    #[test]
    fn video_macro_should_not_use_imagesdir_attribute_to_resolve_target_if_target_is_a_url() {
        verifies!(
            r#"
    test 'video macro should not use imagesdir attribute to resolve target if target is a URL' do
      input = <<~'EOS'
      :imagesdir: assets

      video::http://example.org/videos/cats-vs-dogs.avi[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="http://example.org/videos/cats-vs-dogs.avi"]', output, 1
    end

"#
        );

        let output =
            convert(":imagesdir: assets\n\nvideo::http://example.org/videos/cats-vs-dogs.avi[]\n");
        assert_css(&output, "video", 1);
        assert_css(
            &output,
            r#"video[src="http://example.org/videos/cats-vs-dogs.avi"]"#,
            1,
        );
    }

    #[test]
    fn video_macro_should_output_custom_html_with_iframe_for_vimeo_service() {
        verifies!(
            r#"
    test 'video macro should output custom HTML with iframe for vimeo service' do
      input = 'video::67480300[vimeo, 400, 300, start=60, options="autoplay,muted"]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 0
      assert_css 'iframe', output, 1
      assert_css 'iframe[src="https://player.vimeo.com/video/67480300?autoplay=1&muted=1#at=60"]', output, 1
      assert_css 'iframe[width="400"]', output, 1
      assert_css 'iframe[height="300"]', output, 1
    end

"#
        );

        let output =
            convert(r#"video::67480300[vimeo, 400, 300, start=60, options="autoplay,muted"]"#);
        assert_css(&output, "video", 0);
        assert_css(&output, "iframe", 1);
        assert_css(
            &output,
            r#"iframe[src="https://player.vimeo.com/video/67480300?autoplay=1&muted=1#at=60"]"#,
            1,
        );
        assert_css(&output, r#"iframe[width="400"]"#, 1);
        assert_css(&output, r#"iframe[height="300"]"#, 1);
    }

    #[test]
    fn video_macro_should_allow_hash_for_vimeo_video_to_be_specified_in_video_id() {
        verifies!(
            r#"
    test 'video macro should allow hash for vimeo video to be specified in video ID' do
      input = 'video::67480300/123456789[vimeo, 400, 300, options=loop]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 0
      assert_css 'iframe', output, 1
      assert_css 'iframe[src="https://player.vimeo.com/video/67480300?h=123456789&loop=1"]', output, 1
      assert_css 'iframe[width="400"]', output, 1
      assert_css 'iframe[height="300"]', output, 1
    end

"#
        );

        let output = convert("video::67480300/123456789[vimeo, 400, 300, options=loop]");
        assert_css(&output, "video", 0);
        assert_css(&output, "iframe", 1);
        assert_css(
            &output,
            r#"iframe[src="https://player.vimeo.com/video/67480300?h=123456789&loop=1"]"#,
            1,
        );
        assert_css(&output, r#"iframe[width="400"]"#, 1);
        assert_css(&output, r#"iframe[height="300"]"#, 1);
    }

    #[test]
    fn video_macro_should_allow_hash_for_vimeo_video_to_be_specified_using_hash_attribute() {
        verifies!(
            r#"
    test 'video macro should allow hash for vimeo video to be specified using hash attribute' do
      input = 'video::67480300[vimeo, 400, 300, options=loop, hash=123456789]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 0
      assert_css 'iframe', output, 1
      assert_css 'iframe[src="https://player.vimeo.com/video/67480300?h=123456789&loop=1"]', output, 1
      assert_css 'iframe[width="400"]', output, 1
      assert_css 'iframe[height="300"]', output, 1
    end

"#
        );

        let output = convert("video::67480300[vimeo, 400, 300, options=loop, hash=123456789]");
        assert_css(&output, "video", 0);
        assert_css(&output, "iframe", 1);
        assert_css(
            &output,
            r#"iframe[src="https://player.vimeo.com/video/67480300?h=123456789&loop=1"]"#,
            1,
        );
        assert_css(&output, r#"iframe[width="400"]"#, 1);
        assert_css(&output, r#"iframe[height="300"]"#, 1);
    }

    #[test]
    fn video_macro_should_output_custom_html_with_iframe_for_youtube_service() {
        verifies!(
            r#"
    test 'video macro should output custom HTML with iframe for youtube service' do
      input = 'video::U8GBXvdmHT4/PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe[youtube, 640, 360, start=60, options="autoplay,muted,modest", theme=light]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 0
      assert_css 'iframe', output, 1
      assert_css 'iframe[src="https://www.youtube.com/embed/U8GBXvdmHT4?rel=0&start=60&autoplay=1&mute=1&list=PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe&modestbranding=1&theme=light"]', output, 1
      assert_css 'iframe[width="640"]', output, 1
      assert_css 'iframe[height="360"]', output, 1
    end

"#
        );

        let output = convert(
            r#"video::U8GBXvdmHT4/PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe[youtube, 640, 360, start=60, options="autoplay,muted,modest", theme=light]"#,
        );
        assert_css(&output, "video", 0);
        assert_css(&output, "iframe", 1);
        assert_css(
            &output,
            r#"iframe[src="https://www.youtube.com/embed/U8GBXvdmHT4?rel=0&start=60&autoplay=1&mute=1&list=PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe&modestbranding=1&theme=light"]"#,
            1,
        );
        assert_css(&output, r#"iframe[width="640"]"#, 1);
        assert_css(&output, r#"iframe[height="360"]"#, 1);
    }

    #[test]
    fn video_macro_should_output_custom_html_with_iframe_for_youtube_service_with_dynamic_playlist()
    {
        verifies!(
            r#"
    test 'video macro should output custom HTML with iframe for youtube service with dynamic playlist' do
      input = 'video::SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM[youtube, 640, 360, start=60, options=autoplay]'
      output = convert_string_to_embedded input
      assert_css 'video', output, 0
      assert_css 'iframe', output, 1
      assert_css 'iframe[src="https://www.youtube.com/embed/SCZF6I-Rc4I?rel=0&start=60&autoplay=1&playlist=SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM"]', output, 1
      assert_css 'iframe[width="640"]', output, 1
      assert_css 'iframe[height="360"]', output, 1
    end

"#
        );

        let output = convert(
            "video::SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM[youtube, 640, 360, start=60, options=autoplay]",
        );
        assert_css(&output, "video", 0);
        assert_css(&output, "iframe", 1);
        assert_css(
            &output,
            r#"iframe[src="https://www.youtube.com/embed/SCZF6I-Rc4I?rel=0&start=60&autoplay=1&playlist=SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM"]"#,
            1,
        );
        assert_css(&output, r#"iframe[width="640"]"#, 1);
        assert_css(&output, r#"iframe[height="360"]"#, 1);
    }

    #[test]
    fn should_detect_and_convert_audio_macro() {
        verifies!(
            r#"
    test 'should detect and convert audio macro' do
      input = 'audio::podcast.mp3[]'
      output = convert_string_to_embedded input
      assert_css 'audio', output, 1
      assert_css 'audio[src="podcast.mp3"]', output, 1
    end

"#
        );

        let output = convert("audio::podcast.mp3[]");
        assert_css(&output, "audio", 1);
        assert_css(&output, r#"audio[src="podcast.mp3"]"#, 1);
    }

    #[test]
    fn audio_macro_should_use_imagesdir_attribute_to_resolve_target() {
        verifies!(
            r#"
    test 'audio macro should use imagesdir attribute to resolve target' do
      input = <<~'EOS'
      :imagesdir: assets

      audio::podcast.mp3[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'audio', output, 1
      assert_css 'audio[src="assets/podcast.mp3"]', output, 1
    end

"#
        );

        let output = convert(":imagesdir: assets\n\naudio::podcast.mp3[]\n");
        assert_css(&output, "audio", 1);
        assert_css(&output, r#"audio[src="assets/podcast.mp3"]"#, 1);
    }

    #[test]
    fn audio_macro_should_not_use_imagesdir_attribute_to_resolve_target_if_target_is_a_url() {
        verifies!(
            r#"
    test 'audio macro should not use imagesdir attribute to resolve target if target is a URL' do
      input = <<~'EOS'
      :imagesdir: assets

      video::http://example.org/podcast.mp3[]
      EOS

      output = convert_string_to_embedded input
      assert_css 'video', output, 1
      assert_css 'video[src="http://example.org/podcast.mp3"]', output, 1
    end

"#
        );

        let output = convert(":imagesdir: assets\n\nvideo::http://example.org/podcast.mp3[]\n");
        assert_css(&output, "video", 1);
        assert_css(&output, r#"video[src="http://example.org/podcast.mp3"]"#, 1);
    }

    #[test]
    fn audio_macro_should_honor_all_options() {
        verifies!(
            r#"
    test 'audio macro should honor all options' do
      input = 'audio::podcast.mp3[options="autoplay,nocontrols,loop"]'
      output = convert_string_to_embedded input
      assert_css 'audio', output, 1
      assert_css 'audio[autoplay]', output, 1
      assert_css 'audio:not([controls])', output, 1
      assert_css 'audio[loop]', output, 1
    end

"#
        );

        let output = convert(r#"audio::podcast.mp3[options="autoplay,nocontrols,loop"]"#);
        assert_css(&output, "audio", 1);
        assert_css(&output, "audio[autoplay]", 1);
        assert_css(&output, "audio:not([controls])", 1);
        assert_css(&output, "audio[loop]", 1);
    }

    #[test]
    fn audio_macro_should_support_start_and_end_time() {
        verifies!(
            r#"
    test 'audio macro should support start and end time' do
      input = 'audio::podcast.mp3[start=1,end=2]'
      output = convert_string_to_embedded input
      assert_css 'audio', output, 1
      assert_css 'audio[controls]', output, 1
      assert_css 'audio[src="podcast.mp3#t=1,2"]', output, 1
    end
  end

"#
        );

        let output = convert("audio::podcast.mp3[start=1,end=2]");
        assert_css(&output, "audio", 1);
        assert_css(&output, "audio[controls]", 1);
        assert_css(&output, r#"audio[src="podcast.mp3#t=1,2"]"#, 1);
    }
}

mod admonition_icons {
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    non_normative!(
        r#"
  context 'Admonition icons' do
"#
    );

    /// The 35-byte `ref/asciidoctor/test/fixtures/tip.gif` — a 1x1 GIF, byte
    /// for byte the same file as `dot.gif` — and the `data:` URI Asciidoctor
    /// embeds for it under `data-uri`.
    const TIP_GIF: &[u8] = &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x05, 0x04,
        0x04, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02,
        0x02, 0x44, 0x01, 0x00, 0x3b,
    ];
    const TIP_GIF_DATA_URI: &str =
        "data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs=";

    /// A throwaway on-disk `docdir` holding `fixtures/tip.gif`, the icon the
    /// `data-uri` admonition-icon tests read (a stand-in for Asciidoctor's
    /// `test/fixtures`, reached in Ruby via `docdir: testdir`). Removed on
    /// drop. Point `base_dir` at [`IconFixtures::dir`] and resolve the icon
    /// under `iconsdir=fixtures`.
    struct IconFixtures {
        dir: PathBuf,
    }

    impl IconFixtures {
        #[track_caller]
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("adoc-icons-{}-{unique}", std::process::id()));

            let fixtures = dir.join("fixtures");
            std::fs::create_dir_all(&fixtures).expect("create fixtures dir");
            std::fs::write(fixtures.join("tip.gif"), TIP_GIF).expect("write tip.gif");

            Self { dir }
        }

        fn dir(&self) -> &Path {
            &self.dir
        }
    }

    impl Drop for IconFixtures {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn can_resolve_icon_relative_to_default_iconsdir() {
        verifies!(
            r#"
    test 'can resolve icon relative to default iconsdir' do
      input = <<~'EOS'
      :icons:

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="./images/icons/tip.png"][@alt="Tip"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons:\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="./images/icons/tip.png"][@alt="Tip"]"#,
            1,
        );
    }

    #[test]
    fn can_resolve_icon_relative_to_custom_iconsdir() {
        verifies!(
            r#"
    test 'can resolve icon relative to custom iconsdir' do
      input = <<~'EOS'
      :icons:
      :iconsdir: icons

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="icons/tip.png"][@alt="Tip"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons:\n:iconsdir: icons\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="icons/tip.png"][@alt="Tip"]"#,
            1,
        );
    }

    #[test]
    fn should_add_file_extension_to_custom_icon_if_not_specified() {
        verifies!(
            r#"
    test 'should add file extension to custom icon if not specified' do
      input = <<~'EOS'
      :icons: font
      :iconsdir: images/icons

      [TIP,icon=a]
      Override the icon of an admonition block using an attribute
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="images/icons/a.png"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons: font\n:iconsdir: images/icons\n\n[TIP,icon=a]\nOverride the icon of an admonition block using an attribute\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="images/icons/a.png"]"#,
            1,
        );
    }

    #[test]
    fn should_allow_icontype_to_be_specified_when_using_built_in_admonition_icon() {
        verifies!(
            r#"
    test 'should allow icontype to be specified when using built-in admonition icon' do
      input = 'TIP: Set the icontype using either the icontype attribute on the icons attribute.'
      [
        { 'icons' => '', 'ext' => 'png' },
        { 'icons' => '', 'icontype' => 'jpg', 'ext' => 'jpg' },
        { 'icons' => 'jpg', 'ext' => 'jpg' },
        { 'icons' => 'image', 'ext' => 'png' },
      ].each do |attributes|
        expected_src = %(./images/icons/tip.#{attributes.delete 'ext'})
        output = convert_string input, attributes: attributes
        assert_xpath %(//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="#{expected_src}"]), output, 1
      end
    end

"#
        );

        let input =
            "TIP: Set the icontype using either the icontype attribute on the icons attribute.";
        for (attributes, ext) in [
            (vec![("icons", "")], "png"),
            (vec![("icons", ""), ("icontype", "jpg")], "jpg"),
            (vec![("icons", "jpg")], "jpg"),
            (vec![("icons", "image")], "png"),
        ] {
            let mut options = Options::new().standalone(true);
            for (name, value) in attributes {
                options = options.attribute(name, value);
            }

            let output = convert_with(input, &options);
            assert_xpath(
                &output,
                &format!(
                    r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="./images/icons/tip.{ext}"]"#
                ),
                1,
            );
        }
    }

    #[test]
    fn should_allow_icontype_to_be_specified_when_using_custom_admonition_icon() {
        verifies!(
            r#"
    test 'should allow icontype to be specified when using custom admonition icon' do
      input = <<~'EOS'
      [TIP,icon=hint]
      Set the icontype using either the icontype attribute on the icons attribute.
      EOS
      [
        { 'icons' => '', 'ext' => 'png' },
        { 'icons' => '', 'icontype' => 'jpg', 'ext' => 'jpg' },
        { 'icons' => 'jpg', 'ext' => 'jpg' },
        { 'icons' => 'image', 'ext' => 'png' },
      ].each do |attributes|
        expected_src = %(./images/icons/hint.#{attributes.delete 'ext'})
        output = convert_string input, attributes: attributes
        assert_xpath %(//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="#{expected_src}"]), output, 1
      end
    end

"#
        );

        let input = "[TIP,icon=hint]\nSet the icontype using either the icontype attribute on the icons attribute.\n";
        for (attributes, ext) in [
            (vec![("icons", "")], "png"),
            (vec![("icons", ""), ("icontype", "jpg")], "jpg"),
            (vec![("icons", "jpg")], "jpg"),
            (vec![("icons", "image")], "png"),
        ] {
            let mut options = Options::new().standalone(true);
            for (name, value) in attributes {
                options = options.attribute(name, value);
            }

            let output = convert_with(input, &options);
            assert_xpath(
                &output,
                &format!(
                    r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="./images/icons/hint.{ext}"]"#
                ),
                1,
            );
        }
    }

    #[test]
    fn embeds_base64_encoded_data_uri_of_icon_when_data_uri_attribute_is_set() {
        verifies!(
            r#"
    test 'embeds base64-encoded data uri of icon when data-uri attribute is set and safe mode level is less than SECURE' do
      input = <<~'EOS'
      :icons:
      :iconsdir: fixtures
      :icontype: gif
      :data-uri:

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Tip"]', output, 1
    end

"#
        );

        let fx = IconFixtures::new();
        let output = convert_with(
            ":icons:\n:iconsdir: fixtures\n:icontype: gif\n:data-uri:\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new()
                .standalone(true)
                .safe_mode(SafeMode::Safe)
                .base_dir(fx.dir()),
        );
        assert_xpath(
            &output,
            &format!(
                r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="{TIP_GIF_DATA_URI}"][@alt="Tip"]"#
            ),
            1,
        );
    }

    #[test]
    fn should_embed_base64_encoded_data_uri_of_custom_icon_when_data_uri_attribute_is_set() {
        verifies!(
            r#"
    test 'should embed base64-encoded data uri of custom icon when data-uri attribute is set' do
      input = <<~'EOS'
      :icons:
      :iconsdir: fixtures
      :icontype: gif
      :data-uri:

      [TIP,icon=tip]
      You can set a custom icon using the icon attribute on the block.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Tip"]', output, 1
    end

"#
        );

        let fx = IconFixtures::new();
        let output = convert_with(
            ":icons:\n:iconsdir: fixtures\n:icontype: gif\n:data-uri:\n\n[TIP,icon=tip]\nYou can set a custom icon using the icon attribute on the block.\n",
            &Options::new()
                .standalone(true)
                .safe_mode(SafeMode::Safe)
                .base_dir(fx.dir()),
        );
        assert_xpath(
            &output,
            &format!(
                r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="{TIP_GIF_DATA_URI}"][@alt="Tip"]"#
            ),
            1,
        );
    }

    #[test]
    fn does_not_embed_base64_encoded_data_uri_of_icon_when_safe_mode_level_is_secure_or_greater() {
        verifies!(
            r#"
    test 'does not embed base64-encoded data uri of icon when safe mode level is SECURE or greater' do
      input = <<~'EOS'
      :icons:
      :iconsdir: fixtures
      :icontype: gif
      :data-uri:

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, attributes: { 'icons' => '' }
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="fixtures/tip.gif"][@alt="Tip"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons:\n:iconsdir: fixtures\n:icontype: gif\n:data-uri:\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new().standalone(true).attribute("icons", ""),
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="fixtures/tip.gif"][@alt="Tip"]"#,
            1,
        );
    }

    #[test]
    fn cleans_reference_to_ancestor_directories_before_reading_icon() {
        verifies!(
            r#"
    test 'cleans reference to ancestor directories before reading icon if safe mode level is at least SAFE' do
      input = <<~'EOS'
      :icons:
      :iconsdir: ../fixtures
      :icontype: gif
      :data-uri:

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => testdir }
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs="][@alt="Tip"]', output, 1
"#
        );

        let fx = IconFixtures::new();
        let output = convert_with(
            ":icons:\n:iconsdir: ../fixtures\n:icontype: gif\n:data-uri:\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new()
                .standalone(true)
                .safe_mode(SafeMode::Safe)
                .base_dir(fx.dir()),
        );
        assert_xpath(
            &output,
            &format!(
                r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="{TIP_GIF_DATA_URI}"][@alt="Tip"]"#
            ),
            1,
        );

        // The `assert_message` WARN ("image has illegal reference to ancestor of
        // jail; recovering automatically") is a render-time diagnostic this crate
        // performs silently while recovering the read back inside the jail.
        non_normative!(
            r#"
      assert_message @logger, :WARN, 'image has illegal reference to ancestor of jail; recovering automatically'
    end

"#
        );
    }

    #[test]
    fn should_import_font_awesome_and_use_font_based_icons_when_value_of_icons_attribute_is_font() {
        verifies!(
            r#"
    test 'should import Font Awesome and use font-based icons when value of icons attribute is font' do
      input = <<~'EOS'
      :icons: font

      [TIP]
      You can use icons for admonitions by setting the 'icons' attribute.
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SERVER
      assert_css %(html > head > link[rel="stylesheet"][href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/#{Asciidoctor::FONT_AWESOME_VERSION}/css/font-awesome.min.css"]), output, 1
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/i[@class="fa icon-tip"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons: font\n\n[TIP]\nYou can use icons for admonitions by setting the 'icons' attribute.\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_css(
            &output,
            r#"html > head > link[rel="stylesheet"][href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/i[@class="fa icon-tip"]"#,
            1,
        );
    }

    #[test]
    fn font_based_icon_should_not_override_icon_specified_on_admonition() {
        verifies!(
            r#"
    test 'font-based icon should not override icon specified on admonition' do
      input = <<~'EOS'
      :icons: font
      :iconsdir: images/icons

      [TIP,icon=a.png]
      Override the icon of an admonition block using an attribute
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SERVER
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/i[@class="fa icon-tip"]', output, 0
      assert_xpath '//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="images/icons/a.png"]', output, 1
    end

"#
        );

        let output = convert_with(
            ":icons: font\n:iconsdir: images/icons\n\n[TIP,icon=a.png]\nOverride the icon of an admonition block using an attribute\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/i[@class="fa icon-tip"]"#,
            0,
        );
        assert_xpath(
            &output,
            r#"//*[@class="admonitionblock tip"]//*[@class="icon"]/img[@src="images/icons/a.png"]"#,
            1,
        );
    }

    #[test]
    fn should_use_http_uri_scheme_for_assets_when_asset_uri_scheme_is_http() {
        verifies!(
            r#"
    test 'should use http uri scheme for assets when asset-uri-scheme is http' do
      input = <<~'EOS'
      :asset-uri-scheme: http
      :icons: font
      :source-highlighter: highlightjs

      TIP: You can control the URI scheme used for assets with the asset-uri-scheme attribute

      [source,ruby]
      puts "AsciiDoc, FTW!"
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SAFE
      assert_css %(html > head > link[rel="stylesheet"][href="http://cdnjs.cloudflare.com/ajax/libs/font-awesome/#{Asciidoctor::FONT_AWESOME_VERSION}/css/font-awesome.min.css"]), output, 1
      assert_css %(html > body > script[src="http://cdnjs.cloudflare.com/ajax/libs/highlight.js/#{Asciidoctor::HIGHLIGHT_JS_VERSION}/highlight.min.js"]), output, 1
    end

"#
        );

        let output = convert_with(
            ":asset-uri-scheme: http\n:icons: font\n:source-highlighter: highlightjs\n\nTIP: You can control the URI scheme used for assets with the asset-uri-scheme attribute\n\n[source,ruby]\nputs \"AsciiDoc, FTW!\"\n",
            &Options::new().standalone(true).safe_mode(SafeMode::Safe),
        );
        assert_css(
            &output,
            r#"html > head > link[rel="stylesheet"][href="http://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css"]"#,
            1,
        );
        assert_css(
            &output,
            r#"html > body > script[src="http://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.18.3/highlight.min.js"]"#,
            1,
        );
    }

    // Asciidoctor normalizes a bare `:asset-uri-scheme:` to an empty value, so
    // its CDN assets load over a protocol-relative `//cdnjs…` URL.
    // `asciidoc-parser` instead resolves a bare set of this attribute to its
    // intrinsic default of `https`, indistinguishable from an explicit
    // `:asset-uri-scheme: https`, so the renderer cannot honor the blank form.
    non_normative!(
        r#"
    test 'should use no uri scheme for assets when asset-uri-scheme is blank' do
      input = <<~'EOS'
      :asset-uri-scheme:
      :icons: font
      :source-highlighter: highlightjs

      TIP: You can control the URI scheme used for assets with the asset-uri-scheme attribute

      [source,ruby]
      puts "AsciiDoc, FTW!"
      EOS

      output = convert_string input, safe: Asciidoctor::SafeMode::SAFE
      assert_css %(html > head > link[rel="stylesheet"][href="//cdnjs.cloudflare.com/ajax/libs/font-awesome/#{Asciidoctor::FONT_AWESOME_VERSION}/css/font-awesome.min.css"]), output, 1
      assert_css %(html > body > script[src="//cdnjs.cloudflare.com/ajax/libs/highlight.js/#{Asciidoctor::HIGHLIGHT_JS_VERSION}/highlight.min.js"]), output, 1
    end
  end

"#
    );
}

mod image_paths {
    use super::*;

    // Both tests drive `block_from_string(...).normalize_asset_path(...)`, the
    // parser's asset-path resolver — an `asciidoc-parser` model API this crate
    // does not re-expose. The crate's own asset-path resolution (jail
    // confinement under `safe`/`server`, and its absence under `unsafe`) is
    // exercised through rendered `img/@src` values in the `Images` and `Media`
    // contexts instead.
    non_normative!(
        r##"
  context 'Image paths' do
    test 'restricts access to ancestor directories when safe mode level is at least SAFE' do
      input = 'image::asciidoctor.png[Asciidoctor]'
      basedir = testdir
      block = block_from_string input, attributes: { 'docdir' => basedir }
      doc = block.document
      assert doc.safe >= Asciidoctor::SafeMode::SAFE

      assert_equal File.join(basedir, 'images'), block.normalize_asset_path('images')
      assert_equal File.join(basedir, 'etc/images'), block.normalize_asset_path("#{disk_root}etc/images")
      assert_equal File.join(basedir, 'images'), block.normalize_asset_path('../../images')
    end

    test 'does not restrict access to ancestor directories when safe mode is disabled' do
      input = 'image::asciidoctor.png[Asciidoctor]'
      basedir = testdir
      block = block_from_string input, safe: Asciidoctor::SafeMode::UNSAFE, attributes: { 'docdir' => basedir }
      doc = block.document
      assert doc.safe == Asciidoctor::SafeMode::UNSAFE

      assert_equal File.join(basedir, 'images'), block.normalize_asset_path('images')
      absolute_path = "#{disk_root}etc/images"
      assert_equal absolute_path, block.normalize_asset_path(absolute_path)
      assert_equal File.expand_path(File.join(basedir, '../../images')), block.normalize_asset_path('../../images')
    end
  end

"##
    );
}

mod source_code {
    use super::*;

    non_normative!(
        r#"
  context 'Source code' do
"#
    );

    #[test]
    fn should_support_fenced_code_block_using_backticks() {
        verifies!(
            r#"
    test 'should support fenced code block using backticks' do
      input = <<~'EOS'
      ```
      puts "Hello, World!"
      ```
      EOS

      block = block_from_string input
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :fenced_code, (block.attr 'cloaked-context')
      assert_nil (block.attr 'language')
      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 1
      assert_css '.listingblock pre code', output, 1
      assert_css '.listingblock pre code:not([class])', output, 1
    end

"#
        );

        // The `block.context`/`style`/`cloaked-context`/`language` checks are
        // `asciidoc-parser` model state (verified there); here the rendered output
        // is driven. `asciidoc-parser` does not mark a bare fence's `source` style,
        // so the renderer promotes it (see `opens_with_backtick_fence`), matching
        // Asciidoctor's `<pre class="highlight"><code>` (no language).
        let output = convert("```\nputs \"Hello, World!\"\n```\n");
        assert_css(&output, ".listingblock", 1);
        assert_css(&output, ".listingblock pre code", 1);
        assert_css(&output, ".listingblock pre code:not([class])", 1);
    }

    #[test]
    fn should_not_recognize_fenced_code_blocks_with_more_than_three_delimiters() {
        verifies!(
            r#"
    test 'should not recognize fenced code blocks with more than three delimiters' do
      input = <<~'EOS'
      ````ruby
      puts "Hello, World!"
      ````

      ~~~~ javascript
      alert("Hello, World!")
      ~~~~
      EOS

      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 0
    end

"#
        );

        let output = convert(
            "````ruby\nputs \"Hello, World!\"\n````\n\n~~~~ javascript\nalert(\"Hello, World!\")\n~~~~\n",
        );
        assert_css(&output, ".listingblock", 0);
    }

    #[test]
    fn should_support_fenced_code_blocks_with_languages() {
        verifies!(
            r#"
    test 'should support fenced code blocks with languages' do
      input = <<~'EOS'
      ```ruby
      puts "Hello, World!"
      ```

      ``` javascript
      alert("Hello, World!")
      ```
      EOS

      block = (document_from_string input).blocks[0]
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :fenced_code, (block.attr 'cloaked-context')
      assert_equal 'ruby', (block.attr 'language')
      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 2
      assert_css '.listingblock pre code.language-ruby[data-lang=ruby]', output, 1
      assert_css '.listingblock pre code.language-javascript[data-lang=javascript]', output, 1
    end

"#
        );

        // The `block.context`/`style`/`cloaked-context`/`language` checks are
        // `asciidoc-parser` model state (verified there); here the rendered output
        // is driven.
        let output = convert(
            "```ruby\nputs \"Hello, World!\"\n```\n\n``` javascript\nalert(\"Hello, World!\")\n```\n",
        );
        assert_css(&output, ".listingblock", 2);
        assert_css(
            &output,
            ".listingblock pre code.language-ruby[data-lang=ruby]",
            1,
        );
        assert_css(
            &output,
            ".listingblock pre code.language-javascript[data-lang=javascript]",
            1,
        );
    }

    #[test]
    fn should_support_fenced_code_blocks_with_languages_and_numbering() {
        verifies!(
            r#"
    test 'should support fenced code blocks with languages and numbering' do
      input = <<~'EOS'
      ```ruby,numbered
      puts "Hello, World!"
      ```

      ``` javascript, numbered
      alert("Hello, World!")
      ```
      EOS

      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 2
      assert_css '.listingblock pre code.language-ruby[data-lang=ruby]', output, 1
      assert_css '.listingblock pre code.language-javascript[data-lang=javascript]', output, 1
    end

"#
        );

        // `asciidoc-parser` hands a fence's info string over as a single language
        // attribute (`ruby,numbered`) rather than splitting it into positional
        // attributes; the renderer keeps only the language token before the comma
        // (see `Renderer::source`), so the `data-lang` matches Asciidoctor.
        let output = convert(
            "```ruby,numbered\nputs \"Hello, World!\"\n```\n\n``` javascript, numbered\nalert(\"Hello, World!\")\n```\n",
        );
        assert_css(&output, ".listingblock", 2);
        assert_css(
            &output,
            ".listingblock pre code.language-ruby[data-lang=ruby]",
            1,
        );
        assert_css(
            &output,
            ".listingblock pre code.language-javascript[data-lang=javascript]",
            1,
        );
    }

    #[test]
    fn should_allow_source_style_to_be_specified_on_literal_block() {
        verifies!(
            r#"
    test 'should allow source style to be specified on literal block' do
      input = <<~'EOS'
      [source]
      ....
      console.log('Hello, World!')
      ....
      EOS

      block = block_from_string input
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :literal, (block.attr 'cloaked-context')
      assert_nil (block.attr 'language')
      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 1
      assert_css '.listingblock pre', output, 1
      assert_css '.listingblock pre code', output, 1
      assert_css '.listingblock pre code[data-lang]', output, 0
    end

"#
        );

        // The `block.context`/`style`/`cloaked-context`/`language` checks are
        // `asciidoc-parser` model state (verified there); here the rendered output
        // is driven.
        let output = convert("[source]\n....\nconsole.log('Hello, World!')\n....\n");
        assert_css(&output, ".listingblock", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_css(&output, ".listingblock pre code", 1);
        assert_css(&output, ".listingblock pre code[data-lang]", 0);
    }

    #[test]
    fn should_allow_source_style_and_language_to_be_specified_on_literal_block() {
        verifies!(
            r#"
    test 'should allow source style and language to be specified on literal block' do
      input = <<~'EOS'
      [source,js]
      ....
      console.log('Hello, World!')
      ....
      EOS

      block = block_from_string input
      assert_equal :listing, block.context
      assert_equal 'source', (block.attr 'style')
      assert_equal :literal, (block.attr 'cloaked-context')
      assert_equal 'js', (block.attr 'language')
      output = convert_string_to_embedded input
      assert_css '.listingblock', output, 1
      assert_css '.listingblock pre', output, 1
      assert_css '.listingblock pre code', output, 1
      assert_css '.listingblock pre code[data-lang]', output, 1
    end
  end

"#
        );

        // The `block.context`/`style`/`cloaked-context`/`language` checks are
        // `asciidoc-parser` model state (verified there); here the rendered output
        // is driven.
        let output = convert("[source,js]\n....\nconsole.log('Hello, World!')\n....\n");
        assert_css(&output, ".listingblock", 1);
        assert_css(&output, ".listingblock pre", 1);
        assert_css(&output, ".listingblock pre code", 1);
        assert_css(&output, ".listingblock pre code[data-lang]", 1);
    }
}

mod abstract_and_part_intro {
    use super::*;

    non_normative!(
        r#"
  context 'Abstract and Part Intro' do
"#
    );

    #[test]
    fn should_make_abstract_on_open_block_without_title_a_quote_block_for_article() {
        verifies!(
            r#"
    test 'should make abstract on open block without title a quote block for article' do
      input = <<~'EOS'
      = Article

      [abstract]
      --
      This article is about stuff.

      And other stuff.
      --

      == Section One

      content
      EOS

      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock.abstract', output, 1
      assert_css '#preamble .quoteblock', output, 1
      assert_css '.quoteblock > blockquote', output, 1
      assert_css '.quoteblock > blockquote > .paragraph', output, 2
    end

"#
        );

        let output = convert_with(
            "= Article\n\n[abstract]\n--\nThis article is about stuff.\n\nAnd other stuff.\n--\n\n== Section One\n\ncontent\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock.abstract", 1);
        assert_css(&output, "#preamble .quoteblock", 1);
        assert_css(&output, ".quoteblock > blockquote", 1);
        assert_css(&output, ".quoteblock > blockquote > .paragraph", 2);
    }

    #[test]
    fn should_make_abstract_on_open_block_with_title_a_quote_block_with_title_for_article() {
        verifies!(
            r#"
    test 'should make abstract on open block with title a quote block with title for article' do
      input = <<~'EOS'
      = Article

      .My abstract
      [abstract]
      --
      This article is about stuff.
      --

      == Section One

      content
      EOS

      output = convert_string input
      assert_css '.quoteblock', output, 1
      assert_css '.quoteblock.abstract', output, 1
      assert_css '#preamble .quoteblock', output, 1
      assert_css '.quoteblock > .title', output, 1
      assert_css '.quoteblock > .title + blockquote', output, 1
      assert_css '.quoteblock > .title + blockquote > .paragraph', output, 1
    end

"#
        );

        let output = convert_with(
            "= Article\n\n.My abstract\n[abstract]\n--\nThis article is about stuff.\n--\n\n== Section One\n\ncontent\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".quoteblock", 1);
        assert_css(&output, ".quoteblock.abstract", 1);
        assert_css(&output, "#preamble .quoteblock", 1);
        assert_css(&output, ".quoteblock > .title", 1);
        assert_css(&output, ".quoteblock > .title + blockquote", 1);
        assert_css(&output, ".quoteblock > .title + blockquote > .paragraph", 1);
    }

    #[test]
    fn should_allow_abstract_in_document_with_title_if_doctype_is_book() {
        verifies!(
            r#"
    test 'should allow abstract in document with title if doctype is book' do
      input = <<~'EOS'
      = Book
      :doctype: book

      [abstract]
      Abstract for book with title is valid
      EOS

      output = convert_string input
      assert_css '.abstract', output, 1
    end

"#
        );

        let output = convert_with(
            "= Book\n:doctype: book\n\n[abstract]\nAbstract for book with title is valid\n",
            &Options::new().standalone(true),
        );
        assert_css(&output, ".abstract", 1);
    }

    // This crate does not implement the book-without-doctitle exclusion:
    // Asciidoctor drops an `[abstract]` used as a direct child of a book document
    // that has no doctitle (logging a WARN), whereas this crate renders it. The
    // `.abstract` count therefore does not match, so the test is not verified.
    non_normative!(
        r#"
    test 'should not allow abstract as direct child of document if doctype is book' do
      input = <<~'EOS'
      :doctype: book

      [abstract]
      Abstract for book without title is invalid.
      EOS

      output = convert_string input
      assert_css '.abstract', output, 0
      assert_message @logger, :WARN, 'abstract block cannot be used in a document without a doctitle when doctype is book. Excluding block content.'
    end

"#
    );

    // These four abstract tests target the DocBook backend; this crate renders
    // only the `html5` backend.
    non_normative!(
        r#"
    test 'should make abstract on open block without title converted to DocBook' do
      input = <<~'EOS'
      = Article

      [abstract]
      --
      This article is about stuff.

      And other stuff.
      --
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'info > abstract', output, 1
      assert_css 'info > abstract > simpara', output, 2
    end

    test 'should make abstract on open block with title converted to DocBook' do
      input = <<~'EOS'
      = Article

      .My abstract
      [abstract]
      --
      This article is about stuff.
      --
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'info > abstract', output, 1
      assert_css 'info > abstract > title', output, 1
      assert_css 'info > abstract > title + simpara', output, 1
    end

    test 'should allow abstract in document with title if doctype is book converted to DocBook' do
      input = <<~'EOS'
      = Book
      :doctype: book

      [abstract]
      Abstract for book with title is valid
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'info > abstract', output, 1
      assert_css 'preface', output, 0
    end

    test 'should not allow abstract as direct child of document if doctype is book converted to DocBook' do
      input = <<~'EOS'
      :doctype: book

      [abstract]
      Abstract for book is invalid.
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'abstract', output, 0
      assert_message @logger, :WARN, 'abstract block cannot be used in a document without a doctitle when doctype is book. Excluding block content.'
    end

"#
    );

    // The partintro tests are non_normative: this crate does not fully support
    // the book doctype's level-0 part structure (html5 #188) — a valid partintro
    // renders without its `partintro` class and the crate emits a "level 0
    // section headings not supported" warning — nor does it validate and exclude
    // a misplaced partintro the way Asciidoctor does (which logs an ERROR and
    // drops the block content). The remaining variants also target the DocBook
    // backend, which this crate does not render.
    non_normative!(
        r#"
    # TODO partintro shouldn't be recognized if doctype is not book, should be in proper place
    test 'should accept partintro on open block without title' do
      input = <<~'EOS'
      = Book
      :doctype: book

      = Part 1

      [partintro]
      --
      This is a part intro.

      It can have multiple paragraphs.
      --

      == Chapter 1

      content
      EOS

      output = convert_string input
      assert_css '.openblock', output, 1
      assert_css '.openblock.partintro', output, 1
      assert_css '.openblock .title', output, 0
      assert_css '.openblock .content', output, 1
      assert_xpath %(//h1[@id="_part_1"]/following-sibling::*[#{contains_class(:openblock)}]), output, 1
      assert_xpath %(//*[#{contains_class(:openblock)}]/*[@class="content"]/*[@class="paragraph"]), output, 2
    end

    test 'should accept partintro on open block with title' do
      input = <<~'EOS'
      = Book
      :doctype: book

      = Part 1

      .Intro title
      [partintro]
      --
      This is a part intro with a title.
      --

      == Chapter 1

      content
      EOS

      output = convert_string input
      assert_css '.openblock', output, 1
      assert_css '.openblock.partintro', output, 1
      assert_css '.openblock .title', output, 1
      assert_css '.openblock .content', output, 1
      assert_xpath %(//h1[@id="_part_1"]/following-sibling::*[#{contains_class(:openblock)}]), output, 1
      assert_xpath %(//*[#{contains_class(:openblock)}]/*[@class="title"][text()="Intro title"]), output, 1
      assert_xpath %(//*[#{contains_class(:openblock)}]/*[@class="content"]/*[@class="paragraph"]), output, 1
    end

    test 'should exclude partintro if not a child of part' do
      input = <<~'EOS'
      = Book
      :doctype: book

      [partintro]
      part intro paragraph
      EOS

      output = convert_string input
      assert_css '.partintro', output, 0
      assert_message @logger, :ERROR, 'partintro block can only be used when doctype is book and must be a child of a book part. Excluding block content.'
    end

    test 'should not allow partintro unless doctype is book' do
      input = <<~'EOS'
      [partintro]
      part intro paragraph
      EOS

      output = convert_string input
      assert_css '.partintro', output, 0
      assert_message @logger, :ERROR, 'partintro block can only be used when doctype is book and must be a child of a book part. Excluding block content.'
    end

    test 'should accept partintro on open block without title converted to DocBook' do
      input = <<~'EOS'
      = Book
      :doctype: book

      = Part 1

      [partintro]
      --
      This is a part intro.

      It can have multiple paragraphs.
      --

      == Chapter 1

      content
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'partintro', output, 1
      assert_css 'part[xml|id="_part_1"] > partintro', output, 1
      assert_css 'partintro > simpara', output, 2
    end

    test 'should accept partintro on open block with title converted to DocBook' do
      input = <<~'EOS'
      = Book
      :doctype: book

      = Part 1

      .Intro title
      [partintro]
      --
      This is a part intro with a title.
      --

      == Chapter 1

      content
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'partintro', output, 1
      assert_css 'part[xml|id="_part_1"] > partintro', output, 1
      assert_css 'partintro > title', output, 1
      assert_css 'partintro > title + simpara', output, 1
    end

    test 'should exclude partintro if not a child of part converted to DocBook' do
      input = <<~'EOS'
      = Book
      :doctype: book

      [partintro]
      part intro paragraph
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'partintro', output, 0
      assert_message @logger, :ERROR, 'partintro block can only be used when doctype is book and must be a child of a book part. Excluding block content.'
    end

    test 'should not allow partintro unless doctype is book converted to DocBook' do
      input = <<~'EOS'
      [partintro]
      part intro paragraph
      EOS

      output = convert_string input, backend: 'docbook'
      assert_css 'partintro', output, 0
      assert_message @logger, :ERROR, 'partintro block can only be used when doctype is book and must be a child of a book part. Excluding block content.'
    end
  end

"#
    );
}

mod substitutions {
    use super::*;

    non_normative!(
        r#"
  context 'Substitutions' do
"#
    );

    // The first four tests assert only `block.subs` — the resolved substitution
    // list on the parser's block model, which `asciidoc-parser` verifies. They
    // have no rendered form to drive here.
    non_normative!(
        r#"
    test 'processor should not crash if subs are empty' do
      input = <<~'EOS'
      [subs=","]
      ....
      content
      ....
      EOS

      doc = document_from_string input
      block = doc.blocks.first
      assert_equal [], block.subs
    end

    test 'should be able to append subs to default block substitution list' do
      input = <<~'EOS'
      :application: Asciidoctor

      [subs="+attributes,+macros"]
      ....
      {application}
      ....
      EOS

      doc = document_from_string input
      block = doc.blocks.first
      assert_equal [:specialcharacters, :attributes, :macros], block.subs
    end

    test 'should be able to prepend subs to default block substitution list' do
      input = <<~'EOS'
      :application: Asciidoctor

      [subs="attributes+"]
      ....
      {application}
      ....
      EOS

      doc = document_from_string input
      block = doc.blocks.first
      assert_equal [:attributes, :specialcharacters], block.subs
    end

    test 'should be able to remove subs to default block substitution list' do
      input = <<~'EOS'
      [subs="-quotes,-replacements"]
      content
      EOS

      doc = document_from_string input
      block = doc.blocks.first
      assert_equal [:specialcharacters, :attributes, :macros, :post_replacements], block.subs
    end

"#
    );

    #[test]
    fn should_be_able_to_prepend_append_and_remove_subs_from_default_block_substitution_list() {
        verifies!(
            r#"
    test 'should be able to prepend, append and remove subs from default block substitution list' do
      input = <<~'EOS'
      :application: asciidoctor

      [subs="attributes+,-verbatim,+specialcharacters,+macros"]
      ....
      https://{application}.org[{gt}{gt}] <1>
      ....
      EOS

      doc = document_from_string input, standalone: false
      block = doc.blocks.first
      assert_equal [:attributes, :specialcharacters, :macros], block.subs
      result = doc.convert
      assert_includes result, '<pre><a href="https://asciidoctor.org">&gt;&gt;</a> &lt;1&gt;</pre>'
    end

"#
        );

        // `block.subs` is an `asciidoc-parser` model assertion; here the rendered
        // output is driven.
        let result = convert(
            ":application: asciidoctor\n\n[subs=\"attributes+,-verbatim,+specialcharacters,+macros\"]\n....\nhttps://{application}.org[{gt}{gt}] <1>\n....\n",
        );
        assert!(
            result
                .contains(r#"<pre><a href="https://asciidoctor.org">&gt;&gt;</a> &lt;1&gt;</pre>"#),
            "{result}"
        );
    }

    #[test]
    fn should_be_able_to_set_subs_then_modify_them() {
        verifies!(
            r#"
    test 'should be able to set subs then modify them' do
      input = <<~'EOS'
      [subs="verbatim,-callouts"]
      _hey now_ <1>
      EOS

      doc = document_from_string input, standalone: false
      block = doc.blocks.first
      assert_equal [:specialcharacters], block.subs
      result = doc.convert
      assert_includes result, '_hey now_ &lt;1&gt;'
    end
  end

"#
        );

        // `block.subs` is an `asciidoc-parser` model assertion; here the rendered
        // output is driven.
        let result = convert("[subs=\"verbatim,-callouts\"]\n_hey now_ <1>\n");
        assert!(result.contains("_hey now_ &lt;1&gt;"), "{result}");
    }
}

mod references {
    use super::*;

    non_normative!(
        r#"
  context 'References' do
"#
    );

    #[test]
    fn should_not_recognize_block_anchor_with_illegal_id_characters() {
        verifies!(
            r#"
    test 'should not recognize block anchor with illegal id characters' do
      input = <<~'EOS'
      [[illegal$id,Reference Text]]
      ----
      content
      ----
      EOS

      doc = document_from_string input
      block = doc.blocks.first
      assert_nil block.id
      assert_nil(block.attr 'reftext')
      refute doc.catalog[:refs].key? 'illegal$id'
    end

"#
        );

        // `block.id`/`block.attr 'reftext'` are `asciidoc-parser` model assertions;
        // the catalog `refute … key?` is driven here through the document catalog.
        let doc = load("[[illegal$id,Reference Text]]\n----\ncontent\n----\n");
        assert!(!doc.catalog().contains_id("illegal$id"));
    }

    #[test]
    fn should_not_recognize_block_anchor_that_starts_with_digit() {
        verifies!(
            r#"
    test 'should not recognize block anchor that starts with digit' do
      input = <<~'EOS'
      [[3-blind-mice]]
      --
      see how they run
      --
      EOS

      output = convert_string_to_embedded input
      assert_includes output, '[[3-blind-mice]]'
      assert_xpath '/*[@id=":3-blind-mice"]', output, 0
    end

"#
        );

        let output = convert("[[3-blind-mice]]\n--\nsee how they run\n--\n");
        assert!(output.contains("[[3-blind-mice]]"), "{output}");
        assert_xpath(&output, r#"/*[@id=":3-blind-mice"]"#, 0);
    }

    #[test]
    fn should_recognize_block_anchor_that_starts_with_colon() {
        verifies!(
            r#"
    test 'should recognize block anchor that starts with colon' do
      input = <<~'EOS'
      [[:idname]]
      --
      content
      --
      EOS

      output = convert_string_to_embedded input
      assert_xpath '/*[@id=":idname"]', output, 1
    end

"#
        );

        let output = convert("[[:idname]]\n--\ncontent\n--\n");
        assert_xpath(&output, r#"/*[@id=":idname"]"#, 1);
    }

    #[test]
    fn should_use_specified_id_and_reftext_when_registering_block_reference() {
        verifies!(
            r#"
    test 'should use specified id and reftext when registering block reference' do
      input = <<~'EOS'
      [[debian,Debian Install]]
      .Installation on Debian
      ----
      $ apt-get install asciidoctor
      ----
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['debian']
      refute_nil ref
      assert_equal 'Debian Install', ref.reftext
      assert_equal 'debian', (doc.resolve_id 'Debian Install')
    end

"#
        );

        let doc = load(
            "[[debian,Debian Install]]\n.Installation on Debian\n----\n$ apt-get install asciidoctor\n----\n",
        );
        let entry = doc.catalog().get_ref("debian");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().reftext.as_deref(), Some("Debian Install"));
        assert_eq!(
            doc.catalog().resolve_id("Debian Install"),
            Some("debian".to_string())
        );
    }

    #[test]
    fn should_allow_square_brackets_in_block_reference_text() {
        verifies!(
            r#"
    test 'should allow square brackets in block reference text' do
      input = <<~'EOS'
      [[debian,[Debian] Install]]
      .Installation on Debian
      ----
      $ apt-get install asciidoctor
      ----
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['debian']
      refute_nil ref
      assert_equal '[Debian] Install', ref.reftext
      assert_equal 'debian', (doc.resolve_id '[Debian] Install')
    end

"#
        );

        let doc = load(
            "[[debian,[Debian] Install]]\n.Installation on Debian\n----\n$ apt-get install asciidoctor\n----\n",
        );
        let entry = doc.catalog().get_ref("debian");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().reftext.as_deref(), Some("[Debian] Install"));
        assert_eq!(
            doc.catalog().resolve_id("[Debian] Install"),
            Some("debian".to_string())
        );
    }

    #[test]
    fn should_allow_comma_in_block_reference_text() {
        verifies!(
            r#"
    test 'should allow comma in block reference text' do
      input = <<~'EOS'
      [[debian, Debian, Ubuntu]]
      .Installation on Debian
      ----
      $ apt-get install asciidoctor
      ----
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['debian']
      refute_nil ref
      assert_equal 'Debian, Ubuntu', ref.reftext
      assert_equal 'debian', (doc.resolve_id 'Debian, Ubuntu')
    end

"#
        );

        let doc = load(
            "[[debian, Debian, Ubuntu]]\n.Installation on Debian\n----\n$ apt-get install asciidoctor\n----\n",
        );
        let entry = doc.catalog().get_ref("debian");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().reftext.as_deref(), Some("Debian, Ubuntu"));
        assert_eq!(
            doc.catalog().resolve_id("Debian, Ubuntu"),
            Some("debian".to_string())
        );
    }

    #[test]
    fn should_resolve_attribute_reference_in_title_using_attribute_defined_at_location_of_block() {
        verifies!(
            r##"
    test 'should resolve attribute reference in title using attribute defined at location of block' do
      input = <<~'EOS'
      = Document Title
      :foo: baz

      intro paragraph. see <<free-standing>>.

      :foo: bar

      .foo is {foo}
      [#formal-para]
      paragraph with title

      [discrete#free-standing]
      == foo is still {foo}
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['formal-para']
      refute_nil ref
      assert_equal 'foo is bar', ref.title
      assert_equal 'formal-para', (doc.resolve_id 'foo is bar')
      output = doc.convert standalone: false
      assert_include '<a href="#free-standing">foo is still bar</a>', output
      assert_include '<h2 id="free-standing" class="discrete">foo is still bar</h2>', output
    end

"##
        );

        // `ref.title` is an `asciidoctor` model assertion (the parser records the
        // resolved title as the reference's reftext); the reverse lookup and the
        // rendered output are driven here.
        let input = "= Document Title\n:foo: baz\n\nintro paragraph. see <<free-standing>>.\n\n:foo: bar\n\n.foo is {foo}\n[#formal-para]\nparagraph with title\n\n[discrete#free-standing]\n== foo is still {foo}\n";
        let doc = load(input);
        assert_eq!(
            doc.catalog().resolve_id("foo is bar"),
            Some("formal-para".to_string())
        );

        let output = convert(input);
        assert!(
            output.contains(r##"<a href="#free-standing">foo is still bar</a>"##),
            "{output}"
        );
        assert!(
            output.contains(r##"<h2 id="free-standing" class="discrete">foo is still bar</h2>"##),
            "{output}"
        );
    }

    #[test]
    fn should_substitute_attribute_references_in_reftext_when_registering_block_reference() {
        verifies!(
            r#"
    test 'should substitute attribute references in reftext when registering block reference' do
      input = <<~'EOS'
      :label-tiger: Tiger

      [[tiger-evolution,Evolution of the {label-tiger}]]
      ****
      Information about the evolution of the tiger.
      ****
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['tiger-evolution']
      refute_nil ref
      assert_equal 'Evolution of the Tiger', ref.attributes['reftext']
      assert_equal 'tiger-evolution', (doc.resolve_id 'Evolution of the Tiger')
    end

"#
        );

        let doc = load(
            ":label-tiger: Tiger\n\n[[tiger-evolution,Evolution of the {label-tiger}]]\n****\nInformation about the evolution of the tiger.\n****\n",
        );
        let entry = doc.catalog().get_ref("tiger-evolution");
        assert!(entry.is_some());
        assert_eq!(
            entry.unwrap().reftext.as_deref(),
            Some("Evolution of the Tiger")
        );
        assert_eq!(
            doc.catalog().resolve_id("Evolution of the Tiger"),
            Some("tiger-evolution".to_string())
        );
    }

    #[test]
    fn should_use_specified_reftext_when_registering_block_reference() {
        verifies!(
            r#"
    test 'should use specified reftext when registering block reference' do
      input = <<~'EOS'
      [[debian]]
      [reftext="Debian Install"]
      .Installation on Debian
      ----
      $ apt-get install asciidoctor
      ----
      EOS

      doc = document_from_string input
      ref = doc.catalog[:refs]['debian']
      refute_nil ref
      assert_equal 'Debian Install', ref.reftext
      assert_equal 'debian', (doc.resolve_id 'Debian Install')
    end
  end
end
"#
        );

        let doc = load(
            "[[debian]]\n[reftext=\"Debian Install\"]\n.Installation on Debian\n----\n$ apt-get install asciidoctor\n----\n",
        );
        let entry = doc.catalog().get_ref("debian");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().reftext.as_deref(), Some("Debian Install"));
        assert_eq!(
            doc.catalog().resolve_id("Debian Install"),
            Some("debian".to_string())
        );
    }
}
