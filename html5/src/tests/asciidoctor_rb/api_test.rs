//! Tracks Asciidoctor's `api_test.rb` – exercises of the public
//! `Asciidoctor.load` / `convert` Ruby API surface.
//!
//! `api_test.rb` is mostly a statement about Asciidoctor's *Ruby* object model:
//! the `Asciidoctor.load` / `load_file` entry points and the parse tree they
//! return (`find_by`, `source_location`/sourcemap, `next_adjacent_block`,
//! `sections?`, `Table::Cell`, `set_option`), plus the file-*writing* mechanics
//! of `convert_file` (`to_dir` / `to_file` / `mkdirs`, adjacent-file output,
//! overwrite protection). Those two families have no counterpart in this crate:
//! the parse-tree traversal API belongs to `asciidoc-parser` (and is exercised
//! by its own suite), and file *output* is the `adoc` binary's job, not the
//! library's (`convert_file` here returns a `String`). Both are tracked
//! `non_normative!` with a reason, honestly accounting for the lines rather
//! than counting them uncovered.
//!
//! What this crate *does* own – and so verifies here – is the library-facing
//! behavior: the `docfile`/`docdir`/`docfilesuffix` input-file attribute family
//! (via [`Document::attribute_value`](crate::Document::attribute_value)),
//! author metadata ([`Document::authors`](crate::Document::authors)), the
//! footer timestamp and the `reproducible` attribute, and the standalone
//! `<head>` stylesheet machinery (`linkcss`, `stylesheet`, `stylesdir`, and the
//! embed-vs-link safe-mode default).
//!
//! A recurring blocker is that many fixtures (`sample.adoc`, and the inline
//! `Title\n====` inputs) use a **setext** document title, which
//! `asciidoc-parser` does not recognize (setext is out of scope for this
//! project); those tests' `doctitle` / `#preamble` assertions cannot hold, so
//! they are tracked `non_normative!` unless their real subject is verifiable
//! another way. The parse-tree traversal API is `asciidoc-parser`'s to verify;
//! the CLI-side file-output tests could be lit up by a companion port under the
//! `cli` crate (its output naming/collision already has CLI tests).

use std::path::{Path, PathBuf};

use asciidoc_parser::document::{Author, InterpretedValue};

use crate::{
    convert_file_with, convert_with, load, load_file_with, load_with,
    tests::{assert_html::assert_css, sdd::*},
    Document, Options, SafeMode,
};

track_file!("ref/asciidoctor/test/api_test.rb");

/// The banner comment that opens the embedded default stylesheet; its presence
/// in a standalone document's `<style>` element is how these tests confirm the
/// stylesheet was *embedded* (Asciidoctor reads back the `<style>` node's
/// content and asserts it is non-empty).
const DEFAULT_STYLESHEET_MARKER: &str = "Asciidoctor default stylesheet";

/// Resolves a name under Asciidoctor's vendored `test/fixtures/` tree – the
/// counterpart to the Ruby suite's `fixture_path`. The fixtures live under the
/// pinned `ref/asciidoctor` tree, so we point at them rather than re-vendoring
/// copies into this crate.
fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ref/asciidoctor/test/fixtures")
        .join(name)
}

/// Extracts a resolved attribute's string value, panicking if it is not set –
/// the counterpart to the Ruby suite reading `doc.attr('name')`.
fn attr_str(doc: &Document, name: &str) -> String {
    match doc.attribute_value(name) {
        InterpretedValue::Value(v) => v,
        other => panic!("attribute {name:?} is {other:?}, expected a value"),
    }
}

/// Asserts an [`Author`]'s decomposed fields, the counterpart to the Ruby
/// suite's per-field `assert_equal author.email` / `.name` / `.firstname` / …
/// checks.
fn assert_author(
    author: &Author,
    name: &str,
    firstname: &str,
    middlename: Option<&str>,
    lastname: Option<&str>,
    initials: &str,
    email: Option<&str>,
) {
    assert_eq!(author.email(), email);
    assert_eq!(author.name(), name);
    assert_eq!(author.firstname(), firstname);
    assert_eq!(author.middlename(), middlename);
    assert_eq!(author.lastname(), lastname);
    assert_eq!(author.initials(), initials);
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'API' do
  context 'Load' do
"#
);

mod load {
    use super::*;

    // Not verified: opens the file through Ruby `File.open` and asserts the
    // setext `doctitle` of `sample.adoc`, a title style this crate does not
    // recognize.
    non_normative!(
        r#"
    test 'should load input file' do
      sample_input_path = fixture_path('sample.adoc')
      doc = File.open(sample_input_path, Asciidoctor::FILE_READ_MODE) {|file| Asciidoctor.load file, safe: Asciidoctor::SafeMode::SAFE }
      assert_equal 'Document Title', doc.doctitle
      assert_equal File.expand_path(sample_input_path), doc.attr('docfile')
      assert_equal File.expand_path(File.dirname(sample_input_path)), doc.attr('docdir')
      assert_equal '.adoc', doc.attr('docfilesuffix')
    end

"#
    );

    #[test]
    fn should_load_input_file_from_filename() {
        verifies!(
            r#"
    test 'should load input file from filename' do
      sample_input_path = fixture_path('sample.adoc')
      doc = Asciidoctor.load_file(sample_input_path, safe: Asciidoctor::SafeMode::SAFE)
      assert_equal 'Document Title', doc.doctitle
      assert_equal File.expand_path(sample_input_path), doc.attr('docfile')
      assert_equal File.expand_path(File.dirname(sample_input_path)), doc.attr('docdir')
      assert_equal '.adoc', doc.attr('docfilesuffix')
    end

"#
        );
        let path = fixture_path("sample.adoc");
        let doc = load_file_with(&path, &Options::new().safe_mode(SafeMode::Safe))
            .expect("read fixtures/sample.adoc");

        // The setext `doctitle` assertion is intentionally not reproduced – this
        // crate does not recognize setext (two-line) titles – so verify instead
        // the input-file attribute family this test derives: Asciidoctor's
        // `File.expand_path` of the source path, its directory, and the suffix.
        // The suffix comparisons normalize `\` to `/` so they hold on Windows.
        let docfile = attr_str(&doc, "docfile");
        assert!(Path::new(&docfile).is_absolute(), "{docfile}");
        assert!(
            docfile
                .replace('\\', "/")
                .ends_with("ref/asciidoctor/test/fixtures/sample.adoc"),
            "{docfile}"
        );
        assert!(attr_str(&doc, "docdir")
            .replace('\\', "/")
            .ends_with("ref/asciidoctor/test/fixtures"));
        assert_eq!(attr_str(&doc, "docfilesuffix"), ".adoc");
    }

    // Not verified: Ruby `Pathname` input and the setext `doctitle` of
    // `sample.adoc`.
    non_normative!(
        r#"
    test 'should load input file from pathname' do
      sample_input_path = Pathname fixture_path 'sample.adoc'
      doc = Asciidoctor.load_file sample_input_path, safe: :safe
      assert_equal 'Document Title', doc.doctitle
      assert_equal sample_input_path.expand_path.to_s, (doc.attr 'docfile')
      assert_equal sample_input_path.expand_path.dirname.to_s, (doc.attr 'docdir')
      assert_equal '.adoc', (doc.attr 'docfilesuffix')
    end

"#
    );

    #[test]
    fn should_load_input_file_with_alternate_file_extension() {
        verifies!(
            r#"
    test 'should load input file with alternate file extension' do
      sample_input_path = fixture_path 'sample-alt-extension.asciidoc'
      doc = Asciidoctor.load_file sample_input_path, safe: :safe
      assert_equal 'Document Title', doc.doctitle
      assert_equal File.expand_path(sample_input_path), doc.attr('docfile')
      assert_equal File.expand_path(File.dirname(sample_input_path)), doc.attr('docdir')
      assert_equal '.asciidoc', doc.attr('docfilesuffix')
    end

"#
        );
        let path = fixture_path("sample-alt-extension.asciidoc");
        let doc = load_file_with(&path, &Options::new().safe_mode(SafeMode::Safe))
            .expect("read fixtures/sample-alt-extension.asciidoc");

        // As with the `.adoc` case above, the setext `doctitle` assertion is not
        // reproduced (setext is unsupported); the behavior under test – that the
        // `docfilesuffix` follows the alternate `.asciidoc` extension – is what
        // we verify. The suffix comparisons normalize `\` to `/` for Windows.
        assert!(attr_str(&doc, "docfile")
            .replace('\\', "/")
            .ends_with("ref/asciidoctor/test/fixtures/sample-alt-extension.asciidoc"));
        assert!(attr_str(&doc, "docdir")
            .replace('\\', "/")
            .ends_with("ref/asciidoctor/test/fixtures"));
        assert_eq!(attr_str(&doc, "docfilesuffix"), ".asciidoc");
    }

    // Not verified: manipulates Ruby's default external/internal `Encoding`;
    // this crate reads UTF-8 source only.
    non_normative!(
        r#"
    test 'should coerce encoding of file to UTF-8' do
      old_external = Encoding.default_external
      old_internal = Encoding.default_internal
      old_verbose = $VERBOSE
      begin
        $VERBOSE = nil # disable warnings since we have to modify constants
        input_path = fixture_path 'encoding.adoc'
        Encoding.default_external = Encoding.default_internal = Encoding::IBM437
        output = Asciidoctor.convert_file input_path, to_file: false, safe: :safe
        assert_equal Encoding::UTF_8, output.encoding
        assert_include 'Romé', output
      ensure
        Encoding.default_external = old_external
        Encoding.default_internal = old_internal
        $VERBOSE = old_verbose
      end
    end

"#
    );

    // Not verified: expects a Ruby `ArgumentError` for a non-UTF-8 file; this
    // crate surfaces such failures as `io::Error` with no matching Ruby
    // encoding setup.
    non_normative!(
        r#"
    test 'should not load file with unrecognized encoding' do
      begin
        tmp_input = Tempfile.new %w(test- .adoc), encoding: Encoding::IBM437
        # NOTE using a character whose code differs between UTF-8 and IBM437
        tmp_input.write %(ƒ\n)
        tmp_input.close
        exception = assert_raises ArgumentError do
          Asciidoctor.load_file tmp_input.path, safe: :safe
        end
        expected_message = 'Failed to load AsciiDoc document - source is either binary or contains invalid Unicode data'
        assert_include expected_message, exception.message
      ensure
        tmp_input.close!
      end
    end

"#
    );

    // Not verified: asserts a Ruby `ArgumentError` and a `reader.rb` backtrace
    // for a binary file.
    non_normative!(
        r#"
    test 'should not load invalid file' do
      sample_input_path = fixture_path('hello-asciidoctor.pdf')
      exception = assert_raises ArgumentError do
        Asciidoctor.load_file(sample_input_path, safe: Asciidoctor::SafeMode::SAFE)
      end
      expected_message = 'Failed to load AsciiDoc document - source is either binary or contains invalid Unicode data'
      assert_include expected_message, exception.message
      # verify we have the correct backtrace (should be at least in the first 5 lines)
      assert_match(/reader\.rb.*prepare_lines/, exception.backtrace[0..4].join(?\n))
    end

"#
    );

    // Not verified: Ruby default-encoding manipulation with file output.
    non_normative!(
        r#"
    # NOTE JRuby for Windows does not permit creating a file with non-Windows-1252 characters in the filename
    test 'should convert filename that contains non-ASCII characters independent of default encodings', unless: (jruby? && windows?) do
      old_external = Encoding.default_external
      old_internal = Encoding.default_internal
      old_verbose = $VERBOSE
      begin
        $VERBOSE = nil # disable warnings since we have to modify constants
        tmp_input = Tempfile.new %w(test-ＵＴＦ８- .adoc)
        tmp_input.write %(ＵＴＦ８\n)
        tmp_input.close
        Encoding.default_external = Encoding.default_internal = Encoding::IBM437
        tmp_output = tmp_input.path.sub '.adoc', '.html'
        Asciidoctor.convert_file tmp_input.path, safe: :safe, attributes: 'linkcss !copycss'
        assert File.exist? tmp_output
        output = File.binread tmp_output
        refute_empty output
        # force encoding to UTF-8 and we should see that the string is in fact UTF-8 encoded
        output = String.new output, encoding: Encoding::UTF_8
        assert_equal Encoding::UTF_8, output.encoding
        assert_include 'ＵＴＦ８', output
      ensure
        tmp_input.close!
        FileUtils.rm_f tmp_output
        Encoding.default_external = old_external
        Encoding.default_internal = old_internal
        $VERBOSE = old_verbose
      end
    end

"#
    );

    // Not verified: Ruby `StringIO` input, the setext `doctitle`, and
    // `doc.base_dir` (no base-dir accessor here).
    non_normative!(
        r#"
    test 'should load input IO' do
      input = StringIO.new <<~'EOS'
      Document Title
      ==============

      preamble
      EOS
      doc = Asciidoctor.load(input, safe: Asciidoctor::SafeMode::SAFE)
      assert_equal 'Document Title', doc.doctitle
      refute doc.attr?('docfile')
      assert_equal doc.base_dir, doc.attr('docdir')
    end

"#
    );

    // Not verified: asserts the setext `doctitle` and `doc.base_dir` (no base-
    // dir accessor here).
    non_normative!(
        r#"
    test 'should load input string' do
      input = <<~'EOS'
      Document Title
      ==============

      preamble
      EOS
      doc = Asciidoctor.load(input, safe: Asciidoctor::SafeMode::SAFE)
      assert_equal 'Document Title', doc.doctitle
      refute doc.attr?('docfile')
      assert_equal doc.base_dir, doc.attr('docdir')
    end

"#
    );

    // Not verified: the Ruby string-array (`String#lines`) input form.
    non_normative!(
        r#"
    test 'should load input string array' do
      input = <<~'EOS'
      Document Title
      ==============

      preamble
      EOS
      doc = Asciidoctor.load(input.lines, safe: Asciidoctor::SafeMode::SAFE)
      assert_equal 'Document Title', doc.doctitle
      refute doc.attr?('docfile')
      assert_equal doc.base_dir, doc.attr('docdir')
    end

"#
    );

    // Not verified: Ruby `nil` input and `doc.blocks` (no public block
    // accessor here).
    non_normative!(
        r#"
    test 'should load nil input' do
      doc = Asciidoctor.load nil, safe: :safe
      refute_nil doc
      assert_empty doc.blocks
    end

"#
    );

    // Not verified: the `:to_file` load option, which has no counterpart in
    // this crate's `load`.
    non_normative!(
        r#"
    test 'should ignore :to_file option if value is truthy but not a string' do
      sample_input_path = fixture_path 'sample.adoc'
      doc = Asciidoctor.load_file sample_input_path, safe: :safe, to_file: true
      refute_nil doc
      assert_equal 'Document Title', doc.doctitle
      assert_equal '.html', (doc.attr 'outfilesuffix')
      assert_equal doc.convert, (Asciidoctor.convert_file sample_input_path, safe: :safe, to_file: false)
    end

"#
    );

    // Not verified: the `:to_file` load option and its `outfilesuffix`
    // derivation, absent from this crate's `load`.
    non_normative!(
        r#"
    test 'should set outfilesuffix attribute to file extension of value of :to_file option if value is a string' do
      sample_input_path = fixture_path 'sample.adoc'
      doc = Asciidoctor.load_file sample_input_path, safe: :safe, to_file: 'out.htm'
      refute_nil doc
      assert_equal 'Document Title', doc.doctitle
      assert_equal '.htm', (doc.attr 'outfilesuffix')
    end

"#
    );

    // Not verified: Asciidoctor's attribute-string/array parsing; this crate's
    // typed `Options` sets attributes individually, with no bulk parse.
    non_normative!(
        r#"
    test 'should accept attributes as array' do
      # NOTE there's a tab character before idseparator
      doc = Asciidoctor.load('text', attributes: %w(toc sectnums   source-highlighter=coderay idprefix	idseparator=-))
      assert_kind_of Hash, doc.attributes
      assert doc.attr?('toc')
      assert_equal '', doc.attr('toc')
      assert doc.attr?('sectnums')
      assert_equal '', doc.attr('sectnums')
      assert doc.attr?('source-highlighter')
      assert_equal 'coderay', doc.attr('source-highlighter')
      assert doc.attr?('idprefix')
      assert_equal '', doc.attr('idprefix')
      assert doc.attr?('idseparator')
      assert_equal '-', doc.attr('idseparator')
    end

"#
    );

    // Not verified: the Ruby `attributes: []` / `assert_kind_of Hash` type
    // check.
    non_normative!(
        r#"
    test 'should accept attributes as empty array' do
      doc = Asciidoctor.load('text', attributes: [])
      assert_kind_of Hash, doc.attributes
    end

"#
    );

    // Not verified: Asciidoctor's attribute-string/array parsing; this crate's
    // typed `Options` sets attributes individually, with no bulk parse.
    non_normative!(
        r#"
    test 'should accept attributes as string' do
      doc = Asciidoctor.load 'text', attributes: %(toc sectnums\nsource-highlighter=coderay\nidprefix\nidseparator=-)
      assert_kind_of Hash, doc.attributes
      assert doc.attr?('toc')
      assert_equal '', doc.attr('toc')
      assert doc.attr?('sectnums')
      assert_equal '', doc.attr('sectnums')
      assert doc.attr?('source-highlighter')
      assert_equal 'coderay', doc.attr('source-highlighter')
      assert doc.attr?('idprefix')
      assert_equal '', doc.attr('idprefix')
      assert doc.attr?('idseparator')
      assert_equal '-', doc.attr('idseparator')
    end

"#
    );

    // Not verified: Asciidoctor's attribute-string escaping/parsing, which the
    // typed `Options` API does not perform.
    non_normative!(
        r#"
    test 'should accept values containing spaces in attributes string' do
      doc = Asciidoctor.load('text', attributes: %(idprefix idseparator=-   note-caption=Note\\ to\\\tself toc))
      assert_kind_of Hash, doc.attributes
      assert doc.attr?('idprefix')
      assert_equal '', doc.attr('idprefix')
      assert doc.attr?('idseparator')
      assert_equal '-', doc.attr('idseparator')
      assert doc.attr?('note-caption')
      assert_equal "Note to\tself", doc.attr('note-caption')
    end

"#
    );

    // Not verified: the Ruby `attributes: ''` / `assert_kind_of Hash` type
    // check.
    non_normative!(
        r#"
    test 'should accept attributes as empty string' do
      doc = Asciidoctor.load('text', attributes: '')
      assert_kind_of Hash, doc.attributes
    end

"#
    );

    // Not verified: the Ruby `attributes: nil` / `assert_kind_of Hash` type
    // check.
    non_normative!(
        r#"
    test 'should accept attributes as nil' do
      doc = Asciidoctor.load('text', attributes: nil)
      assert_kind_of Hash, doc.attributes
    end

"#
    );

    // Not verified: a Ruby duck-typed (`Hashlike`) attributes object.
    non_normative!(
        r#"
    test 'should accept attributes if hash like' do
      class Hashlike
        def initialize
          @table = { 'toc' => '' }
        end

        def keys
          @table.keys
        end

        def [](key)
          @table[key]
        end
      end

      doc = Asciidoctor.load 'text', attributes: Hashlike.new
      assert_kind_of Hash, doc.attributes
      assert doc.attributes.key?('toc')
    end

"#
    );

    #[test]
    fn should_not_expand_value_of_docdir_attribute_if_specified_via_api() {
        verifies!(
            r#"
    test 'should not expand value of docdir attribute if specified via API' do
      docdir = 'virtual/directory'
      doc = document_from_string '', safe: :safe, attributes: { 'docdir' => docdir }
      assert_equal docdir, (doc.attr 'docdir')
      assert_equal docdir, doc.base_dir
    end

"#
        );
        let docdir = "virtual/directory";
        let doc = load_with(
            "",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute("docdir", docdir),
        );
        assert_eq!(
            doc.attribute_value("docdir"),
            InterpretedValue::Value(docdir.to_string())
        );

        // Ruby also asserts `doc.base_dir` equals the unexpanded value; this
        // crate's parsed `Document` exposes no base-dir accessor, so the
        // "docdir set via the API is not expanded" behavior is verified
        // through the stored attribute value alone.
    }

    // Not verified: block-level `convert` on a parse node; this crate renders
    // whole documents, not individual `asciidoc-parser` blocks.
    non_normative!(
        r#"
    test 'converts block to output format when convert is called' do
      doc = Asciidoctor.load 'paragraph text'
      expected = <<~'EOS'.chop
      <div class="paragraph">
      <p>paragraph text</p>
      </div>
      EOS
      assert_equal 1, doc.blocks.length
      assert_equal :paragraph, doc.blocks[0].context
      assert_equal expected, doc.blocks[0].convert
    end

"#
    );

    // Not verified: the Ruby `render`→`convert` method alias.
    non_normative!(
        r#"
    test 'render method on node is aliased to convert method' do
      input = <<~'EOS'
      paragraph text

      * list item
      EOS
      doc = Asciidoctor.load input
      assert_equal 2, doc.blocks.length
      ([doc] + doc.blocks).each do |block|
        assert_equal block.method(:convert), block.method(:render)
      end
      inline = Asciidoctor::Inline.new doc.blocks[0], :image, nil, type: 'image', target: 'tiger.png'
      assert_equal inline.method(:convert), inline.method(:render)
    end

"#
    );

    #[test]
    fn should_output_timestamps_by_default() {
        verifies!(
            r#"
    test 'should output timestamps by default' do
      doc = document_from_string 'text', backend: :html5, attributes: nil
      result = doc.convert
      assert doc.attr?('docdate')
      refute doc.attr? 'reproducible'
      assert_xpath '//div[@id="footer-text" and contains(string(.//text()), "Last updated")]', result, 1
    end

"#
        );
        let doc = load_with("text", &Options::new());
        let result = convert_with("text", &Options::new().standalone(true));
        assert!(doc.has_attribute("docdate"));
        assert!(!doc.has_attribute("reproducible"));

        // The Ruby XPath drills into the footer's descendant text with
        // `string(.//text())`, a form outside this crate's XPath subset; verify
        // the same claim by locating the footer-text div and confirming it
        // carries the "Last updated" timestamp.
        assert_css(&result, "#footer-text", 1);
        assert!(result.contains("Last updated"));
    }

    #[test]
    fn should_not_output_timestamps_if_reproducible_attribute_is_set_in_html_5() {
        verifies!(
            r#"
    test 'should not output timestamps if reproducible attribute is set in HTML 5' do
      doc = document_from_string 'text', backend: :html5, attributes: { 'reproducible' => '' }
      result = doc.convert
      assert doc.attr?('docdate')
      assert doc.attr?('reproducible')
      assert_xpath '//div[@id="footer-text" and contains(string(.//text()), "Last updated")]', result, 0
    end

"#
        );
        let doc = load_with("text", &Options::new().set("reproducible"));
        let result = convert_with("text", &Options::new().standalone(true).set("reproducible"));
        assert!(doc.has_attribute("docdate"));
        assert!(doc.has_attribute("reproducible"));

        // With `reproducible` set the footer omits the timestamp (see the
        // default-timestamp test above for the note on the Ruby XPath form).
        assert!(!result.contains("Last updated"));
    }

    // Not verified: the DocBook backend, which this crate does not implement.
    non_normative!(
        r#"
    test 'should not output timestamps if reproducible attribute is set in DocBook' do
      doc = document_from_string 'text', backend: :docbook, attributes: { 'reproducible' => '' }
      result = doc.convert
      assert doc.attr?('docdate')
      assert doc.attr?('reproducible')
      assert_xpath '/article/info/date', result, 0
    end

"#
    );

    // Not verified: that a Ruby options Hash is not mutated; this crate takes
    // `&Options` by shared reference.
    non_normative!(
        r#"
    test 'should not modify options argument' do
      options = { safe: Asciidoctor::SafeMode::SAFE }
      options.freeze
      sample_input_path = fixture_path('sample.adoc')
      begin
        Asciidoctor.load_file sample_input_path, options
      rescue
        flunk %(options argument should not be modified)
      end
    end

"#
    );

    // Not verified: that a Ruby attributes Hash is not mutated; `Options` is
    // passed by shared reference here.
    non_normative!(
        r#"
    test 'should not modify attributes Hash argument' do
      attributes = {}
      attributes.freeze
      options = {
        safe: Asciidoctor::SafeMode::SAFE,
        attributes: attributes,
      }
      sample_input_path = fixture_path('sample.adoc')
      begin
        Asciidoctor.load_file sample_input_path, options
      rescue
        flunk %(attributes argument should not be modified)
      end
    end

"#
    );

    // Not verified: `doc.restore_attributes`, a parse-model mutation with no
    // counterpart here.
    non_normative!(
        r#"
    test 'should be able to restore header attributes after call to convert' do
      input = <<~'EOS'
      = Document Title
      :foo: bar

      content

      :foo: baz

      content
      EOS
      doc = Asciidoctor.load input
      assert_equal 'bar', (doc.attr 'foo')
      doc.convert
      assert_equal 'baz', (doc.attr 'foo')
      doc.restore_attributes
      assert_equal 'bar', (doc.attr 'foo')
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should track file and line information with blocks if sourcemap option is set' do
      doc = Asciidoctor.load_file fixture_path('sample.adoc'), sourcemap: true

      refute_nil doc.source_location
      assert_equal 'sample.adoc', doc.file
      assert_equal 1, doc.lineno

      preamble = doc.blocks[0]
      refute_nil preamble.source_location
      assert_equal 'sample.adoc', preamble.file
      assert_equal 6, preamble.lineno

      section_1 = doc.sections[0]
      assert_equal 'Section A', section_1.title
      refute_nil section_1.source_location
      assert_equal 'sample.adoc', section_1.file
      assert_equal 10, section_1.lineno

      section_2 = doc.sections[1]
      assert_equal 'Section B', section_2.title
      refute_nil section_2.source_location
      assert_equal 'sample.adoc', section_2.file
      assert_equal 18, section_2.lineno

      table_block = section_2.blocks[1]
      assert_equal :table, table_block.context
      refute_nil table_block.source_location
      assert_equal 'sample.adoc', table_block.file
      assert_equal 22, table_block.lineno
      first_cell = table_block.rows.body[0][0]
      refute_nil first_cell.source_location
      assert_equal 'sample.adoc', first_cell.file
      assert_equal 23, first_cell.lineno
      second_cell = table_block.rows.body[0][1]
      refute_nil second_cell.source_location
      assert_equal 'sample.adoc', second_cell.file
      assert_equal 23, second_cell.lineno
      last_cell = table_block.rows.body[-1][-1]
      refute_nil last_cell.source_location
      assert_equal 'sample.adoc', last_cell.file
      assert_equal 24, last_cell.lineno

      last_block = section_2.blocks[-1]
      assert_equal :ulist, last_block.context
      refute_nil last_block.source_location
      assert_equal 'sample.adoc', last_block.file
      assert_equal 28, last_block.lineno

      list_items = last_block.blocks
      refute_nil list_items[0].source_location
      assert_equal 'sample.adoc', list_items[0].file
      assert_equal 28, list_items[0].lineno

      refute_nil list_items[1].source_location
      assert_equal 'sample.adoc', list_items[1].file
      assert_equal 29, list_items[1].lineno

      refute_nil list_items[2].source_location
      assert_equal 'sample.adoc', list_items[2].file
      assert_equal 30, list_items[2].lineno

      doc = Asciidoctor.load_file fixture_path('main.adoc'), sourcemap: true, safe: :safe

      section_1 = doc.sections[0]
      assert_equal 'Chapter A', section_1.title
      refute_nil section_1.source_location
      assert_equal fixture_path('chapter-a.adoc'), section_1.file
      assert_equal 1, section_1.lineno
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should track file and line information on list items if sourcemap option is set' do
      doc = Asciidoctor.load_file fixture_path('lists.adoc'), sourcemap: true

      first_section = doc.blocks[1]

      unordered_basic_list = first_section.blocks[0]
      assert_equal 11, unordered_basic_list.lineno

      unordered_basic_list_items = unordered_basic_list.find_by context: :list_item
      assert_equal 11, unordered_basic_list_items[0].lineno
      assert_equal 12, unordered_basic_list_items[1].lineno
      assert_equal 13, unordered_basic_list_items[2].lineno

      unordered_max_nesting = first_section.blocks[1]
      assert_equal 16, unordered_max_nesting.lineno
      unordered_max_nesting_items = unordered_max_nesting.find_by context: :list_item
      assert_equal 16, unordered_max_nesting_items[0].lineno
      assert_equal 17, unordered_max_nesting_items[1].lineno
      assert_equal 18, unordered_max_nesting_items[2].lineno
      assert_equal 19, unordered_max_nesting_items[3].lineno
      assert_equal 20, unordered_max_nesting_items[4].lineno
      assert_equal 21, unordered_max_nesting_items[5].lineno

      checklist = first_section.blocks[2]
      assert_equal 24, checklist.lineno
      checklist_list_items = checklist.find_by context: :list_item
      assert_equal 24, checklist_list_items[0].lineno
      assert_equal 25, checklist_list_items[1].lineno
      assert_equal 26, checklist_list_items[2].lineno
      assert_equal 27, checklist_list_items[3].lineno

      ordered_basic = first_section.blocks[3]
      assert_equal 30, ordered_basic.lineno
      ordered_basic_list_items = ordered_basic.find_by context: :list_item
      assert_equal 30, ordered_basic_list_items[0].lineno
      assert_equal 31, ordered_basic_list_items[1].lineno
      assert_equal 32, ordered_basic_list_items[2].lineno

      ordered_nested = first_section.blocks[4]
      assert_equal 35, ordered_nested.lineno
      ordered_nested_list_items = ordered_nested.find_by context: :list_item
      assert_equal 35, ordered_nested_list_items[0].lineno
      assert_equal 36, ordered_nested_list_items[1].lineno
      assert_equal 37, ordered_nested_list_items[2].lineno
      assert_equal 38, ordered_nested_list_items[3].lineno
      assert_equal 39, ordered_nested_list_items[4].lineno

      ordered_max_nesting = first_section.blocks[5]
      assert_equal 42, ordered_max_nesting.lineno
      ordered_max_nesting_items = ordered_max_nesting.find_by context: :list_item
      assert_equal 42, ordered_max_nesting_items[0].lineno
      assert_equal 43, ordered_max_nesting_items[1].lineno
      assert_equal 44, ordered_max_nesting_items[2].lineno
      assert_equal 45, ordered_max_nesting_items[3].lineno
      assert_equal 46, ordered_max_nesting_items[4].lineno
      assert_equal 47, ordered_max_nesting_items[5].lineno

      labeled_singleline = first_section.blocks[6]
      assert_equal 50, labeled_singleline.lineno
      labeled_singleline_items = labeled_singleline.find_by context: :list_item
      assert_equal 50, labeled_singleline_items[0].lineno
      assert_equal 50, labeled_singleline_items[1].lineno
      assert_equal 51, labeled_singleline_items[2].lineno
      assert_equal 51, labeled_singleline_items[3].lineno

      labeled_multiline = first_section.blocks[7]
      assert_equal 54, labeled_multiline.lineno
      labeled_multiline_items = labeled_multiline.find_by context: :list_item
      assert_equal 54, labeled_multiline_items[0].lineno
      assert_equal 55, labeled_multiline_items[1].lineno
      assert_equal 56, labeled_multiline_items[2].lineno
      assert_equal 57, labeled_multiline_items[3].lineno

      qanda = first_section.blocks[8]
      assert_equal 61, qanda.lineno
      qanda_items = qanda.find_by context: :list_item
      assert_equal 61, qanda_items[0].lineno
      assert_equal 62, qanda_items[1].lineno
      assert_equal 63, qanda_items[2].lineno
      assert_equal 63, qanda_items[3].lineno

      mixed = first_section.blocks[9]
      assert_equal 66, mixed.lineno
      mixed_items = mixed.find_by(context: :list_item) {|block| block.text? }
      assert_equal 66, mixed_items[0].lineno
      assert_equal 67, mixed_items[1].lineno
      assert_equal 68, mixed_items[2].lineno
      assert_equal 69, mixed_items[3].lineno
      assert_equal 70, mixed_items[4].lineno
      assert_equal 71, mixed_items[5].lineno
      assert_equal 72, mixed_items[6].lineno
      assert_equal 73, mixed_items[7].lineno
      assert_equal 74, mixed_items[8].lineno
      assert_equal 75, mixed_items[9].lineno
      assert_equal 77, mixed_items[10].lineno
      assert_equal 78, mixed_items[11].lineno
      assert_equal 79, mixed_items[12].lineno
      assert_equal 80, mixed_items[13].lineno
      assert_equal 81, mixed_items[14].lineno
      assert_equal 82, mixed_items[15].lineno
      assert_equal 83, mixed_items[16].lineno

      unordered_complex_list = first_section.blocks[10]
      assert_equal 86, unordered_complex_list.lineno
      unordered_complex_items = unordered_complex_list.find_by context: :list_item
      assert_equal 86, unordered_complex_items[0].lineno
      assert_equal 87, unordered_complex_items[1].lineno
      assert_equal 88, unordered_complex_items[2].lineno
      assert_equal 92, unordered_complex_items[3].lineno
      assert_equal 96, unordered_complex_items[4].lineno
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    # FIXME see #3966
    test 'should assign incorrect lineno for single-line paragraph inside a conditional preprocessor directive' do
      input = <<~'EOS'
      :conditional-attribute:

      before

      ifdef::conditional-attribute[]
      subject
      endif::[]

      after
      EOS

      doc = document_from_string input, sourcemap: true
      # FIXME the second line number should be 6 instead of 7
      assert_equal [3, 7, 9], (doc.find_by context: :paragraph).map(&:lineno)
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should assign correct lineno for multi-line paragraph inside a conditional preprocessor directive' do
      input = <<~'EOS'
      :conditional-attribute:

      before

      ifdef::conditional-attribute[]
      subject
      subject
      endif::[]

      after
      EOS

      doc = document_from_string input, sourcemap: true
      assert_equal [3, 6, 10], (doc.find_by context: :paragraph).map(&:lineno)
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    # NOTE this does not work for a list continuation that attached to a grandparent
    test 'should assign correct source location to blocks that follow a detached list continuation' do
      input = <<~'EOS'
      * parent
       ** child

      +
      paragraph attached to parent

      ****
      sidebar outside list
      ****
      EOS

      doc = document_from_string input, sourcemap: true
      assert_equal [5, 8], (doc.find_by context: :paragraph).map(&:lineno)
    end

"#
    );

    // Not verified: the sourcemap `source_location`/`file`/`lineno` parse-tree
    // API, which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should assign correct source location if section occurs on last line of input' do
      input = <<~'EOS'
      = Document Title

      == Section A

      content

      == Section B
      EOS

      doc = document_from_string input, sourcemap: true
      assert_equal [1, 3, 7], (doc.find_by context: :section).map(&:lineno)
    end

"#
    );

    // Not verified: the deferred-parse (`parse: false` then `doc.parse`) API,
    // which `asciidoc-parser` owns.
    non_normative!(
        r#"
    test 'should allow sourcemap option on document to be modified before document is parsed' do
      doc = Asciidoctor.load_file fixture_path('sample.adoc'), parse: false
      doc.sourcemap = true
      refute doc.parsed?
      doc = doc.parse
      assert doc.parsed?

      section_1 = doc.sections[0]
      assert_equal 'Section A', section_1.title
      refute_nil section_1.source_location
      assert_equal 'sample.adoc', section_1.file
      assert_equal 10, section_1.lineno
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return Array of blocks anywhere in document tree that match criteria' do
      input = <<~'EOS'
      = Document Title

      preamble

      == Section A

      paragraph

      --
      Exhibit A::
      +
      [#tiger.animal]
      image::tiger.png[Tiger]
      --

      image::shoe.png[Shoe]

      == Section B

      paragraph
      EOS

      doc = Asciidoctor.load input
      result = doc.find_by context: :image
      assert_equal 2, result.size
      assert_equal :image, result[0].context
      assert_equal 'tiger.png', result[0].attr('target')
      assert_equal :image, result[1].context
      assert_equal 'shoe.png', result[1].attr('target')
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return an empty Array if no matches are found' do
      input = 'paragraph'
      doc = Asciidoctor.load input
      result = doc.find_by context: :section
      refute_nil result
      assert_equal 0, result.size
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should only return matched node when return value of block argument is :prune' do
      input = <<~'EOS'
      * foo
       ** yin
        *** zen
       ** yang
      * bar
      * baz
      EOS

      doc = Asciidoctor.load input
      result = doc.find_by context: :list_item do |li|
        li.text == 'yin' ? :prune : false
      end
      assert_equal 1, result.size
      assert_equal 'yin', result[0].text
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should discover blocks inside AsciiDoc table cells if traverse_documents selector option is true' do
      input = <<~'EOS'
      paragraph in parent document (before)

      [%footer,cols=2*]
      |===
      a|
      paragraph in nested document (body)
      |normal table cell

      a|
      paragraph in nested document (foot)
      |normal table cell
      |===

      paragraph in parent document (after)
      EOS

      doc = Asciidoctor.load input
      result = doc.find_by context: :paragraph
      assert_equal 2, result.size
      result = doc.find_by context: :paragraph, traverse_documents: true
      assert_equal 4, result.size
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return inner document of AsciiDoc table cell if traverse_documents selector option is true' do
      input = <<~'EOS'
      |===
      a|paragraph in nested document
      |===
      EOS

      doc = Asciidoctor.load input
      inner_doc = doc.blocks[0].rows.body[0][0].inner_document
      result = doc.find_by traverse_documents: true
      assert_include inner_doc, result
      result = doc.find_by context: :inner_document, traverse_documents: true
      assert_equal 1, result.size
      assert_equal inner_doc, result[0]
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should match table cells' do
      input = <<~'EOS'
      |===
      |a |b |c

      |1
      one
      a|NOTE: 2, as it goes.
      l|
      3
       you
        me
      |===
      EOS

      doc = document_from_string input
      table = doc.blocks[0]
      first_head_cell = table.rows.head[0][0]
      first_body_cell = table.rows.body[0][0]
      result = doc.find_by
      assert_include first_head_cell, result
      assert_include first_body_cell, result
      assert_equal 'a', first_head_cell.source
      assert_equal ['a'], first_head_cell.lines
      assert_equal %(1\none), first_body_cell.source
      assert_equal ['1', 'one'], first_body_cell.lines
      result = doc.find_by context: :table_cell, style: :asciidoc
      assert_equal 1, result.size
      assert_kind_of Asciidoctor::Table::Cell, result[0]
      assert_equal :asciidoc, result[0].style
      assert_equal 'NOTE: 2, as it goes.', result[0].source
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return Array of blocks that match style criteria' do
      input = <<~'EOS'
      [square]
      * one
      * two
      * three

      ---

      * apples
      * bananas
      * pears
      EOS

      doc = Asciidoctor.load input
      result = doc.find_by context: :ulist, style: 'square'
      assert_equal 1, result.size
      assert_equal :ulist, result[0].context
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return Array of blocks that match role criteria' do
      input = <<~'EOS'
      [#tiger.animal]
      image::tiger.png[Tiger]

      image::shoe.png[Shoe]
      EOS

      doc = Asciidoctor.load input
      result = doc.find_by context: :image, role: 'animal'
      assert_equal 1, result.size
      assert_equal :image, result[0].context
      assert_equal 'tiger.png', result[0].attr('target')
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return the document title section if context selector is :section' do
      input = <<~'EOS'
      = Document Title

      preamble

      == Section One

      content
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by context: :section
      refute_nil result
      assert_equal 2, result.size
      assert_equal :section, result[0].context
      assert_equal 'Document Title', result[0].title
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should only return results for which the block argument yields true' do
      input = <<~'EOS'
      == Section

      content

      === Subsection

      content
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by(context: :section) {|sect| sect.level == 1 }
      refute_nil result
      assert_equal 1, result.size
      assert_equal :section, result[0].context
      assert_equal 'Section', result[0].title
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should reject node and its children if block returns :reject' do
      input = <<~'EOS'
      paragraph 1

      ====
      paragraph 2

      term::
      +
      paragraph 3
      ====

      paragraph 4
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by do |candidate|
        case candidate.context
        when :example
          :reject
        when :paragraph
          true
        end
      end
      refute_nil result
      assert_equal 2, result.size
      assert_equal :paragraph, result[0].context
      assert_equal :paragraph, result[1].context
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should reject node matched by ID selector if block returns :reject' do
      input = <<~'EOS'
      [.rolename]
      paragraph 1

      [.rolename#idname]
      paragraph 2
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by id: 'idname', role: 'rolename'
      refute_nil result
      assert_equal 1, result.size
      assert_equal doc.blocks[1], result[0]
      result = doc.find_by(id: 'idname', role: 'rolename') { :reject }
      refute_nil result
      assert_equal 0, result.size
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should accept node matched by ID selector if block returns :prune' do
      input = <<~'EOS'
      [.rolename]
      paragraph 1

      [.rolename#idname]
      ====
      paragraph 2
      ====
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by id: 'idname', role: 'rolename'
      refute_nil result
      assert_equal 1, result.size
      assert_equal doc.blocks[1], result[0]
      result = doc.find_by(id: 'idname', role: 'rolename') { :prune }
      refute_nil result
      assert_equal 1, result.size
      assert_equal doc.blocks[1], result[0]
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should accept node but reject its children if block returns :prune' do
      input = <<~'EOS'
      ====
      paragraph 2

      term::
      +
      paragraph 3
      ====
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by do |candidate|
        if candidate.context == :example
          :prune
        end
      end
      refute_nil result
      assert_equal 1, result.size
      assert_equal :example, result[0].context
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should stop looking for blocks when StopIteration is raised' do
      input = <<~'EOS'
      paragraph 1

      ====
      paragraph 2

      ****
      paragraph 3
      ****
      ====

      paragraph 4

      * item
      +
      paragraph 5
      EOS
      doc = Asciidoctor.load input

      stop_at_next = false
      result = doc.find_by do |candidate|
        raise StopIteration if stop_at_next
        if candidate.context == :paragraph
          candidate.parent.context == :sidebar ? (stop_at_next = true) : true
        end
      end
      refute_nil result
      assert_equal 3, result.size
      assert_equal 'paragraph 1', result[0].content
      assert_equal 'paragraph 2', result[1].content
      assert_equal 'paragraph 3', result[2].content
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should stop looking for blocks when filter block returns :stop directive' do
      input = <<~'EOS'
      paragraph 1

      ====
      paragraph 2

      ****
      paragraph 3
      ****
      ====

      paragraph 4

      * item
      +
      paragraph 5
      EOS
      doc = Asciidoctor.load input

      stop_at_next = false
      result = doc.find_by do |candidate|
        next :stop if stop_at_next
        if candidate.context == :paragraph
          candidate.parent.context == :sidebar ? (stop_at_next = true) : true
        end
      end
      refute_nil result
      assert_equal 3, result.size
      assert_equal 'paragraph 1', result[0].content
      assert_equal 'paragraph 2', result[1].content
      assert_equal 'paragraph 3', result[2].content
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should only return one result when matching by id' do
      input = <<~'EOS'
      == Section

      content

      [#subsection]
      === Subsection

      content
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by(context: :section, id: 'subsection')
      refute_nil result
      assert_equal 1, result.size
      assert_equal :section, result[0].context
      assert_equal 'Subsection', result[0].title
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should stop seeking once match is found' do
      input = <<~'EOS'
      == Section

      content

      [#subsection]
      === Subsection

      [#last]
      content
      EOS
      doc = Asciidoctor.load input
      visited_last = false
      result = doc.find_by(id: 'subsection') do |candidate|
        visited_last = true if candidate.id == 'last'
        true
      end
      refute_nil result
      assert_equal 1, result.size
      refute visited_last
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should return an empty Array if the id criteria matches but the block argument yields false' do
      input = <<~'EOS'
      == Section

      content

      [#subsection]
      === Subsection

      content
      EOS
      doc = Asciidoctor.load input
      result = doc.find_by(context: :section, id: 'subsection') {|sect| false }
      refute_nil result
      assert_equal 0, result.size
    end

"#
    );

    // Not verified: `Document#find_by`, the parse-tree traversal API, which
    // belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'find_by should not crash if dlist entry does not have description' do
      input = 'term without description::'
      doc = Asciidoctor.load input
      result = doc.find_by
      refute_nil result
      assert_equal 3, result.size
      assert_kind_of Asciidoctor::Document, result[0]
      assert_kind_of Asciidoctor::List, result[1]
      assert_kind_of Asciidoctor::ListItem, result[2]
    end

"#
    );

    // Not verified: the `List#items` parse-model structure, which belongs to
    // `asciidoc-parser`.
    non_normative!(
        r#"
    test 'dlist item should always have two entries for terms and desc' do
      [
        'term w/o desc::',
        %(term::\nalias::),
        %(primary:: 1\nsecondary:: 2),
      ].each do |input|
        dlist = (Asciidoctor.load input).blocks[0]
        dlist.items.each do |item|
          assert_equal 2, item.size
          assert_kind_of ::Array, item[0]
          assert_kind_of Asciidoctor::ListItem, item[1] if item[1]
        end
      end
    end

"#
    );

    // Not verified: the `Timings` instrumentation API, which this crate does
    // not provide.
    non_normative!(
        r#"
    test 'timings are recorded for each step when load and convert are called separately' do
      sample_input_path = fixture_path 'asciidoc_index.txt'
      (Asciidoctor.load_file sample_input_path, timings: (timings = Asciidoctor::Timings.new)).convert
      refute_equal '0.00000', '%05.5f' % timings.read_parse.to_f
      refute_equal '0.00000', '%05.5f' % timings.convert.to_f
      refute_equal timings.read_parse, timings.total
    end

"#
    );

    // Not verified: the pluggable `:syntax_highlighters` /
    // `:syntax_highlighter_factory` API, which this crate does not expose.
    non_normative!(
        r#"
    test 'can disable syntax highlighter by setting value to nil in :syntax_highlighters option' do
      doc = Asciidoctor.load '', safe: :safe, syntax_highlighters: { 'coderay' => nil }, attributes: { 'source-highlighter' => 'coderay' }
      assert_nil doc.syntax_highlighter
    end

"#
    );

    // Not verified: the pluggable `:syntax_highlighters` /
    // `:syntax_highlighter_factory` API, which this crate does not expose.
    non_normative!(
        r#"
    test 'can substitute a custom syntax highlighter factory instance using the :syntax_highlighter_factory option' do
      input = <<~'EOS'
      [source,ruby]
      ----
      puts 'Hello, World!'
      ----
      EOS
      # NOTE this tests both the lazy loading and the custom factory
      syntax_hl_factory = Asciidoctor::SyntaxHighlighter::CustomFactory.new 'github' => (Asciidoctor::SyntaxHighlighter.for 'html-pipeline')
      doc = Asciidoctor.load input, safe: :safe, syntax_highlighter_factory: syntax_hl_factory, attributes: { 'source-highlighter' => 'github' }
      refute_nil doc.syntax_highlighter
      assert_kind_of Asciidoctor::SyntaxHighlighter::HtmlPipelineAdapter, doc.syntax_highlighter
      assert_include '<pre lang="ruby"><code>', doc.convert
    end

"#
    );

    // Not verified: the pluggable `:syntax_highlighters` /
    // `:syntax_highlighter_factory` API, which this crate does not expose.
    non_normative!(
        r#"
    test 'can substitute an extended syntax highlighter factory implementation using the :syntax_highlighters option' do
      input = <<~'EOS'
      [source,ruby]
      ----
      puts 'Hello, World!'
      ----
      EOS
      syntax_hl_factory_class = Class.new do
        include Asciidoctor::SyntaxHighlighter::DefaultFactory

        def for name
          super 'highlight.js'
        end
      end
      doc = Asciidoctor.load input, safe: :safe, syntax_highlighter_factory: syntax_hl_factory_class.new, attributes: { 'source-highlighter' => 'coderay' }
      refute_nil doc.syntax_highlighter
      output = doc.convert
      refute_include 'CodeRay', output
      assert_include 'hljs', output
    end
"#
    );

    non_normative!(
        r#"
  end

"#
    );
}

mod convert {
    use super::*;

    non_normative!(
        r#"
  context 'Convert' do
"#
    );

    // Not verified: the Ruby `render_file`→`convert_file` method alias.
    non_normative!(
        r#"
    test 'render_file is aliased to convert_file' do
      assert_equal Asciidoctor.method(:convert_file), Asciidoctor.method(:render_file)
    end

"#
    );

    // Not verified: the Ruby `render`→`convert` method alias.
    non_normative!(
        r#"
    test 'render is aliased to convert' do
      assert_equal Asciidoctor.method(:convert), Asciidoctor.method(:render)
    end

"#
    );

    // Not verified: writes output to a file and asserts `#preamble`, which
    // depends on `sample.adoc`'s unrecognized setext title.
    non_normative!(
        r#"
    test 'should convert source document to embedded document when header_footer is false' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('sample.html')

      [{ header_footer: false }, { header_footer: false, to_file: sample_output_path }].each do |opts|
        begin
          Asciidoctor.convert_file sample_input_path, opts
          assert File.exist?(sample_output_path)
          output = File.read(sample_output_path, mode: Asciidoctor::FILE_READ_MODE)
          refute_empty output
          assert_xpath '/html', output, 0
          assert_css '#preamble', output, 1
        ensure
          FileUtils.rm(sample_output_path)
        end
      end
    end

"#
    );

    // Not verified: asserts the setext `doctitle`/`<h1>` of `sample.adoc`, a
    // title style this crate does not recognize.
    non_normative!(
        r#"
    test 'should convert source document to standalone document string when to_file is false and standalone is true' do
      sample_input_path = fixture_path('sample.adoc')

      output = Asciidoctor.convert_file sample_input_path, standalone: true, to_file: false
      refute_empty output
      assert_xpath '/html', output, 1
      assert_xpath '/html/head', output, 1
      assert_xpath '/html/body', output, 1
      assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
      assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
    end

"#
    );

    // Not verified: asserts the setext `doctitle`/`<h1>` of `sample.adoc` (the
    // `header_footer` alias of standalone), a title style this crate does not
    // recognize.
    non_normative!(
        r#"
    test 'should convert source document to standalone document string when to_file is false and header_footer is true' do
      sample_input_path = fixture_path('sample.adoc')

      output = Asciidoctor.convert_file sample_input_path, header_footer: true, to_file: false
      refute_empty output
      assert_xpath '/html', output, 1
      assert_xpath '/html/head', output, 1
      assert_xpath '/html/body', output, 1
      assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
      assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
    end

"#
    );

    #[test]
    fn lines_in_output_should_be_separated_by_line_feed_universal_newline() {
        verifies!(
            r#"
    test 'lines in output should be separated by line feed (universal newline)' do
      sample_input_path = fixture_path('sample.adoc')

      output = Asciidoctor.convert_file sample_input_path, standalone: true, to_file: false
      refute_empty output
      refute_includes output, ?\r
      lines = output.split ?\n
      assert_equal lines.map(&:length), lines.map(&:rstrip).map(&:length)
    end

"#
        );
        let output = convert_file_with(
            fixture_path("sample.adoc"),
            &Options::new().standalone(true),
        )
        .expect("read fixtures/sample.adoc");
        assert!(!output.is_empty());
        assert!(!output.contains('\r'));

        // Every emitted line is already right-trimmed: stripping trailing
        // whitespace leaves each line's length unchanged.
        for line in output.split('\n') {
            assert_eq!(line.len(), line.trim_end().len());
        }
    }

    // Not verified: Asciidoctor's attribute-string/array parsing; this crate's
    // typed `Options` sets attributes individually, with no bulk parse.
    non_normative!(
        r#"
    test 'should accept attributes as array' do
      sample_input_path = fixture_path('sample.adoc')
      output = Asciidoctor.convert_file sample_input_path, attributes: %w(sectnums idprefix idseparator=-), to_file: false
      assert_css '#section-a', output, 1
    end

"#
    );

    // Not verified: Asciidoctor's attribute-string/array parsing; this crate's
    // typed `Options` sets attributes individually, with no bulk parse.
    non_normative!(
        r#"
    test 'should accept attributes as string' do
      sample_input_path = fixture_path('sample.adoc')
      output = Asciidoctor.convert_file sample_input_path, attributes: 'sectnums idprefix idseparator=-', to_file: false
      assert_css '#section-a', output, 1
    end

"#
    );

    #[test]
    fn should_link_to_default_stylesheet_by_default_when_safe_mode_is_secure_or_greater() {
        verifies!(
            r#"
    test 'should link to default stylesheet by default when safe mode is SECURE or greater' do
      sample_input_path = fixture_path('basic.adoc')
      output = Asciidoctor.convert_file sample_input_path, standalone: true, to_file: false
      assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 1
      assert_css 'html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]', output, 1
    end

"#
        );
        let output =
            convert_file_with(fixture_path("basic.adoc"), &Options::new().standalone(true))
                .expect("read fixtures/basic.adoc");
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            1,
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]"#,
            1,
        );
    }

    #[test]
    fn should_embed_default_stylesheet_by_default_if_safemode_is_less_than_secure() {
        verifies!(
            r#"
    test 'should embed default stylesheet by default if SafeMode is less than SECURE' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = Asciidoctor.convert input, safe: Asciidoctor::SafeMode::SERVER, standalone: true
      assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 1
      assert_css 'html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]', output, 0
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
    end

"#
        );
        let input = "= Document Title\n\ntext\n";
        let output = convert_with(
            input,
            &Options::new().standalone(true).safe_mode(SafeMode::Server),
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            1,
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]"#,
            0,
        );
        assert_css(&output, "html:root > head > style", 1);
        assert!(output.contains(DEFAULT_STYLESHEET_MARKER));
    }

    // Not verified: remote stylesheet fetching (`allow-uri-read`), a non-goal
    // for this crate.
    non_normative!(
        r#"
    test 'should embed remote stylesheet by default if SafeMode is less than SECURE and allow-uri-read is set' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = using_test_webserver do
        Asciidoctor.convert input, safe: Asciidoctor::SafeMode::SERVER, standalone: true, attributes: { 'allow-uri-read' => '', 'stylesheet' => %(http://#{resolve_localhost}:9876/fixtures/custom.css) }
      end
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
      assert_include 'color: green', styles
    end

"#
    );

    #[test]
    fn should_not_allow_linkcss_be_unset_from_document_if_safemode_is_secure_or_greater() {
        verifies!(
            r#"
    test 'should not allow linkcss be unset from document if SafeMode is SECURE or greater' do
      input = <<~'EOS'
      = Document Title
      :linkcss!:

      text
      EOS

      output = Asciidoctor.convert input, standalone: true
      assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 1
      assert_css 'html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]', output, 1
    end

"#
        );
        let input = "= Document Title\n:linkcss!:\n\ntext\n";
        let output = convert_with(input, &Options::new().standalone(true));
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            1,
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]"#,
            1,
        );
    }

    #[test]
    fn should_embed_default_stylesheet_if_linkcss_is_unset_from_api_and_safemode_is_secure_or_greater(
    ) {
        verifies!(
            r#"
    test 'should embed default stylesheet if linkcss is unset from API and SafeMode is SECURE or greater' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      [{ 'linkcss!' => '' }, { 'linkcss' => nil }, { 'linkcss' => false }].each do |attrs|
        output = Asciidoctor.convert input, standalone: true, attributes: attrs
        assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 1
        assert_css 'html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]', output, 0
        stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
        styles = stylenode.content
        refute_nil styles
        refute_empty styles.strip
      end
    end

"#
        );
        let input = "= Document Title\n\ntext\n";

        // Asciidoctor accepts three API spellings to unset an attribute
        // (`{'linkcss!'=>''}`, `{'linkcss'=>nil}`, `{'linkcss'=>false}`); all
        // map to `Options::unset` here.
        let output = convert_with(input, &Options::new().standalone(true).unset("linkcss"));
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            1,
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./asciidoctor.css"]"#,
            0,
        );
        assert_css(&output, "html:root > head > style", 1);
        assert!(output.contains(DEFAULT_STYLESHEET_MARKER));
    }

    #[test]
    fn should_embed_default_stylesheet_if_safe_mode_is_less_than_secure_and_linkcss_is_unset_from_api(
    ) {
        verifies!(
            r#"
    test 'should embed default stylesheet if safe mode is less than SECURE and linkcss is unset from API' do
      sample_input_path = fixture_path('basic.adoc')
      output = Asciidoctor.convert_file sample_input_path, standalone: true, to_file: false,
          safe: Asciidoctor::SafeMode::SAFE, attributes: { 'linkcss!' => '' }
      assert_css 'html:root > head > style', output, 1
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
    end

"#
        );
        let output = convert_file_with(
            fixture_path("basic.adoc"),
            &Options::new()
                .standalone(true)
                .safe_mode(SafeMode::Safe)
                .unset("linkcss"),
        )
        .expect("read fixtures/basic.adoc");
        assert_css(&output, "html:root > head > style", 1);
        assert!(output.contains(DEFAULT_STYLESHEET_MARKER));
    }

    #[test]
    fn should_not_link_to_stylesheet_if_stylesheet_is_unset() {
        verifies!(
            r#"
    test 'should not link to stylesheet if stylesheet is unset' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = Asciidoctor.convert input, standalone: true, attributes: { 'stylesheet!' => '' }
      assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 0
      assert_css 'html:root > head > link[rel="stylesheet"]', output, 0
    end

"#
        );
        let input = "= Document Title\n\ntext\n";
        let output = convert_with(input, &Options::new().standalone(true).unset("stylesheet"));
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            0,
        );
        assert_css(&output, r#"html:root > head > link[rel="stylesheet"]"#, 0);
    }

    #[test]
    fn should_link_to_custom_stylesheet_if_specified_in_stylesheet_attribute() {
        verifies!(
            r#"
    test 'should link to custom stylesheet if specified in stylesheet attribute' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = Asciidoctor.convert input, standalone: true, attributes: { 'stylesheet' => './custom.css' }
      assert_css 'html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]', output, 0
      assert_css 'html:root > head > link[rel="stylesheet"][href="./custom.css"]', output, 1

      output = Asciidoctor.convert input, standalone: true, attributes: { 'stylesheet' => 'file:///home/username/custom.css' }
      assert_css 'html:root > head > link[rel="stylesheet"][href="file:///home/username/custom.css"]', output, 1
    end

"#
        );
        let input = "= Document Title\n\ntext\n";
        let output = convert_with(
            input,
            &Options::new()
                .standalone(true)
                .attribute("stylesheet", "./custom.css"),
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href^="https://fonts.googleapis.com"]"#,
            0,
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./custom.css"]"#,
            1,
        );

        let output = convert_with(
            input,
            &Options::new()
                .standalone(true)
                .attribute("stylesheet", "file:///home/username/custom.css"),
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="file:///home/username/custom.css"]"#,
            1,
        );
    }

    #[test]
    fn should_resolve_custom_stylesheet_relative_to_stylesdir() {
        verifies!(
            r#"
    test 'should resolve custom stylesheet relative to stylesdir' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = Asciidoctor.convert input, standalone: true, attributes: { 'stylesheet' => 'custom.css', 'stylesdir' => './stylesheets' }
      assert_css 'html:root > head > link[rel="stylesheet"][href="./stylesheets/custom.css"]', output, 1
    end

"#
        );
        let input = "= Document Title\n\ntext\n";
        let output = convert_with(
            input,
            &Options::new()
                .standalone(true)
                .attribute("stylesheet", "custom.css")
                .attribute("stylesdir", "./stylesheets"),
        );
        assert_css(
            &output,
            r#"html:root > head > link[rel="stylesheet"][href="./stylesheets/custom.css"]"#,
            1,
        );
    }

    // Not verified: reads and embeds a custom stylesheet file from
    // `stylesdir`; custom-stylesheet embedding is tracked in asciidoc-html5#36
    // (this crate embeds only the built-in default stylesheet).
    non_normative!(
        r#"
    test 'should resolve custom stylesheet to embed relative to stylesdir' do
      sample_input_path = fixture_path('basic.adoc')
      output = Asciidoctor.convert_file sample_input_path, standalone: true, safe: Asciidoctor::SafeMode::SAFE, to_file: false,
          attributes: { 'stylesheet' => 'custom.css', 'stylesdir' => './stylesheets', 'linkcss!' => '' }
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
    end

"#
    );

    // Not verified: remote stylesheet fetching (`allow-uri-read`), a non-goal
    // for this crate.
    non_normative!(
        r#"
    test 'should embed custom remote stylesheet if SafeMode is less than SECURE and allow-uri-read is set' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = using_test_webserver do
        Asciidoctor.convert input, safe: Asciidoctor::SafeMode::SERVER, standalone: true, attributes: { 'allow-uri-read' => '', 'stylesheet' => %(http://#{resolve_localhost}:9876/fixtures/custom.css) }
      end
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
      assert_include 'color: green', styles
    end

"#
    );

    // Not verified: a JRuby classloader URI.
    non_normative!(
        r#"
    test 'should embed custom stylesheet read from classloader URI', if: jruby? do
      require fixture_path 'assets.jar'
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = Asciidoctor.convert input, safe: :unsafe, standalone: true, attributes: { 'stylesdir' => 'uri:classloader:/styles-in-jar', 'stylesheet' => 'custom.css' }
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
      assert_include 'color: green', styles
    end

"#
    );

    // Not verified: remote stylesheet fetching (`allow-uri-read`), a non-goal
    // for this crate.
    non_normative!(
        r#"
    test 'should embed custom stylesheet in remote stylesdir if SafeMode is less than SECURE and allow-uri-read is set' do
      input = <<~'EOS'
      = Document Title

      text
      EOS

      output = using_test_webserver do
        Asciidoctor.convert input, safe: Asciidoctor::SafeMode::SERVER, standalone: true, attributes: { 'allow-uri-read' => '', 'stylesdir' => %(http://#{resolve_localhost}:9876/fixtures), 'stylesheet' => 'custom.css' }
      end
      stylenode = xmlnodes_at_css 'html:root > head > style', output, 1
      styles = stylenode.content
      refute_nil styles
      refute_empty styles.strip
      assert_include 'color: green', styles
    end

"#
    );

    // Not verified: the `copycss` file copy driven through `convert_file`
    // `to_dir`/`mkdirs` output; file output is the `adoc` binary's job, and
    // copycss is verified on this project's own docs page.
    non_normative!(
        r#"
    test 'should copy custom stylesheet in folder to same folder in destination dir if copycss is set' do
      begin
        output_dir = fixture_path 'output'
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_path = File.join output_dir, 'sample.html'
        custom_stylesheet_output_path = File.join output_dir, 'stylesheets', 'custom.css'
        Asciidoctor.convert_file sample_input_path, safe: :safe, to_dir: output_dir, mkdirs: true,
            attributes: { 'stylesheet' => 'stylesheets/custom.css', 'linkcss' => '', 'copycss' => '' }
        assert File.exist? sample_output_path
        assert File.exist? custom_stylesheet_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        assert_xpath '/html/head/link[@rel="stylesheet"][@href="./stylesheets/custom.css"]', output, 1
        assert_xpath 'style', output, 0
      ensure
        FileUtils.rm_r output_dir, force: true, secure: true
      end
    end

"#
    );

    // Not verified: the `copycss` file copy driven through `convert_file`
    // `to_dir` output; file output is the `adoc` binary's job, and copycss is
    // verified on this project's own docs page.
    non_normative!(
        r#"
    test 'should copy custom stylesheet to destination dir if copycss is true' do
      begin
        output_dir = fixture_path 'output'
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_path = File.join output_dir, 'sample.html'
        custom_stylesheet_output_path = File.join output_dir, 'custom.css'
        Asciidoctor.convert_file sample_input_path, safe: :safe, to_dir: output_dir, mkdirs: true,
            attributes: { 'stylesheet' => 'custom.css', 'linkcss' => true, 'copycss' => true }
        assert File.exist? sample_output_path
        assert File.exist? custom_stylesheet_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        assert_xpath '/html/head/link[@rel="stylesheet"][@href="./custom.css"]', output, 1
        assert_xpath 'style', output, 0
      ensure
        FileUtils.rm_r output_dir, force: true, secure: true
      end
    end

"#
    );

    // Not verified: the `copycss` file copy driven through `convert_file`
    // `to_dir` output; file output is the `adoc` binary's job, and copycss is
    // verified on this project's own docs page.
    non_normative!(
        r#"
    test 'should copy custom stylesheet to destination dir if copycss is a path string' do
      begin
        output_dir = fixture_path 'output'
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_path = File.join output_dir, 'sample.html'
        custom_stylesheet_output_path = File.join output_dir, 'styles.css'
        Asciidoctor.convert_file sample_input_path,
          safe: :safe, to_dir: output_dir, mkdirs: true, attributes: { 'stylesheet' => 'styles.css', 'linkcss' => true, 'copycss' => 'custom.css' }
        assert_path_exists sample_output_path
        assert_path_exists custom_stylesheet_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        assert_xpath '/html/head/link[@rel="stylesheet"][@href="./styles.css"]', output, 1
        assert_xpath 'style', output, 0
      ensure
        FileUtils.rm_r output_dir, force: true, secure: true
      end
    end

"#
    );

    // Not verified: the `copycss` file copy (Ruby `Pathname` source) driven
    // through `convert_file` `to_dir` output; file output is the `adoc`
    // binary's job.
    non_normative!(
        r#"
    test 'should copy custom stylesheet to destination dir if copycss is a Pathname object' do
      begin
        output_dir = fixture_path 'output'
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_path = File.join output_dir, 'sample.html'
        custom_stylesheet_src_path = Pathname.new fixture_path 'custom.css'
        custom_stylesheet_output_path = File.join output_dir, 'styles.css'
        Asciidoctor.convert_file sample_input_path,
          safe: :safe, to_dir: output_dir, mkdirs: true, attributes: { 'stylesheet' => 'styles.css', 'linkcss' => true, 'copycss' => custom_stylesheet_src_path }
        assert_path_exists sample_output_path
        assert_path_exists custom_stylesheet_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        assert_xpath '/html/head/link[@rel="stylesheet"][@href="./styles.css"]', output, 1
        assert_xpath 'style', output, 0
      ensure
        FileUtils.rm_r output_dir, force: true, secure: true
      end
    end

"#
    );

    // Not verified: a JRuby classloader URI copycss source.
    non_normative!(
        r#"
    test 'should copy custom stylesheet to destination dir if copycss is a classloader URI', if: jruby? do
      require fixture_path 'assets.jar'
      begin
        output_dir = fixture_path 'output'
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_path = File.join output_dir, 'sample.html'
        custom_stylesheet_src_path = 'uri:classloader:/styles-in-jar/custom.css'
        custom_stylesheet_output_path = File.join output_dir, 'styles.css'
        Asciidoctor.convert_file sample_input_path,
          safe: :unsafe, to_dir: output_dir, mkdirs: true, attributes: { 'stylesheet' => 'styles.css', 'linkcss' => true, 'copycss' => custom_stylesheet_src_path }
        assert_path_exists sample_output_path
        assert_path_exists custom_stylesheet_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        assert_xpath '/html/head/link[@rel="stylesheet"][@href="./styles.css"]', output, 1
        assert_xpath 'style', output, 0
      ensure
        FileUtils.rm_r output_dir, force: true, secure: true
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should convert source file and write result to adjacent file by default' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('sample.html')
      begin
        Asciidoctor.convert_file sample_input_path
        assert File.exist?(sample_output_path)
        output = File.read(sample_output_path, mode: Asciidoctor::FILE_READ_MODE)
        refute_empty output
        assert_xpath '/html', output, 1
        assert_xpath '/html/head', output, 1
        assert_xpath '/html/body', output, 1
        assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
        assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should convert source file specified by pathname and write result to adjacent file by default' do
      sample_input_path = Pathname fixture_path 'sample.adoc'
      sample_output_path = Pathname fixture_path 'sample.html'
      begin
        doc = Asciidoctor.convert_file sample_input_path, safe: :safe
        assert_equal sample_output_path.expand_path.to_s, (doc.attr 'outfile')
        assert sample_output_path.file?
        output = sample_output_path.read mode: Asciidoctor::FILE_READ_MODE
        refute_empty output
        assert_xpath '/html', output, 1
        assert_xpath '/html/head', output, 1
        assert_xpath '/html/body', output, 1
        assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
        assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
      ensure
        sample_output_path.delete
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should convert source file and write to specified file' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('result.html')
      begin
        Asciidoctor.convert_file sample_input_path, to_file: sample_output_path
        assert File.exist?(sample_output_path)
        output = File.read(sample_output_path, mode: Asciidoctor::FILE_READ_MODE)
        refute_empty output
        assert_xpath '/html', output, 1
        assert_xpath '/html/head', output, 1
        assert_xpath '/html/body', output, 1
        assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
        assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should convert source file and write to specified file in base_dir' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('result.html')
      fixture_dir = fixture_path('')
      begin
        Asciidoctor.convert_file sample_input_path, to_file: 'result.html', base_dir: fixture_dir
        assert File.exist?(sample_output_path)
        output = File.read(sample_output_path, mode: Asciidoctor::FILE_READ_MODE)
        refute_empty output
        assert_xpath '/html', output, 1
        assert_xpath '/html/head', output, 1
        assert_xpath '/html/body', output, 1
        assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
        assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
      rescue => e
        flunk e.message
      ensure
        FileUtils.rm(sample_output_path, force: true)
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should write file in bin mode and thus not convert line feeds to system-dependent newline' do
      sample_input_path = fixture_path 'sample.adoc'
      sample_output_path = fixture_path 'sample.html'
      begin
        Asciidoctor.convert_file sample_input_path
        assert_path_exists sample_output_path
        output = File.read sample_output_path, mode: Asciidoctor::FILE_READ_MODE
        refute_empty output
        assert_includes output, ?\n
        refute_includes output, ?\r
        assert_includes output, %(<\/body>\n<\/html>)
        refute output.end_with? ?\n
      ensure
        FileUtils.rm sample_output_path
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'should resolve :to_dir option correctly when both :to_dir and :to_file options are set to an absolute path' do
      begin
        sample_input_path = fixture_path 'sample.adoc'
        sample_output_file = Tempfile.new %w(out- .html)
        sample_output_path = sample_output_file.path
        sample_output_dir = File.dirname sample_output_path
        sample_output_file.close
        doc = Asciidoctor.convert_file sample_input_path, to_file: sample_output_path, to_dir: sample_output_dir, safe: :unsafe
        assert File.exist? sample_output_path
        assert_equal sample_output_path, doc.options[:to_file]
        assert_equal sample_output_dir, doc.options[:to_dir]
      ensure
        sample_output_file.close!
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'in_place option is ignored when to_file is specified' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('result.html')
      begin
        Asciidoctor.convert_file sample_input_path, to_file: sample_output_path, in_place: true
        assert File.exist?(sample_output_path)
      ensure
        FileUtils.rm(sample_output_path) if File.exist? sample_output_path
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'in_place option is ignored when to_dir is specified' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('sample.html')
      begin
        Asciidoctor.convert_file sample_input_path, to_dir: File.dirname(sample_output_path), in_place: true
        assert File.exist?(sample_output_path)
      ensure
        FileUtils.rm(sample_output_path) if File.exist? sample_output_path
      end
    end

"#
    );

    // Not verified: the `to_file`-driven `outfilesuffix` derivation; file
    // output is the `adoc` binary's job.
    non_normative!(
        r#"
    test 'should set outfilesuffix to match file extension of target file' do
      sample_input = '{outfilesuffix}'
      sample_output_path = fixture_path('result.htm')
      begin
        Asciidoctor.convert sample_input, to_file: sample_output_path
        assert File.exist?(sample_output_path)
        output = File.read(sample_output_path, mode: Asciidoctor::FILE_READ_MODE)
        refute_empty output
        assert_include '<p>.htm</p>', output
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: the `to_dir`-driven soft-set `outfilesuffix` output; file
    // output is the `adoc` binary's job.
    non_normative!(
        r#"
    test 'should respect outfilesuffix soft set from API' do
      sample_input_path = fixture_path('sample.adoc')
      sample_output_path = fixture_path('sample.htm')
      begin
        Asciidoctor.convert_file sample_input_path, to_dir: (File.dirname sample_input_path), attributes: { 'outfilesuffix' => '.htm@' }
        assert File.exist?(sample_output_path)
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'output should be relative to to_dir option' do
      sample_input_path = fixture_path('sample.adoc')
      output_dir = File.join(File.dirname(sample_input_path), 'test_output')
      Dir.mkdir output_dir unless File.exist? output_dir
      sample_output_path = File.join(output_dir, 'sample.html')
      begin
        Asciidoctor.convert_file sample_input_path, to_dir: output_dir
        assert File.exist? sample_output_path
      ensure
        FileUtils.rm(sample_output_path) if File.exist? sample_output_path
        FileUtils.rmdir output_dir
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'missing directories should be created if mkdirs is enabled' do
      sample_input_path = fixture_path('sample.adoc')
      output_dir = File.join(File.join(File.dirname(sample_input_path), 'test_output'), 'subdir')
      sample_output_path = File.join(output_dir, 'sample.html')
      begin
        Asciidoctor.convert_file sample_input_path, to_dir: output_dir, mkdirs: true
        assert File.exist? sample_output_path
      ensure
        FileUtils.rm(sample_output_path) if File.exist? sample_output_path
        FileUtils.rmdir output_dir
        FileUtils.rmdir File.dirname(output_dir)
      end
    end

"#
    );

    // Not verified: the input-overwrite `IOError` guard, part of
    // `convert_file`'s file-writing path (absent here).
    non_normative!(
        r#"
    # TODO need similar test for when to_dir is specified
    test 'should raise exception if an attempt is made to overwrite input file' do
      sample_input_path = fixture_path('sample.adoc')

      assert_raises IOError do
        Asciidoctor.convert_file sample_input_path, attributes: { 'outfilesuffix' => '.adoc' }
      end
    end

"#
    );

    // Not verified: `convert_file`'s file-*writing* mechanics
    // (`to_file`/`to_dir`/`mkdirs`/adjacent-file output); this crate's
    // `convert_file` returns a `String` and writes no files.
    non_normative!(
        r#"
    test 'to_file should be relative to to_dir when both given' do
      sample_input_path = fixture_path('sample.adoc')
      base_dir = File.dirname(sample_input_path)
      sample_rel_output_path = File.join('test_output', 'result.html')
      output_dir = File.dirname(File.join(base_dir, sample_rel_output_path))
      Dir.mkdir output_dir unless File.exist? output_dir
      sample_output_path = File.join(base_dir, sample_rel_output_path)
      begin
        Asciidoctor.convert_file sample_input_path, to_dir: base_dir, to_file: sample_rel_output_path
        assert File.exist? sample_output_path
      ensure
        FileUtils.rm(sample_output_path) if File.exist? sample_output_path
        FileUtils.rmdir output_dir
      end
    end

"#
    );

    // Not verified: that a Ruby options Hash is not mutated; `Options` is
    // passed by shared reference here.
    non_normative!(
        r#"
    test 'should not modify options argument' do
      options = {
        safe: Asciidoctor::SafeMode::SAFE,
        to_file: false,
      }
      options.freeze
      sample_input_path = fixture_path('sample.adoc')
      begin
        Asciidoctor.convert_file sample_input_path, options
      rescue
        flunk %(options argument should not be modified)
      end
    end

"#
    );

    // Not verified: the `:to_dir` output-option plumbing on `doc.options`,
    // part of the file-writing path (absent here).
    non_normative!(
        r#"
    test 'should set to_dir option to parent directory of specified output file' do
      sample_input_path = fixture_path 'basic.adoc'
      sample_output_path = fixture_path 'basic.html'
      begin
        doc = Asciidoctor.convert_file sample_input_path, to_file: sample_output_path
        assert_equal File.dirname(sample_output_path), doc.options[:to_dir]
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: the `:to_dir` output-option plumbing on `doc.options`,
    // part of the file-writing path (absent here).
    non_normative!(
        r#"
    test 'should set to_dir option to parent directory of specified output directory and file' do
      sample_input_path = fixture_path 'basic.adoc'
      sample_output_path = fixture_path 'basic.html'
      fixture_base_path = File.dirname sample_output_path
      fixture_parent_path = File.dirname fixture_base_path
      sample_output_relpath = File.join 'fixtures', 'basic.html'
      begin
        doc = Asciidoctor.convert_file sample_input_path, to_dir: fixture_parent_path, to_file: sample_output_relpath
        assert_equal fixture_base_path, doc.options[:to_dir]
      ensure
        FileUtils.rm(sample_output_path)
      end
    end

"#
    );

    // Not verified: the `Timings` instrumentation API, which this crate does
    // not provide.
    non_normative!(
        r#"
    test 'timings are recorded for each step' do
      sample_input_path = fixture_path 'asciidoc_index.txt'
      Asciidoctor.convert_file sample_input_path, timings: (timings = Asciidoctor::Timings.new), to_file: false
      refute_equal '0.00000', '%05.5f' % timings.read_parse.to_f
      refute_equal '0.00000', '%05.5f' % timings.convert.to_f
      refute_equal timings.read_parse, timings.total
    end

"#
    );

    // Not verified: the pluggable `:syntax_highlighters` /
    // `:syntax_highlighter_factory` API, which this crate does not expose.
    non_normative!(
        r#"
    test 'can override syntax highlighter using syntax_highlighters option' do
      syntax_hl = Class.new Asciidoctor::SyntaxHighlighter::Base do
        def highlight?
          true
        end

        def highlight node, source, lang, opts
          'highlighted'
        end
      end
      input = <<~'EOS'
      [source,ruby]
      ----
      puts 'Hello, World!'
      ----
      EOS
      output = Asciidoctor.convert input, safe: :safe, syntax_highlighters: { 'coderay' => syntax_hl }, attributes: { 'source-highlighter' => 'coderay' }
      assert_css 'pre.highlight > code[data-lang="ruby"]', output, 1
      assert_xpath '//pre[@class="coderay highlight"]/code[text()="highlighted"]', output, 1
    end
"#
    );

    non_normative!(
        r#"
  end

"#
    );
}

mod ast {
    use super::*;

    non_normative!(
        r#"
  context 'AST' do
"#
    );

    #[test]
    fn with_no_author() {
        verifies!(
            r#"
    test 'with no author' do
      input = <<~'EOS'
      = Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application

      Getting Real details the business, design, programming, and marketing principles of 37signals.
      EOS

      doc = document_from_string input
      assert_equal 0, doc.authors.size
    end

"#
        );
        let doc = load("= Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application\n\nGetting Real details the business, design, programming, and marketing principles of 37signals.\n");
        assert_eq!(doc.authors().len(), 0);
    }

    #[test]
    fn with_one_author() {
        verifies!(
            r#"
    test 'with one author' do
      input = <<~'EOS'
      = Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application
      David Heinemeier Hansson <david@37signals.com>

      Getting Real details the business, design, programming, and marketing principles of 37signals.
      EOS

      doc = document_from_string input
      authors = doc.authors
      assert_equal 1, authors.size
      author_1 = authors[0]
      assert_equal 'david@37signals.com', author_1.email
      assert_equal 'David Heinemeier Hansson', author_1.name
      assert_equal 'David', author_1.firstname
      assert_equal 'Heinemeier', author_1.middlename
      assert_equal 'Hansson', author_1.lastname
      assert_equal 'DHH', author_1.initials
    end

"#
        );
        let doc = load("= Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application\nDavid Heinemeier Hansson <david@37signals.com>\n\nGetting Real details the business, design, programming, and marketing principles of 37signals.\n");
        let authors = doc.authors();
        assert_eq!(authors.len(), 1);
        assert_author(
            &authors[0],
            "David Heinemeier Hansson",
            "David",
            Some("Heinemeier"),
            Some("Hansson"),
            "DHH",
            Some("david@37signals.com"),
        );
    }

    #[test]
    fn with_two_authors() {
        verifies!(
            r#"
    test 'with two authors' do
      input = <<~'EOS'
      = Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application
      David Heinemeier Hansson <david@37signals.com>; Jason Fried <jason@37signals.com>

      Getting Real details the business, design, programming, and marketing principles of 37signals.
      EOS

      doc = document_from_string input
      authors = doc.authors
      assert_equal 2, authors.size
      author_1 = authors[0]
      assert_equal 'david@37signals.com', author_1.email
      assert_equal 'David Heinemeier Hansson', author_1.name
      assert_equal 'David', author_1.firstname
      assert_equal 'Heinemeier', author_1.middlename
      assert_equal 'Hansson', author_1.lastname
      assert_equal 'DHH', author_1.initials
      author_2 = authors[1]
      assert_equal 'jason@37signals.com', author_2.email
      assert_equal 'Jason Fried', author_2.name
      assert_equal 'Jason', author_2.firstname
      assert_nil author_2.middlename
      assert_equal 'Fried', author_2.lastname
      assert_equal 'JF', author_2.initials
    end

"#
        );
        let doc = load("= Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application\nDavid Heinemeier Hansson <david@37signals.com>; Jason Fried <jason@37signals.com>\n\nGetting Real details the business, design, programming, and marketing principles of 37signals.\n");
        let authors = doc.authors();
        assert_eq!(authors.len(), 2);
        assert_author(
            &authors[0],
            "David Heinemeier Hansson",
            "David",
            Some("Heinemeier"),
            Some("Hansson"),
            "DHH",
            Some("david@37signals.com"),
        );
        assert_author(
            &authors[1],
            "Jason Fried",
            "Jason",
            None,
            Some("Fried"),
            "JF",
            Some("jason@37signals.com"),
        );
    }

    #[test]
    fn with_authors_as_attributes() {
        verifies!(
            r#"
    test 'with authors as attributes' do
      input = <<~'EOS'
      = Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application
      :author_1: David Heinemeier Hansson
      :email_1: david@37signals.com
      :author_2: Jason Fried
      :email_2: jason@37signals.com

      Getting Real details the business, design, programming, and marketing principles of 37signals.
      EOS

      doc = document_from_string input
      authors = doc.authors
      assert_equal 2, authors.size
      author_1 = authors[0]
      assert_equal 'david@37signals.com', author_1.email
      assert_equal 'David Heinemeier Hansson', author_1.name
      assert_equal 'David', author_1.firstname
      assert_equal 'Heinemeier', author_1.middlename
      assert_equal 'Hansson', author_1.lastname
      assert_equal 'DHH', author_1.initials
      author_2 = authors[1]
      assert_equal 'jason@37signals.com', author_2.email
      assert_equal 'Jason Fried', author_2.name
      assert_equal 'Jason', author_2.firstname
      assert_nil author_2.middlename
      assert_equal 'Fried', author_2.lastname
      assert_equal 'JF', author_2.initials
    end

"#
        );
        let doc = load("= Getting Real: The Smarter, Faster, Easier Way to Build a Successful Web Application\n:author_1: David Heinemeier Hansson\n:email_1: david@37signals.com\n:author_2: Jason Fried\n:email_2: jason@37signals.com\n\nGetting Real details the business, design, programming, and marketing principles of 37signals.\n");
        let authors = doc.authors();
        assert_eq!(authors.len(), 2);
        assert_author(
            &authors[0],
            "David Heinemeier Hansson",
            "David",
            Some("Heinemeier"),
            Some("Hansson"),
            "DHH",
            Some("david@37signals.com"),
        );
        assert_author(
            &authors[1],
            "Jason Fried",
            "Jason",
            None,
            Some("Fried"),
            "JF",
            Some("jason@37signals.com"),
        );
    }

    // Not verified: direct construction of an `asciidoc-parser` `Table::Cell`,
    // a parse-model API.
    non_normative!(
        r#"
    test 'should not crash if nil cell text is passed to Cell constructor' do
      input = <<~'EOS'
      |===
      |a
      |===
      EOS
      table = (document_from_string input).blocks[0]
      cell = Asciidoctor::Table::Cell.new table.rows.body[0][0].column, nil
      refute cell.style
      assert_same Asciidoctor::AbstractNode::NORMAL_SUBS, cell.subs
      assert_equal '', cell.text
    end

"#
    );

    // Not verified: `AbstractBlock#set_option`/`enabled_options`, parse-model
    // APIs, which belong to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should set option on node when set_option is called' do
      input = <<~'EOS'
      . three
      . two
      . one
      EOS

      block = (document_from_string input).blocks[0]
      block.set_option('reversed')
      assert block.option? 'reversed'
      assert_equal '', block.attributes['reversed-option']
    end

"#
    );

    // Not verified: `AbstractBlock#set_option`/`enabled_options`, parse-model
    // APIs, which belong to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'enabled_options should return all options which are set' do
      input = <<~'EOS'
      [%interactive]
      * [x] code
      * [ ] test
      * [ ] profit
      EOS

      block = (document_from_string input).blocks[0]
      assert_equal %w(checklist interactive).to_set, block.enabled_options
    end

"#
    );

    // Not verified: `AbstractBlock#set_option`/`enabled_options`, parse-model
    // APIs, which belong to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should append option to existing options' do
      input = <<~'EOS'
      [%fancy]
      . three
      . two
      . one
      EOS

      block = (document_from_string input).blocks[0]
      block.set_option('reversed')
      assert block.option? 'fancy'
      assert block.option? 'reversed'
    end

"#
    );

    // Not verified: `AbstractBlock#set_option`/`enabled_options`, parse-model
    // APIs, which belong to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should not append option if option is already set' do
      input = <<~'EOS'
      [%reversed]
      . three
      . two
      . one
      EOS

      block = (document_from_string input).blocks[0]
      refute block.set_option('reversed')
      assert_equal '', block.attributes['reversed-option']
    end

"#
    );

    // Not verified: `AbstractBlock#set_option`/`enabled_options`, parse-model
    // APIs, which belong to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should return set of option names' do
      input = <<~'EOS'
      [%compact%reversed]
      . three
      . two
      . one
      EOS

      block = (document_from_string input).blocks[0]
      assert_equal %w(compact reversed).to_set, block.enabled_options
    end

"#
    );

    // Not verified: the `block?`/`inline?` node predicates on a table
    // column/cell, a parse-model API.
    non_normative!(
        r#"
    test 'table column should not be a block or inline' do
      input = <<~'EOS'
      |===
      |a
      |===
      EOS

      col = (document_from_string input).blocks[0].columns[0]
      refute col.block?
      refute col.inline?
    end

"#
    );

    // Not verified: the `block?`/`inline?` node predicates on a table
    // column/cell, a parse-model API.
    non_normative!(
        r#"
    test 'table cell should be a block' do
      input = <<~'EOS'
      |===
      |a
      |===
      EOS

      cell = (document_from_string input).blocks[0].rows.body[0][0]
      assert_kind_of ::Asciidoctor::AbstractBlock, cell
      assert cell.block?
      refute cell.inline?
    end

"#
    );

    // Not verified: `next_adjacent_block`, a parse-tree navigation method,
    // which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'next_adjacent_block should return next block' do
      input = <<~'EOS'
      first

      second
      EOS

      doc = document_from_string input
      assert_equal doc.blocks[1], doc.blocks[0].next_adjacent_block
    end

"#
    );

    // Not verified: `next_adjacent_block`, a parse-tree navigation method,
    // which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'next_adjacent_block should return next sibling of parent if called on last sibling' do
      input = <<~'EOS'
      --
      first
      --

      second
      EOS

      doc = document_from_string input
      assert_equal doc.blocks[1], doc.blocks[0].blocks[0].next_adjacent_block
    end

"#
    );

    // Not verified: `next_adjacent_block`, a parse-tree navigation method,
    // which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'next_adjacent_block should return next sibling of list if called on last item' do
      input = <<~'EOS'
      * first

      second
      EOS

      doc = document_from_string input
      assert_equal doc.blocks[1], doc.blocks[0].blocks[0].next_adjacent_block
    end

"#
    );

    // Not verified: `next_adjacent_block`, a parse-tree navigation method,
    // which belongs to `asciidoc-parser`.
    non_normative!(
        r#"
    test 'next_adjacent_block should return next item in dlist if called on last block of list item' do
      input = <<~'EOS'
      first::
      desc
      +
      more desc

      second::
      desc
      EOS

      doc = document_from_string input
      assert_equal doc.blocks[0].items[1], doc.blocks[0].items[0][1].blocks[0].next_adjacent_block
    end

"#
    );

    // Not verified: `sections?`, a parse-model predicate, which belongs to
    // `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should return true when sections? is called on a document or section that has sections' do
      input = <<~'EOS'
      = Document Title

      == First Section

      === First subsection

      content
      EOS

      doc = document_from_string input
      assert doc.sections?
      assert doc.blocks[0].sections?
    end

"#
    );

    // Not verified: `sections?`, a parse-model predicate, which belongs to
    // `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should return false when sections? is called on a document with no sections' do
      input = <<~'EOS'
      = Document Title

      content
      EOS

      doc = document_from_string input
      refute doc.sections?
    end

"#
    );

    // Not verified: `sections?`, a parse-model predicate, which belongs to
    // `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should return false when sections? is called on a section with no sections' do
      input = <<~'EOS'
      = Document Title

      == First Section
      EOS

      doc = document_from_string input
      refute doc.blocks[0].sections?
    end

"#
    );

    // Not verified: `sections?`, a parse-model predicate, which belongs to
    // `asciidoc-parser`.
    non_normative!(
        r#"
    test 'should return false when sections? is called on anything that is not a section' do
      input = <<~'EOS'
      .Title
      ====
      I'm not section!
      ====

      [NOTE]
      I'm not a section either!
      EOS

      doc = document_from_string input
      refute doc.blocks[0].sections?
      refute doc.blocks[1].sections?
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
