//! Port of Asciidoctor's `reader_test.rb`.
//!
//! `reader_test.rb` exercises Asciidoctor's `Reader` and `PreprocessorReader` —
//! the line buffer and preprocessor that feed the parser. Most of it drives
//! that machinery directly (`peek_line`, `read_line`, the include stack, cursor
//! bookkeeping) and has no rendered-HTML form, so it is tracked
//! `non_normative!` here and verified in `asciidoc-parser`'s own port of this
//! file.
//!
//! What *is* document-visible — and therefore ported to `verifies!`, driven
//! through `convert`/`convert_with` per the crate memory
//! `html5-favor-convert-over-parse` — is the behavior the preprocessor makes
//! observable in the output: CRLF cleaning, front-matter skipping, `include::`
//! directive resolution (content injection, line/tag selection, the link-macro
//! replacement under a secure safe mode, unresolved/optional handling,
//! leveloffset, attribute-substituted targets, escaping, and
//! max-include-depth), and conditional preprocessing (`ifdef`/`ifndef`/`ifeval`
//! inclusion and the malformed/unmatched/unterminated-directive warnings). The
//! warning cases are verified against the re-exported `Document`'s warnings
//! inventory (crate memory `ruby-logger-warnings-map-to-document-warnings`).
//!
//! Local `include::` targets resolve against a `base_dir` pointed at
//! `ref/asciidoctor/test`, so a `fixtures/<name>` target matches the Ruby
//! suite's `DIRNAME`-relative include paths; see [`fixtures_base_dir`]. Remote
//! (URI) includes are out of scope for this crate (crate memory
//! `remote-fetch-not-planned`) and stay `non_normative!`, as do the
//! jruby/classloader, non-UTF-8-encoding, and chmod-unreadable cases and every
//! test that asserts only on `Reader` internals.
//!
//! A handful of *document-visible* tests are also kept `non_normative!` because
//! this crate diverges from the Asciidoctor oracle. One divergence is
//! permanent: compat-mode role handling on a replacement link is out of scope –
//! this crate will not implement compat mode. The rest are tracked by a
//! follow-up issue rather than asserting the divergent output:
//!
//! - a three-level nested include from a subdirectory leaves the inner include
//!   unresolved — [#131]
//! - an absolute include path is not resolved — [#132]
//! - an `include::` inside an `ifdef[...]` bracket is not processed — [#133]
//! - a remote `include::` target under a non-secure safe mode is reported
//!   unresolved (and warns) instead of falling back to a link macro — [#136]
//! - a non-UTF-8 include file cannot be read (this crate is UTF-8 only) —
//!   [#138]
//! - an unreadable include file is not distinguished from a missing one, so it
//!   reports the generic not-found diagnostic — [#146]
//!
//! [#131]: https://github.com/asciidoc-rs/asciidoc-html5/issues/131
//! [#132]: https://github.com/asciidoc-rs/asciidoc-html5/issues/132
//! [#133]: https://github.com/asciidoc-rs/asciidoc-html5/issues/133
//! [#136]: https://github.com/asciidoc-rs/asciidoc-html5/issues/136
//! [#138]: https://github.com/asciidoc-rs/asciidoc-html5/issues/138
//! [#146]: https://github.com/asciidoc-rs/asciidoc-html5/issues/146

use std::path::PathBuf;

use asciidoc_parser::warnings::WarningType;

use crate::{
    convert, convert_with, load_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

track_file!("ref/asciidoctor/test/reader_test.rb");

/// The base directory local `include::` targets resolve against: the vendored
/// Asciidoctor test tree, so a `fixtures/<name>` target matches the Ruby
/// suite's `DIRNAME`-relative include paths.
fn fixtures_base_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("ref/asciidoctor/test")
}

/// Converts `src` (embedded) under the `Safe` safe mode with `base_dir`
/// anchored at the fixtures tree — the counterpart to the Ruby suite's
/// `convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME`.
fn convert_safe_with_fixtures(src: &str) -> String {
    convert_with(
        src,
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(fixtures_base_dir()),
    )
}

/// The typed warnings raised while loading `src` under the `Safe` safe mode
/// with the fixtures `base_dir`, as `(WarningType, line)` pairs in source
/// order.
fn fixture_warnings(src: &str) -> Vec<(WarningType, usize)> {
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

/// The typed warnings raised while loading `src` (no fixtures, default safe
/// mode) with `attrs` seeded as document attributes, as `(WarningType, line)`
/// pairs in source order.
fn conditional_warnings(src: &str, attrs: &[(&str, &str)]) -> Vec<(WarningType, usize)> {
    let mut options = Options::new();
    for (name, value) in attrs {
        options = options.attribute(*name, *value);
    }
    let doc = load_with(src, &options);
    doc.warnings()
        .map(|w| (w.warning.clone(), w.source.line()))
        .collect()
}

/// Converts `src` (embedded) with `attrs` seeded as document attributes — the
/// counterpart to `Asciidoctor::Document.new input, attributes: {…}` read as
/// rendered HTML rather than as the preprocessed line stream.
fn convert_with_attrs(src: &str, attrs: &[(&str, &str)]) -> String {
    let mut options = Options::new();
    for (name, value) in attrs {
        options = options.attribute(*name, *value);
    }
    convert_with(src, &options)
}

/// Writes `content` to a file named `name` in a fresh temp directory tagged by
/// `tag`, returning that directory for use as an include `base_dir`. Lets a
/// test exercise include resolution against a file it fully controls — a space
/// in the name, CRLF line endings, no trailing newline — without touching the
/// vendored fixtures (the counterpart to the Ruby suite's `Tempfile` includes).
fn temp_include_dir(tag: &str, name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("adoc-reader-inc-{}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp include dir");
    std::fs::write(dir.join(name), content).expect("write temp include file");
    dir
}

/// Converts `src` (embedded) under the `Safe` safe mode with `base_dir` set to
/// `dir` — the temp-directory counterpart of [`convert_safe_with_fixtures`].
fn convert_safe_in(dir: &std::path::Path, src: &str) -> String {
    convert_with(
        src,
        &Options::new()
            .safe_mode(SafeMode::Safe)
            .base_dir(dir.to_path_buf()),
    )
}

/// The inner text of the sole `<pre>…</pre>` in `html`.
fn pre_content(html: &str) -> &str {
    let start = html.find("<pre>").expect("a <pre> block") + "<pre>".len();
    let end = html[start..].find("</pre>").expect("a closing </pre>") + start;
    &html[start..end]
}

/// Asserts the sole listing block's `<pre>` content equals `expected`. Like
/// Asciidoctor, this crate strips leading and trailing blank lines from
/// verbatim block content (so a tag region that opens on a blank source line,
/// as `tagged-class.rb`'s `bark` region does, renders without that blank), so
/// the tag-selected lines are compared directly.
fn assert_listing_selection(html: &str, expected: &str) {
    assert_eq!(pre_content(html), expected, "in:\n{html}");
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

class ReaderTest < Minitest::Test
  DIRNAME = ASCIIDOCTOR_TEST_DIR

  SAMPLE_DATA = ['first line', 'second line', 'third line']

"#
);

mod reader {
    use super::*;

    non_normative!(
        r#"
  context 'Reader' do
    context 'Prepare lines' do
      test 'should prepare lines from Array data' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA, reader.lines
      end

      test 'should prepare lines from String data' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA.join(Asciidoctor::LF)
        assert_equal SAMPLE_DATA, reader.lines
      end

      test 'should prepare lines from String data with trailing newline' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA.join(Asciidoctor::LF) + Asciidoctor::LF
        assert_equal SAMPLE_DATA, reader.lines
      end

      test 'should remove UTF-8 BOM from first line of String data' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          data = String.new %(\xef\xbb\xbf#{SAMPLE_DATA.join ::Asciidoctor::LF}), encoding: start_encoding
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end

      test 'should remove UTF-8 BOM from first line of Array data' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          data = SAMPLE_DATA.drop 0
          data[0] = String.new %(\xef\xbb\xbf#{data.first}), encoding: start_encoding
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end

      test 'should encode UTF-16LE string to UTF-8 when BOM is found' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          data = "\ufeff#{SAMPLE_DATA.join ::Asciidoctor::LF}".encode('UTF-16LE').force_encoding(start_encoding)
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end

      test 'should encode UTF-16LE string array to UTF-8 when BOM is found' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          # NOTE can't split a UTF-16LE string using .lines when encoding is set to UTF-8
          data = SAMPLE_DATA.drop 0
          data.unshift %(\ufeff#{data.shift})
          data.each {|line| (line.encode 'UTF-16LE').force_encoding start_encoding }
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end

      test 'should encode UTF-16BE string to UTF-8 when BOM is found' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          data = "\ufeff#{SAMPLE_DATA.join ::Asciidoctor::LF}".encode('UTF-16BE').force_encoding(start_encoding)
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end

      test 'should encode UTF-16BE string array to UTF-8 when BOM is found' do
        ['UTF-8', 'ASCII-8BIT'].each do |start_encoding|
          data = SAMPLE_DATA.drop 0
          data.unshift %(\ufeff#{data.shift})
          data = data.map {|line| (line.encode 'UTF-16BE').force_encoding start_encoding }
          reader = Asciidoctor::Reader.new data, nil, normalize: true
          assert_equal Encoding::UTF_8, reader.lines[0].encoding
          assert_equal 'f', reader.lines[0].chr
          assert_equal SAMPLE_DATA, reader.lines
        end
      end
    end

    context 'With empty data' do
      test 'has_more_lines? should return false with empty data' do
        refute Asciidoctor::Reader.new.has_more_lines?
      end

      test 'empty? should return true with empty data' do
        assert Asciidoctor::Reader.new.empty?
        assert Asciidoctor::Reader.new.eof?
      end

      test 'next_line_empty? should return true with empty data' do
        assert Asciidoctor::Reader.new.next_line_empty?
      end

      test 'peek_line should return nil with empty data' do
        assert_nil Asciidoctor::Reader.new.peek_line
      end

      test 'peek_lines should return empty Array with empty data' do
        assert_equal [], Asciidoctor::Reader.new.peek_lines(1)
      end

      test 'read_line should return nil with empty data' do
        assert_nil Asciidoctor::Reader.new.read_line
        #assert_nil Asciidoctor::Reader.new.get_line
      end

      test 'read_lines should return empty Array with empty data' do
        assert_equal [], Asciidoctor::Reader.new.read_lines
        #assert_equal [], Asciidoctor::Reader.new.get_lines
      end
    end

    context 'With data' do
      test 'has_more_lines? should return true if there are lines remaining' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert reader.has_more_lines?
      end

      test 'empty? should return false if there are lines remaining' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        refute reader.empty?
        refute reader.eof?
      end

      test 'next_line_empty? should return false if next line is not blank' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        refute reader.next_line_empty?
      end

      test 'next_line_empty? should return true if next line is blank' do
        reader = Asciidoctor::Reader.new ['', 'second line']
        assert reader.next_line_empty?
      end

      test 'peek_line should return nil if next entry is nil' do
        assert_nil (Asciidoctor::Reader.new [nil]).peek_line
      end

      test 'peek_line should return next line if there are lines remaining' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA.first, reader.peek_line
      end

      test 'peek_line should not consume line or increment line number' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA.first, reader.peek_line
        assert_equal SAMPLE_DATA.first, reader.peek_line
        assert_equal 1, reader.lineno
      end

      test 'peek_line should return next lines if there are lines remaining' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA[0..1], reader.peek_lines(2)
      end

      test 'peek_lines should not consume lines or increment line number' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA[0..1], reader.peek_lines(2)
        assert_equal SAMPLE_DATA[0..1], reader.peek_lines(2)
        assert_equal 1, reader.lineno
      end

      test 'peek_lines should not increment line number if reader overruns buffer' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA, (reader.peek_lines SAMPLE_DATA.size * 2)
        assert_equal 1, reader.lineno
      end

      test 'peek_lines should peek all lines if no arguments are given' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA, reader.peek_lines
        assert_equal 1, reader.lineno
      end

      test 'peek_lines should not invert order of lines' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA, reader.lines
        reader.peek_lines 3
        assert_equal SAMPLE_DATA, reader.lines
      end

      test 'read_line should return next line if there are lines remaining' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA.first, reader.read_line
      end

      test 'read_line should consume next line and increment line number' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA[0], reader.read_line
        assert_equal SAMPLE_DATA[1], reader.read_line
        assert_equal 3, reader.lineno
      end

      test 'advance should consume next line and return a Boolean indicating if a line was consumed' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert reader.advance
        assert reader.advance
        assert reader.advance
        refute reader.advance
      end

      test 'read_lines should return all lines' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA, reader.read_lines
      end

      test 'read should return all lines joined as String' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        assert_equal SAMPLE_DATA.join(::Asciidoctor::LF), reader.read
      end

      test 'has_more_lines? should return false after read_lines is invoked' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        reader.read_lines
        refute reader.has_more_lines?
      end

      test 'unshift puts line onto Reader as next line to read' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA, nil, normalize: true
        reader.unshift 'line zero'
        assert_equal 'line zero', reader.peek_line
        assert_equal 'line zero', reader.read_line
        assert_equal 1, reader.lineno
      end

      test 'terminate should consume all lines and update line number' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        reader.terminate
        assert reader.eof?
        assert_equal 4, reader.lineno
      end

      test 'skip_blank_lines should skip blank lines' do
        reader = Asciidoctor::Reader.new ['', ''].concat(SAMPLE_DATA)
        reader.skip_blank_lines
        assert_equal SAMPLE_DATA.first, reader.peek_line
      end

      test 'lines should return remaining lines' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        reader.read_line
        assert_equal SAMPLE_DATA[1..-1], reader.lines
      end

      test 'source_lines should return copy of original data Array' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        reader.read_lines
        assert_equal SAMPLE_DATA, reader.source_lines
      end

      test 'source should return original data Array joined as String' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA
        reader.read_lines
        assert_equal SAMPLE_DATA.join(::Asciidoctor::LF), reader.source
      end

    end

    context 'Line context' do
      test 'cursor.to_s should return file name and line number of current line' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA, 'sample.adoc'
        reader.read_line
        assert_equal 'sample.adoc: line 2', reader.cursor.to_s
      end

      test 'line_info should return file name and line number of current line' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA, 'sample.adoc'
        reader.read_line
        assert_equal 'sample.adoc: line 2', reader.line_info
      end

      test 'cursor_at_prev_line should return file name and line number of previous line read' do
        reader = Asciidoctor::Reader.new SAMPLE_DATA, 'sample.adoc'
        reader.read_line
        assert_equal 'sample.adoc: line 1', reader.cursor_at_prev_line.to_s
      end
    end

    context 'Read lines until' do
      test 'Read lines until until end' do
        lines = <<~'EOS'.lines
        This is one paragraph.

        This is another paragraph.
        EOS

        reader = Asciidoctor::Reader.new lines, nil, normalize: true
        result = reader.read_lines_until
        assert_equal 3, result.size
        assert_equal lines.map(&:chomp), result
        refute reader.has_more_lines?
        assert reader.eof?
      end

      test 'Read lines until until blank line' do
        lines = <<~'EOS'.lines
        This is one paragraph.

        This is another paragraph.
        EOS

        reader = Asciidoctor::Reader.new lines, nil, normalize: true
        result = reader.read_lines_until break_on_blank_lines: true
        assert_equal 1, result.size
        assert_equal lines.first.chomp, result.first
        assert_equal lines.last.chomp, reader.peek_line
      end

      test 'Read lines until until blank line preserving last line' do
        lines = <<~'EOS'.split ::Asciidoctor::LF
        This is one paragraph.

        This is another paragraph.
        EOS

        reader = Asciidoctor::Reader.new lines
        result = reader.read_lines_until break_on_blank_lines: true, preserve_last_line: true
        assert_equal 1, result.size
        assert_equal lines.first.chomp, result.first
        assert reader.next_line_empty?
      end

      test 'Read lines until until condition is true' do
        lines = <<~'EOS'.split ::Asciidoctor::LF
        --
        This is one paragraph inside the block.

        This is another paragraph inside the block.
        --

        This is a paragraph outside the block.
        EOS

        reader = Asciidoctor::Reader.new lines
        reader.read_line
        result = reader.read_lines_until {|line| line == '--' }
        assert_equal 3, result.size
        assert_equal lines[1, 3], result
        assert reader.next_line_empty?
      end

      test 'Read lines until until condition is true, taking last line' do
        lines = <<~'EOS'.split ::Asciidoctor::LF
        --
        This is one paragraph inside the block.

        This is another paragraph inside the block.
        --

        This is a paragraph outside the block.
        EOS

        reader = Asciidoctor::Reader.new lines
        reader.read_line
        result = reader.read_lines_until(read_last_line: true) {|line| line == '--' }
        assert_equal 4, result.size
        assert_equal lines[1, 4], result
        assert reader.next_line_empty?
      end

      test 'Read lines until until condition is true, taking and preserving last line' do
        lines = <<~'EOS'.split ::Asciidoctor::LF
        --
        This is one paragraph inside the block.

        This is another paragraph inside the block.
        --

        This is a paragraph outside the block.
        EOS

        reader = Asciidoctor::Reader.new lines
        reader.read_line
        result = reader.read_lines_until(read_last_line: true, preserve_last_line: true) {|line| line == '--' }
        assert_equal 4, result.size
        assert_equal lines[1, 4], result
        assert_equal '--', reader.peek_line
      end

      test 'read lines until terminator' do
        lines = <<~'EOS'.lines
        ****
        captured

        also captured
        ****

        not captured
        EOS

        expected = ['captured', '', 'also captured']

        doc = empty_safe_document base_dir: DIRNAME
        reader = Asciidoctor::PreprocessorReader.new doc, lines, nil, normalize: true
        terminator = reader.read_line
        result = reader.read_lines_until terminator: terminator, skip_processing: true
        assert_equal expected, result
        refute reader.unterminated
      end

      test 'should flag reader as unterminated if reader reaches end of source without finding terminator' do
        lines = <<~'EOS'.lines
        ****
        captured

        also captured

        captured yet again
        EOS

        expected = lines[1..-1].map(&:chomp)

        using_memory_logger do |logger|
          doc = empty_safe_document base_dir: DIRNAME
          reader = Asciidoctor::PreprocessorReader.new doc, lines, nil, normalize: true
          terminator = reader.peek_line
          result = reader.read_lines_until terminator: terminator, skip_first_line: true, skip_processing: true
          assert_equal expected, result
          assert reader.unterminated
          assert_message logger, :WARN, '<stdin>: line 1: unterminated **** block', Hash
        end
      end
    end
  end

"#
    );
}

mod preprocessor_reader {
    use super::*;

    non_normative!(
        r#"
  context 'PreprocessorReader' do
"#
    );

    mod type_hierarchy {
        use super::*;

        non_normative!(
            r#"
    context 'Type hierarchy' do
      test 'PreprocessorReader should extend from Reader' do
        reader = empty_document.reader
        assert_kind_of Asciidoctor::PreprocessorReader, reader
      end

      test 'PreprocessorReader should invoke or emulate Reader initializer' do
        doc = Asciidoctor::Document.new SAMPLE_DATA
        reader = doc.reader
        assert_equal SAMPLE_DATA, reader.lines
        assert_equal 1, reader.lineno
      end
    end

"#
        );
    }

    mod prepare_lines {
        use super::*;

        non_normative!(
            r#"
    context 'Prepare lines' do
      test 'should prepare and normalize lines from Array data' do
        data = SAMPLE_DATA.drop 0
        data.unshift ''
        data.push ''
        doc = Asciidoctor::Document.new data
        reader = doc.reader
        assert_equal [''] + SAMPLE_DATA, reader.lines
      end

      test 'should prepare and normalize lines from String data' do
        data = SAMPLE_DATA.drop 0
        data.unshift ' '
        data.push ' '
        data_as_string = data * ::Asciidoctor::LF
        doc = Asciidoctor::Document.new data_as_string
        reader = doc.reader
        assert_equal [''] + SAMPLE_DATA, reader.lines
      end

      test 'should drop all lines if all lines are empty' do
        data = ['', ' ', '', ' ']
        doc = Asciidoctor::Document.new data
        reader = doc.reader
        assert reader.lines.empty?
      end

"#
        );

        #[test]
        fn should_clean_crlf_from_end_of_lines() {
            verifies!(
                r#"
      test 'should clean CRLF from end of lines' do
        input = <<~EOS
        source\r
        with\r
        CRLF\r
        line endings\r
        EOS

        [input, input.lines, input.split(::Asciidoctor::LF), input.split(::Asciidoctor::LF).join(::Asciidoctor::LF)].each do |lines|
          doc = Asciidoctor::Document.new lines
          reader = doc.reader
          reader.lines.each do |line|
            refute line.end_with?("\r"), "CRLF not properly cleaned for source lines: #{lines.inspect}"
            refute line.end_with?("\r\n"), "CRLF not properly cleaned for source lines: #{lines.inspect}"
            refute line.end_with?("\n"), "CRLF not properly cleaned for source lines: #{lines.inspect}"
          end
        end
      end

"#
            );

            // The preprocessor strips CR (and the CRLF pair) from line ends, so the
            // rendered paragraph carries none.
            let html = convert("source\r\nwith\r\nCRLF\r\nline endings\r\n");
            assert!(html.contains("source\nwith\nCRLF\nline endings"), "{html}");
            assert!(!html.contains('\r'), "{html}");
        }

        #[test]
        fn should_not_skip_front_matter_by_default() {
            verifies!(
                r#"
      test 'should not skip front matter by default' do
        input = <<~'EOS'
        ---
        layout: post
        title: Document Title
        author: username
        tags: [ first, second ]
        ---
        = Document Title
        Author Name

        preamble
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        refute doc.attributes.key?('front-matter')
        assert_equal '---', reader.peek_line
        assert_equal 1, reader.lineno
      end

"#
            );

            // Without `skip-front-matter`, the front matter is left in the source as
            // ordinary content rather than being consumed, so its text renders.
            let html = convert("---\nlayout: post\ntitle: Document Title\nauthor: username\ntags: [ first, second ]\n---\n= Document Title\nAuthor Name\n\npreamble\n");
            assert!(html.contains("layout: post"), "{html}");
        }

        #[test]
        fn should_not_skip_front_matter_if_ending_delimiter_is_not_found() {
            verifies!(
                r#"
      test 'should not skip front matter if ending delimiter is not found' do
        input = <<~'EOS'
        ---
        title: Document Title
        tags: [ first, second ]
        = Document Title
        Author Name

        preamble
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'skip-front-matter' => '' }
        reader = doc.reader
        assert_equal '---', reader.peek_line
        refute doc.attributes.key? 'front-matter'
        assert_equal 1, reader.lineno
      end

"#
            );

            // With no closing `---`, the front matter is not skipped even though
            // `skip-front-matter` is set, so its text still renders as content.
            let html = convert_with(
                "---\ntitle: Document Title\ntags: [ first, second ]\n= Document Title\nAuthor Name\n\npreamble\n",
                &Options::new().set("skip-front-matter"),
            );
            assert!(html.contains("title: Document Title"), "{html}");
        }

        #[test]
        fn should_skip_front_matter_if_specified_by_skip_front_matter_attribute() {
            verifies!(
                r#"
      test 'should skip front matter if specified by skip-front-matter attribute' do
        front_matter = <<~'EOS'.chop
        layout: post
        title: Document Title
        author: username
        tags: [ first, second ]
        EOS

        input = <<~EOS
        ---
        #{front_matter}
        ---
        = Document Title
        Author Name

        preamble
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'skip-front-matter' => '' }
        reader = doc.reader
        assert_equal '= Document Title', reader.peek_line
        assert_equal front_matter, doc.attributes['front-matter']
        assert_equal 7, reader.lineno
      end
"#
            );

            // With `skip-front-matter` set and a closing `---`, the front matter is
            // consumed: its text does not render, and the body after it (the preamble)
            // does.
            let html = convert_with(
                "---\nlayout: post\ntitle: Document Title\nauthor: username\ntags: [ first, second ]\n---\n= Document Title\nAuthor Name\n\npreamble\n",
                &Options::new().set("skip-front-matter"),
            );
            assert!(!html.contains("layout: post"), "{html}");
            assert!(html.contains("preamble"), "{html}");
        }

        non_normative!(
            r#"
    end

"#
        );
    }

    mod include_stack {
        use super::*;

        non_normative!(
            r#"
    context 'Include Stack' do
      test 'PreprocessorReader#push_include method should return reader' do
        reader = empty_document.reader
        append_lines = %w(one two three)
        result = reader.push_include append_lines, '<stdin>', '<stdin>'
        assert_equal reader, result
      end

      test 'PreprocessorReader#push_include method should put lines on top of stack' do
        lines = %w(a b c)
        doc = Asciidoctor::Document.new lines
        reader = doc.reader
        append_lines = %w(one two three)
        reader.push_include append_lines, '', '<stdin>'
        assert_equal 1, reader.include_stack.size
        assert_equal 'one', reader.read_line.rstrip
      end

      test 'PreprocessorReader#push_include method should gracefully handle file and path' do
        lines = %w(a b c)
        doc = Asciidoctor::Document.new lines
        reader = doc.reader
        append_lines = %w(one two three)
        reader.push_include append_lines
        assert_equal 1, reader.include_stack.size
        assert_equal 'one', reader.read_line.rstrip
        assert_nil reader.file
        assert_equal '<stdin>', reader.path
      end

      test 'PreprocessorReader#push_include method should set path from file automatically if not specified' do
        lines = %w(a b c)
        doc = Asciidoctor::Document.new lines
        reader = doc.reader
        append_lines = %w(one two three)
        reader.push_include append_lines, '/tmp/lines.adoc'
        assert_equal '/tmp/lines.adoc', reader.file
        assert_equal 'lines.adoc', reader.path
        assert doc.catalog[:includes]['lines']
      end

      test 'PreprocessorReader#push_include method should accept file as a URI and compute dir and path' do
        file_uri = ::URI.parse 'http://example.com/docs/file.adoc'
        dir_uri = ::URI.parse 'http://example.com/docs'
        reader = empty_document.reader
        reader.push_include %w(one two three), file_uri
        assert_same file_uri, reader.file
        assert_equal dir_uri, reader.dir
        assert_equal 'file.adoc', reader.path
      end

      test 'PreprocessorReader#push_include method should accept file as a top-level URI and compute dir and path' do
        file_uri = ::URI.parse 'http://example.com/index.adoc'
        dir_uri = ::URI.parse 'http://example.com'
        reader = empty_document.reader
        reader.push_include %w(one two three), file_uri
        assert_same file_uri, reader.file
        assert_equal dir_uri, reader.dir
        assert_equal 'index.adoc', reader.path
      end

      test 'PreprocessorReader#push_include method should not fail if data is nil' do
        lines = %w(a b c)
        doc = Asciidoctor::Document.new lines
        reader = doc.reader
        reader.push_include nil, '', '<stdin>'
        assert_equal 0, reader.include_stack.size
        assert_equal 'a', reader.read_line.rstrip
      end

      test 'PreprocessorReader#push_include method should ignore dot in directory name when computing include path' do
        lines = %w(a b c)
        doc = Asciidoctor::Document.new lines
        reader = doc.reader
        append_lines = %w(one two three)
        reader.push_include append_lines, nil, 'include.d/data'
        assert_nil reader.file
        assert_equal 'include.d/data', reader.path
        assert doc.catalog[:includes]['include.d/data']
      end
    end

"#
        );
    }

    mod include_directive {
        use super::*;

        non_normative!(
            r#"
    context 'Include Directive' do
"#
        );

        #[test]
        fn should_replace_include_directive_with_link_macro_in_default_safe_mode() {
            verifies!(
                r#"
      test 'should replace include directive with link macro in default safe mode' do
        input = 'include::include-file.adoc[]'
        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 'link:include-file.adoc[role=include]', reader.read_line
      end

"#
            );

            // Under the default (secure) safe mode the include directive is not
            // resolved; it becomes a link macro carrying the `include` role.
            let html = convert("include::include-file.adoc[]");
            assert_css(&html, "a.include", 1);
            assert_xpath(&html, r#"//a[@href="include-file.adoc"]"#, 1);
        }

        // Non-normative: compat-mode role handling is permanently out of scope –
        // this crate will not implement compat mode, so it does not drop the
        // include role on the replacement link.
        non_normative!(
            r#"
      test 'should not add role to link macro used to replace include directive in compat mode' do
        input = 'include::include-file.adoc[]'
        doc = Asciidoctor::Document.new input, attributes: { 'compat-mode' => '' }
        reader = doc.reader
        assert_equal 'link:include-file.adoc[]', reader.read_line
      end

"#
        );

        #[test]
        fn should_escape_spaces_in_target_when_generating_link_from_include_directive() {
            verifies!(
                r#"
      test 'should escape spaces in target when generating link from include directive' do
        input = 'include::foo bar baz.adoc[]'
        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 'link:pass:c[foo bar baz.adoc][role=include]', reader.read_line
      end

"#
            );

            // Spaces in the target survive into the generated link's href.
            let html = convert("include::foo bar baz.adoc[]");
            assert_css(&html, "a.include", 1);
            assert_xpath(&html, r#"//a[@href="foo bar baz.adoc"]"#, 1);
        }

        // Non-normative: a remote target under a non-secure safe mode is reported
        // unresolved instead of falling back to a link (#136).
        non_normative!(
            r#"
      test 'should replace include directive with link macro if safe mode allows it, but allow-uri-read is not set' do
        using_memory_logger do |logger|
          input = 'include::https://example.org/dist/info.adoc[]'
          doc = Asciidoctor::Document.new input, safe: :safe
          reader = doc.reader
          assert_equal 'link:https://example.org/dist/info.adoc[role=include]', reader.read_line
          assert_empty logger
        end
      end

"#
        );

        // Non-normative on two counts: compat-mode role suppression is
        // permanently out of scope, and the remote link fallback (#136) is still
        // unimplemented.
        non_normative!(
            r#"
      test 'should not add role to link macro that replaces include directive with remote target in compat mode' do
        input = 'include::https://example.org/dist/info.adoc[]'
        doc = Asciidoctor::Document.new input, safe: :safe, attributes: { 'compat-mode' => '' }
        reader = doc.reader
        assert_equal 'link:https://example.org/dist/info.adoc[]', reader.read_line
      end

"#
        );

        // Non-normative: the remote link fallback under a non-secure safe mode (#136).
        non_normative!(
            r#"
      test 'should escape spaces in target when generating link from remote include directive' do
        input = 'include::https://example.org/no such file.adoc[]'
        doc = Asciidoctor::Document.new input, safe: :safe
        reader = doc.reader
        assert_equal 'link:pass:c[https://example.org/no such file.adoc][role=include]', reader.read_line
      end

"#
        );

        #[test]
        fn include_directive_is_enabled_when_safe_mode_is_less_than_secure() {
            verifies!(
                r#"
      test 'include directive is enabled when safe mode is less than SECURE' do
        input = 'include::fixtures/include-file.adoc[]'
        doc = document_from_string input, safe: :safe, standalone: false, base_dir: DIRNAME
        output = doc.convert
        assert_match(/included content/, output)
        assert doc.catalog[:includes]['fixtures/include-file']
      end

"#
            );

            let html = convert_safe_with_fixtures("include::fixtures/include-file.adoc[]");
            assert!(html.contains("included content"), "{html}");
        }

        #[test]
        fn should_strip_bom_from_include_file() {
            verifies!(
                r#"
      test 'should strip BOM from include file' do
        input = %(:showtitle:\ninclude::fixtures/file-with-utf8-bom.adoc[])
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_css '.paragraph', output, 0
        assert_css 'h1', output, 1
        assert_match(/<h1>人<\/h1>/, output)
      end

"#
            );

            // The UTF-8 BOM at the start of the include file is stripped, so the title
            // heading renders with its first real character.
            let html = convert_safe_with_fixtures(
                ":showtitle:\ninclude::fixtures/file-with-utf8-bom.adoc[]",
            );
            assert_css(&html, ".paragraph", 0);
            assert_css(&html, "h1", 1);
            assert!(html.contains("<h1>\u{4eba}</h1>"), "{html}");
        }

        // Non-normative: reads from the JRuby classloader (jruby-only; out of scope).
        non_normative!(
            r#"
      test 'should include content from a file on the classloader', if: jruby? do
        require fixture_path 'assets.jar'
        input = 'include::uri:classloader:/includes-in-jar/include-file.adoc[]'
        doc = document_from_string input, safe: :unsafe, standalone: false, base_dir: DIRNAME
        output = doc.convert
        assert_match(/included from a file/, output)
        assert doc.catalog[:includes]['uri:classloader:/includes-in-jar/include-file']
      end

"#
        );

        #[test]
        fn should_not_track_include_in_catalog_for_non_asciidoc_include_files() {
            verifies!(
                r#"
      test 'should not track include in catalog for non-AsciiDoc include files' do
        input = <<~'EOS'
        ----
        include::fixtures/circle.svg[]
        ----
        EOS

        doc = document_from_string input, safe: :safe, standalone: false, base_dir: DIRNAME
        assert doc.catalog[:includes].empty?
      end

"#
            );

            // A non-AsciiDoc include is pulled in but not recorded in the document's
            // include catalog.
            let opts = Options::new()
                .safe_mode(SafeMode::Safe)
                .base_dir(fixtures_base_dir());
            // Control: a normal AsciiDoc include *is* tracked (confirms the key form).
            let adoc = load_with("include::fixtures/include-file.adoc[]", &opts);
            assert!(adoc.catalog().was_included("fixtures/include-file"));
            // An .svg pulled into a listing block is not tracked.
            let svg = load_with("----\ninclude::fixtures/circle.svg[]\n----", &opts);
            assert!(!svg.catalog().was_included("fixtures/circle"));
        }

        #[test]
        fn include_directive_should_resolve_file_with_spaces_in_name() {
            verifies!(
                r#"
      test 'include directive should resolve file with spaces in name' do
        input = 'include::fixtures/include file.adoc[]'
        include_file = File.join DIRNAME, 'fixtures', 'include-file.adoc'
        include_file_with_sp = File.join DIRNAME, 'fixtures', 'include file.adoc'
        begin
          FileUtils.cp include_file, include_file_with_sp
          doc = document_from_string input, safe: :safe, standalone: false, base_dir: DIRNAME
          output = doc.convert
          assert_match(/included content/, output)
        ensure
          FileUtils.rm include_file_with_sp
        end
      end

"#
            );

            // A target whose file name contains a space resolves.
            let dir = temp_include_dir("spaces", "include file.adoc", "included content\n");
            let html = convert_safe_in(&dir, "include::include file.adoc[]");
            assert!(html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_should_resolve_file_with_sp_in_name() {
            verifies!(
                r#"
      test 'include directive should resolve file with {sp} in name' do
        input = 'include::fixtures/include{sp}file.adoc[]'
        include_file = File.join DIRNAME, 'fixtures', 'include-file.adoc'
        include_file_with_sp = File.join DIRNAME, 'fixtures', 'include file.adoc'
        begin
          FileUtils.cp include_file, include_file_with_sp
          doc = document_from_string input, safe: :safe, standalone: false, base_dir: DIRNAME
          output = doc.convert
          assert_match(/included content/, output)
        ensure
          FileUtils.rm include_file_with_sp
        end
      end

"#
            );

            // `{sp}` in the target resolves to a space, so the spaced file resolves.
            let dir = temp_include_dir("sp-attr", "include file.adoc", "included content\n");
            let html = convert_safe_in(&dir, "include::include{sp}file.adoc[]");
            assert!(html.contains("included content"), "{html}");
        }

        // Non-normative: asserts the raw reader line for a non-include; there is no
        // rendered form.
        non_normative!(
            r#"
      test 'include directive should not match if target is empty or starts or ends with space' do
        ['include::[]', 'include:: []', 'include:: not-include[]', 'include::not-include []'].each do |input|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          assert_equal input, reader.read_line
        end
      end

"#
        );

        // Non-normative: remote include target handling (#136).
        non_normative!(
            r#"
      test 'include directive should not attempt to resolve target as remote if allow-uri-read is set and URL is not on first line' do
        using_memory_logger do |logger|
          input = <<~'EOS'
          :target: not-a-file.adoc + \
          http://example.org/team.adoc

          include::{target}[]
          EOS
          doc = Asciidoctor.load input, safe: :safe, base_dir: fixturedir
          lines = doc.blocks[0].lines
          assert_equal [%(Unresolved directive in <stdin> - include::not-a-file.adoc +\nhttp://example.org/team.adoc[])], lines
          assert_message logger, :ERROR, %(<stdin>: line 4: include file not found: #{fixture_path 'not-a-file.adoc'} +\nhttp://example.org/team.adoc), Hash
        end
      end

"#
        );

        // Non-normative: asserts PreprocessorReader internals (file/dir/path/cursor);
        // no rendered form.
        non_normative!(
            r#"
      test 'include directive should resolve file relative to current include' do
        input = 'include::fixtures/parent-include.adoc[]'
        pseudo_docfile = File.join DIRNAME, 'main.adoc'
        fixtures_dir = File.join DIRNAME, 'fixtures'
        parent_include_docfile = File.join fixtures_dir, 'parent-include.adoc'
        child_include_docfile = File.join fixtures_dir, 'child-include.adoc'
        grandchild_include_docfile = File.join fixtures_dir, 'grandchild-include.adoc'

        doc = empty_safe_document base_dir: DIRNAME
        reader = Asciidoctor::PreprocessorReader.new doc, input, pseudo_docfile, normalize: true

        assert_equal pseudo_docfile, reader.file
        assert_equal DIRNAME, reader.dir
        assert_equal 'main.adoc', reader.path

        assert_equal 'first line of parent', reader.read_line

        assert_equal 'fixtures/parent-include.adoc: line 1', reader.cursor_at_prev_line.to_s
        assert_equal parent_include_docfile, reader.file
        assert_equal fixtures_dir, reader.dir
        assert_equal 'fixtures/parent-include.adoc', reader.path

        reader.skip_blank_lines

        assert_equal 'first line of child', reader.read_line

        assert_equal 'fixtures/child-include.adoc: line 1', reader.cursor_at_prev_line.to_s
        assert_equal child_include_docfile, reader.file
        assert_equal fixtures_dir, reader.dir
        assert_equal 'fixtures/child-include.adoc', reader.path

        reader.skip_blank_lines

        assert_equal 'first line of grandchild', reader.read_line

        assert_equal 'fixtures/grandchild-include.adoc: line 1', reader.cursor_at_prev_line.to_s
        assert_equal grandchild_include_docfile, reader.file
        assert_equal fixtures_dir, reader.dir
        assert_equal 'fixtures/grandchild-include.adoc', reader.path

        reader.skip_blank_lines

        assert_equal 'last line of grandchild', reader.read_line

        reader.skip_blank_lines

        assert_equal 'last line of child', reader.read_line

        reader.skip_blank_lines

        assert_equal 'last line of parent', reader.read_line

        assert_equal 'fixtures/parent-include.adoc: line 5', reader.cursor_at_prev_line.to_s
        assert_equal parent_include_docfile, reader.file
        assert_equal fixtures_dir, reader.dir
        assert_equal 'fixtures/parent-include.adoc', reader.path
      end

"#
        );

        #[test]
        fn include_directive_should_process_lines_when_file_extension_of_target_is_asciidoc() {
            verifies!(
                r#"
      test 'include directive should process lines when file extension of target is .asciidoc' do
        input = 'include::fixtures/include-alt-extension.asciidoc[]'
        doc = document_from_string input, safe: :safe, base_dir: DIRNAME
        assert_equal 3, doc.blocks.size
        assert_equal ['first line'], doc.blocks[0].lines
        assert_equal ['Asciidoctor!'], doc.blocks[1].lines
        assert_equal ['last line'], doc.blocks[2].lines
      end

"#
            );

            let html =
                convert_safe_with_fixtures("include::fixtures/include-alt-extension.asciidoc[]");
            assert!(html.contains("first line"), "{html}");
            assert!(html.contains("Asciidoctor!"), "{html}");
            assert!(html.contains("last line"), "{html}");
        }

        #[test]
        fn should_only_strip_trailing_newlines_not_trailing_whitespace_if_include_file_is_not_asciidoc(
        ) {
            verifies!(
                r#"
      test 'should only strip trailing newlines, not trailing whitespace, if include file is not AsciiDoc' do
        input = <<~'EOS'
        ....
        include::fixtures/data.tsv[]
        ....
        EOS

        doc = document_from_string input, safe: :safe, base_dir: DIRNAME
        assert_equal 1, doc.blocks.size
        assert doc.blocks[0].lines[2].end_with? ?\t
      end

"#
            );

            // Only trailing newlines are stripped from a non-AsciiDoc include, not
            // trailing whitespace: the tab ending the third data line survives.
            let html = convert_safe_with_fixtures("....\ninclude::fixtures/data.tsv[]\n....");
            assert!(html.contains("1\t2\t\n"), "{html:?}");
        }

        // Non-normative: reads a non-UTF-8 include file; this crate is UTF-8 only
        // (#138).
        non_normative!(
            r#"
      test 'should fail to read include file if not UTF-8 encoded and encoding is not specified' do
        input = <<~'EOS'
        ....
        include::fixtures/iso-8859-1.txt[]
        ....
        EOS

        assert_raises StandardError, 'invalid byte sequence in UTF-8' do
          doc = document_from_string input, safe: :safe, base_dir: DIRNAME
          assert_equal 1, doc.blocks.size
          refute_equal ['Où est l\'hôpital ?'], doc.blocks[0].lines
          doc.convert
        end
      end

"#
        );

        #[test]
        fn should_ignore_encoding_attribute_if_value_is_not_a_valid_encoding() {
            verifies!(
                r#"
      test 'should ignore encoding attribute if value is not a valid encoding' do
        input = <<~'EOS'
        ....
        include::fixtures/encoding.adoc[tag=romé,encoding=iso-1000-1]
        ....
        EOS

        doc = document_from_string input, safe: :safe, base_dir: DIRNAME
        assert_equal 1, doc.blocks.size
        assert_equal doc.blocks[0].lines[0].encoding, Encoding::UTF_8
        assert_equal ['Gregory Romé has written an AsciiDoc plugin for the Redmine project management application.'], doc.blocks[0].lines
      end

"#
            );

            // The file is UTF-8; an invalid `encoding` value is ignored and it reads
            // normally (the non-ASCII `romé` tag name resolves too).
            let html = convert_safe_with_fixtures(
                "....\ninclude::fixtures/encoding.adoc[tag=rom\u{e9},encoding=iso-1000-1]\n....",
            );
            assert!(
                html.contains("Gregory Rom\u{e9} has written an AsciiDoc plugin"),
                "{html}"
            );
        }

        // Non-normative: reads a non-UTF-8 include file per the encoding attribute;
        // this crate is UTF-8 only (#138).
        non_normative!(
            r#"
      test 'should use encoding specified by encoding attribute when reading include file' do
        input = <<~'EOS'
        ....
        include::fixtures/iso-8859-1.txt[encoding=iso-8859-1]
        ....
        EOS

        doc = document_from_string input, safe: :safe, base_dir: DIRNAME
        assert_equal 1, doc.blocks.size
        assert_equal doc.blocks[0].lines[0].encoding, Encoding::UTF_8
        assert_equal ['Où est l\'hôpital ?'], doc.blocks[0].lines
      end

"#
        );

        #[test]
        fn unresolved_target_referenced_by_include_directive_is_skipped_when_optional_option_is_set(
        ) {
            verifies!(
                r#"
      test 'unresolved target referenced by include directive is skipped when optional option is set' do
        input = <<~'EOS'
        include::fixtures/{no-such-file}[opts=optional]

        trailing content
        EOS

        begin
          using_memory_logger do |logger|
            doc = document_from_string input, safe: :safe, base_dir: DIRNAME
            assert_equal 1, doc.blocks.size
            assert_equal ['trailing content'], doc.blocks[0].lines
            assert_message logger, :INFO, '~<stdin>: line 1: optional include dropped because include file not found', Hash
          end
        rescue
          flunk 'include directive should not raise exception on unresolved target'
        end
      end

"#
            );

            // An optional include whose target is unresolved (missing attribute) is
            // dropped silently; only the trailing content remains.
            let html = convert_safe_with_fixtures(
                "include::fixtures/{no-such-file}[opts=optional]\n\ntrailing content",
            );
            assert!(html.contains("trailing content"), "{html}");
            assert!(!html.contains("Unresolved directive"), "{html}");
        }

        #[test]
        fn should_skip_include_directive_that_references_missing_file_if_optional_option_is_set() {
            verifies!(
                r#"
      test 'should skip include directive that references missing file if optional option is set' do
        input = <<~'EOS'
        include::fixtures/no-such-file.adoc[opts=optional]

        trailing content
        EOS

        begin
          using_memory_logger do |logger|
            doc = document_from_string input, safe: :safe, base_dir: DIRNAME
            assert_equal 1, doc.blocks.size
            assert_equal ['trailing content'], doc.blocks[0].lines
            assert_message logger, :INFO, '~<stdin>: line 1: optional include dropped because include file not found', Hash
          end
        rescue
          flunk 'include directive should not raise exception on missing file'
        end
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "include::fixtures/no-such-file.adoc[opts=optional]\n\ntrailing content",
            );
            assert!(html.contains("trailing content"), "{html}");
            assert!(!html.contains("Unresolved directive"), "{html}");
        }

        #[test]
        fn should_replace_include_directive_that_references_missing_file_with_message() {
            verifies!(
                r#"
      test 'should replace include directive that references missing file with message' do
        input = <<~'EOS'
        include::fixtures/no-such-file.adoc[]

        trailing content
        EOS

        begin
          using_memory_logger do |logger|
            doc = document_from_string input, safe: :safe, base_dir: DIRNAME
            assert_equal 2, doc.blocks.size
            assert_equal ['Unresolved directive in <stdin> - include::fixtures/no-such-file.adoc[]'], doc.blocks[0].lines
            assert_equal ['trailing content'], doc.blocks[1].lines
            assert_message logger, :ERROR, '~<stdin>: line 1: include file not found', Hash
          end
        rescue
          flunk 'include directive should not raise exception on missing file'
        end
      end

"#
            );

            // A required include of a missing file is replaced with an "Unresolved
            // directive" message (this crate names the origin "(root file)" rather than
            // Asciidoctor's "<stdin>"), and an include-file-not-found warning is raised.
            let src = "include::fixtures/no-such-file.adoc[]\n\ntrailing content";
            let html = convert_safe_with_fixtures(src);
            assert!(html.contains("Unresolved directive"), "{html}");
            assert!(
                html.contains("include::fixtures/no-such-file.adoc[]"),
                "{html}"
            );
            assert!(html.contains("trailing content"), "{html}");
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeFileNotFound(_))),
                "{warnings:?}"
            );
        }

        // Non-normative: an unreadable include file is not distinguished from a missing
        // one (#146).
        non_normative!(
            r#"
      test 'should replace include directive that references unreadable file with message', unless: (windows? || Process.euid == 0) do
        include_file = File.join DIRNAME, 'fixtures', 'chapter-a.adoc'
        old_mode = (File.stat include_file).mode
        FileUtils.chmod 0o000, include_file
        input = <<~'EOS'
        include::fixtures/chapter-a.adoc[]

        trailing content
        EOS

        begin
          using_memory_logger do |logger|
            doc = document_from_string input, safe: :safe, base_dir: DIRNAME
            assert_equal 2, doc.blocks.size
            assert_equal ['Unresolved directive in <stdin> - include::fixtures/chapter-a.adoc[]'], doc.blocks[0].lines
            assert_equal ['trailing content'], doc.blocks[1].lines
            assert_message logger, :ERROR, '~<stdin>: line 1: include file not readable', Hash
          end
        rescue
          flunk 'include directive should not raise exception on missing file'
        ensure
          FileUtils.chmod old_mode, include_file
        end
      end

"#
        );

        non_normative!(
            r#"
      # IMPORTANT this test needs to be run on Windows to verify proper behavior in Windows
"#
        );

        // Non-normative: an absolute include path is not resolved (#132).
        non_normative!(
            r#"
      test 'can resolve include directive with absolute path' do
        include_path = ::File.join DIRNAME, 'fixtures', 'chapter-a.adoc'
        input = %(include::#{include_path}[])
        result = document_from_string input, safe: :safe
        assert_equal 'Chapter A', result.doctitle

        result = document_from_string input, safe: :unsafe, base_dir: ::Dir.tmpdir
        assert_equal 'Chapter A', result.doctitle
      end

"#
        );

        // Non-normative: fetches a remote (URI) include; remote fetch is a non-goal
        // (remote-fetch-not-planned).
        non_normative!(
            r#"
      test 'include directive can retrieve data from uri' do
        url = %(http://#{resolve_localhost}:9876/name/asciidoctor)
        input = <<~EOS
        ....
        include::#{url}[]
        ....
        EOS
        expect = /\{"name": "asciidoctor"\}/
        output = using_test_webserver do
          convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
        end

        refute_nil output
        assert_match(expect, output)
      end

"#
        );

        // Non-normative: a nested include from a subdirectory leaves the inner include
        // unresolved (#131).
        non_normative!(
            r#"
      test 'nested include directives are resolved relative to current file' do
        input = <<~'EOS'
        ....
        include::fixtures/outer-include.adoc[]
        ....
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~'EOS'.chop
        first line of outer

        first line of middle

        first line of inner

        last line of inner

        last line of middle

        last line of outer
        EOS
        assert_includes output, expected
      end

"#
        );

        // Non-normative: fetches a remote (URI) include; remote fetch is a non-goal
        // (remote-fetch-not-planned).
        non_normative!(
            r#"
      test 'nested remote include directive is resolved relative to uri of current file' do
        url = %(http://#{resolve_localhost}:9876/fixtures/outer-include.adoc)
        input = <<~EOS
        ....
        include::#{url}[]
        ....
        EOS
        output = using_test_webserver do
          convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
        end

        expected = <<~'EOS'.chop
        first line of outer

        first line of middle

        first line of inner

        last line of inner

        last line of middle

        last line of outer
        EOS
        assert_includes output, expected
      end

"#
        );

        // Non-normative: fetches a remote (URI) include; remote fetch is a non-goal
        // (remote-fetch-not-planned).
        non_normative!(
            r#"
      test 'nested remote include directive that cannot be resolved does not crash processor' do
        include_url = %(http://#{resolve_localhost}:9876/fixtures/file-with-missing-include.adoc)
        nested_include_url = 'no-such-file.adoc'
        input = <<~EOS
        ....
        include::#{include_url}[]
        ....
        EOS
        begin
          using_memory_logger do |logger|
            result = using_test_webserver do
              convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
            end
            assert_includes result, %(Unresolved directive in #{include_url} - include::#{nested_include_url}[])
            assert_message logger, :ERROR, %(#{include_url}: line 1: include uri not readable: http://#{resolve_localhost}:9876/fixtures/#{nested_include_url}), Hash
          end
        rescue
          flunk 'include directive should not raise exception on missing file'
        end
      end

"#
        );

        // Non-normative: fetches a remote (URI) include; remote fetch is a non-goal
        // (remote-fetch-not-planned).
        non_normative!(
            r#"
      test 'should support tag filtering for remote includes' do
        url = %(http://#{resolve_localhost}:9876/fixtures/tagged-class.rb)
        input = <<~EOS
        [source,ruby]
        ----
        include::#{url}[tag=init,indent=0]
        ----
        EOS
        output = using_test_webserver do
          convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
        end

        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        <code class="language-ruby" data-lang="ruby">def initialize breed
          @breed = breed
        end</code>
        EOS
        assert_includes output, expected
      end

"#
        );

        // Non-normative: fetches a remote (URI) include; remote fetch is a non-goal
        // (remote-fetch-not-planned).
        non_normative!(
            r#"
      test 'should not crash if include directive references inaccessible uri' do
        url = %(http://#{resolve_localhost}:9876/no_such_file)
        input = <<~EOS
        ....
        include::#{url}[]
        ....
        EOS

        begin
          using_memory_logger do |logger|
            output = using_test_webserver do
              convert_string_to_embedded input, safe: :safe, attributes: { 'allow-uri-read' => '' }
            end
            refute_nil output
            assert_match(/Unresolved directive/, output)
            assert_message logger, :ERROR, %(<stdin>: line 2: include uri not readable: #{url}), Hash
          end
        rescue
          flunk 'include directive should not raise exception on inaccessible uri'
        end
      end

"#
        );

        #[test]
        fn include_directive_supports_selecting_lines_by_line_number() {
            verifies!(
                r#"
      test 'include directive supports selecting lines by line number' do
        input = 'include::fixtures/include-file.adoc[lines=1;3..4;6..-1]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/first line/, output)
        refute_match(/second line/, output)
        assert_match(/third line/, output)
        assert_match(/fourth line/, output)
        refute_match(/fifth line/, output)
        assert_match(/sixth line/, output)
        assert_match(/seventh line/, output)
        assert_match(/eighth line/, output)
        assert_match(/last line of included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "include::fixtures/include-file.adoc[lines=1;3..4;6..-1]",
            );
            assert!(html.contains("first line"), "{html}");
            assert!(!html.contains("second line"), "{html}");
            assert!(html.contains("third line"), "{html}");
            assert!(html.contains("fourth line"), "{html}");
            assert!(!html.contains("fifth line"), "{html}");
            assert!(html.contains("sixth line"), "{html}");
            assert!(html.contains("seventh line"), "{html}");
            assert!(html.contains("eighth line"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_supports_line_ranges_separated_by_commas_in_quoted_attribute_value() {
            verifies!(
                r#"
      test 'include directive supports line ranges separated by commas in quoted attribute value' do
        input = 'include::fixtures/include-file.adoc[lines="1,3..4,6..-1"]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/first line/, output)
        refute_match(/second line/, output)
        assert_match(/third line/, output)
        assert_match(/fourth line/, output)
        refute_match(/fifth line/, output)
        assert_match(/sixth line/, output)
        assert_match(/seventh line/, output)
        assert_match(/eighth line/, output)
        assert_match(/last line of included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                r#"include::fixtures/include-file.adoc[lines="1,3..4,6..-1"]"#,
            );
            assert!(html.contains("first line"), "{html}");
            assert!(!html.contains("second line"), "{html}");
            assert!(html.contains("third line"), "{html}");
            assert!(html.contains("fourth line"), "{html}");
            assert!(!html.contains("fifth line"), "{html}");
            assert!(html.contains("sixth line"), "{html}");
            assert!(html.contains("seventh line"), "{html}");
            assert!(html.contains("eighth line"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_ignores_spaces_between_line_ranges_in_quoted_attribute_value() {
            verifies!(
                r#"
      test 'include directive ignores spaces between line ranges in quoted attribute value' do
        input = 'include::fixtures/include-file.adoc[lines="1, 3..4 , 6 .. -1"]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/first line/, output)
        refute_match(/second line/, output)
        assert_match(/third line/, output)
        assert_match(/fourth line/, output)
        refute_match(/fifth line/, output)
        assert_match(/sixth line/, output)
        assert_match(/seventh line/, output)
        assert_match(/eighth line/, output)
        assert_match(/last line of included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                r#"include::fixtures/include-file.adoc[lines="1, 3..4 , 6 .. -1"]"#,
            );
            assert!(html.contains("first line"), "{html}");
            assert!(!html.contains("second line"), "{html}");
            assert!(html.contains("third line"), "{html}");
            assert!(html.contains("fourth line"), "{html}");
            assert!(!html.contains("fifth line"), "{html}");
            assert!(html.contains("sixth line"), "{html}");
            assert!(html.contains("seventh line"), "{html}");
            assert!(html.contains("eighth line"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_supports_implicit_endless_range() {
            verifies!(
                r#"
      test 'include directive supports implicit endless range' do
        input = 'include::fixtures/include-file.adoc[lines=6..]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        refute_match(/first line/, output)
        refute_match(/second line/, output)
        refute_match(/third line/, output)
        refute_match(/fourth line/, output)
        refute_match(/fifth line/, output)
        assert_match(/sixth line/, output)
        assert_match(/seventh line/, output)
        assert_match(/eighth line/, output)
        assert_match(/last line of included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures("include::fixtures/include-file.adoc[lines=6..]");
            assert!(!html.contains("first line"), "{html}");
            assert!(!html.contains("fifth line"), "{html}");
            assert!(html.contains("sixth line"), "{html}");
            assert!(html.contains("seventh line"), "{html}");
            assert!(html.contains("eighth line"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_ignores_lines_attribute_if_empty() {
            verifies!(
                r#"
      test 'include directive ignores lines attribute if empty' do
        input = <<~'EOS'
        ++++
        include::fixtures/include-file.adoc[lines=]
        ++++
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_includes output, 'first line of included content'
        assert_includes output, 'last line of included content'
      end

"#
            );

            // An empty `lines` value is ignored, so the whole file is included;
            // the `++++` passthrough block emits it raw.
            let html = convert_safe_with_fixtures(
                "++++\ninclude::fixtures/include-file.adoc[lines=]\n++++",
            );
            assert!(html.contains("first line of included content"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_ignores_lines_attribute_with_invalid_range() {
            verifies!(
                r#"
      test 'include directive ignores lines attribute with invalid range' do
        input = <<~'EOS'
        ++++
        include::fixtures/include-file.adoc[lines=10..5]
        ++++
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_includes output, 'first line of included content'
        assert_includes output, 'last line of included content'
      end

"#
            );

            // An invalid range (start after end) is ignored, so the whole file is
            // included; the `++++` passthrough block emits it raw.
            let html = convert_safe_with_fixtures(
                "++++\ninclude::fixtures/include-file.adoc[lines=10..5]\n++++",
            );
            assert!(html.contains("first line of included content"), "{html}");
            assert!(html.contains("last line of included content"), "{html}");
        }

        #[test]
        fn include_directive_supports_selecting_lines_by_tag() {
            verifies!(
                r#"
      test 'include directive supports selecting lines by tag' do
        input = 'include::fixtures/include-file.adoc[tag=snippetA]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/snippetA content/, output)
        refute_match(/snippetB content/, output)
        refute_match(/non-tagged content/, output)
        refute_match(/included content/, output)
      end

"#
            );

            let html =
                convert_safe_with_fixtures("include::fixtures/include-file.adoc[tag=snippetA]");
            assert!(html.contains("snippetA content"), "{html}");
            assert!(!html.contains("snippetB content"), "{html}");
            assert!(!html.contains("non-tagged content"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_supports_selecting_lines_by_tags() {
            verifies!(
                r#"
      test 'include directive supports selecting lines by tags' do
        input = 'include::fixtures/include-file.adoc[tags=snippetA;snippetB]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/snippetA content/, output)
        assert_match(/snippetB content/, output)
        refute_match(/non-tagged content/, output)
        refute_match(/included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "include::fixtures/include-file.adoc[tags=snippetA;snippetB]",
            );
            assert!(html.contains("snippetA content"), "{html}");
            assert!(html.contains("snippetB content"), "{html}");
            assert!(!html.contains("non-tagged content"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_supports_selecting_lines_by_tag_in_language_that_uses_circumfix_comments(
        ) {
            verifies!(
                r#"
      test 'include directive supports selecting lines by tag in language that uses circumfix comments' do
        {
          'include-file.xml' => '<snippet>content</snippet>',
          'include-file.ml' => 'let s = SS.empty;;',
          'include-file.jsx' => '<p>Welcome to the club.</p>',
        }.each do |filename, expect|
          input = <<~EOS
          [source,xml]
          ----
          include::fixtures/#{filename}[tag=snippet,indent=0]
          ----
          EOS

          doc = document_from_string input, safe: :safe, base_dir: DIRNAME
          assert_equal expect, doc.blocks[0].source
        end
      end

"#
            );

            for (filename, expect) in [
                ("include-file.xml", "&lt;snippet&gt;content&lt;/snippet&gt;"),
                ("include-file.ml", "let s = SS.empty;;"),
                (
                    "include-file.jsx",
                    "&lt;p&gt;Welcome to the club.&lt;/p&gt;",
                ),
            ] {
                let src = format!(
                    "[source,xml]\n----\ninclude::fixtures/{filename}[tag=snippet,indent=0]\n----"
                );
                let html = convert_safe_with_fixtures(&src);
                assert!(html.contains(expect), "{filename}: {html}");
            }
        }

        #[test]
        fn include_directive_supports_selecting_lines_by_tag_in_file_that_has_crlf_line_endings() {
            verifies!(
                r#"
      test 'include directive supports selecting lines by tag in file that has CRLF line endings' do
        begin
          tmp_include = Tempfile.new %w(include- .adoc)
          tmp_include_dir, tmp_include_path = File.split tmp_include.path
          tmp_include.write %(do not include\r\ntag::include-me[]\r\nincluded line\r\nend::include-me[]\r\ndo not include\r\n)
          tmp_include.close
          input = %(include::#{tmp_include_path}[tag=include-me])
          output = convert_string_to_embedded input, safe: :safe, base_dir: tmp_include_dir
          assert_includes output, 'included line'
          refute_includes output, 'do not include'
        ensure
          tmp_include.close!
        end
      end

"#
            );

            // Tag selection works against an include file with CRLF line endings.
            let dir = temp_include_dir(
                "crlf",
                "include-crlf.adoc",
                "do not include\r\ntag::include-me[]\r\nincluded line\r\nend::include-me[]\r\ndo not include\r\n",
            );
            let html = convert_safe_in(&dir, "include::include-crlf.adoc[tag=include-me]");
            assert!(html.contains("included line"), "{html}");
            assert!(!html.contains("do not include"), "{html}");
        }

        #[test]
        fn include_directive_finds_closing_tag_on_last_line_of_file_without_a_trailing_newline() {
            verifies!(
                r#"
      test 'include directive finds closing tag on last line of file without a trailing newline' do
        begin
          tmp_include = Tempfile.new %w(include- .adoc)
          tmp_include_dir, tmp_include_path = File.split tmp_include.path
          tmp_include.write %(line not included\ntag::include-me[]\nline included\nend::include-me[])
          tmp_include.close
          input = %(include::#{tmp_include_path}[tag=include-me])
          using_memory_logger do |logger|
            output = convert_string_to_embedded input, safe: :safe, base_dir: tmp_include_dir
            assert_empty logger.messages
            assert_includes output, 'line included'
            refute_includes output, 'line not included'
          end
        ensure
          tmp_include.close!
        end
      end

"#
            );

            // The closing tag directive on the last line of a file with no trailing
            // newline is still recognized.
            let dir = temp_include_dir(
                "no-trailing-nl",
                "include-no-nl.adoc",
                "line not included\ntag::include-me[]\nline included\nend::include-me[]",
            );
            let html = convert_safe_in(&dir, "include::include-no-nl.adoc[tag=include-me]");
            assert!(html.contains("line included"), "{html}");
            assert!(!html.contains("line not included"), "{html}");
        }

        #[test]
        fn include_directive_does_not_select_lines_containing_tag_directives_within_selected_tag_region(
        ) {
            verifies!(
                r#"
      test 'include directive does not select lines containing tag directives within selected tag region' do
        input = <<~'EOS'
        ++++
        include::fixtures/include-file.adoc[tags=snippet]
        ++++
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~'EOS'.chop
        snippetA content

        non-tagged content

        snippetB content
        EOS
        assert_equal expected, output
      end

"#
            );

            // The `++++` passthrough block emits the tag-selected include content
            // raw, so selecting the outer `snippet` tag yields the inner content
            // without the nested tag directive lines. (This crate appends a
            // trailing newline to embedded output that Asciidoctor omits, so it
            // is trimmed for the comparison.)
            let output = convert_safe_with_fixtures(
                "++++\ninclude::fixtures/include-file.adoc[tags=snippet]\n++++",
            );
            assert_eq!(
                output.trim_end_matches('\n'),
                "snippetA content\n\nnon-tagged content\n\nsnippetB content",
            );
        }

        #[test]
        fn include_directive_skips_lines_inside_tag_which_is_negated() {
            verifies!(
                r#"
      test 'include directive skips lines inside tag which is negated' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class-enclosed.rb[tags=all;!bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class-enclosed.rb[tags=all;!bark]\n----",
            );
            assert_listing_selection(
                &html,
                "class Dog\n  def initialize breed\n    @breed = breed\n  end\nend",
            );
        }

        #[test]
        fn include_directive_selects_all_lines_without_a_tag_directive_when_value_is_double_asterisk(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines without a tag directive when value is double asterisk' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**]\n----",
            );
            assert_listing_selection(&html, "class Dog\n  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_selects_all_lines_except_lines_inside_tag_which_is_negated_when_value_starts_with_double_asterisk(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines except lines inside tag which is negated when value starts with double asterisk' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**;!bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**;!bark]\n----",
            );
            assert_listing_selection(
                &html,
                "class Dog\n  def initialize breed\n    @breed = breed\n  end\nend",
            );
        }

        #[test]
        fn include_directive_selects_all_lines_including_lines_inside_nested_tags_except_lines_inside_tag_which_is_negated_when_value_starts_with_double_asterisk(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines, including lines inside nested tags, except lines inside tag which is negated when value starts with double asterisk' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**;!init]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**;!init]\n----",
            );
            assert_listing_selection(&html, "class Dog\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_selects_all_lines_outside_of_tags_when_value_is_double_asterisk_followed_by_negated_wildcard(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines outside of tags when value is double asterisk followed by negated wildcard' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**;!*]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~'EOS'.chop
        class Dog
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**;!*]\n----",
            );
            assert_listing_selection(&html, "class Dog\nend");
        }

        #[test]
        fn include_directive_skips_all_tagged_regions_when_value_of_tags_attribute_is_negated_wildcard(
        ) {
            verifies!(
                r#"
      test 'include directive skips all tagged regions when value of tags attribute is negated wildcard' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!*]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = %(class Dog\nend)
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!*]\n----",
            );
            assert_listing_selection(&html, "class Dog\nend");
        }

        non_normative!(
            r#"
      # FIXME this is a weird one since we'd expect it to only select the specified tags; but it's always been this way
"#
        );

        #[test]
        fn include_directive_selects_all_lines_except_for_lines_containing_tag_directive_if_value_is_double_asterisk_followed_by_nested_tag_names(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines except for lines containing tag directive if value is double asterisk followed by nested tag names' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**;bark-beagle;bark-all]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**;bark-beagle;bark-all]\n----",
            );
            assert_listing_selection(&html, "class Dog\n  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        non_normative!(
            r#"
      # FIXME this is a weird one since we'd expect it to only select the specified tags; but it's always been this way
"#
        );

        #[test]
        fn include_directive_selects_all_lines_except_for_lines_containing_tag_directive_when_value_is_double_asterisk_followed_by_outer_tag_name(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines except for lines containing tag directive when value is double asterisk followed by outer tag name' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=**;bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=**;bark]\n----",
            );
            assert_listing_selection(&html, "class Dog\n  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_selects_all_lines_inside_unspecified_tags_when_value_is_negated_double_asterisk_followed_by_negated_tags(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines inside unspecified tags when value is negated double asterisk followed by negated tags' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!**;!init]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~EOS.chop
        \x20 def bark
        \x20   if @breed == 'beagle'
        \x20     'woof woof woof woof woof'
        \x20   else
        \x20     'woof woof'
        \x20   end
        \x20 end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!**;!init]\n----",
            );
            assert_listing_selection(&html, "  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end");
        }

        #[test]
        fn include_directive_selects_all_lines_except_tag_which_is_negated_when_value_only_contains_negated_tag(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines except tag which is negated when value only contains negated tag' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tag=!bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tag=!bark]\n----",
            );
            assert_listing_selection(
                &html,
                "class Dog\n  def initialize breed\n    @breed = breed\n  end\nend",
            );
        }

        #[test]
        fn include_directive_selects_all_lines_except_tags_which_are_negated_when_value_only_contains_negated_tags(
        ) {
            verifies!(
                r#"
      test 'include directive selects all lines except tags which are negated when value only contains negated tags' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!bark;!init]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~'EOS'.chop
        class Dog
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!bark;!init]\n----",
            );
            assert_listing_selection(&html, "class Dog\nend");
        }

        #[test]
        fn should_recognize_tag_wildcard_if_not_at_start_of_tags_list() {
            verifies!(
                r#"
      test 'should recognize tag wildcard if not at start of tags list' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=init;**;*;!bark-other]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=init;**;*;!bark-other]\n----",
            );
            assert_listing_selection(&html, "class Dog\n  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_selects_lines_between_tags_when_value_of_tags_attribute_is_wildcard() {
            verifies!(
                r#"
      test 'include directive selects lines between tags when value of tags attribute is wildcard' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=*]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~EOS.chop
        \x20 def initialize breed
        \x20   @breed = breed
        \x20 end

        \x20 def bark
        \x20   if @breed == 'beagle'
        \x20     'woof woof woof woof woof'
        \x20   else
        \x20     'woof woof'
        \x20   end
        \x20 end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html =
                convert_safe_with_fixtures("----\ninclude::fixtures/tagged-class.rb[tags=*]\n----");
            assert_listing_selection(&html, "  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end");
        }

        #[test]
        fn include_directive_selects_lines_inside_tags_when_value_of_tags_attribute_is_wildcard_and_tag_surrounds_content(
        ) {
            verifies!(
                r#"
      test 'include directive selects lines inside tags when value of tags attribute is wildcard and tag surrounds content' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class-enclosed.rb[tags=*]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class-enclosed.rb[tags=*]\n----",
            );
            assert_listing_selection(&html, "class Dog\n  def initialize breed\n    @breed = breed\n  end\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_selects_lines_inside_all_tags_except_tag_which_is_negated_when_value_of_tags_attribute_is_wildcard_followed_by_negated_tag(
        ) {
            verifies!(
                r#"
      test 'include directive selects lines inside all tags except tag which is negated when value of tags attribute is wildcard followed by negated tag' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class-enclosed.rb[tags=*;!init]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class-enclosed.rb[tags=*;!init]\n----",
            );
            assert_listing_selection(&html, "class Dog\n\n  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end\nend");
        }

        #[test]
        fn include_directive_skips_all_tagged_regions_except_ones_re_enabled_when_value_of_tags_attribute_is_negated_wildcard_followed_by_tag_name(
        ) {
            verifies!(
                r#"
      test 'include directive skips all tagged regions except ones re-enabled when value of tags attribute is negated wildcard followed by tag name' do
        ['!*;init', '**;!*;init'].each do |pattern|
          input = <<~EOS
          ----
          include::fixtures/tagged-class.rb[tags=#{pattern}]
          ----
          EOS

          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
          expected = <<~EOS.chop
          class Dog
            def initialize breed
              @breed = breed
            end
          end
          EOS
          assert_includes output, %(<pre>#{expected}</pre>)
        end
      end

"#
            );

            for pattern in ["!*;init", "**;!*;init"] {
                let src = format!("----\ninclude::fixtures/tagged-class.rb[tags={pattern}]\n----");
                let html = convert_safe_with_fixtures(&src);
                assert_listing_selection(
                    &html,
                    "class Dog\n  def initialize breed\n    @breed = breed\n  end\nend",
                );
            }
        }

        #[test]
        fn include_directive_includes_regions_outside_tags_and_inside_specified_tags_when_value_begins_with_negated_wildcard(
        ) {
            verifies!(
                r#"
      test 'include directive includes regions outside tags and inside specified tags when value begins with negated wildcard' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!*;bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        class Dog

          def bark
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!*;bark]\n----",
            );
            assert_listing_selection(&html, "class Dog\n\n  def bark\n  end\nend");
        }

        #[test]
        fn include_directive_includes_lines_inside_tag_except_for_lines_inside_nested_tags_when_tag_is_followed_by_negated_wildcard(
        ) {
            verifies!(
                r#"
      test 'include directive includes lines inside tag except for lines inside nested tags when tag is followed by negated wildcard' do
        ['bark;!*', '!**;bark;!*', '!**;!*;bark'].each do |pattern|
          input = <<~EOS
          ----
          include::fixtures/tagged-class.rb[tags=#{pattern}]
          ----
          EOS

          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          expected = <<~EOS.chop
          \x20 def bark
          \x20 end
          EOS
          assert_includes output, %(<pre>#{expected}</pre>)
        end
      end

"#
            );

            for pattern in ["bark;!*", "!**;bark;!*", "!**;!*;bark"] {
                let src = format!("----\ninclude::fixtures/tagged-class.rb[tags={pattern}]\n----");
                let html = convert_safe_with_fixtures(&src);
                assert_listing_selection(&html, "  def bark\n  end");
            }
        }

        #[test]
        fn include_directive_selects_lines_inside_tag_except_for_lines_inside_nested_tags_when_tag_is_preceded_by_negated_double_asterisk_and_negated_wildcard(
        ) {
            verifies!(
                r#"
      test 'include directive selects lines inside tag except for lines inside nested tags when tag is preceded by negated double asterisk and negated wildcard' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!**;!*;bark]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~EOS.chop
        \x20 def bark
        \x20 end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!**;!*;bark]\n----",
            );
            assert_listing_selection(&html, "  def bark\n  end");
        }

        #[test]
        fn include_directive_does_not_select_lines_inside_tag_that_has_been_included_then_excluded()
        {
            verifies!(
                r#"
      test 'include directive does not select lines inside tag that has been included then excluded' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class.rb[tags=!*;init;!init]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        expected = <<~'EOS'.chop
        class Dog
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "----\ninclude::fixtures/tagged-class.rb[tags=!*;init;!init]\n----",
            );
            assert_listing_selection(&html, "class Dog\nend");
        }

        #[test]
        fn include_directive_only_selects_lines_inside_specified_tag_even_if_proceeded_by_negated_double_asterisk(
        ) {
            verifies!(
                r#"
      test 'include directive only selects lines inside specified tag, even if proceeded by negated double asterisk' do
        ['bark', '!**;bark'].each do |pattern|
          input = <<~EOS
          ----
          include::fixtures/tagged-class.rb[tags=#{pattern}]
          ----
          EOS

          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          expected = <<~EOS.chop
          \x20 def bark
          \x20   if @breed == 'beagle'
          \x20     'woof woof woof woof woof'
          \x20   else
          \x20     'woof woof'
          \x20   end
          \x20 end
          EOS
          assert_includes output, %(<pre>#{expected}</pre>)
        end
      end

"#
            );

            for pattern in ["bark", "!**;bark"] {
                let src = format!("----\ninclude::fixtures/tagged-class.rb[tags={pattern}]\n----");
                let html = convert_safe_with_fixtures(&src);
                assert_listing_selection(&html, "  def bark\n    if @breed == 'beagle'\n      'woof woof woof woof woof'\n    else\n      'woof woof'\n    end\n  end");
            }
        }

        #[test]
        fn include_directive_selects_lines_inside_specified_tag_and_ignores_lines_inside_a_negated_tag(
        ) {
            verifies!(
                r#"
      test 'include directive selects lines inside specified tag and ignores lines inside a negated tag' do
        input = <<~'EOS'
        [indent=0]
        ----
        include::fixtures/tagged-class.rb[tags=bark;!bark-other]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        # NOTE cannot use single-quoted heredoc because of https://github.com/jruby/jruby/issues/4260
        expected = <<~EOS.chop
        def bark
          if @breed == 'beagle'
            'woof woof woof woof woof'
          end
        end
        EOS
        assert_includes output, %(<pre>#{expected}</pre>)
      end

"#
            );

            // The `[indent=0]` attribute removes the two-space block indent from
            // the tag-selected `bark` region.
            let html = convert_safe_with_fixtures(
                "[indent=0]\n----\ninclude::fixtures/tagged-class.rb[tags=bark;!bark-other]\n----",
            );
            assert_listing_selection(
                &html,
                "def bark\n  if @breed == 'beagle'\n    'woof woof woof woof woof'\n  end\nend",
            );
        }

        #[test]
        fn should_warn_if_specified_tag_is_not_found_in_include_file() {
            verifies!(
                r#"
      test 'should warn if specified tag is not found in include file' do
        input = 'include::fixtures/include-file.adoc[tag=no-such-tag]'
        using_memory_logger do |logger|
          convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_message logger, :WARN, %(~<stdin>: line 1: tag 'no-such-tag' not found in include file), Hash
        end
      end

"#
            );

            let warnings = fixture_warnings("include::fixtures/include-file.adoc[tag=no-such-tag]");
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeTagNotFound(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_not_warn_if_specified_negated_tag_is_not_found_in_include_file() {
            verifies!(
                r#"
      test 'should not warn if specified negated tag is not found in include file' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class-enclosed.rb[tag=!no-such-tag]
        ----
        EOS
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        using_memory_logger do |logger|
          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_includes output, %(<pre>#{expected}</pre>)
          assert_empty logger.messages
        end
      end

"#
            );

            let src = "----\ninclude::fixtures/tagged-class-enclosed.rb[tag=!no-such-tag]\n----";
            let html = convert_safe_with_fixtures(src);
            assert!(html.contains("class Dog"), "{html}");
            assert!(
                fixture_warnings(src).is_empty(),
                "{:?}",
                fixture_warnings(src)
            );
        }

        #[test]
        fn should_warn_if_specified_tags_are_not_found_in_include_file() {
            verifies!(
                r#"
      test 'should warn if specified tags are not found in include file' do
        input = <<~'EOS'
        ++++
        include::fixtures/include-file.adoc[tags=no-such-tag-b;no-such-tag-a]
        ++++
        EOS

        using_memory_logger do |logger|
          convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          expected_tags = 'no-such-tag-b, no-such-tag-a'
          assert_message logger, :WARN, %(~<stdin>: line 2: tags '#{expected_tags}' not found in include file), Hash
        end
      end

"#
            );

            let warnings = fixture_warnings(
                "++++\ninclude::fixtures/include-file.adoc[tags=no-such-tag-b;no-such-tag-a]\n++++",
            );
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeTagNotFound(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_not_warn_if_specified_negated_tags_are_not_found_in_include_file() {
            verifies!(
                r#"
      test 'should not warn if specified negated tags are not found in include file' do
        input = <<~'EOS'
        ----
        include::fixtures/tagged-class-enclosed.rb[tags=all;!no-such-tag;!unknown-tag]
        ----
        EOS
        expected = <<~EOS.chop
        class Dog
          def initialize breed
            @breed = breed
          end

          def bark
            if @breed == 'beagle'
              'woof woof woof woof woof'
            else
              'woof woof'
            end
          end
        end
        EOS
        using_memory_logger do |logger|
          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_includes output, %(<pre>#{expected}</pre>)
          assert_empty logger.messages
        end
      end

"#
            );

            let src = "----\ninclude::fixtures/tagged-class-enclosed.rb[tags=all;!no-such-tag;!unknown-tag]\n----";
            let html = convert_safe_with_fixtures(src);
            assert!(html.contains("class Dog"), "{html}");
            assert!(
                fixture_warnings(src).is_empty(),
                "{:?}",
                fixture_warnings(src)
            );
        }

        #[test]
        fn should_warn_if_specified_tag_in_include_file_is_not_closed() {
            verifies!(
                r#"
      test 'should warn if specified tag in include file is not closed' do
        input = <<~'EOS'
        ++++
        include::fixtures/unclosed-tag.adoc[tag=a]
        ++++
        EOS

        using_memory_logger do |logger|
          result = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_equal 'a', result
          assert_message logger, :WARN, %(~<stdin>: line 2: detected unclosed tag 'a' starting at line 2 of include file), Hash
          refute_nil logger.messages[0][:message][:include_location]
        end
      end

"#
            );

            // The `++++` passthrough block renders the included line raw, and the
            // include still preprocesses and warns. (Embedded output carries a
            // trailing newline this crate adds and Asciidoctor omits.)
            let src = "++++\ninclude::fixtures/unclosed-tag.adoc[tag=a]\n++++";
            assert_eq!(convert_safe_with_fixtures(src).trim_end_matches('\n'), "a");
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeTagUnclosed(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_warn_if_end_tag_in_included_file_is_mismatched() {
            verifies!(
                r#"
      test 'should warn if end tag in included file is mismatched' do
        input = <<~'EOS'
        ++++
        include::fixtures/mismatched-end-tag.adoc[tags=a;b]
        ++++
        EOS

        inc_path = File.join DIRNAME, 'fixtures/mismatched-end-tag.adoc'
        using_memory_logger do |logger|
          result = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_equal %(a\nb), result
          assert_message logger, :WARN, %(<stdin>: line 2: mismatched end tag (expected 'b' but found 'a') at line 5 of include file: #{inc_path}), Hash
          refute_nil logger.messages[0][:message][:include_location]
        end
      end

"#
            );

            // The `++++` passthrough block renders both included lines raw.
            // (Embedded output carries a trailing newline this crate adds and
            // Asciidoctor omits.)
            let src = "++++\ninclude::fixtures/mismatched-end-tag.adoc[tags=a;b]\n++++";
            assert_eq!(
                convert_safe_with_fixtures(src).trim_end_matches('\n'),
                "a\nb"
            );
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeTagMismatchedEnd(_, _))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_warn_if_unexpected_end_tag_is_found_in_included_file() {
            verifies!(
                r#"
      test 'should warn if unexpected end tag is found in included file' do
        input = <<~'EOS'
        ++++
        include::fixtures/unexpected-end-tag.adoc[tags=a]
        ++++
        EOS

        inc_path = File.join DIRNAME, 'fixtures/unexpected-end-tag.adoc'
        using_memory_logger do |logger|
          result = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_equal 'a', result
          assert_message logger, :WARN, %(<stdin>: line 2: unexpected end tag 'a' at line 4 of include file: #{inc_path}), Hash
          refute_nil logger.messages[0][:message][:include_location]
        end
      end

"#
            );

            // The `++++` passthrough block renders the included line raw.
            // (Embedded output carries a trailing newline this crate adds and
            // Asciidoctor omits.)
            let src = "++++\ninclude::fixtures/unexpected-end-tag.adoc[tags=a]\n++++";
            assert_eq!(convert_safe_with_fixtures(src).trim_end_matches('\n'), "a");
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::IncludeTagUnexpectedEnd(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn include_directive_ignores_tags_attribute_when_empty() {
            verifies!(
                r#"
      test 'include directive ignores tags attribute when empty' do
        ['tag', 'tags'].each do |attr_name|
          input = <<~EOS
          ++++
          include::fixtures/include-file.xml[#{attr_name}=]
          ++++
          EOS

          output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
          assert_match(/(?:tag|end)::/, output, 2)
        end
      end

"#
            );

            // An empty `tag`/`tags` value is ignored, so every line — including the
            // tag directive lines themselves — is included; the `++++` passthrough
            // block emits them raw.
            for attr_name in ["tag", "tags"] {
                let src = format!("++++\ninclude::fixtures/include-file.xml[{attr_name}=]\n++++");
                let html = convert_safe_with_fixtures(&src);
                assert!(html.contains("tag::"), "{html}");
                assert!(html.contains("end::"), "{html}");
            }
        }

        #[test]
        fn lines_attribute_takes_precedence_over_tags_attribute_in_include_directive() {
            verifies!(
                r#"
      test 'lines attribute takes precedence over tags attribute in include directive' do
        input = 'include::fixtures/include-file.adoc[lines=1, tags=snippetA;snippetB]'
        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/first line of included content/, output)
        refute_match(/snippetA content/, output)
        refute_match(/snippetB content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                "include::fixtures/include-file.adoc[lines=1, tags=snippetA;snippetB]",
            );
            assert!(html.contains("first line of included content"), "{html}");
            assert!(!html.contains("snippetA content"), "{html}");
            assert!(!html.contains("snippetB content"), "{html}");
        }

        #[test]
        fn indent_of_included_file_can_be_reset_to_size_of_indent_attribute() {
            verifies!(
                r#"
      test 'indent of included file can be reset to size of indent attribute' do
        input = <<~'EOS'
        [source, xml]
        ----
        include::fixtures/basic-docinfo.xml[lines=2..3, indent=0]
        ----
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        result = xmlnodes_at_xpath('//pre', output, 1).text
        assert_equal "<year>2013</year>\n<holder>Acme™, Inc.</holder>", result
      end

"#
            );

            let html = convert_safe_with_fixtures("[source, xml]\n----\ninclude::fixtures/basic-docinfo.xml[lines=2..3, indent=0]\n----");
            assert!(html.contains("&lt;year&gt;2013&lt;/year&gt;\n&lt;holder&gt;Acme\u{2122}, Inc.&lt;/holder&gt;"), "{html}");
        }

        #[test]
        fn should_substitute_attribute_references_in_attrlist() {
            verifies!(
                r#"
      test 'should substitute attribute references in attrlist' do
        input = <<~'EOS'
        :name-of-tag: snippetA
        include::fixtures/include-file.adoc[tag={name-of-tag}]
        EOS

        output = convert_string_to_embedded input, safe: :safe, base_dir: DIRNAME
        assert_match(/snippetA content/, output)
        refute_match(/snippetB content/, output)
        refute_match(/non-tagged content/, output)
        refute_match(/included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                ":name-of-tag: snippetA\ninclude::fixtures/include-file.adoc[tag={name-of-tag}]",
            );
            assert!(html.contains("snippetA content"), "{html}");
            assert!(!html.contains("snippetB content"), "{html}");
            assert!(!html.contains("non-tagged content"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        // Non-normative: requires a custom include processor (the extension mechanism
        // is out of scope).
        non_normative!(
            r#"
      test 'should fall back to built-in include directive behavior when not handled by include processor' do
        input = 'include::fixtures/include-file.adoc[]'
        include_processor = Class.new do
          def initialize document; end

          def handles? target
            false
          end

          def process reader, target, attributes
            raise 'TestIncludeHandler should not have been invoked'
          end
        end

        document = empty_safe_document base_dir: DIRNAME
        reader = Asciidoctor::PreprocessorReader.new document, input, nil, normalize: true
        reader.instance_variable_set '@include_processors', [include_processor.new(document)]
        lines = reader.read_lines
        source = lines * ::Asciidoctor::LF
        assert_match(/included content/, source)
      end

"#
        );

        #[test]
        fn leveloffset_attribute_entries_should_be_added_to_content_if_leveloffset_attribute_is_specified(
        ) {
            verifies!(
                r#"
      test 'leveloffset attribute entries should be added to content if leveloffset attribute is specified' do
        input = 'include::fixtures/main.adoc[]'
        expected = <<~'EOS'.split ::Asciidoctor::LF
        = Main Document

        preamble

        :leveloffset: +1

        = Chapter A

        content

        :leveloffset!:
        EOS

        document = Asciidoctor.load input, safe: :safe, base_dir: DIRNAME, parse: false
        assert_equal expected, document.reader.read_lines
      end

"#
            );

            // The include applies a +1 leveloffset around the child document, so its
            // `= Chapter A` renders as a nested section, and the main content survives.
            let html = convert_safe_with_fixtures("include::fixtures/main.adoc[]");
            assert!(html.contains("Chapter A"), "{html}");
            assert!(html.contains("preamble"), "{html}");
            assert!(html.contains("content"), "{html}");
        }

        #[test]
        fn attributes_are_substituted_in_target_of_include_directive() {
            verifies!(
                r#"
      test 'attributes are substituted in target of include directive' do
        input = <<~'EOS'
        :fixturesdir: fixtures
        :ext: adoc

        include::{fixturesdir}/include-file.{ext}[]
        EOS

        doc = document_from_string input, safe: :safe, base_dir: DIRNAME
        output = doc.convert
        assert_match(/included content/, output)
      end

"#
            );

            let html = convert_safe_with_fixtures(
                ":fixturesdir: fixtures\n:ext: adoc\n\ninclude::{fixturesdir}/include-file.{ext}[]",
            );
            assert!(html.contains("included content"), "{html}");
        }

        #[test]
        fn line_is_skipped_by_default_if_target_of_include_directive_resolves_to_empty() {
            verifies!(
                r#"
      test 'line is skipped by default if target of include directive resolves to empty' do
        input = 'include::{blank}[]'
        using_memory_logger do |logger|
          doc = empty_safe_document base_dir: DIRNAME
          reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
          line = reader.read_line
          assert_equal 'Unresolved directive in <stdin> - include::{blank}[]', line
          assert_message logger, :WARN, '<stdin>: line 1: include dropped because resolved target is blank: include::{blank}[]', Hash
        end
      end

"#
            );

            // A directive whose target resolves to blank is replaced with an
            // unresolved-directive message.
            let html = convert_safe_with_fixtures("include::{blank}[]");
            assert!(html.contains("Unresolved directive"), "{html}");
        }

        #[test]
        fn include_is_dropped_if_target_contains_missing_attribute_and_attribute_missing_is_drop_line(
        ) {
            verifies!(
                r#"
      test 'include is dropped if target contains missing attribute and attribute-missing is drop-line' do
        input = 'include::{foodir}/include-file.adoc[]'
        using_memory_logger Logger::INFO do |logger|
          doc = empty_safe_document base_dir: DIRNAME, attributes: { 'attribute-missing' => 'drop-line' }
          reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
          line = reader.read_line
          assert_nil line
          assert_messages logger, [
            [:INFO, 'dropping line containing reference to missing attribute: foodir'],
            [:INFO, '<stdin>: line 1: include dropped due to missing attribute: include::{foodir}/include-file.adoc[]', Hash],
          ]
        end
      end

"#
            );

            // Under attribute-missing=drop-line, the whole directive line is removed,
            // leaving no content.
            let html = convert_with(
                "include::{foodir}/include-file.adoc[]",
                &Options::new()
                    .safe_mode(SafeMode::Safe)
                    .base_dir(fixtures_base_dir())
                    .attribute("attribute-missing", "drop-line"),
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn line_following_dropped_include_is_not_dropped() {
            verifies!(
                r#"
      test 'line following dropped include is not dropped' do
        input = <<~'EOS'
        include::{foodir}/include-file.adoc[]
        yo
        EOS

        using_memory_logger do |logger|
          doc = empty_safe_document base_dir: DIRNAME, attributes: { 'attribute-missing' => 'warn' }
          reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
          line = reader.read_line
          assert_equal 'Unresolved directive in <stdin> - include::{foodir}/include-file.adoc[]', line
          line = reader.read_line
          assert_equal 'yo', line
          assert_messages logger, [
            [:INFO, 'dropping line containing reference to missing attribute: foodir'],
            [:WARN, '<stdin>: line 1: include dropped due to missing attribute: include::{foodir}/include-file.adoc[]', Hash],
          ]
        end
      end

"#
            );

            // Under attribute-missing=warn a missing-attribute include is replaced with
            // an unresolved-directive message; the following line survives.
            let src = "include::{foodir}/include-file.adoc[]\nyo";
            let html = convert_with(
                src,
                &Options::new()
                    .safe_mode(SafeMode::Safe)
                    .base_dir(fixtures_base_dir())
                    .attribute("attribute-missing", "warn"),
            );
            assert!(html.contains("Unresolved directive"), "{html}");
            assert!(html.contains("yo"), "{html}");
        }

        #[test]
        fn escaped_include_directive_is_left_unprocessed() {
            verifies!(
                r#"
      test 'escaped include directive is left unprocessed' do
        input = <<~'EOS'
        \include::fixtures/include-file.adoc[]
        \escape preserved here
        EOS
        doc = empty_safe_document base_dir: DIRNAME
        reader = Asciidoctor::PreprocessorReader.new doc, input, nil, normalize: true
        # we should be able to peek it multiple times and still have the backslash preserved
        # this is the test for @unescape_next_line
        assert_equal 'include::fixtures/include-file.adoc[]', reader.peek_line
        assert_equal 'include::fixtures/include-file.adoc[]', reader.peek_line
        assert_equal 'include::fixtures/include-file.adoc[]', reader.read_line
        assert_equal '\\escape preserved here', reader.read_line
      end

"#
            );

            // A backslash-escaped include is emitted as literal text (the backslash
            // removed) and never resolved.
            let html = convert_safe_with_fixtures(
                "\\include::fixtures/include-file.adoc[]\n\\escape preserved here",
            );
            assert!(
                html.contains("include::fixtures/include-file.adoc[]"),
                "{html}"
            );
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_not_at_start_of_line_is_ignored() {
            verifies!(
                r#"
      test 'include directive not at start of line is ignored' do
        input = ' include::include-file.adoc[]'
        para = block_from_string input
        assert_equal 1, para.lines.size
        # NOTE the space gets stripped because the line is treated as an inline literal
        assert_equal :literal, para.context
        assert_equal 'include::include-file.adoc[]', para.source
      end

"#
            );

            // A leading space makes the line a literal block, not an include.
            let html = convert(" include::include-file.adoc[]");
            assert!(html.contains("include::include-file.adoc[]"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_is_disabled_when_max_include_depth_attribute_is_0() {
            verifies!(
                r#"
      test 'include directive is disabled when max-include-depth attribute is 0' do
        input = 'include::include-file.adoc[]'
        para = block_from_string input, safe: :safe, attributes: { 'max-include-depth' => 0 }
        assert_equal 1, para.lines.size
        assert_equal 'include::include-file.adoc[]', para.source
      end

"#
            );

            let html = convert_with(
                "include::include-file.adoc[]",
                &Options::new()
                    .safe_mode(SafeMode::Safe)
                    .attribute("max-include-depth", "0"),
            );
            assert!(html.contains("include::include-file.adoc[]"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn max_include_depth_cannot_be_set_by_document() {
            verifies!(
                r#"
      test 'max-include-depth cannot be set by document' do
        input = <<~'EOS'
        :max-include-depth: 1

        include::include-file.adoc[]
        EOS
        para = block_from_string input, safe: :safe, attributes: { 'max-include-depth' => 0 }
        assert_equal 1, para.lines.size
        assert_equal 'include::include-file.adoc[]', para.source
      end

"#
            );

            // An API `max-include-depth` of 0 wins over a document `:max-include-depth: 1`.
            let html = convert_with(
                ":max-include-depth: 1\n\ninclude::include-file.adoc[]",
                &Options::new()
                    .safe_mode(SafeMode::Safe)
                    .attribute("max-include-depth", "0"),
            );
            assert!(html.contains("include::include-file.adoc[]"), "{html}");
            assert!(!html.contains("included content"), "{html}");
        }

        #[test]
        fn include_directive_should_be_disabled_if_max_include_depth_has_been_exceeded() {
            verifies!(
                r#"
      test 'include directive should be disabled if max include depth has been exceeded' do
        input = 'include::fixtures/parent-include.adoc[depth=1]'
        using_memory_logger do |logger|
          pseudo_docfile = File.join DIRNAME, 'main.adoc'
          doc = empty_safe_document base_dir: DIRNAME
          reader = Asciidoctor::PreprocessorReader.new doc, input, Asciidoctor::Reader::Cursor.new(pseudo_docfile), normalize: true
          lines = reader.readlines
          assert_includes lines, 'include::grandchild-include.adoc[]'
          assert_message logger, :ERROR, 'fixtures/child-include.adoc: line 3: maximum include depth of 1 exceeded', Hash
        end
      end

"#
            );

            // With `depth=1`, the grandchild include (two levels down) is left
            // unexpanded and a max-include-depth warning is raised.
            let src = "include::fixtures/parent-include.adoc[depth=1]";
            let html = convert_safe_with_fixtures(src);
            assert!(
                html.contains("include::grandchild-include.adoc[]"),
                "{html}"
            );
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::MaxIncludeDepthExceeded(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn include_directive_should_be_disabled_if_max_include_depth_set_in_nested_context_has_been_exceeded(
        ) {
            verifies!(
                r#"
      test 'include directive should be disabled if max include depth set in nested context has been exceeded' do
        input = 'include::fixtures/parent-include-restricted.adoc[depth=3]'
        using_memory_logger do |logger|
          pseudo_docfile = File.join DIRNAME, 'main.adoc'
          doc = empty_safe_document base_dir: DIRNAME
          reader = Asciidoctor::PreprocessorReader.new doc, input, Asciidoctor::Reader::Cursor.new(pseudo_docfile), normalize: true
          lines = reader.readlines
          assert_includes lines, 'first line of child'
          assert_includes lines, 'include::grandchild-include.adoc[]'
          assert_message logger, :ERROR, 'fixtures/child-include.adoc: line 3: maximum include depth of 0 exceeded', Hash
        end
      end

"#
            );

            let src = "include::fixtures/parent-include-restricted.adoc[depth=3]";
            let html = convert_safe_with_fixtures(src);
            assert!(html.contains("first line of child"), "{html}");
            assert!(
                html.contains("include::grandchild-include.adoc[]"),
                "{html}"
            );
            let warnings = fixture_warnings(src);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::MaxIncludeDepthExceeded(_))),
                "{warnings:?}"
            );
        }

        non_normative!(
            r#"
      test 'read_lines_until should not process lines if process option is false' do
        lines = <<~'EOS'.lines
        ////
        include::fixtures/no-such-file.adoc[]
        ////
        EOS

        doc = empty_safe_document base_dir: DIRNAME
        reader = Asciidoctor::PreprocessorReader.new doc, lines, nil, normalize: true
        reader.read_line
        result = reader.read_lines_until(terminator: '////', skip_processing: true)
        assert_equal lines.map(&:chomp)[1..1], result
      end

      test 'skip_comment_lines should not process lines read' do
        lines = <<~'EOS'.lines
        ////
        include::fixtures/no-such-file.adoc[]
        ////
        EOS

        using_memory_logger do |logger|
          doc = empty_safe_document base_dir: DIRNAME
          reader = Asciidoctor::PreprocessorReader.new doc, lines, nil, normalize: true
          reader.skip_comment_lines
          assert reader.empty?
          assert logger.empty?
        end
      end
    end

"#
        );
    }

    mod conditional_inclusions {
        use super::*;

        non_normative!(
            r#"
    context 'Conditional Inclusions' do
"#
        );

        // Non-normative: these drive PreprocessorReader cursor mechanics (process_line
        // / peek_line / peek_lines); no rendered form.
        non_normative!(
            r#"
      test 'process_line returns nil if cursor advanced' do
        input = <<~'EOS'
        ifdef::asciidoctor[]
        Asciidoctor!
        endif::asciidoctor[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_nil reader.send :process_line, reader.lines.first
      end

      test 'peek_line advances cursor to next conditional line of content' do
        input = <<~'EOS'
        ifdef::asciidoctor[]
        Asciidoctor!
        endif::asciidoctor[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 1, reader.lineno
        assert_equal 'Asciidoctor!', reader.peek_line
        assert_equal 2, reader.lineno
      end

      test 'peek_lines should preprocess lines if direct is false' do
        input = <<~'EOS'
        The Asciidoctor
        ifdef::asciidoctor[is in.]
        EOS
        doc = Asciidoctor::Document.new input
        reader = doc.reader
        result = reader.peek_lines 2, false
        assert_equal ['The Asciidoctor', 'is in.'], result
      end

      test 'peek_lines should not preprocess lines if direct is true' do
        input = <<~'EOS'
        The Asciidoctor
        ifdef::asciidoctor[is in.]
        EOS
        doc = Asciidoctor::Document.new input
        reader = doc.reader
        result = reader.peek_lines 2, true
        assert_equal ['The Asciidoctor', 'ifdef::asciidoctor[is in.]'], result
      end

      test 'peek_lines should not prevent subsequent preprocessing of peeked lines' do
        input = <<~'EOS'
        The Asciidoctor
        ifdef::asciidoctor[is in.]
        EOS
        doc = Asciidoctor::Document.new input
        reader = doc.reader
        result = reader.peek_lines 2, true
        result = reader.peek_lines 2, false
        assert_equal ['The Asciidoctor', 'is in.'], result
      end

      test 'process_line returns line if cursor not advanced' do
        input = <<~'EOS'
        content
        ifdef::asciidoctor[]
        Asciidoctor!
        endif::asciidoctor[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        refute_nil reader.send :process_line, reader.lines.first
      end

      test 'peek_line does not advance cursor when on a regular content line' do
        input = <<~'EOS'
        content
        ifdef::asciidoctor[]
        Asciidoctor!
        endif::asciidoctor[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 1, reader.lineno
        assert_equal 'content', reader.peek_line
        assert_equal 1, reader.lineno
      end

      test 'peek_line returns nil if cursor advances past end of source' do
        input = <<~'EOS'
        ifdef::foobar[]
        swallowed content
        endif::foobar[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 1, reader.lineno
        assert_nil reader.peek_line
        assert_equal 4, reader.lineno
      end

      test 'peek_line returns nil if contents of skipped conditional is empty line' do
        input = <<~'EOS'
        ifdef::foobar[]

        endif::foobar[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        assert_equal 1, reader.lineno
        assert_nil reader.peek_line
      end

"#
        );

        #[test]
        fn ifdef_with_defined_attribute_includes_content() {
            verifies!(
                r#"
      test 'ifdef with defined attribute includes content' do
        input = <<~'EOS'
        ifdef::holygrail[]
        There is a holy grail!
        endif::holygrail[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'holygrail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'There is a holy grail!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::holygrail[]\nThere is a holy grail!\nendif::holygrail[]",
                &[("holygrail", "")],
            );
            assert!(html.contains("There is a holy grail!"), "{html}");
        }

        #[test]
        fn ifdef_with_defined_attribute_includes_text_in_brackets() {
            verifies!(
                r#"
      test 'ifdef with defined attribute includes text in brackets' do
        input = <<~'EOS'
        On our quest we go...
        ifdef::holygrail[There is a holy grail!]
        There was much rejoicing.
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'holygrail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "On our quest we go...\nThere is a holy grail!\nThere was much rejoicing.", (lines * ::Asciidoctor::LF)
      end

"#
            );

            // `holygrail` is set, so the bracketed text is included between the
            // surrounding lines.
            let html = convert_with_attrs(
                "On our quest we go...\nifdef::holygrail[There is a holy grail!]\nThere was much rejoicing.",
                &[("holygrail", "")],
            );
            assert!(
                html.contains("There is a holy grail!\nThere was much rejoicing."),
                "{html}"
            );
        }

        // Non-normative: an include directive inside an ifdef[...] bracket is not
        // processed (#133).
        non_normative!(
            r#"
      test 'ifdef with defined attribute processes include directive in brackets' do
        input = 'ifdef::asciidoctor-version[include::fixtures/include-file.adoc[tag=snippetA]]'
        doc = Asciidoctor::Document.new input, safe: :safe, base_dir: DIRNAME
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'snippetA content', lines[0]
      end

"#
        );

        #[test]
        fn ifdef_attribute_name_is_not_case_sensitive() {
            verifies!(
                r#"
      test 'ifdef attribute name is not case sensitive' do
        input = <<~'EOS'
        ifdef::showScript[]
        The script is shown!
        endif::showScript[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'showscript' => '' }
        result = doc.reader.read
        assert_equal 'The script is shown!', result
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::showScript[]\nThe script is shown!\nendif::showScript[]",
                &[("showscript", "")],
            );
            assert!(html.contains("The script is shown!"), "{html}");
        }

        #[test]
        fn ifndef_with_defined_attribute_does_not_include_text_in_brackets() {
            verifies!(
                r#"
      test 'ifndef with defined attribute does not include text in brackets' do
        input = <<~'EOS'
        On our quest we go...
        ifndef::hardships[There is a holy grail!]
        There was no rejoicing.
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'hardships' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "On our quest we go...\nThere was no rejoicing.", (lines * ::Asciidoctor::LF)
      end

"#
            );

            // `hardships` is set, so the `ifndef` bracketed text is dropped.
            let html = convert_with_attrs(
                "On our quest we go...\nifndef::hardships[There is a holy grail!]\nThere was no rejoicing.",
                &[("hardships", "")],
            );
            assert!(!html.contains("There is a holy grail!"), "{html}");
            assert!(html.contains("There was no rejoicing."), "{html}");
        }

        #[test]
        fn include_with_non_matching_nested_exclude() {
            verifies!(
                r#"
      test 'include with non-matching nested exclude' do
        input = <<~'EOS'
        ifdef::grail[]
        holy
        ifdef::swallow[]
        swallow
        endif::swallow[]
        grail
        endif::grail[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'grail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "holy\ngrail", (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs("ifdef::grail[]\nholy\nifdef::swallow[]\nswallow\nendif::swallow[]\ngrail\nendif::grail[]", &[("grail", "")]);
            assert!(html.contains("holy\ngrail"), "{html}");
        }

        #[test]
        fn nested_excludes_with_same_condition() {
            verifies!(
                r#"
      test 'nested excludes with same condition' do
        input = <<~'EOS'
        ifndef::grail[]
        ifndef::grail[]
        not here
        endif::grail[]
        endif::grail[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'grail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::grail[]\nifndef::grail[]\nnot here\nendif::grail[]\nendif::grail[]",
                &[("grail", "")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn include_with_nested_exclude_of_inverted_condition() {
            verifies!(
                r#"
      test 'include with nested exclude of inverted condition' do
        input = <<~'EOS'
        ifdef::grail[]
        holy
        ifndef::grail[]
        not here
        endif::grail[]
        grail
        endif::grail[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'grail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "holy\ngrail", (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs("ifdef::grail[]\nholy\nifndef::grail[]\nnot here\nendif::grail[]\ngrail\nendif::grail[]", &[("grail", "")]);
            assert!(html.contains("holy\ngrail"), "{html}");
        }

        #[test]
        fn exclude_with_matching_nested_exclude() {
            verifies!(
                r#"
      test 'exclude with matching nested exclude' do
        input = <<~'EOS'
        poof
        ifdef::swallow[]
        no
        ifdef::swallow[]
        swallow
        endif::swallow[]
        here
        endif::swallow[]
        gone
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'grail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "poof\ngone", (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs("poof\nifdef::swallow[]\nno\nifdef::swallow[]\nswallow\nendif::swallow[]\nhere\nendif::swallow[]\ngone", &[("grail", "")]);
            assert!(html.contains("poof\ngone"), "{html}");
        }

        #[test]
        fn exclude_with_nested_include_using_shorthand_end() {
            verifies!(
                r#"
      test 'exclude with nested include using shorthand end' do
        input = <<~'EOS'
        poof
        ifndef::grail[]
        no grail
        ifndef::swallow[]
        or swallow
        endif::[]
        in here
        endif::[]
        gone
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'grail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "poof\ngone", (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs("poof\nifndef::grail[]\nno grail\nifndef::swallow[]\nor swallow\nendif::[]\nin here\nendif::[]\ngone", &[("grail", "")]);
            assert!(html.contains("poof\ngone"), "{html}");
        }

        #[test]
        fn ifdef_with_one_alternative_attribute_set_includes_content() {
            verifies!(
                r#"
      test 'ifdef with one alternative attribute set includes content' do
        input = <<~'EOS'
        ifdef::holygrail,swallow[]
        Our quest is complete!
        endif::holygrail,swallow[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'swallow' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Our quest is complete!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::holygrail,swallow[]\nOur quest is complete!\nendif::holygrail,swallow[]",
                &[("swallow", "")],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
        }

        #[test]
        fn ifdef_with_no_alternative_attributes_set_does_not_include_content() {
            verifies!(
                r#"
      test 'ifdef with no alternative attributes set does not include content' do
        input = <<~'EOS'
        ifdef::holygrail,swallow[]
        Our quest is complete!
        endif::holygrail,swallow[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::holygrail,swallow[]\nOur quest is complete!\nendif::holygrail,swallow[]",
                &[],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifdef_with_all_required_attributes_set_includes_content() {
            verifies!(
                r#"
      test 'ifdef with all required attributes set includes content' do
        input = <<~'EOS'
        ifdef::holygrail+swallow[]
        Our quest is complete!
        endif::holygrail+swallow[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'holygrail' => '', 'swallow' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Our quest is complete!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::holygrail+swallow[]\nOur quest is complete!\nendif::holygrail+swallow[]",
                &[("holygrail", ""), ("swallow", "")],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
        }

        #[test]
        fn ifdef_with_missing_required_attributes_does_not_include_content() {
            verifies!(
                r#"
      test 'ifdef with missing required attributes does not include content' do
        input = <<~'EOS'
        ifdef::holygrail+swallow[]
        Our quest is complete!
        endif::holygrail+swallow[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'holygrail' => '' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifdef::holygrail+swallow[]\nOur quest is complete!\nendif::holygrail+swallow[]",
                &[("holygrail", "")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifdef_should_permit_leading_trailing_and_repeat_operators() {
            verifies!(
                r#"
      test 'ifdef should permit leading, trailing, and repeat operators' do
        {
          'asciidoctor,' => 'content',
          ',asciidoctor' => 'content',
          'asciidoctor+' => '',
          '+asciidoctor' => '',
          'asciidoctor,,asciidoctor-version' => 'content',
          'asciidoctor++asciidoctor-version' => '',
        }.each do |condition, expected|
          input = <<~EOS
          ifdef::#{condition}[]
          content
          endif::[]
          EOS
          assert_equal expected, (document_from_string input, parse: false).reader.read
        end
      end

"#
            );

            for (condition, expected) in [
                ("asciidoctor,", "content"),
                (",asciidoctor", "content"),
                ("asciidoctor+", ""),
                ("+asciidoctor", ""),
                ("asciidoctor,,asciidoctor-version", "content"),
                ("asciidoctor++asciidoctor-version", ""),
            ] {
                let src = format!("ifdef::{condition}[]\ncontent\nendif::[]");
                let html = convert(&src);
                if expected.is_empty() {
                    assert!(html.trim().is_empty(), "{condition}: {html}");
                } else {
                    assert!(html.contains(expected), "{condition}: {html}");
                }
            }
        }

        #[test]
        fn ifndef_with_undefined_attribute_includes_block() {
            verifies!(
                r#"
      test 'ifndef with undefined attribute includes block' do
        input = <<~'EOS'
        ifndef::holygrail[]
        Our quest continues to find the holy grail!
        endif::holygrail[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Our quest continues to find the holy grail!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs("ifndef::holygrail[]\nOur quest continues to find the holy grail!\nendif::holygrail[]", &[]);
            assert!(
                html.contains("Our quest continues to find the holy grail!"),
                "{html}"
            );
        }

        #[test]
        fn ifndef_with_one_alternative_attribute_set_does_not_include_content() {
            verifies!(
                r#"
      test 'ifndef with one alternative attribute set does not include content' do
        input = <<~'EOS'
        ifndef::holygrail,swallow[]
        Our quest is complete!
        endif::holygrail,swallow[]
        EOS

        result = (Asciidoctor::Document.new input, attributes: { 'swallow' => '' }).reader.read
        assert_empty result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail,swallow[]\nOur quest is complete!\nendif::holygrail,swallow[]",
                &[("swallow", "")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifndef_with_both_alternative_attributes_set_does_not_include_content() {
            verifies!(
                r#"
      test 'ifndef with both alternative attributes set does not include content' do
        input = <<~'EOS'
        ifndef::holygrail,swallow[]
        Our quest is complete!
        endif::holygrail,swallow[]
        EOS

        result = (Asciidoctor::Document.new input, attributes: { 'swallow' => '', 'holygrail' => '' }).reader.read
        assert_empty result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail,swallow[]\nOur quest is complete!\nendif::holygrail,swallow[]",
                &[("swallow", ""), ("holygrail", "")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifndef_with_no_alternative_attributes_set_includes_content() {
            verifies!(
                r#"
      test 'ifndef with no alternative attributes set includes content' do
        input = <<~'EOS'
        ifndef::holygrail,swallow[]
        Our quest is complete!
        endif::holygrail,swallow[]
        EOS

        result = (Asciidoctor::Document.new input).reader.read
        assert_equal 'Our quest is complete!', result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail,swallow[]\nOur quest is complete!\nendif::holygrail,swallow[]",
                &[],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
        }

        #[test]
        fn ifndef_with_no_required_attributes_set_includes_content() {
            verifies!(
                r#"
      test 'ifndef with no required attributes set includes content' do
        input = <<~'EOS'
        ifndef::holygrail+swallow[]
        Our quest is complete!
        endif::holygrail+swallow[]
        EOS

        result = (Asciidoctor::Document.new input).reader.read
        assert_equal 'Our quest is complete!', result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail+swallow[]\nOur quest is complete!\nendif::holygrail+swallow[]",
                &[],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
        }

        #[test]
        fn ifndef_with_all_required_attributes_set_does_not_include_content() {
            verifies!(
                r#"
      test 'ifndef with all required attributes set does not include content' do
        input = <<~'EOS'
        ifndef::holygrail+swallow[]
        Our quest is complete!
        endif::holygrail+swallow[]
        EOS

        result = (Asciidoctor::Document.new input, attributes: { 'swallow' => '', 'holygrail' => '' }).reader.read
        assert_empty result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail+swallow[]\nOur quest is complete!\nendif::holygrail+swallow[]",
                &[("swallow", ""), ("holygrail", "")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifndef_with_at_least_one_required_attributes_set_does_not_include_content() {
            verifies!(
                r#"
      test 'ifndef with at least one required attributes set does not include content' do
        input = <<~'EOS'
        ifndef::holygrail+swallow[]
        Our quest is complete!
        endif::holygrail+swallow[]
        EOS

        result = (Asciidoctor::Document.new input, attributes: { 'swallow' => '' }).reader.read
        assert_equal 'Our quest is complete!', result
      end

"#
            );

            let html = convert_with_attrs(
                "ifndef::holygrail+swallow[]\nOur quest is complete!\nendif::holygrail+swallow[]",
                &[("swallow", "")],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
        }

        #[test]
        fn ifdef_around_empty_line_does_not_introduce_extra_line() {
            verifies!(
                r#"
      test 'ifdef around empty line does not introduce extra line' do
        input = <<~'EOS'
        before
        ifdef::no-such-attribute[]

        endif::[]
        after
        EOS

        result = (Asciidoctor::Document.new input).reader.read
        assert_equal %(before\nafter), result
      end

"#
            );

            let html = convert_with_attrs(
                "before\nifdef::no-such-attribute[]\n\nendif::[]\nafter",
                &[],
            );
            assert!(html.contains("before\nafter"), "{html}");
        }

        #[test]
        fn should_log_warning_if_endif_is_unmatched() {
            verifies!(
                r#"
      test 'should log warning if endif is unmatched' do
        input = <<~'EOS'
        Our quest is complete!
        endif::on-quest[]
        EOS

        using_memory_logger do |logger|
          result = (Asciidoctor::Document.new input, attributes: { 'on-quest' => '' }).reader.read
          assert_equal 'Our quest is complete!', result
          assert_message logger, :ERROR, '~<stdin>: line 2: unmatched preprocessor directive: endif::on-quest[]', Hash
        end
      end

"#
            );

            let html = convert_with_attrs(
                "Our quest is complete!\nendif::on-quest[]",
                &[("on-quest", "")],
            );
            assert!(html.contains("Our quest is complete!"), "{html}");
            let warnings = conditional_warnings(
                "Our quest is complete!\nendif::on-quest[]",
                &[("on-quest", "")],
            );
            assert_eq!(
                warnings,
                vec![(
                    WarningType::UnmatchedConditionalDirective("endif::on-quest[]".to_owned()),
                    2
                )]
            );
        }

        #[test]
        fn should_log_warning_if_endif_is_mismatched() {
            verifies!(
                r#"
      test 'should log warning if endif is mismatched' do
        input = <<~'EOS'
        ifdef::on-quest[]
        Our quest is complete!
        endif::on-journey[]
        EOS

        using_memory_logger do |logger|
          result = (Asciidoctor::Document.new input, attributes: { 'on-quest' => '' }, sourcemap: true).reader.read
          assert_equal 'Our quest is complete!', result
          assert_messages logger, [
            [:ERROR, '~<stdin>: line 3: mismatched preprocessor directive: endif::on-journey[]', Hash],
            [:ERROR, '~<stdin>: line 1: detected unterminated preprocessor conditional directive: ifdef::on-quest[]', Hash],
          ]
        end
      end

"#
            );

            let src = "ifdef::on-quest[]\nOur quest is complete!\nendif::on-journey[]";
            let html = convert_with_attrs(src, &[("on-quest", "")]);
            assert!(html.contains("Our quest is complete!"), "{html}");
            // The mismatched `endif` closes nothing, so the opening `ifdef` is left
            // unterminated: both diagnostics are raised. (Line numbers are not asserted;
            // this crate reports both on the preprocessed span rather than Asciidoctor's
            // original 1 / 3.)
            let warnings = conditional_warnings(src, &[("on-quest", "")]);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::UnterminatedConditionalDirective(_))),
                "{warnings:?}"
            );
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::MismatchedConditionalDirective(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_log_warning_if_endif_contains_text() {
            verifies!(
                r#"
      test 'should log warning if endif contains text' do
        input = <<~'EOS'
        ifdef::on-quest[]
        Our quest is complete!
        endif::on-quest[complete!]
        fin
        EOS

        using_memory_logger do |logger|
          result = (Asciidoctor::Document.new input, attributes: { 'on-quest' => '' }, sourcemap: true).reader.read
          assert_equal %(Our quest is complete!\nfin), result
          assert_messages logger, [
            [:ERROR, '~<stdin>: line 3: malformed preprocessor directive - text not permitted: endif::on-quest[complete!]', Hash],
            [:ERROR, '~<stdin>: line 1: detected unterminated preprocessor conditional directive: ifdef::on-quest[]', Hash],
          ]
        end
      end

"#
            );

            let src = "ifdef::on-quest[]\nOur quest is complete!\nendif::on-quest[complete!]\nfin";
            let html = convert_with_attrs(src, &[("on-quest", "")]);
            assert!(html.contains("Our quest is complete!"), "{html}");
            assert!(html.contains("fin"), "{html}");
            let warnings = conditional_warnings(src, &[("on-quest", "")]);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::MalformedConditionalDirective(_, _))),
                "{warnings:?}"
            );
        }

        #[test]
        fn escaped_ifdef_is_unescaped_and_ignored() {
            verifies!(
                r#"
      test 'escaped ifdef is unescaped and ignored' do
        input = <<~'EOS'
        \ifdef::holygrail[]
        content
        \endif::holygrail[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal "ifdef::holygrail[]\ncontent\nendif::holygrail[]", (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html =
                convert_with_attrs("\\ifdef::holygrail[]\ncontent\n\\endif::holygrail[]", &[]);
            assert!(
                html.contains("ifdef::holygrail[]\ncontent\nendif::holygrail[]"),
                "{html}"
            );
        }

        #[test]
        fn ifeval_comparing_missing_attribute_to_nil_includes_content() {
            verifies!(
                r#"
      test 'ifeval comparing missing attribute to nil includes content' do
        input = <<~'EOS'
        ifeval::['{foo}' == '']
        No foo for you!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'No foo for you!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html =
                convert_with_attrs("ifeval::['{foo}' == '']\nNo foo for you!\nendif::[]", &[]);
            assert!(html.contains("No foo for you!"), "{html}");
        }

        #[test]
        fn ifeval_comparing_missing_attribute_to_0_drops_content() {
            verifies!(
                r#"
      test 'ifeval comparing missing attribute to 0 drops content' do
        input = <<~'EOS'
        ifeval::[{leveloffset} == 0]
        I didn't make the cut!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[{leveloffset} == 0]\nI didn't make the cut!\nendif::[]",
                &[],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifeval_running_unsupported_operation_on_missing_attribute_drops_content() {
            verifies!(
                r#"
      test 'ifeval running unsupported operation on missing attribute drops content' do
        input = <<~'EOS'
        ifeval::[{leveloffset} >= 3]
        I didn't make the cut!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[{leveloffset} >= 3]\nI didn't make the cut!\nendif::[]",
                &[],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifeval_running_invalid_operation_drops_content() {
            verifies!(
                r#"
      test 'ifeval running invalid operation drops content' do
        input = <<~'EOS'
        ifeval::[{asciidoctor-version} > true]
        I didn't make the cut!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[{asciidoctor-version} > true]\nI didn't make the cut!\nendif::[]",
                &[],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifeval_comparing_double_quoted_attribute_to_matching_string_includes_content() {
            verifies!(
                r#"
      test 'ifeval comparing double-quoted attribute to matching string includes content' do
        input = <<~'EOS'
        ifeval::["{gem}" == "asciidoctor"]
        Asciidoctor it is!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'gem' => 'asciidoctor' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Asciidoctor it is!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[\"{gem}\" == \"asciidoctor\"]\nAsciidoctor it is!\nendif::[]",
                &[("gem", "asciidoctor")],
            );
            assert!(html.contains("Asciidoctor it is!"), "{html}");
        }

        #[test]
        fn ifeval_comparing_single_quoted_attribute_to_matching_string_includes_content() {
            verifies!(
                r#"
      test 'ifeval comparing single-quoted attribute to matching string includes content' do
        input = <<~'EOS'
        ifeval::['{gem}' == 'asciidoctor']
        Asciidoctor it is!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'gem' => 'asciidoctor' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Asciidoctor it is!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::['{gem}' == 'asciidoctor']\nAsciidoctor it is!\nendif::[]",
                &[("gem", "asciidoctor")],
            );
            assert!(html.contains("Asciidoctor it is!"), "{html}");
        }

        #[test]
        fn ifeval_comparing_quoted_attribute_to_non_matching_string_drops_content() {
            verifies!(
                r#"
      test 'ifeval comparing quoted attribute to non-matching string drops content' do
        input = <<~'EOS'
        ifeval::['{gem}' == 'asciidoctor']
        Asciidoctor it is!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'gem' => 'tilt' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal '', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::['{gem}' == 'asciidoctor']\nAsciidoctor it is!\nendif::[]",
                &[("gem", "tilt")],
            );
            assert!(html.trim().is_empty(), "{html}");
        }

        #[test]
        fn ifeval_comparing_attribute_to_lower_version_number_includes_content() {
            verifies!(
                r#"
      test 'ifeval comparing attribute to lower version number includes content' do
        input = <<~'EOS'
        ifeval::['{asciidoctor-version}' >= '0.1.0']
        That version will do!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'That version will do!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::['{asciidoctor-version}' >= '0.1.0']\nThat version will do!\nendif::[]",
                &[],
            );
            assert!(html.contains("That version will do!"), "{html}");
        }

        #[test]
        fn ifeval_comparing_attribute_to_self_includes_content() {
            verifies!(
                r#"
      test 'ifeval comparing attribute to self includes content' do
        input = <<~'EOS'
        ifeval::['{asciidoctor-version}' == '{asciidoctor-version}']
        Of course it's the same!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'Of course it\'s the same!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert("ifeval::['{asciidoctor-version}' == '{asciidoctor-version}']\nOf course it's the same!\nendif::[]");
            assert!(html.contains("the same!"), "{html}");
        }

        #[test]
        fn ifeval_arguments_can_be_transposed() {
            verifies!(
                r#"
      test 'ifeval arguments can be transposed' do
        input = <<~'EOS'
        ifeval::['0.1.0' <= '{asciidoctor-version}']
        That version will do!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'That version will do!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::['0.1.0' <= '{asciidoctor-version}']\nThat version will do!\nendif::[]",
                &[],
            );
            assert!(html.contains("That version will do!"), "{html}");
        }

        #[test]
        fn ifeval_matching_numeric_equality_includes_content() {
            verifies!(
                r#"
      test 'ifeval matching numeric equality includes content' do
        input = <<~'EOS'
        ifeval::[{rings} == 1]
        One ring to rule them all!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'rings' => '1' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'One ring to rule them all!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[{rings} == 1]\nOne ring to rule them all!\nendif::[]",
                &[("rings", "1")],
            );
            assert!(html.contains("One ring to rule them all!"), "{html}");
        }

        #[test]
        fn ifeval_matching_numeric_inequality_includes_content() {
            verifies!(
                r#"
      test 'ifeval matching numeric inequality includes content' do
        input = <<~'EOS'
        ifeval::[{rings} != 0]
        One ring to rule them all!
        endif::[]
        EOS

        doc = Asciidoctor::Document.new input, attributes: { 'rings' => '1' }
        reader = doc.reader
        lines = []
        while reader.has_more_lines?
          lines << reader.read_line
        end
        assert_equal 'One ring to rule them all!', (lines * ::Asciidoctor::LF)
      end

"#
            );

            let html = convert_with_attrs(
                "ifeval::[{rings} != 0]\nOne ring to rule them all!\nendif::[]",
                &[("rings", "1")],
            );
            assert!(html.contains("One ring to rule them all!"), "{html}");
        }

        #[test]
        fn should_warn_if_ifeval_has_target() {
            verifies!(
                r#"
      test 'should warn if ifeval has target' do
        input = <<~'EOS'
        ifeval::target[1 == 1]
        content
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'content', (lines * ::Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 1: malformed preprocessor directive - target not permitted: ifeval::target[1 == 1]', Hash
        end
      end

"#
            );

            let html = convert("ifeval::target[1 == 1]\ncontent");
            assert!(html.contains("content"), "{html}");
            let warnings = conditional_warnings("ifeval::target[1 == 1]\ncontent", &[]);
            assert_eq!(
                warnings,
                vec![(
                    WarningType::MalformedConditionalDirective(
                        "target not permitted".to_owned(),
                        "ifeval::target[1 == 1]".to_owned()
                    ),
                    1
                )]
            );
        }

        #[test]
        fn should_warn_if_ifeval_has_invalid_expression() {
            verifies!(
                r#"
      test 'should warn if ifeval has invalid expression' do
        input = <<~'EOS'
        ifeval::[1 | 2]
        content
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'content', (lines * ::Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 1: malformed preprocessor directive - invalid expression: ifeval::[1 | 2]', Hash
        end
      end

"#
            );

            let html = convert("ifeval::[1 | 2]\ncontent");
            assert!(html.contains("content"), "{html}");
            let warnings = conditional_warnings("ifeval::[1 | 2]\ncontent", &[]);
            assert_eq!(
                warnings,
                vec![(
                    WarningType::MalformedConditionalDirective(
                        "invalid expression".to_owned(),
                        "ifeval::[1 | 2]".to_owned()
                    ),
                    1
                )]
            );
        }

        #[test]
        fn should_warn_if_ifeval_is_missing_expression() {
            verifies!(
                r#"
      test 'should warn if ifeval is missing expression' do
        input = <<~'EOS'
        ifeval::[]
        content
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'content', (lines * ::Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 1: malformed preprocessor directive - missing expression: ifeval::[]', Hash
        end
      end

"#
            );

            let html = convert("ifeval::[]\ncontent");
            assert!(html.contains("content"), "{html}");
            let warnings = conditional_warnings("ifeval::[]\ncontent", &[]);
            assert_eq!(
                warnings,
                vec![(
                    WarningType::MalformedConditionalDirective(
                        "missing expression".to_owned(),
                        "ifeval::[]".to_owned()
                    ),
                    1
                )]
            );
        }

        #[test]
        fn ifdef_with_no_target_is_ignored() {
            verifies!(
                r#"
      test 'ifdef with no target is ignored' do
        input = <<~'EOS'
        ifdef::[]
        content
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'content', (lines * ::Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 1: malformed preprocessor directive - missing target: ifdef::[]', Hash
        end
      end

"#
            );

            let html = convert("ifdef::[]\ncontent");
            assert!(html.contains("content"), "{html}");
            let warnings = conditional_warnings("ifdef::[]\ncontent", &[]);
            assert_eq!(
                warnings,
                vec![(
                    WarningType::MalformedConditionalDirective(
                        "missing target".to_owned(),
                        "ifdef::[]".to_owned()
                    ),
                    1
                )]
            );
        }

        #[test]
        fn should_not_warn_about_invalid_ifdef_preprocessor_directive_if_already_skipping() {
            verifies!(
                r#"
      test 'should not warn about invalid ifdef preprocessor directive if already skipping' do
        input = <<~'EOS'
        ifdef::attribute-not-set[]
        foo
        ifdef::[]
        bar
        endif::[]
        baz
        EOS

        using_memory_logger do |logger|
          result = (Asciidoctor::Document.new input).reader.read
          assert_equal 'baz', result
          assert_empty logger
        end
      end

"#
            );

            let src = "ifdef::attribute-not-set[]\nfoo\nifdef::[]\nbar\nendif::[]\nbaz";
            let html = convert(src);
            assert!(html.contains("baz"), "{html}");
            assert!(
                conditional_warnings(src, &[]).is_empty(),
                "{:?}",
                conditional_warnings(src, &[])
            );
        }

        #[test]
        fn should_not_warn_about_invalid_ifeval_preprocessor_directive_if_already_skipping() {
            verifies!(
                r#"
      test 'should not warn about invalid ifeval preprocessor directive if already skipping' do
        input = <<~'EOS'
        ifdef::attribute-not-set[]
        foo
        ifeval::[]
        bar
        endif::[]
        baz
        EOS

        using_memory_logger do |logger|
          result = (Asciidoctor::Document.new input).reader.read
          assert_equal 'baz', result
          assert_empty logger
        end
      end

"#
            );

            let src = "ifdef::attribute-not-set[]\nfoo\nifeval::[]\nbar\nendif::[]\nbaz";
            let html = convert(src);
            assert!(html.contains("baz"), "{html}");
            assert!(
                conditional_warnings(src, &[]).is_empty(),
                "{:?}",
                conditional_warnings(src, &[])
            );
        }

        #[test]
        fn should_log_error_with_end_position_if_preprocessor_conditional_directive_is_unterminated(
        ) {
            verifies!(
                r#"
      test 'should log error with end position if preprocessor conditional directive is unterminated' do
        input = <<~'EOS'
        before
        ifdef::not-set[]
        skip
        these
        lines
        fin
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'before', (lines * Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 6: detected unterminated preprocessor conditional directive: ifdef::not-set[]', Hash
        end
      end

"#
            );

            let src = "before\nifdef::not-set[]\nskip\nthese\nlines\nfin";
            let html = convert(src);
            assert!(html.contains("before"), "{html}");
            let warnings = conditional_warnings(src, &[]);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::UnterminatedConditionalDirective(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_log_error_with_start_location_if_preprocessor_conditional_directive_is_unterminated_and_sourcemap_is_set(
        ) {
            verifies!(
                r#"
      test 'should log error with start location if preprocessor conditional directive is unterminated and sourcemap is set' do
        input = <<~'EOS'
        before
        ifdef::not-set[]
        skip
        these
        lines
        fin
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input, sourcemap: true
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'before', (lines * Asciidoctor::LF)
          assert_message logger, :ERROR, '~<stdin>: line 2: detected unterminated preprocessor conditional directive: ifdef::not-set[]', Hash
        end
      end

"#
            );

            // This crate has a single unterminated-conditional diagnostic (it does not
            // model Asciidoctor's sourcemap-dependent start/end position choice).
            let src = "before\nifdef::not-set[]\nskip\nthese\nlines\nfin";
            let html = convert(src);
            assert!(html.contains("before"), "{html}");
            let warnings = conditional_warnings(src, &[]);
            assert!(
                warnings
                    .iter()
                    .any(|(w, _)| matches!(w, WarningType::UnterminatedConditionalDirective(_))),
                "{warnings:?}"
            );
        }

        #[test]
        fn should_log_error_if_multiple_preprocessor_conditional_directives_are_unterminated() {
            verifies!(
                r#"
      test 'should log error if multiple preprocessor conditional directives are unterminated' do
        input = <<~'EOS'
        before
        ifdef::not-set[]
        skip
        these
        lines
        ifeval::[1 == 2]
        {asciidoctor-version}
        fin
        EOS

        using_memory_logger do |logger|
          doc = Asciidoctor::Document.new input, sourcemap: true
          reader = doc.reader
          lines = []
          lines << reader.read_line while reader.has_more_lines?
          assert_equal 'before', (lines * Asciidoctor::LF)
          assert_messages logger, [
            [:ERROR, '~<stdin>: line 2: detected unterminated preprocessor conditional directive: ifdef::not-set[]', Hash],
            [:ERROR, '~<stdin>: line 6: detected unterminated preprocessor conditional directive: ifeval::[1 == 2]', Hash],
          ]
        end
      end

"#
            );

            let src = "before\nifdef::not-set[]\nskip\nthese\nlines\nifeval::[1 == 2]\n{asciidoctor-version}\nfin";
            let html = convert(src);
            assert!(html.contains("before"), "{html}");
            let warnings = conditional_warnings(src, &[]);
            let count = warnings
                .iter()
                .filter(|(w, _)| matches!(w, WarningType::UnterminatedConditionalDirective(_)))
                .count();
            assert_eq!(count, 2, "{warnings:?}");
        }

        #[test]
        fn should_not_fail_to_process_preprocessor_directive_that_evaluates_to_false_and_has_a_large_number_of_lines(
        ) {
            verifies!(
                r#"
      test 'should not fail to process preprocessor directive that evaluates to false and has a large number of lines' do
        lines = (%w(data) * 5000) * ?\n
        input = <<~EOS
        before

        ifdef::attribute-not-set[]
        #{lines}
        endif::attribute-not-set[]

        after
        EOS

        doc = Asciidoctor.load input
        assert_equal 2, doc.blocks.size
        assert_equal 'before', doc.blocks[0].source
        assert_equal 'after', doc.blocks[1].source
      end

"#
            );

            // A large false conditional block is skipped without failure; only the
            // surrounding paragraphs remain.
            let big = vec!["data"; 5000].join("\n");
            let src = format!(
                "before\n\nifdef::attribute-not-set[]\n{big}\nendif::attribute-not-set[]\n\nafter"
            );
            let html = convert(&src);
            assert!(html.contains("before"), "{html}");
            assert!(html.contains("after"), "{html}");
            assert!(!html.contains("data"), "large block should be dropped");
        }

        // Non-normative: requires the extension/preprocessor mechanism (out of scope).
        non_normative!(
            r#"
      test 'should not fail to process lines if reader contains a nil entry' do
        input = ['before', '', '', '', 'after']
        doc = Asciidoctor.load input, extensions: proc {
          preprocessor do
            process do |_, reader|
              reader.source_lines[2] = nil
              nil
            end
          end
        }
        assert_equal 2, doc.blocks.size
        assert_equal 'before', doc.blocks[0].source
        assert_equal 'after', doc.blocks[1].source
      end
"#
        );

        non_normative!(
            r#"
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

non_normative!(
    r#"
end
"#
);
