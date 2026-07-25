//! Port of Asciidoctor's `text_test.rb`.
//!
//! This file is almost entirely about *inline* text substitutions — emphasis,
//! strong, monospace, superscript/subscript, smart quotes, line breaks, and
//! plus/passthrough spans. `asciidoc-parser` applies those substitutions at
//! parse time, so this crate's rendered output already carries them; each test
//! is driven through `convert` (embedded, the counterpart to
//! `convert_string_to_embedded`), `convert_with(..standalone(true)..)` (the
//! counterpart to `convert_string`), or `convert_with(..doctype("inline")..)`
//! (the counterpart to `convert_inline_string`).
//!
//! The UTF-8 encoding-from-included-file test *is* verified: `include::`
//! resolution is this crate's own filesystem layer
//! ([`FsIncludeFileHandler`](crate::include_handler)), so it is driven for real
//! against the vendored `fixtures/encoding.adoc`.
//!
//! Kept `non_normative!` are the tests this crate's stack cannot satisfy:
//!
//! * The UTF-8 *encoding* document/embedded pair converts the whole `:encoding`
//!   sample. The sample ends in a bulleted list, so its fourth `<p>` comes from
//!   a list item, which this crate does not yet emit — revisit those two once
//!   lists land (#120). Encoding itself is not a concern here — this crate
//!   converts `&str` that is already valid UTF-8. The matching DocBook-backend
//!   tests are out of scope, and the `arbitrary block` test reaches for the
//!   `PreprocessorReader` / `Parser.next_block` Ruby APIs (its verse-`<pre>`
//!   behavior is already covered by the paragraphs suite).
//! * *Compat mode* is not implemented by `asciidoc-parser`, so tests that carry
//!   both a compat-mode and a modern assertion are ported as `verifies!` with
//!   only the modern assertion driven in Rust; the compat-mode assertion
//!   remains in the reproduced Ruby text.
//! * The `markdown horizontal rules` (positive) test asserts a thematic break
//!   for every variant at 0–3 leading spaces. This crate recognizes the `---` /
//!   `***` / `___` variants only at column 1 — a deliberate, settled divergence
//!   from Asciidoctor's leading-space tolerance — so the offset dimension of
//!   the test is left unsatisfied by design and the whole test is
//!   non-normative. The negative case *is* verified, since it asserts no
//!   thematic break either way.

use std::path::Path;

use crate::{
    convert, convert_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

track_file!("ref/asciidoctor/test/text_test.rb");

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context "Text" do
"#
);

// UTF-8 encoding test: converts the whole `:encoding` sample standalone. The
// sample ends in a bulleted list, so the fourth `<p>` the test counts comes
// from a list item, which this crate does not yet render (light this up once
// lists land — #120); encoding itself is not a parser concern here.
non_normative!(
    r#"
  test "proper encoding to handle utf8 characters in document using html backend" do
    output = example_document(:encoding).convert
    assert_xpath '//p', output, 4
    assert_xpath '//a', output, 1
  end

"#
);

// As above, for the embedded (`standalone: false`) HTML conversion.
non_normative!(
    r#"
  test "proper encoding to handle utf8 characters in embedded document using html backend" do
    output = example_document(:encoding, standalone: false).convert
    assert_xpath '//p', output, 4
    assert_xpath '//a', output, 1
  end

"#
);

// DocBook backend is out of scope for this crate.
non_normative!(
    r#"
  test 'proper encoding to handle utf8 characters in document using docbook backend' do
    output = example_document(:encoding, attributes: { 'backend' => 'docbook', 'xmlns' => '' }).convert
    assert_xpath '//xmlns:simpara', output, 4
    assert_xpath '//xmlns:link', output, 1
  end

"#
);

// DocBook backend is out of scope for this crate.
non_normative!(
    r#"
  test 'proper encoding to handle utf8 characters in embedded document using docbook backend' do
    output = example_document(:encoding, standalone: false, attributes: { 'backend' => 'docbook' }).convert
    assert_xpath '//simpara', output, 4
    assert_xpath '//link', output, 1
  end

"#
);

// Reads the `:encoding` sample through the `PreprocessorReader` and
// `Parser.next_block` Ruby APIs, which this crate does not expose. A verse
// block rendering a `<pre>` from UTF-8 content is already covered by the
// paragraphs suite.
non_normative!(
    r#"
  # NOTE this test ensures we have the encoding line on block templates too
  test 'proper encoding to handle utf8 characters in arbitrary block' do
    input = []
    input << "[verse]\n"
    input += (File.readlines (sample_doc_path :encoding), mode: Asciidoctor::FILE_READ_MODE)
    doc = empty_document
    reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
    block = Asciidoctor::Parser.next_block(reader, doc)
    assert_xpath '//pre', block.convert.gsub(/^\s*\n/, ''), 1
  end

"#
);

// Unlike the parser crate — which delegates `include::` resolution and marks
// this non-normative — filesystem access lives in *this* crate, in
// [`FsIncludeFileHandler`](crate::include_handler). So this test is driven for
// real: `convert_with` with a `base_dir` installs the include handler even for
// string input, and the `tags=romé` selector (a UTF-8 tag name) filters the
// vendored `fixtures/encoding.adoc` down to its single tagged paragraph. The
// Ruby `empty_safe_document base_dir: testdir` maps to `SafeMode::Safe`
// anchored at `ref/asciidoctor/test`; the `PreprocessorReader` / `next_block`
// scaffolding is Ruby plumbing for converting one block, which the whole-string
// `convert` subsumes here (the include expands to exactly one paragraph).
#[test]
fn proper_encoding_to_handle_utf8_characters_from_included_file() {
    verifies!(
        r#"
  test 'proper encoding to handle utf8 characters from included file' do
    input = 'include::fixtures/encoding.adoc[tags=romé]'
    doc = empty_safe_document base_dir: testdir
    reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
    block = Asciidoctor::Parser.next_block(reader, doc)
    output = block.convert
    assert_css '.paragraph', output, 1
  end

"#
    );

    // `testdir` is Asciidoctor's `ref/asciidoctor/test`; this crate resolves the
    // include relative to it under the `safe` jail.
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ref/asciidoctor/test");
    let html = convert_with(
        "include::fixtures/encoding.adoc[tags=romé]",
        &Options::new().safe_mode(SafeMode::Safe).base_dir(base_dir),
    );
    assert_css(&html, ".paragraph", 1);
}

#[test]
fn escaped_text_markup() {
    verifies!(
        r#"
  test 'escaped text markup' do
    assert_match(/All your &lt;em&gt;inline&lt;\/em&gt; markup belongs to &lt;strong&gt;us&lt;\/strong&gt;!/,
        convert_string('All your <em>inline</em> markup belongs to <strong>us</strong>!'))
  end

"#
    );

    let html = convert_with(
        "All your <em>inline</em> markup belongs to <strong>us</strong>!",
        &Options::new().standalone(true),
    );
    assert!(html.contains(
        "All your &lt;em&gt;inline&lt;/em&gt; markup belongs to &lt;strong&gt;us&lt;/strong&gt;!"
    ));
}

#[test]
fn line_breaks() {
    verifies!(
        r#"
  test "line breaks" do
    assert_xpath "//br", convert_string("Well this is +\njust fine and dandy, isn't it?"), 1
  end

"#
    );

    let html = convert_with(
        "Well this is +\njust fine and dandy, isn't it?",
        &Options::new().standalone(true),
    );
    assert_xpath(&html, "//br", 1);
}

#[test]
fn single_and_double_quoted_text() {
    verifies!(
        r#"
  test 'single- and double-quoted text' do
    output = convert_string_to_embedded(%q(``Where?,'' she said, flipping through her copy of `The New Yorker.'), attributes: { 'compat-mode' => '' })
    assert_match(/&#8220;Where\?,&#8221;/, output)
    assert_match(/&#8216;The New Yorker.&#8217;/, output)

    output = convert_string_to_embedded(%q("`Where?,`" she said, flipping through her copy of '`The New Yorker.`'))
    assert_match(/&#8220;Where\?,&#8221;/, output)
    assert_match(/&#8216;The New Yorker.&#8217;/, output)
  end

"#
    );

    // Compat mode is unsupported; only the modern (second) form is driven here.
    let html = convert(r#""`Where?,`" she said, flipping through her copy of '`The New Yorker.`'"#);
    assert!(html.contains("&#8220;Where?,&#8221;"));
    assert!(html.contains("&#8216;The New Yorker.&#8217;"));
}

#[test]
fn multiple_double_quoted_text_on_a_single_line() {
    verifies!(
        r#"
  test 'multiple double-quoted text on a single line' do
    assert_equal '&#8220;Our business is constantly changing&#8221; or &#8220;We need faster time to market.&#8221;',
        convert_inline_string(%q(``Our business is constantly changing'' or ``We need faster time to market.''), attributes: { 'compat-mode' => '' })
    assert_equal '&#8220;Our business is constantly changing&#8221; or &#8220;We need faster time to market.&#8221;',
        convert_inline_string(%q("`Our business is constantly changing`" or "`We need faster time to market.`"))
  end

"#
    );

    // Compat mode is unsupported; only the modern (second) form is driven here.
    // The inline doctype emits the fragment with the single trailing newline
    // this crate — matching the `asciidoctor` CLI — always appends.
    let output = convert_with(
        r#""`Our business is constantly changing`" or "`We need faster time to market.`""#,
        &Options::new().doctype("inline"),
    );
    assert_eq!(
        output,
        "&#8220;Our business is constantly changing&#8221; or \
         &#8220;We need faster time to market.&#8221;\n"
    );
}

#[test]
fn horizontal_rule() {
    verifies!(
        r#"
  test 'horizontal rule' do
    input = <<~'EOS'
    This line is separated by a horizontal rule...

    '''

    ...from this line.
    EOS
    output = convert_string_to_embedded input
    assert_xpath "//hr", output, 1
    assert_xpath "/*[@class='paragraph']", output, 2
    assert_xpath "(/*[@class='paragraph'])[1]/following-sibling::hr", output, 1
    assert_xpath "/hr/following-sibling::*[@class='paragraph']", output, 1
  end

"#
    );

    let html =
        convert("This line is separated by a horizontal rule...\n\n'''\n\n...from this line.\n");

    // The test xpath engine handles double-quoted attribute values only.
    assert_xpath(&html, "//hr", 1);
    assert_xpath(&html, "/*[@class=\"paragraph\"]", 2);
    assert_xpath(
        &html,
        "(/*[@class=\"paragraph\"])[1]/following-sibling::hr",
        1,
    );
    assert_xpath(&html, "/hr/following-sibling::*[@class=\"paragraph\"]", 1);
}

// The `markdown horizontal rules` (positive) test asserts a thematic break for
// six variants at four leading offsets. This crate recognizes all six variants
// (`---`, `- - -`, `***`, `* * *`, `___`, `_ _ _`) at column 1 — the `-`/`*`
// forms are spec-recognized, the `_` forms an Asciidoctor-compatibility
// extension — but intentionally diverges on the leading offset: Asciidoctor
// tolerates 0–3 leading spaces, while this crate requires the marker at column
// 1 (an indented line becomes a literal paragraph). That divergence is a
// deliberate, settled decision, so the offset dimension of this test is left
// unsatisfied by design and the whole test is reproduced non-normatively.
non_normative!(
    r#"
  test 'markdown horizontal rules' do
    variants = [
      '---',
      '- - -',
      '***',
      '* * *',
      '___',
      '_ _ _'
    ]

    offsets = [
      '',
      ' ',
      '  ',
      '   '
    ]

    variants.each do |variant|
      offsets.each do |offset|
        input = <<~EOS
        This line is separated by a horizontal rule...

        #{offset}#{variant}

        ...from this line.
        EOS
        output = convert_string_to_embedded input
        assert_xpath "//hr", output, 1
        assert_xpath "/*[@class='paragraph']", output, 2
        assert_xpath "(/*[@class='paragraph'])[1]/following-sibling::hr", output, 1
        assert_xpath "/hr/following-sibling::*[@class='paragraph']", output, 1
      end
    end
  end

"#
);

#[test]
fn markdown_horizontal_rules_negative_case() {
    verifies!(
        r#"
  test 'markdown horizontal rules negative case' do

    bad_variants = [
      '- - - -',
      '* * * *',
      '_ _ _ _'
    ]

    good_offsets = [
      '',
      ' ',
      '  ',
      '   '
    ]

    bad_variants.each do |variant|
      good_offsets.each do |offset|
        input = <<~EOS
        This line is separated by something that is not a horizontal rule...

        #{offset}#{variant}

        ...from this line.
        EOS
        output = convert_string_to_embedded input
        assert_xpath '//hr', output, 0
      end
    end

    good_variants = [
      '- - -',
      '* * *',
      '_ _ _'
    ]

    bad_offsets = [
      "\t",
      '    '
    ]

    good_variants.each do |variant|
      bad_offsets.each do |offset|
        input = <<~EOS
        This line is separated by something that is not a horizontal rule...

        #{offset}#{variant}

        ...from this line.
        EOS
        output = convert_string_to_embedded input
        assert_xpath '//hr', output, 0
      end
    end
  end

"#
    );

    // None of these produce a thematic break in this crate. The four-character
    // variants are rejected by both this crate and Asciidoctor; the tab- and
    // four-space-offset good variants are rejected here because this crate
    // intentionally accepts no leading offset at all (Asciidoctor rejects only
    // these two). That stricter behavior is a deliberate, settled divergence.
    for variant in ["- - - -", "* * * *", "_ _ _ _"] {
        for offset in ["", " ", "  ", "   "] {
            let input = format!(
                "This line is separated by something that is not a horizontal rule...\n\n{offset}{variant}\n\n...from this line.\n"
            );
            assert_xpath(&convert(&input), "//hr", 0);
        }
    }

    for variant in ["- - -", "* * *", "_ _ _"] {
        for offset in ["\t", "    "] {
            let input = format!(
                "This line is separated by something that is not a horizontal rule...\n\n{offset}{variant}\n\n...from this line.\n"
            );
            assert_xpath(&convert(&input), "//hr", 0);
        }
    }
}

#[test]
fn emphasized_text_using_underscore_characters() {
    verifies!(
        r#"
  test "emphasized text using underscore characters" do
    assert_xpath "//em", convert_string("An _emphatic_ no")
  end

"#
    );

    let html = convert_with("An _emphatic_ no", &Options::new().standalone(true));
    assert_xpath(&html, "//em", 1);
}

#[test]
fn emphasized_text_with_single_quote_using_apostrophe_characters() {
    verifies!(
        r#"
  test 'emphasized text with single quote using apostrophe characters' do
    rsquo = decode_char 8217
    assert_xpath %(//em[text()="Johnny#{rsquo}s"]), convert_string(%q(It's 'Johnny's' phone), attributes: { 'compat-mode' => '' })
    assert_xpath %(//p[text()="It#{rsquo}s 'Johnny#{rsquo}s' phone"]), convert_string(%q(It's 'Johnny's' phone))
  end

"#
    );

    // Compat mode is unsupported; only the modern (second) assertion is driven.
    // `#{rsquo}` decodes to U+2019 (right single quotation mark).
    let html = convert_with("It's 'Johnny's' phone", &Options::new().standalone(true));
    assert_xpath(
        &html,
        "//p[text()=\"It\u{2019}s 'Johnny\u{2019}s' phone\"]",
        1,
    );
}

#[test]
fn emphasized_text_with_escaped_single_quote_using_apostrophe_characters() {
    verifies!(
        r#"
  test 'emphasized text with escaped single quote using apostrophe characters' do
    assert_xpath %(//em[text()="Johnny's"]), convert_string(%q(It's 'Johnny\\'s' phone), attributes: { 'compat-mode' => '' })
    assert_xpath %(//p[text()="It's 'Johnny's' phone"]), convert_string(%q(It\\'s 'Johnny\\'s' phone))
  end

"#
    );

    // Compat mode is unsupported; only the modern (second) assertion is driven.
    // In the `%q(..)` Ruby literal, `\\'` is a single escaped apostrophe.
    let html = convert_with(
        "It\\'s 'Johnny\\'s' phone",
        &Options::new().standalone(true),
    );
    assert_xpath(&html, "//p[text()=\"It's 'Johnny's' phone\"]", 1);
}

#[test]
fn escaped_single_quote_is_restored_as_single_quote() {
    verifies!(
        r#"
  test "escaped single quote is restored as single quote" do
    assert_xpath "//p[contains(text(), \"Let's do it!\")]", convert_string("Let\\'s do it!")
  end

"#
    );

    // The whole paragraph is `Let's do it!`, so an exact `text()` match is
    // equivalent to the Ruby `contains(text(), ..)` here.
    let html = convert_with("Let\\'s do it!", &Options::new().standalone(true));
    assert_xpath(&html, "//p[text()=\"Let's do it!\"]", 1);
}

#[test]
fn unescape_escaped_single_quote_emphasis_in_compat_mode_only() {
    verifies!(
        r#"
  test 'unescape escaped single quote emphasis in compat mode only' do
    assert_xpath %(//p[text()="A 'single quoted string' example"]), convert_string_to_embedded(%(A \\'single quoted string' example), attributes: { 'compat-mode' => '' })
    assert_xpath %(//p[text()="'single quoted string'"]), convert_string_to_embedded(%(\\'single quoted string'), attributes: { 'compat-mode' => '' })

    assert_xpath %(//p[text()="A \\'single quoted string' example"]), convert_string_to_embedded(%(A \\'single quoted string' example))
    assert_xpath %(//p[text()="\\'single quoted string'"]), convert_string_to_embedded(%(\\'single quoted string'))
  end

"#
    );

    // Compat mode is unsupported; only the two modern assertions are driven.
    // Outside compat mode the leading escaped apostrophe keeps its backslash.
    let html = convert("A \\'single quoted string' example");
    assert_xpath(
        &html,
        r#"//p[text()="A \'single quoted string' example"]"#,
        1,
    );

    let html = convert("\\'single quoted string'");
    assert_xpath(&html, r#"//p[text()="\'single quoted string'"]"#, 1);
}

#[test]
fn emphasized_text_at_end_of_line() {
    verifies!(
        r#"
  test "emphasized text at end of line" do
    assert_xpath "//em", convert_string("This library is _awesome_")
  end

"#
    );

    let html = convert_with(
        "This library is _awesome_",
        &Options::new().standalone(true),
    );
    assert_xpath(&html, "//em", 1);
}

#[test]
fn emphasized_text_at_beginning_of_line() {
    verifies!(
        r#"
  test "emphasized text at beginning of line" do
    assert_xpath "//em", convert_string("_drop_ it")
  end

"#
    );

    let html = convert_with("_drop_ it", &Options::new().standalone(true));
    assert_xpath(&html, "//em", 1);
}

#[test]
fn emphasized_text_across_line() {
    verifies!(
        r#"
  test "emphasized text across line" do
    assert_xpath "//em", convert_string("_check it_")
  end

"#
    );

    let html = convert_with("_check it_", &Options::new().standalone(true));
    assert_xpath(&html, "//em", 1);
}

#[test]
fn unquoted_text() {
    verifies!(
        r#"
  test "unquoted text" do
    refute_match(/#/, convert_string("An #unquoted# word"))
  end

"#
    );

    let html = convert_with("An #unquoted# word", &Options::new().standalone(true));
    assert!(!html.contains('#'));
}

#[test]
fn backticks_and_straight_quotes_in_text() {
    verifies!(
        r#"
  test 'backticks and straight quotes in text' do
    backslash = '\\'
    assert_equal %q(run <code>foo</code> <em>dog</em>), convert_inline_string(%q(run `foo` 'dog'), attributes: { 'compat-mode' => '' })
    assert_equal %q(run <code>foo</code> 'dog'), convert_inline_string(%q(run `foo` 'dog'))
    assert_equal %q(run `foo` 'dog'), convert_inline_string(%(run #{backslash}`foo` 'dog'))
    assert_equal %q(run &#8216;foo` 'dog&#8217;), convert_inline_string(%q(run '`foo` 'dog`'))
    assert_equal %q(run '`foo` 'dog`'), convert_inline_string(%(run #{backslash}'`foo` 'dog#{backslash}`'))
  end

"#
    );

    // Compat mode is unsupported; the four modern assertions are driven here.
    // The inline doctype appends the single trailing newline this crate always
    // emits.
    let inline = |src: &str| convert_with(src, &Options::new().doctype("inline"));
    assert_eq!(inline("run `foo` 'dog'"), "run <code>foo</code> 'dog'\n");
    assert_eq!(inline("run \\`foo` 'dog'"), "run `foo` 'dog'\n");
    assert_eq!(inline("run '`foo` 'dog`'"), "run &#8216;foo` 'dog&#8217;\n");
    assert_eq!(inline("run \\'`foo` 'dog\\`'"), "run '`foo` 'dog`'\n");
}

#[test]
fn plus_characters_inside_single_plus_passthrough() {
    verifies!(
        r#"
  test 'plus characters inside single plus passthrough' do
    assert_xpath '//p[text()="+"]', convert_string_to_embedded('+++')
    assert_xpath '//p[text()="+="]', convert_string_to_embedded('++=+')
  end

"#
    );

    assert_xpath(&convert("+++"), "//p[text()=\"+\"]", 1);
    assert_xpath(&convert("++=+"), "//p[text()=\"+=\"]", 1);
}

#[test]
fn plus_passthrough_escapes_entity_reference() {
    verifies!(
        r#"
  test 'plus passthrough escapes entity reference' do
    assert_match(/&amp;#44;/, convert_string_to_embedded('+&#44;+'))
    assert_match(/one&amp;#44;two/, convert_string_to_embedded('one++&#44;++two'))
  end

"#
    );

    assert!(convert("+&#44;+").contains("&amp;#44;"));
    assert!(convert("one++&#44;++two").contains("one&amp;#44;two"));
}

mod basic_styling {
    use super::*;

    non_normative!(
        r#"
  context "basic styling" do
    setup do
      @output = convert_string("A *BOLD* word.  An _italic_ word.  A `mono` word.  ^superscript!^ and some ~subscript~.")
    end

"#
    );

    // The Ruby `setup` renders `@output` once for the whole context; each Rust
    // test reconverts the same input standalone.
    fn output() -> String {
        convert_with(
            "A *BOLD* word.  An _italic_ word.  A `mono` word.  ^superscript!^ and some ~subscript~.",
            &Options::new().standalone(true),
        )
    }

    #[test]
    fn strong() {
        verifies!(
            r#"
    test "strong" do
      assert_xpath "//strong", @output, 1
    end

"#
        );

        assert_xpath(&output(), "//strong", 1);
    }

    #[test]
    fn italic() {
        verifies!(
            r#"
    test "italic" do
      assert_xpath "//em", @output, 1
    end

"#
        );

        assert_xpath(&output(), "//em", 1);
    }

    #[test]
    fn monospaced() {
        verifies!(
            r#"
    test "monospaced" do
      assert_xpath "//code", @output, 1
    end

"#
        );

        assert_xpath(&output(), "//code", 1);
    }

    #[test]
    fn superscript() {
        verifies!(
            r#"
    test "superscript" do
      assert_xpath "//sup", @output, 1
    end

"#
        );

        assert_xpath(&output(), "//sup", 1);
    }

    #[test]
    fn subscript() {
        verifies!(
            r#"
    test "subscript" do
      assert_xpath "//sub", @output, 1
    end

"#
        );

        assert_xpath(&output(), "//sub", 1);
    }

    #[test]
    fn passthrough() {
        verifies!(
            r#"
    test "passthrough" do
      assert_xpath "//code", convert_string("This is +passed through+."), 0
      assert_xpath "//code", convert_string("This is +passed through and monospaced+.", attributes: { 'compat-mode' => '' }), 1
    end

"#
        );

        // Modern: single-plus is a passthrough, not a monospaced phrase. The
        // second assertion is compat-mode (unsupported) and is not driven.
        let html = convert_with(
            "This is +passed through+.",
            &Options::new().standalone(true),
        );
        assert_xpath(&html, "//code", 0);
    }

    #[test]
    fn nested_styles() {
        verifies!(
            r#"
    test "nested styles" do
      output = convert_string("Winning *big _time_* in the +city *boyeeee*+.", attributes: { 'compat-mode' => '' })

      assert_xpath "//strong/em", output
      assert_xpath "//code/strong", output

      output = convert_string("Winning *big _time_* in the `city *boyeeee*`.")

      assert_xpath "//strong/em", output
      assert_xpath "//code/strong", output
    end

"#
        );

        // Compat mode is unsupported; only the modern form is driven here.
        let html = convert_with(
            "Winning *big _time_* in the `city *boyeeee*`.",
            &Options::new().standalone(true),
        );
        assert_xpath(&html, "//strong/em", 1);
        assert_xpath(&html, "//code/strong", 1);
    }

    #[test]
    fn unconstrained_quotes() {
        verifies!(
            r#"
    test 'unconstrained quotes' do
      output = convert_string('**B**__I__++M++[role]++M++', attributes: { 'compat-mode' => '' })
      assert_xpath '//strong', output, 1
      assert_xpath '//em', output, 1
      assert_xpath '//code[not(@class)]', output, 1
      assert_xpath '//code[@class="role"]', output, 1

      output = convert_string('**B**__I__``M``[role]``M``')
      assert_xpath '//strong', output, 1
      assert_xpath '//em', output, 1
      assert_xpath '//code[not(@class)]', output, 1
      assert_xpath '//code[@class="role"]', output, 1
    end
"#
        );

        // Compat mode is unsupported; only the modern form is driven here.
        let html = convert_with(
            "**B**__I__``M``[role]``M``",
            &Options::new().standalone(true),
        );
        assert_xpath(&html, "//strong", 1);
        assert_xpath(&html, "//em", 1);
        assert_xpath(&html, "//code[@class=\"role\"]", 1);

        // The test xpath engine has no `not(@class)`, so the Ruby assertion that
        // exactly one `<code>` has no class is instead pinned by asserting the
        // full rendered paragraph (which also subsumes the `//code[@class="role"]`
        // check above).
        assert!(html
            .contains(r#"<strong>B</strong><em>I</em><code>M</code><code class="role">M</code>"#));
    }

    non_normative!(
        r#"
  end

"#
    );
}

#[test]
fn should_format_asian_characters_as_words() {
    verifies!(
        r#"
  test 'should format Asian characters as words' do
    assert_xpath '//strong', (convert_string_to_embedded 'bold *要* bold')
    assert_xpath '//strong', (convert_string_to_embedded 'bold *素* bold')
    assert_xpath '//strong', (convert_string_to_embedded 'bold *要素* bold')
  end
"#
    );

    assert_xpath(&convert("bold *要* bold"), "//strong", 1);
    assert_xpath(&convert("bold *素* bold"), "//strong", 1);
    assert_xpath(&convert("bold *要素* bold"), "//strong", 1);
}

non_normative!(
    r#"
end
"#
);
