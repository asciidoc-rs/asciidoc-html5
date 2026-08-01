//! Port of Asciidoctor's `invoker_test.rb`, tracked from the CLI crate.
//!
//! `invoker_test.rb` exercises Asciidoctor's command-line front end
//! (`Asciidoctor::Cli::Invoker`): option parsing, reading from files and
//! standard input, output-file routing (`-o`/`-D`), attribute assignment
//! (`-a`), safe mode, and error paths. `adoc` is a leaner binary than the Ruby
//! CLI, so each ported `#[test]` drives `adoc`'s own pipeline (`Cli` parsing
//! plus `run`/`run_with_input`) and re-expresses the Ruby assertions against
//! what `adoc` exposes: the rendered HTML and the files it writes, rather than
//! the `Asciidoctor::Document` model the Ruby suite can inspect.
//!
//! Kept `non_normative!` are the tests for behavior `adoc` does not have,
//! grouped by why:
//!
//! - Ruby CLI internals with no `adoc` analog: the `Invoker`/`Options`
//!   constructor signatures, the Ruby-only `--eruby`, `-E` stdio encoding, `-r`
//!   require, and `Dir.home` fixtures, and the `-w` flag's Ruby-VM effect of
//!   toggling `$VERBOSE` script warnings (the flag itself is accepted).
//! - Out of scope for this html5-only renderer: other backends (DocBook via
//!   `-b`, manpage), the non-`article` doctypes (`book`, `manpage`, `inline`)
//!   that `-d`/`--doctype` rejects, and custom template engines (`-T`/`-E`
//!   haml/slim).
//! - Tracked for later work: image-based admonition icons (<https://github.com/asciidoc-rs/asciidoc-html5/issues/50>),
//!   and the table of contents that `toc-title` renders into (<https://github.com/asciidoc-rs/asciidoc-html5/issues/86>).
//!
//! The document date/time attributes (`docdate`/`doctime`/`docdatetime`/
//! `docyear` and their `local*` siblings), their `-a` overrides, and
//! `SOURCE_DATE_EPOCH` are now wired through `adoc`: they drive the footer's
//! "Last updated" stamp, which the ported tests assert against. The one
//! remaining gap is a local timezone *offset* derived from the system `TZ`:
//! this toolchain carries no timezone database, so an unpinned clock reads as
//! UTC (the `-d inline` tests below, which also need a doctype `adoc` rejects,
//! stay `non_normative!`).

use std::path::PathBuf;

use asciidoc_html5::{Options, ReferenceTime, SafeMode};
use clap::Parser as _;

use crate::{
    print_usage, resolve_safe_mode, run, run_with_input, run_with_streams, should_report_usage,
    tests::sdd::*, Cli, TEST_STDIN_NOT_A_TERMINAL,
};

track_file!("ref/asciidoctor/test/invoker_test.rb");

/// A throwaway on-disk project rooted at a unique temp directory, used by the
/// file-based tests to write inputs and inspect the outputs `adoc` produces.
struct Project {
    dir: PathBuf,
}

impl Project {
    /// Creates a fresh, empty project directory named for `label`.
    fn new(label: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("adoc-cli-invoker-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create project dir");
        Self { dir }
    }

    /// The absolute path of `relative` within the project.
    fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }

    /// Writes `contents` to `relative`, creating parent directories as needed,
    /// and returns the absolute path written.
    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        std::fs::write(&path, contents).expect("write project file");
        path
    }

    /// Reads `relative` back as a string (empty if it does not exist).
    fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative)).unwrap_or_default()
    }

    /// Whether `relative` exists within the project.
    fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    /// Runs `adoc` with `args` (which already include any input file paths),
    /// capturing the bytes written to standard output. Standard input is never
    /// read because these invocations name real files.
    fn run(&self, args: &[&str]) -> std::io::Result<Vec<u8>> {
        let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
        let mut stdout = Vec::new();
        run(&cli, &mut stdout)?;
        Ok(stdout)
    }

    /// Like [`Project::run`], but drives [`run_with_streams`] so it can capture
    /// standard error (the warning stream) and whether the run reached its
    /// `--failure-level`. Returns `(failure_reached, stderr)`; standard output
    /// is discarded, as these warning tests only inspect stderr and the code.
    fn run_streams(&self, args: &[&str]) -> (bool, String) {
        let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
        let mut stdin = std::io::empty();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let failed = run_with_streams(
            &cli,
            TEST_STDIN_NOT_A_TERMINAL,
            &mut stdin,
            &mut stdout,
            &mut stderr,
        )
        .expect("adoc converts");
        (failed, String::from_utf8(stderr).expect("stderr is UTF-8"))
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Runs `adoc` with `args`, feeding `source` in as standard input, capturing
/// the bytes written to standard output. This drives the real stdin read path
/// via [`run_with_input`], the injectable-reader core of `run`.
fn run_stdin(args: &[&str], source: &str) -> std::io::Result<Vec<u8>> {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout)?;
    Ok(stdout)
}

/// Like [`run_stdin`], but drives [`run_with_streams`] so it can capture
/// standard error (the warning stream) and whether the run reached its
/// `--failure-level`. Returns `(failure_reached, stderr)`; standard output is
/// discarded, as these warning tests only inspect stderr and the code.
fn run_stdin_streams(args: &[&str], source: &str) -> (bool, String) {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let failed = run_with_streams(
        &cli,
        TEST_STDIN_NOT_A_TERMINAL,
        &mut stdin,
        &mut stdout,
        &mut stderr,
    )
    .expect("adoc converts");
    (failed, String::from_utf8(stderr).expect("stderr is UTF-8"))
}

non_normative!(
    r#"
# frozen_string_literal: false
require_relative 'test_helper'
require File.join Asciidoctor::LIB_DIR, 'asciidoctor/cli'

context 'Invoker' do
"#
);

// Ruby-internal: `Asciidoctor::Cli::Options`/`Invoker` accept a pre-built
// options object. `adoc` has no such object — it parses argv straight into a
// clap `Cli` — so there is no equivalent construction path to verify.
non_normative!(
    r#"
  test 'should allow Options to be passed as first argument of constructor' do
    opts = Asciidoctor::Cli::Options.new attributes: { 'toc' => '' }, doctype: 'book', eruby: 'erubis'
    invoker = Asciidoctor::Cli::Invoker.new opts
    assert_same invoker.options, opts
  end

"#
);

// Ruby-internal: same as above for an options `Hash` (`:attributes`,
// `:doctype`, `:eruby`). These are `Invoker` constructor conveniences with no
// `adoc` counterpart.
non_normative!(
    r#"
  test 'should allow options Hash to be passed as first argument of constructor' do
    opts = { attributes: { 'toc' => '' }, doctype: 'book', eruby: 'erubis' }
    invoker = Asciidoctor::Cli::Invoker.new opts
    resolved_opts = invoker.options
    assert_equal opts[:attributes], resolved_opts[:attributes]
    assert_equal 'book', resolved_opts[:attributes]['doctype']
    assert_equal 'erubis', resolved_opts[:eruby]
  end

"#
);

#[test]
fn should_parse_options_from_array_passed_as_first_argument_of_constructor() {
    verifies!(
        r#"
  test 'should parse options from array passed as first argument of constructor' do
    input_file = fixture_path 'basic.adoc'
    invoker = Asciidoctor::Cli::Invoker.new ['-s', input_file]
    resolved_options = invoker.options
    refute resolved_options[:standalone]
    assert_equal [input_file], resolved_options[:input_files]
  end

"#
    );

    // Asciidoctor's `-s` suppresses the header/footer (`standalone == false`).
    // `adoc`'s primary spelling is `-e`/`--embedded`, and it accepts the legacy
    // `-s`/`--no-header-footer` as compatibility aliases, so the Ruby
    // `['-s', file]` array parses directly. Every spelling selects embedded
    // output and records the lone input file.
    let input_file = "test/fixtures/basic.adoc";
    for flag in ["-s", "-e", "--no-header-footer"] {
        let cli = Cli::parse_from(["adoc", flag, input_file]);
        assert!(cli.embedded, "{flag} should select embedded output");
        assert_eq!(cli.inputs, vec![PathBuf::from(input_file)]);
    }
}

// Ruby-internal: the `Invoker.new '-s', file` splat signature. `adoc` parses a
// single argv slice (verified above via the array form), so the splat variant
// has no analog.
non_normative!(
    r#"
  test 'should parse options from multiple arguments passed to constructor' do
    input_file = fixture_path 'basic.adoc'
    invoker = Asciidoctor::Cli::Invoker.new '-s', input_file
    resolved_options = invoker.options
    refute resolved_options[:standalone]
    assert_equal [input_file], resolved_options[:input_files]
  end

"#
);

#[test]
fn should_parse_source_and_convert_to_html5_article_by_default() {
    verifies!(
        r#"
  test 'should parse source and convert to html5 article by default' do
    invoker = nil
    output = nil
    redirect_streams do |out, err|
      invoker = invoke_cli %w(-o -)
      output = out.string
    end
    refute_nil invoker
    doc = invoker.document
    refute_nil doc
    assert_equal 'Document Title', doc.doctitle
    assert_equal 'Doc Writer', doc.attr('author')
    assert_equal 'html5', doc.attr('backend')
    assert_equal '.html', doc.attr('outfilesuffix')
    assert_equal 'article', doc.attr('doctype')
    assert doc.blocks?
    assert_equal :preamble, doc.blocks.first.context
    refute_empty output
    assert_xpath '/html', output, 1
    assert_xpath '/html/head', output, 1
    assert_xpath '/html/body', output, 1
    assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
    assert_xpath '/html/body[@class="article"]/*[@id="header"]/h1[text() = "Document Title"]', output, 1
  end

"#
    );

    // `adoc` renders a standalone html5 article by default. The Ruby test also
    // reads the document model (doctitle, author, backend, doctype); `adoc`
    // exposes only the rendered output, which carries the same facts: the html
    // shell, the title, and an `article` body class with the header `<h1>`.
    let project = Project::new("default-article");
    let input = project.write(
        "sample.adoc",
        "= Document Title\nDoc Writer <thedoc@asciidoctor.org>\n\nPreamble paragraph.\n\n== Section A\n\n*Section A* paragraph.\n",
    );
    let output = String::from_utf8(
        project
            .run(&["-o", "-", input.to_str().unwrap()])
            .expect("adoc converts"),
    )
    .expect("output is UTF-8");

    assert!(output.contains("<!DOCTYPE html>"));
    assert!(output.contains("<html"));
    assert!(output.contains("<head"));
    assert!(output.contains("<body"));
    assert!(output.contains("<title>Document Title</title>"));
    assert!(output.contains(r#"<body class="article""#));
    assert!(output.contains(r#"id="header""#));
    assert!(output.contains("<h1>Document Title</h1>"));
}

// Not exposed: the test reads implicit doc-info attributes off the `Document`
// object. The date/time members (`docdate`, `doctime`, `docdatetime`,
// `docyear`) now surface in the footer's "Last updated" stamp – covered by the
// override and `SOURCE_DATE_EPOCH` tests below – but the path members
// (`docname`, `docfile`, `docdir`) appear nowhere in `adoc`'s rendered HTML, so
// the whole-`Document` assertion this test makes has no `adoc` analog.
non_normative!(
    r#"
  test 'should set implicit doc info attributes' do
    sample_filepath = fixture_path 'sample.adoc'
    sample_filedir = fixturedir
    invoker = invoke_cli_to_buffer %w(-o /dev/null), sample_filepath
    doc = invoker.document
    assert_equal 'sample', doc.attr('docname')
    assert_equal sample_filepath, doc.attr('docfile')
    assert_equal sample_filedir, doc.attr('docdir')
    assert doc.attr?('docdate')
    assert doc.attr?('docyear')
    assert doc.attr?('doctime')
    assert doc.attr?('docdatetime')
    assert_empty invoker.read_output
  end

"#
);

#[test]
fn should_allow_docdate_and_doctime_to_be_overridden() {
    verifies!(
        r#"
  test 'should allow docdate and doctime to be overridden' do
    sample_filepath = fixture_path 'sample.adoc'
    invoker = invoke_cli_to_buffer %w(-o /dev/null -a docdate=2015-01-01 -a doctime=10:00:00-0700), sample_filepath
    doc = invoker.document
    assert doc.attr?('docdate', '2015-01-01')
    assert doc.attr?('docyear', '2015')
    assert doc.attr?('doctime', '10:00:00-0700')
    assert doc.attr?('docdatetime', '2015-01-01 10:00:00-0700')
  end

"#
    );

    // `adoc` does not expose the `Document`'s attributes, but the derived
    // `docdatetime` surfaces in the standalone footer's "Last updated" stamp.
    // Overriding `docdate`/`doctime` with `-a` therefore drives that stamp:
    // `docdatetime` is "{docdate} {doctime}" = "2015-01-01 10:00:00-0700",
    // built from the two explicit overrides exactly as Asciidoctor computes it.
    let project = Project::new("docdate-doctime-override");
    let input = project.write("sample.adoc", "= Sample\n\nBody.\n");
    let output = String::from_utf8(
        project
            .run(&[
                input.to_str().unwrap(),
                "-o",
                "-",
                "-a",
                "docdate=2015-01-01",
                "-a",
                "doctime=10:00:00-0700",
            ])
            .expect("adoc converts"),
    )
    .expect("output is UTF-8");

    assert!(
        output.contains("Last updated 2015-01-01 10:00:00-0700"),
        "footer should show the overridden docdatetime: {output}"
    );
}

#[test]
fn should_accept_document_from_stdin_and_write_to_stdout() {
    verifies!(
        r#"
  test 'should accept document from stdin and write to stdout' do
    invoker = invoke_cli_to_buffer(%w(-e), '-') { 'content' }
    doc = invoker.document
    refute doc.attr?('docname')
    refute doc.attr?('docfile')
    assert_equal Dir.pwd, doc.attr('docdir')
    assert_equal doc.attr('docdate'), doc.attr('localdate')
    assert_equal doc.attr('docyear'), doc.attr('localyear')
    assert_equal doc.attr('doctime'), doc.attr('localtime')
    assert_equal doc.attr('docdatetime'), doc.attr('localdatetime')
    refute doc.attr?('outfile')
    output = invoker.read_output
    refute_empty output
    assert_xpath '/*[@class="paragraph"]/p[text()="content"]', output, 1
  end

"#
    );

    // The document-model date attributes the Ruby test compares aren't exposed
    // by `adoc`; the rendered body is, and piping `content` through `-e -` yields
    // the single `content` paragraph.
    let output = String::from_utf8(run_stdin(&["-e", "-"], "content").expect("adoc converts"))
        .expect("output is UTF-8");
    assert!(output.contains(r#"<div class="paragraph">"#));
    assert!(output.contains("<p>content</p>"));
}

#[test]
fn should_not_fail_to_rewind_input_if_reading_document_from_stdin() {
    verifies!(
        r#"
  test 'should not fail to rewind input if reading document from stdin' do
    begin
      old_stdin = $stdin
      $stdin = StringIO.new 'paragraph'
      invoker = invoke_cli_to_buffer(%w(-e), '-')
      assert_equal 0, invoker.code
      assert_equal 1, invoker.document.blocks.size
    ensure
      $stdin = old_stdin
    end
  end

"#
    );

    // Piping a bare paragraph succeeds (the Ruby test asserts exit code 0) and
    // yields exactly one block — one rendered paragraph.
    let output = String::from_utf8(run_stdin(&["-e", "-"], "paragraph").expect("adoc converts"))
        .expect("output is UTF-8");
    assert_eq!(output.matches("<p>").count(), 1);
    assert!(output.contains("<p>paragraph</p>"));
}

#[test]
fn should_accept_document_from_stdin_and_write_to_output_file() {
    verifies!(
        r#"
  test 'should accept document from stdin and write to output file' do
    sample_outpath = fixture_path 'sample-output.html'
    begin
      invoker = invoke_cli(%W(-e -o #{sample_outpath}), '-') { 'content' }
      doc = invoker.document
      refute doc.attr?('docname')
      refute doc.attr?('docfile')
      assert_equal Dir.pwd, doc.attr('docdir')
      assert_equal doc.attr('docdate'), doc.attr('localdate')
      assert_equal doc.attr('docyear'), doc.attr('localyear')
      assert_equal doc.attr('doctime'), doc.attr('localtime')
      assert_equal doc.attr('docdatetime'), doc.attr('localdatetime')
      assert doc.attr?('outfile')
      assert_equal sample_outpath, doc.attr('outfile')
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(sample_outpath)
    end
  end

"#
    );

    // The Ruby test checks the document's `outfile` attribute and that the file
    // exists; `adoc` exposes the file itself, whose body holds the converted
    // content read from standard input.
    let project = Project::new("stdin-outfile");
    let out = project.path("sample-output.html");
    let out_str = out.to_str().unwrap();
    run_stdin(&["-e", "-o", out_str, "-"], "content").expect("adoc converts");
    assert!(out.exists());
    let html = std::fs::read_to_string(&out).expect("read output file");
    assert!(html.contains("<p>content</p>"));
}

#[test]
fn should_fail_if_input_file_matches_resolved_output_file() {
    verifies!(
        r#"
  test 'should fail if input file matches resolved output file' do
    invoker = invoke_cli_to_buffer %w(-a outfilesuffix=.adoc), 'sample.adoc'
    assert_match(/input file and output file cannot be the same/, invoker.read_error)
  end

"#
    );

    // `-a outfilesuffix=.adoc` derives the output name `sample.adoc` — the input
    // itself — so `adoc` refuses to convert the file onto itself.
    let project = Project::new("input-eq-resolved-output");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let err = project
        .run(&["-a", "outfilesuffix=.adoc", input.to_str().unwrap()])
        .expect_err("input == resolved output fails");
    assert!(err
        .to_string()
        .contains("input file and output file cannot be the same"));
}

#[test]
fn should_fail_if_input_file_matches_specified_output_file() {
    verifies!(
        r#"
  test 'should fail if input file matches specified output file' do
    sample_outpath = fixture_path 'sample.adoc'
    invoker = invoke_cli_to_buffer %W(-o #{sample_outpath}), 'sample.adoc'
    assert_match(/input file and output file cannot be the same/, invoker.read_error)
  end

"#
    );

    // Naming the output the same file as the input (`-o sample.adoc` on
    // `sample.adoc`) is refused the same way.
    let project = Project::new("input-eq-specified-output");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let err = project
        .run(&["-o", input.to_str().unwrap(), input.to_str().unwrap()])
        .expect_err("input == specified output fails");
    assert!(err
        .to_string()
        .contains("input file and output file cannot be the same"));
}

// Test infrastructure: the test builds a Unix named pipe with `mkfifo` and a
// writer thread (skipped on Windows). This exercises the Ruby harness's
// fixture plumbing, not a rendering rule `adoc` states.
non_normative!(
    r#"
  test 'should accept input from named pipe and output to stdout', unless: windows? do
    sample_inpath = fixture_path 'sample-pipe.adoc'
    begin
      %x(mkfifo #{sample_inpath})
      write_thread = Thread.new do
        File.write sample_inpath, 'pipe content'
      end
      invoker = invoke_cli_to_buffer %w(-a stylesheet!), sample_inpath
      result = invoker.read_output
      assert_match(/pipe content/, result)
      write_thread.join
    ensure
      FileUtils.rm_f sample_inpath
    end
  end

"#
);

#[test]
fn should_allow_docdir_to_be_specified_when_input_is_a_string() {
    verifies!(
        r#"
  test 'should allow docdir to be specified when input is a string' do
    expected_docdir = fixturedir
    invoker = invoke_cli_to_buffer(%w(-e --base-dir test/fixtures -o /dev/null), '-') { 'content' }
    doc = invoker.document
    assert_equal expected_docdir, doc.attr('docdir')
    assert_equal expected_docdir, doc.base_dir
  end

"#
    );

    // The Ruby test reads `docdir`/`base_dir` off the document; `adoc` exposes
    // the parsed option instead. `--base-dir` (Asciidoctor's `-B`) sets the base
    // directory even when the source is a stream. Its include-resolution effect
    // is covered by the io-piping page suite; here the parse is the claim.
    let cli = Cli::parse_from(["adoc", "-e", "--base-dir", "test/fixtures", "-o", "-", "-"]);
    assert_eq!(cli.base_dir, Some(PathBuf::from("test/fixtures")));
}

#[test]
fn should_display_version_and_exit() {
    verifies!(
        r#"
  test 'should display version and exit' do
    expected = %(Asciidoctor #{Asciidoctor::VERSION} [https://asciidoctor.org]\nRuntime Environment (#{RUBY_DESCRIPTION}))
    ['--version', '-V'].each do |switch|
      actual = nil
      redirect_streams do |out, err|
        invoke_cli [switch]
        actual = out.string.rstrip
      end
      refute_nil actual
      assert actual.start_with?(expected), %(Expected to print version when using #{switch} switch)
    end
  end

"#
    );

    // Asciidoctor prints `Asciidoctor <version> [https://asciidoctor.org] ...`;
    // `adoc` prints its own identity, `adoc <version>`. Both `--version` and its
    // short `-V` render the line and exit (clap reports this as a `DisplayVersion`
    // parse outcome rather than an error).
    for switch in ["--version", "-V"] {
        let err = Cli::try_parse_from(["adoc", switch])
            .expect_err("--version exits before producing a Cli");
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
        assert!(
            err.to_string().starts_with("adoc "),
            "expected the version line to name the program for {switch}",
        );
    }
}

#[test]
fn should_print_warnings_to_stderr_by_default() {
    verifies!(
        r#"
  test 'should print warnings to stderr by default' do
    input = <<~'EOS'
    1. first
    3. third
    EOS
    warnings = nil
    redirect_streams do |out, err|
      invoke_cli_to_buffer(%w(-o /dev/null), '-') { input }
      warnings = err.string
    end
    assert_match(/WARNING/, warnings)
  end

"#
    );

    // `adoc` prints the parser's warnings to standard error by default. The
    // out-of-sequence list item in the piped document raises one; its formatted
    // line carries the `WARNING` label the Ruby test matches. (Output goes to
    // `-o -`/stdout, which this helper discards — the assertion is on stderr.)
    let (_failed, stderr) = run_stdin_streams(&["-o", "-"], "1. first\n3. third\n");
    assert!(stderr.contains("WARNING"));
}

#[test]
fn should_not_emit_any_unexpected_warnings() {
    verifies!(
        r#"
  test 'should not emit any unexpected warnings' do
    input_path = fixture_path 'basic.adoc'
    output = run_command(asciidoctor_cmd, '-o', '/dev/null', '-w', input_path) {|out| out.read }
    assert_empty output
  end

"#
    );

    // `-w`/`--warnings` is accepted for compatibility (Asciidoctor's `-w` toggles
    // the Ruby interpreter's script warnings, which have no analog here). A clean
    // document emits nothing on stderr under it — the parser reports no warnings,
    // and `-w` does not manufacture any.
    let project = Project::new("no-unexpected-warnings");
    let input = project.write("basic.adoc", "= Document Title\n\nBody paragraph.\n");
    let out = project.path("basic.html");
    let (failed, stderr) =
        project.run_streams(&["-w", "-o", out.to_str().unwrap(), input.to_str().unwrap()]);
    assert!(!failed);
    assert!(stderr.is_empty(), "expected no warnings, got: {stderr}");
}

// Not applicable: `adoc` accepts `-w`/`--warnings` for compatibility (see
// `should_not_emit_any_unexpected_warnings` above), but this test asserts the
// Ruby-VM behavior the flag drives there — toggling `$VERBOSE` so redefining a
// constant emits an interpreter script warning. A native binary has no such VM
// state, so there is nothing to verify.
non_normative!(
    r#"
  test 'should enable script warnings if -w flag is specified' do
    old_verbose, $VERBOSE = $VERBOSE, false
    begin
      warnings = nil
      redirect_streams do |_, err|
        invoke_cli_to_buffer %w(-w -o /dev/null), '-' do
          A_CONST = 10
          A_CONST = 20
        end
        warnings = err.string
      end
      assert_equal false, $VERBOSE
      refute_empty warnings
    ensure
      $VERBOSE = old_verbose
    end
  end

"#
);

#[test]
fn should_silence_warnings_if_q_flag_is_specified() {
    verifies!(
        r#"
  test 'should silence warnings if -q flag is specified' do
    input = <<~'EOS'
    2. second
    3. third
    EOS
    warnings = nil
    redirect_streams do |out, err|
      invoke_cli_to_buffer(%w(-q -o /dev/null), '-') { input }
      warnings = err.string
    end
    assert_equal '', warnings
  end

"#
    );

    // `-q`/`--quiet` silences the warning stream: the out-of-sequence list item
    // still raises a warning, but nothing is written to stderr.
    let (_failed, stderr) = run_stdin_streams(&["-q", "-o", "-"], "2. second\n3. third\n");
    assert_eq!(stderr, "");
}

#[test]
fn should_not_fail_to_check_log_level_when_q_flag_is_specified() {
    verifies!(
        r#"
  test 'should not fail to check log level when -q flag is specified' do
    input = <<~'EOS'
    skip to <<install>>

    . download
    . install[[install]]
    . run
    EOS
    begin
      old_stderr, $stderr = $stderr, ::StringIO.new
      old_stdout, $stdout = $stdout, ::StringIO.new
      invoker = invoke_cli(%w(-q), '-') { input }
      assert_equal 0, invoker.code
    ensure
      $stderr = old_stderr
      $stdout = old_stdout
    end
  end

"#
    );

    // Under `-q`, computing the failure code (which consults each diagnostic's
    // severity) must not itself fail the run: with the default failure level of
    // `FATAL`, this document's diagnostics leave the exit code at success.
    let input = "skip to <<install>>\n\n. download\n. install[[install]]\n. run\n";
    let (failed, _stderr) = run_stdin_streams(&["-q", "-o", "-"], input);
    assert!(!failed);
}

#[test]
fn should_return_non_zero_exit_code_if_failure_level_is_reached() {
    verifies!(
        r#"
  test 'should return non-zero exit code if failure level is reached' do
    input = <<~'EOS'
    1. first
    3. third
    EOS
    exit_code, messages = redirect_streams do |_, err|
      [invoke_cli(%w(-q --failure-level=WARN -o /dev/null), '-') { input }.code, err.string]
    end
    assert_equal 1, exit_code
    assert messages.empty?
  end

"#
    );

    // `--failure-level=WARN` makes a warning fail the run: the out-of-sequence
    // list item raises one, so `adoc` exits non-zero (the `failure_reached` flag
    // the binary maps to a `1` exit code). `-q` still silences the stream, so the
    // failure is signaled without any message — matching the Ruby test's assertion
    // that the exit code is 1 while stderr stays empty.
    let (failed, stderr) = run_stdin_streams(
        &["-q", "--failure-level=WARN", "-o", "-"],
        "1. first\n3. third\n",
    );
    assert!(failed);
    assert!(stderr.is_empty(), "expected no messages, got: {stderr}");
}

#[test]
fn should_report_usage_if_no_input_file_given() {
    verifies!(
        r#"
  test 'should report usage if no input file given' do
    redirect_streams do |out, err|
      invoke_cli [], nil
      assert_match(/Usage:/, err.string)
    end
  end

"#
    );

    // With no input argument at an interactive terminal, `adoc` prints usage
    // rather than blocking on standard input, matching Asciidoctor, which prints
    // its option summary when no input file is given. Drive the whole pipeline:
    // `run_with_streams` diverts to writing the `Usage:` summary to stderr and
    // reports a non-zero exit, reading no standard input and producing no output.
    let cli = Cli::parse_from(["adoc"]);
    let mut stdin = std::io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let failed = run_with_streams(&cli, true, &mut stdin, &mut stdout, &mut stderr)
        .expect("the usage path does not error");
    let stderr = String::from_utf8(stderr).expect("stderr is UTF-8");
    assert!(failed, "a bare invocation at a terminal exits non-zero");
    assert!(stderr.contains("Usage:"), "{stderr}");
    assert!(stdout.is_empty(), "the usage path converts nothing");

    // Piped input (standard input is not a terminal) and an explicit `-` still
    // read standard input, preserving the piping design.
    assert!(!should_report_usage(&cli.inputs, false));
    assert!(!should_report_usage(
        &Cli::parse_from(["adoc", "-"]).inputs,
        true
    ));

    // The usage text alone matches the Ruby test's `/Usage:/`.
    let mut usage = Vec::new();
    print_usage(&mut usage).expect("write usage");
    assert!(String::from_utf8(usage)
        .expect("usage is UTF-8")
        .contains("Usage:"));

    // An invalid option is still reported specifically, not masked by usage: a
    // no-input terminal run with an unsupported `-b` fails with the backend
    // error, since the option checks run before the usage divert.
    let cli = Cli::parse_from(["adoc", "-b", "docbook5"]);
    let mut stdin = std::io::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let err = run_with_streams(&cli, true, &mut stdin, &mut stdout, &mut stderr)
        .expect_err("an unsupported backend fails even on the terminal path");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("docbook5"), "{err}");
    assert!(
        stderr.is_empty(),
        "no usage is printed when an option is invalid"
    );
}

#[test]
fn should_report_error_if_input_file_does_not_exist() {
    verifies!(
        r#"
  test 'should report error if input file does not exist' do
    redirect_streams do |out, err|
      invoker = invoke_cli [], 'missing_file.adoc'
      assert_match(/input file .* is missing/, err.string)
      assert_equal 1, invoker.code
    end
  end

"#
    );

    // Reading a missing input file fails the pipeline (the binary maps this to
    // exit code 1 with an `adoc:`-prefixed message on stderr — the Ruby test's
    // `input file ... is missing` / `code == 1`).
    let project = Project::new("missing");
    let missing = project.path("missing_file.adoc");
    let _ = std::fs::remove_file(&missing);
    let err = project
        .run(&[missing.to_str().unwrap()])
        .expect_err("a missing input file fails");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn should_treat_extra_arguments_as_files() {
    verifies!(
        r#"
  test 'should treat extra arguments as files' do
    redirect_streams do |out, err|
      invoker = invoke_cli %w(-o /dev/null extra arguments sample.adoc), nil
      assert_match(/input file .* is missing/, err.string)
      assert_equal 1, invoker.code
    end
  end

"#
    );

    // `extra` and `arguments` name no file, so — like Asciidoctor treating extra
    // arguments as inputs — `adoc` tries to read `extra` first and fails.
    let project = Project::new("extra-args");
    let sample = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let out = project.path("out.html");
    let err = project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "extra",
            "arguments",
            sample.to_str().unwrap(),
        ])
        .expect_err("a missing extra argument fails");
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn should_output_to_file_name_based_on_input_file_name() {
    verifies!(
        r#"
  test 'should output to file name based on input file name' do
    sample_outpath = fixture_path 'sample.html'
    begin
      invoker = invoke_cli
      doc = invoker.document
      assert_equal sample_outpath, doc.attr('outfile')
      assert File.exist?(sample_outpath)
      output = File.read(sample_outpath, mode: Asciidoctor::FILE_READ_MODE)
      refute_empty output
      assert_xpath '/html', output, 1
      assert_xpath '/html/head', output, 1
      assert_xpath '/html/body', output, 1
      assert_xpath '/html/head/title[text() = "Document Title"]', output, 1
      assert_xpath '/html/body/*[@id="header"]/h1[text() = "Document Title"]', output, 1
    ensure
      FileUtils.rm_f(sample_outpath)
    end
  end

"#
    );

    // With no `-o`, `adoc` derives the output name by swapping the extension for
    // `.html`, writing `sample.html` next to the input.
    let project = Project::new("derived-name");
    let input = project.write("sample.adoc", "= Document Title\n\n== Section A\n\nx\n");
    project
        .run(&[input.to_str().unwrap()])
        .expect("adoc converts");
    assert!(project.exists("sample.html"));
    let html = project.read("sample.html");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<title>Document Title</title>"));
    assert!(html.contains("<h1>Document Title</h1>"));
}

#[test]
fn should_output_to_file_in_destination_directory_if_set() {
    verifies!(
        r#"
  test 'should output to file in destination directory if set' do
    destination_path = File.join testdir, 'test_output'
    sample_outpath = File.join destination_path, 'sample.html'
    begin
      FileUtils.mkdir_p(destination_path)
      # QUESTION should -D be relative to working directory or source directory?
      invoker = invoke_cli %w(-D test/test_output)
      #invoker = invoke_cli %w(-D ../../test/test_output)
      doc = invoker.document
      assert_equal sample_outpath, doc.attr('outfile')
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rmdir(destination_path)
    end
  end

"#
    );

    // `-D` writes the derived output into the destination directory.
    let project = Project::new("dest-dir");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let dest = project.path("test_output");
    project
        .run(&["-D", dest.to_str().unwrap(), input.to_str().unwrap()])
        .expect("adoc converts");
    assert!(dest.join("sample.html").exists());
}

#[test]
fn should_preserve_directory_structure_in_destination_directory_if_source_directory_is_set() {
    verifies!(
        r#"
  test 'should preserve directory structure in destination directory if source directory is set' do
    sample_inpath = 'subdir/index.adoc'
    destination_path = 'test_output'
    destination_subdir_path = File.join destination_path, 'subdir'
    sample_outpath = File.join destination_subdir_path, 'index.html'
    begin
      FileUtils.mkdir_p(destination_path)
      invoke_cli %W(-D #{destination_path} -R test/fixtures), sample_inpath
      assert File.directory?(destination_subdir_path)
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rmdir(destination_subdir_path)
      FileUtils.rmdir(destination_path)
    end
  end

"#
    );

    // `-R`/`--source-dir` names a source root so that `-D` recreates the input's
    // subdirectory beneath it. Here the input `fixtures/subdir/index.adoc` sits
    // one level under the source root `fixtures`, so its output lands in the
    // mirrored `subdir` under the destination, not flat in it.
    let project = Project::new("source-dir");
    let input = project.write("fixtures/subdir/index.adoc", "= Index\n\nBody.\n");
    let dest = project.path("test_output");
    let source_dir = project.path("fixtures");
    project
        .run(&[
            "-D",
            dest.to_str().unwrap(),
            "-R",
            source_dir.to_str().unwrap(),
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(project.path("test_output/subdir").is_dir());
    assert!(project.exists("test_output/subdir/index.html"));

    // Without `-R`, the same input flattens to the destination directory by base
    // name — the behavior `-R` opts out of.
    let project = Project::new("source-dir-flat");
    let input = project.write("fixtures/subdir/index.adoc", "= Index\n\nBody.\n");
    let dest = project.path("test_output");
    project
        .run(&["-D", dest.to_str().unwrap(), input.to_str().unwrap()])
        .expect("adoc converts");
    assert!(project.exists("test_output/index.html"));
    assert!(!project.exists("test_output/subdir/index.html"));
}

#[test]
fn should_output_to_file_specified() {
    verifies!(
        r#"
  test 'should output to file specified' do
    sample_outpath = fixture_path 'sample-output.html'
    begin
      invoker = invoke_cli %W(-o #{sample_outpath})
      doc = invoker.document
      assert_equal sample_outpath, doc.attr('outfile')
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(sample_outpath)
    end
  end

"#
    );

    // `-o` writes the output to the named file.
    let project = Project::new("outfile");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let out = project.path("sample-output.html");
    project
        .run(&["-o", out.to_str().unwrap(), input.to_str().unwrap()])
        .expect("adoc converts");
    assert!(out.exists());
}

#[test]
fn should_copy_coderay_stylesheet_to_target_directory_if_linkcss_is_specified() {
    verifies!(
        r#"
  test 'should copy default stylesheet to target directory if linkcss is specified' do
    sample_outpath = fixture_path 'sample-output.html'
    asciidoctor_stylesheet = fixture_path 'asciidoctor.css'
    coderay_stylesheet = fixture_path 'coderay-asciidoctor.css'
    begin
      invoke_cli %W(-o #{sample_outpath} -a linkcss -a source-highlighter=coderay), 'source-block.adoc'
      assert_path_exists(sample_outpath)
      assert_path_exists(asciidoctor_stylesheet)
      assert_path_exists(coderay_stylesheet)
      [sample_outpath, asciidoctor_stylesheet, coderay_stylesheet].each do |path|
        contents = File.read path, mode: Asciidoctor::FILE_READ_MODE
        assert_includes contents, ?\n
        refute_includes contents, ?\r
        refute contents.end_with? ?\n
      end
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rm_f(asciidoctor_stylesheet)
      FileUtils.rm_f(coderay_stylesheet)
    end
  end

"#
    );

    // With `linkcss` and `source-highlighter=coderay`, a document that contains
    // a source block gets both companion stylesheets copied next to the output:
    // the default `asciidoctor.css` and the CodeRay `coderay-asciidoctor.css`.
    let project = Project::new("linkcss-coderay");
    let input = project.write(
        "source-block.adoc",
        "= Doc\n\n[source,ruby]\n----\nputs 1\n----\n",
    );
    let out = project.path("sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "source-highlighter=coderay",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(project.exists("asciidoctor.css"));
    assert!(project.exists("coderay-asciidoctor.css"));

    // Both copied stylesheets have the line-ending discipline the Ruby test
    // asserts: each uses LF, carries no CR, and — because Asciidoctor `rstrip`s
    // the stylesheet data it writes — ends without a trailing newline.
    for name in ["asciidoctor.css", "coderay-asciidoctor.css"] {
        let contents = project.read(name);
        assert!(contents.contains('\n'), "{name} should contain LF");
        assert!(!contents.contains('\r'), "{name} should carry no CR");
        assert!(
            !contents.ends_with('\n'),
            "{name} should have no trailing newline"
        );
    }

    // Deviation from the Ruby test: it applies that same discipline to the
    // rendered HTML output too, but `adoc` appends a final newline to the HTML
    // — a separate convention from the stylesheet copies this test covers.
}

#[test]
fn should_not_copy_coderay_stylesheet_when_no_source_blocks_were_highlighted() {
    verifies!(
        r#"
  test 'should not copy coderay stylesheet to target directory when no source blocks where highlighted' do
    sample_outpath = fixture_path 'sample-output.html'
    asciidoctor_stylesheet = fixture_path 'asciidoctor.css'
    coderay_stylesheet = fixture_path 'coderay-asciidoctor.css'
    begin
      invoke_cli %W(-o #{sample_outpath} -a linkcss -a source-highlighter=coderay)
      assert File.exist?(sample_outpath)
      assert File.exist?(asciidoctor_stylesheet)
      refute File.exist?(coderay_stylesheet)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rm_f(asciidoctor_stylesheet)
      FileUtils.rm_f(coderay_stylesheet)
    end
  end

"#
    );

    // A document with no source block highlights nothing, so — even with
    // `linkcss` and `source-highlighter=coderay` — `adoc` copies the default
    // stylesheet but writes no `coderay-asciidoctor.css`.
    let project = Project::new("linkcss-coderay-nosrc");
    let input = project.write("sample.adoc", "= Doc\n\nJust a paragraph.\n");
    let out = project.path("sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "source-highlighter=coderay",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(project.exists("asciidoctor.css"));
    assert!(!project.exists("coderay-asciidoctor.css"));
}

#[test]
fn should_not_copy_default_stylesheet_to_target_directory_if_linkcss_is_set_and_copycss_is_unset() {
    verifies!(
        r#"
  test 'should not copy default stylesheet to target directory if linkcss is set and copycss is unset' do
    sample_outpath = fixture_path 'sample-output.html'
    default_stylesheet = fixture_path 'asciidoctor.css'
    begin
      invoker = invoke_cli %W(-o #{sample_outpath} -a linkcss -a copycss!)
      invoker.document
      assert File.exist?(sample_outpath)
      refute File.exist?(default_stylesheet)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rm_f(default_stylesheet)
    end
  end

"#
    );

    // With `linkcss` set but `copycss` unset, the HTML links the stylesheet yet
    // `adoc` writes no `asciidoctor.css` next to it.
    let project = Project::new("linkcss-nocopy");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let out = project.path("sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "copycss!",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(!project.exists("asciidoctor.css"));
}

#[test]
fn should_copy_custom_stylesheet_to_target_directory_if_stylesheet_and_linkcss_is_specified() {
    verifies!(
        r#"
  test 'should copy custom stylesheet to target directory if stylesheet and linkcss is specified' do
    destdir = fixture_path 'output'
    sample_outpath = File.join destdir, 'sample-output.html'
    stylesdir = File.join destdir, 'styles'
    custom_stylesheet = File.join stylesdir, 'custom.css'
    begin
      invoker = invoke_cli %W(-o #{sample_outpath} -a linkcss -a copycss=stylesheets/custom.css -a stylesdir=./styles -a stylesheet=custom.css)
      invoker.document
      assert File.exist?(sample_outpath)
      assert File.exist?(custom_stylesheet)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rm_f(custom_stylesheet)
      FileUtils.rmdir(stylesdir)
      FileUtils.rmdir(destdir)
    end
  end

"#
    );

    // A custom `stylesheet` under a `copycss=<path>` source is copied to
    // `stylesdir/custom.css` beneath the output directory.
    let project = Project::new("custom-css-copy");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    project.write("stylesheets/custom.css", "body { color: red; }\n");
    let out = project.path("output/sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "copycss=stylesheets/custom.css",
            "-a",
            "stylesdir=./styles",
            "-a",
            "stylesheet=custom.css",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(project.exists("output/styles/custom.css"));
}

#[test]
fn should_not_copy_custom_stylesheet_to_target_directory_if_stylesheet_and_linkcss_are_set_and_copycss_is_unset(
) {
    verifies!(
        r#"
  test 'should not copy custom stylesheet to target directory if stylesheet and linkcss are set and copycss is unset' do
    destdir = fixture_path 'output'
    sample_outpath = File.join destdir, 'sample-output.html'
    stylesdir = File.join destdir, 'styles'
    custom_stylesheet = File.join stylesdir, 'custom.css'
    begin
      invoker = invoke_cli %W(-o #{sample_outpath} -a linkcss -a stylesdir=./styles -a stylesheet=custom.css -a copycss!)
      invoker.document
      assert File.exist?(sample_outpath)
      refute File.exist?(custom_stylesheet)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rm_f(custom_stylesheet)
      FileUtils.rmdir(stylesdir) if File.directory? stylesdir
      FileUtils.rmdir(destdir)
    end
  end

"#
    );

    // The same setup with `copycss!` links the custom stylesheet but copies
    // nothing.
    let project = Project::new("custom-css-nocopy");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    project.write("styles/custom.css", "body {}\n");
    let out = project.path("output/sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "stylesdir=./styles",
            "-a",
            "stylesheet=custom.css",
            "-a",
            "copycss!",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(!project.exists("output/styles/custom.css"));
}

#[test]
fn should_not_copy_custom_stylesheet_to_target_directory_if_stylesdir_is_a_uri() {
    verifies!(
        r#"
  test 'should not copy custom stylesheet to target directory if stylesdir is a URI' do
    destdir = fixture_path 'output'
    sample_outpath = File.join destdir, 'sample-output.html'
    stylesdir = File.join destdir, 'http:'
    begin
      invoker = invoke_cli %W(-o #{sample_outpath} -a linkcss -a stylesdir=http://example.org/styles -a stylesheet=custom.css)
      invoker.document
      assert File.exist?(sample_outpath)
      refute File.exist?(stylesdir)
    ensure
      FileUtils.rm_f(sample_outpath)
      FileUtils.rmdir(stylesdir) if File.directory? stylesdir
      FileUtils.rmdir(destdir)
    end
  end

"#
    );

    // A URI `stylesdir` is not a local directory, so nothing is copied under the
    // output.
    let project = Project::new("custom-css-uri");
    let input = project.write("sample.adoc", "= Doc\n\nBody.\n");
    let out = project.path("output/sample-output.html");
    project
        .run(&[
            "-o",
            out.to_str().unwrap(),
            "-a",
            "linkcss",
            "-a",
            "stylesdir=http://example.org/styles",
            "-a",
            "stylesheet=custom.css",
            input.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(out.exists());
    assert!(!project.path("output/http:").exists());
}

#[test]
fn should_convert_all_passed_files() {
    verifies!(
        r#"
  test 'should convert all passed files' do
    basic_outpath = fixture_path 'basic.html'
    sample_outpath = fixture_path 'sample.html'
    begin
      invoke_cli_with_filenames [], %w(basic.adoc sample.adoc)
      assert File.exist?(basic_outpath)
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(basic_outpath)
      FileUtils.rm_f(sample_outpath)
    end
  end

"#
    );

    // Passing several files converts each to its own derived `.html`.
    let project = Project::new("all-files");
    let basic = project.write("basic.adoc", "= Basic\n\nBody.\n");
    let sample = project.write("sample.adoc", "= Sample\n\nBody.\n");
    project
        .run(&[basic.to_str().unwrap(), sample.to_str().unwrap()])
        .expect("adoc converts");
    assert!(project.exists("basic.html"));
    assert!(project.exists("sample.html"));
}

#[test]
fn options_should_not_be_modified_when_processing_multiple_files() {
    verifies!(
        r#"
  test 'options should not be modified when processing multiple files' do
    destination_path = File.join testdir, 'test_output'
    basic_outpath = File.join destination_path, 'basic.htm'
    sample_outpath = File.join destination_path, 'sample.htm'
    begin
      invoke_cli_with_filenames %w(-D test/test_output -a outfilesuffix=.htm), %w(basic.adoc sample.adoc)
      assert File.exist?(basic_outpath)
      assert File.exist?(sample_outpath)
    ensure
      FileUtils.rm_f(basic_outpath)
      FileUtils.rm_f(sample_outpath)
      FileUtils.rmdir(destination_path)
    end
  end

"#
    );

    // Converting several files does not mutate the shared options: each is
    // written into the `-D` destination under its own derived name, here with the
    // `outfilesuffix=.htm` override applied to both.
    let project = Project::new("multi-file-options");
    let basic = project.write("basic.adoc", "= Basic\n\nBody.\n");
    let sample = project.write("sample.adoc", "= Sample\n\nBody.\n");
    let dest = project.path("test_output");
    project
        .run(&[
            "-D",
            dest.to_str().unwrap(),
            "-a",
            "outfilesuffix=.htm",
            basic.to_str().unwrap(),
            sample.to_str().unwrap(),
        ])
        .expect("adoc converts");
    assert!(dest.join("basic.htm").exists());
    assert!(dest.join("sample.htm").exists());
}

#[test]
fn should_convert_all_files_that_matches_a_glob_expression() {
    verifies!(
        r#"
  test 'should convert all files that matches a glob expression' do
    basic_outpath = fixture_path 'basic.html'
    begin
      invoke_cli_to_buffer [], "ba*.adoc"
      assert File.exist?(basic_outpath)
    ensure
      FileUtils.rm_f(basic_outpath)
    end
  end

"#
    );

    // A glob argument expands to its matches; `ba*.adoc` converts `basic.adoc`.
    // (Given as an absolute pattern so it does not depend on the test's working
    // directory; the absolute-pattern case follows.)
    let project = Project::new("glob");
    project.write("basic.adoc", "= Basic\n\nBody.\n");
    let glob = project.path("ba*.adoc");
    project
        .run(&[glob.to_str().unwrap()])
        .expect("adoc converts");
    assert!(project.exists("basic.html"));
}

#[test]
fn should_convert_all_files_that_matches_an_absolute_path_glob_expression() {
    verifies!(
        r#"
  test 'should convert all files that matches an absolute path glob expression' do
    basic_outpath = fixture_path 'basic.html'
    glob = fixture_path 'ba*.adoc'
    # test Windows using backslash-style pathname
    if File::ALT_SEPARATOR == '\\'
      glob = glob.tr '/', '\\'
    end

    begin
      invoke_cli_to_buffer [], glob
      assert File.exist?(basic_outpath)
    ensure
      FileUtils.rm_f(basic_outpath)
    end
  end

"#
    );

    // An absolute-path glob expands the same way.
    let project = Project::new("abs-glob");
    project.write("basic.adoc", "= Basic\n\nBody.\n");
    let glob = project.path("ba*.adoc");
    assert!(glob.is_absolute());
    project
        .run(&[glob.to_str().unwrap()])
        .expect("adoc converts");
    assert!(project.exists("basic.html"));
}

#[test]
fn should_suppress_header_footer_if_specified() {
    verifies!(
        r#"
  test 'should suppress header footer if specified' do
    # NOTE this verifies support for the legacy alias -s
    [%w(-e -o -), %w(-s -o -)].each do |flags|
      invoker = invoke_cli_to_buffer flags
      output = invoker.read_output
      assert_xpath '/html', output, 0
      assert_xpath '/*[@id="preamble"]', output, 1
    end
  end

"#
    );

    // The Ruby test iterates `-e` and its legacy alias `-s`; `adoc` accepts both
    // (`-e`/`--embedded` primary, `-s`/`--no-header-footer` as compatibility
    // aliases). Each drops the `<html>` shell yet, for a titled document with a
    // section, still emits the preamble wrapper.
    for flag in ["-e", "-s", "--no-header-footer"] {
        let output = String::from_utf8(
            run_stdin(&[flag, "-"], "= T\n\nPreamble.\n\n== Section\n\nbody\n")
                .expect("adoc converts"),
        )
        .expect("output is UTF-8");
        assert!(
            !output.contains("<html"),
            "{flag} should drop the html shell"
        );
        assert!(output.contains(r#"id="preamble""#));
    }
}

// Out of scope: the manpage backend (`-b manpage`), including writing a `.so`
// redirect page per alternate manname. This crate renders only the html5
// backend.
non_normative!(
    r#"
  test 'should write page for each alternate manname' do
    outdir = fixturedir
    outfile_1 = File.join outdir, 'eve.1'
    outfile_2 = File.join outdir, 'islifeform.1'
    input = <<~'EOS'
    = eve(1)
    Andrew Stanton
    v1.0.0
    :doctype: manpage
    :manmanual: EVE
    :mansource: EVE

    == NAME

    eve, islifeform - analyzes an image to determine if it's a picture of a life form

    == SYNOPSIS

    *eve* ['OPTION']... 'FILE'...
    EOS

    begin
      invoke_cli(%W(-b manpage -o #{outfile_1}), '-') { input }
      assert File.exist?(outfile_1)
      assert File.exist?(outfile_2)
      assert_equal '.so eve.1', (File.read outfile_2, mode: Asciidoctor::FILE_READ_MODE).chomp
    ensure
      FileUtils.rm_f outfile_1
      FileUtils.rm_f outfile_2
    end
  end

"#
);

#[test]
fn should_output_a_trailing_newline_to_stdout() {
    verifies!(
        r#"
  test 'should output a trailing newline to stdout' do
    invoker = nil
    output = nil
    redirect_streams do |out, err|
      invoker = invoke_cli %w(-o -)
      output = out.string
    end
    refute_nil invoker
    refute_nil output
    assert output.end_with?("\n")
  end

"#
    );

    // Output written to standard output ends with a newline.
    let output =
        String::from_utf8(run_stdin(&["-o", "-", "-"], "= T\n\nx\n").expect("adoc converts"))
            .expect("output is UTF-8");
    assert!(output.ends_with('\n'));
}

#[test]
fn should_set_backend_to_html5_if_specified() {
    verifies!(
        r#"
  test 'should set backend to html5 if specified' do
    invoker = invoke_cli_to_buffer %w(-b html5 -o -)
    doc = invoker.document
    assert_equal 'html5', doc.attr('backend')
    assert_equal '.html', doc.attr('outfilesuffix')
    output = invoker.read_output
    assert_xpath '/html', output, 1
  end

"#
    );

    // `-b html5` selects the backend explicitly. html5 is the only backend
    // `adoc` produces, so passing it is accepted purely for command-line
    // compatibility and yields the same standalone html5 document as the
    // default: a `<!DOCTYPE html>` prologue and an `<html>` root element.
    let output = String::from_utf8(
        run_stdin(&["-b", "html5", "-o", "-", "-"], "= T\n\nx\n").expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains("<!DOCTYPE html>"));
    assert!(output.contains("<html"));
}

// Deliberate divergence: the DocBook backend (`-b docbook5`). `adoc` models
// only the html5 backend, so `-b docbook5` is rejected rather than producing
// DocBook output.
non_normative!(
    r#"
  test 'should set backend to docbook5 if specified' do
    invoker = invoke_cli_to_buffer %w(-b docbook5 -a xmlns -o -)
    doc = invoker.document
    assert_equal 'docbook5', doc.attr('backend')
    assert_equal '.xml', doc.attr('outfilesuffix')
    output = invoker.read_output
    assert_xpath '/xmlns:article', output, 1
  end

"#
);

#[test]
fn should_set_doctype_to_article_if_specified() {
    verifies!(
        r#"
  test 'should set doctype to article if specified' do
    invoker = invoke_cli_to_buffer %w(-d article -o -)
    doc = invoker.document
    assert_equal 'article', doc.attr('doctype')
    output = invoker.read_output
    assert_xpath '/html/body[@class="article"]', output, 1
  end

"#
    );

    // `article` is the only doctype `adoc` models, and its default, so `-d
    // article` is accepted for `asciidoctor` compatibility as a no-op (the other
    // doctypes are rejected — see below). The Ruby test reads `doctype` off the
    // document model; `adoc` exposes only the rendered output, which carries the
    // same fact: a standalone article renders with an `article` body class.
    let project = Project::new("doctype-article");
    let input = project.write("sample.adoc", "= Document Title\n\nBody paragraph.\n");
    let output = String::from_utf8(
        project
            .run(&["-d", "article", "-o", "-", input.to_str().unwrap()])
            .expect("adoc converts with -d article"),
    )
    .expect("output is UTF-8");

    assert!(output.contains(r#"<body class="article""#));
}

// Out of scope: `adoc` models only the `article` doctype (`-d article` is
// verified above), so `-d book` is rejected rather than rendering a book with a
// `book` body class.
non_normative!(
    r#"
  test 'should set doctype to book if specified' do
    invoker = invoke_cli_to_buffer %w(-d book -o -)
    doc = invoker.document
    assert_equal 'book', doc.attr('doctype')
    output = invoker.read_output
    assert_xpath '/html/body[@class="book"]', output, 1
  end

"#
);

// Out of scope: `adoc`'s `-d`/`--doctype` flag rejects `-d inline` (it models
// only `article`), so there is no CLI path to the `inline` doctype's 'no inline
// candidate' warning.
non_normative!(
    r#"
  test 'should warn if doctype is inline and the first block is not a candidate for inline conversion' do
    ['== Section Title', 'image::tiger.png[]'].each do |input|
      warnings = redirect_streams do |out, err|
        invoke_cli_to_buffer(%w(-d inline), '-') { input }
        err.string
      end
      assert_match(/WARNING: no inline candidate/, warnings)
    end
  end

"#
);

// Out of scope: the `inline` doctype's empty-document case; `-d inline` is
// rejected here (see above).
non_normative!(
    r#"
  test 'should not warn if doctype is inline and the document has no blocks' do
    warnings = redirect_streams do |out, err|
      invoke_cli_to_buffer(%w(-d inline), '-') { '// comment' }
      err.string
    end
    refute_match(/WARNING/, warnings)
  end

"#
);

// Out of scope: the `inline` doctype's multi-block case; `-d inline` is
// rejected here (see above).
non_normative!(
    r#"
  test 'should not warn if doctype is inline and the document contains multiple blocks' do
    warnings = redirect_streams do |out, err|
      invoke_cli_to_buffer(%w(-d inline), '-') { %(paragraph one\n\nparagraph two\n\nparagraph three) }
      err.string
    end
    refute_match(/WARNING/, warnings)
  end

"#
);

// Out of scope: custom template converters located by `-T`/`-E` (haml/slim).
// `adoc` is a fixed Rust renderer with no template-engine layer.
non_normative!(
    r#"
  test 'should locate custom templates based on template dir, template engine and backend' do
    custom_backend_root = fixture_path 'custom-backends'
    invoker = invoke_cli_to_buffer %W(-E haml -T #{custom_backend_root} -o -)
    doc = invoker.document
    assert_kind_of Asciidoctor::Converter::CompositeConverter, doc.converter
    selected = doc.converter.find_converter 'paragraph'
    assert_kind_of Asciidoctor::Converter::TemplateConverter, selected
    assert_kind_of haml_template_class, selected.templates['paragraph']
  end

"#
);

// Out of scope: loading custom templates from multiple `-T` directories. Same
// template-engine gap as above.
non_normative!(
    r#"
  test 'should load custom templates from multiple template directories' do
    custom_backend_1 = fixture_path 'custom-backends/haml/html5'
    custom_backend_2 = fixture_path 'custom-backends/haml/html5-tweaks'
    invoker = invoke_cli_to_buffer %W(-T #{custom_backend_1} -T #{custom_backend_2} -o - -e)
    output = invoker.read_output
    assert_css '.paragraph', output, 0
    assert_css '#preamble > .sectionbody > p', output, 1
  end

"#
);

#[test]
fn should_set_attribute_with_value() {
    verifies!(
        r#"
  test 'should set attribute with value' do
    invoker = invoke_cli_to_buffer %w(--trace -a idprefix=id -e -o -)
    doc = invoker.document
    assert_equal 'id', doc.attr('idprefix')
    output = invoker.read_output
    assert_xpath '//h2[@id="idsection_a"]', output, 1
  end

"#
    );

    // Asciidoctor passes `--trace`; `adoc` has no such flag, so it is dropped.
    // `-a idprefix=id` overrides the id prefix, so the section's generated id is
    // `idsection_a`.
    let output = String::from_utf8(
        run_stdin(
            &["-a", "idprefix=id", "-e", "-"],
            "= T\n\n== Section A\n\nx\n",
        )
        .expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains(r#"id="idsection_a""#));
}

#[test]
fn should_set_attribute_with_value_containing_equal_sign() {
    verifies!(
        r#"
  test 'should set attribute with value containing equal sign' do
    invoker = invoke_cli_to_buffer %w(--trace -a toc -a toc-title=t=o=c -o -)
    doc = invoker.document
    assert_equal 't=o=c', doc.attr('toc-title')
    output = invoker.read_output
    assert_xpath '//*[@id="toctitle"][text() = "t=o=c"]', output, 1
  end

"#
    );

    // Asciidoctor verifies `toc-title=t=o=c` through the rendered TOC (not yet
    // rendered here). The claim under test is that the value keeps every `=` after
    // the first — `adoc` splits an `-a` spec on the first `=` only — shown here by
    // substituting the whole `a=b=c` value through an attribute reference.
    let output = String::from_utf8(
        run_stdin(&["-a", "myattr=a=b=c", "-e", "-"], "= T\n\n{myattr}\n").expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains("a=b=c"));
}

#[test]
fn should_set_attribute_with_quoted_value_containing_a_space() {
    verifies!(
        r#"
  test 'should set attribute with quoted value containing a space' do
    # emulating commandline arguments: --trace -a toc -a note-caption="Note to self:" -o -
    invoker = invoke_cli_to_buffer %w(--trace -a toc -a note-caption=Note\ to\ self: -o -)
    doc = invoker.document
    assert_equal 'Note to self:', doc.attr('note-caption')
    output = invoker.read_output
    assert_xpath %(//*[#{contains_class('admonitionblock')}]//*[@class='title'][text() = 'Note to self:']), output, 1
  end

"#
    );

    // The shell strips the quotes, handing `adoc` a single `note-caption=Note to
    // self:` argument; the space survives in the value and becomes the caption of
    // the NOTE admonition.
    let output = String::from_utf8(
        run_stdin(
            &["-a", "note-caption=Note to self:", "-e", "-"],
            "= T\n\nNOTE: text\n",
        )
        .expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains("Note to self:"));
}

#[test]
fn should_not_set_attribute_ending_in_at_if_defined_in_document() {
    verifies!(
        r#"
  test 'should not set attribute ending in @ if defined in document' do
    invoker = invoke_cli_to_buffer %w(--trace -a idprefix=id@ -e -o -)
    doc = invoker.document
    assert_equal 'id_', doc.attr('idprefix')
    output = invoker.read_output
    assert_xpath '//h2[@id="id_section_a"]', output, 1
  end

"#
    );

    // `-a idprefix=id@` is a soft default: the document's own `:idprefix: id_`
    // wins, so ids use the `id_` prefix (`id_section_a`), not `id`.
    let output = String::from_utf8(
        run_stdin(
            &["-a", "idprefix=id@", "-e", "-"],
            "= T\n:idprefix: id_\n\n== Section A\n\nx\n",
        )
        .expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains(r#"id="id_section_a""#));
}

// Not implemented: `-a icons` (bare) selects image-based admonition icons
// (`<img alt="Note">`). `adoc` accepts the bare-attribute syntax but does not
// render image icons. Tracked in
// <https://github.com/asciidoc-rs/asciidoc-html5/issues/50>.
non_normative!(
    r#"
  test 'should set attribute with no value' do
    invoker = invoke_cli_to_buffer %w(-a icons -e -o -)
    doc = invoker.document
    assert_equal '', doc.attr('icons')
    output = invoker.read_output
    assert_xpath '//*[@class="admonitionblock note"]//img[@alt="Note"]', output, 1
  end

"#
);

#[test]
fn should_unset_attribute_ending_in_bang() {
    verifies!(
        r#"
  test 'should unset attribute ending in bang' do
    invoker = invoke_cli_to_buffer %w(-a sectids! -e -o -)
    doc = invoker.document
    refute doc.attr?('sectids')
    output = invoker.read_output
    # leave the count loose in case we add more sections
    assert_xpath '//h2[not(@id)]', output
  end

"#
    );

    // `-a sectids!` unsets `sectids`, so the section heading gets no id.
    let output = String::from_utf8(
        run_stdin(&["-a", "sectids!", "-e", "-"], "= T\n\n== Section A\n\nx\n")
            .expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains("<h2>"));
    assert!(!output.contains(r#"<h2 id="#));
}

#[test]
fn default_mode_for_cli_should_be_unsafe() {
    verifies!(
        r#"
  test 'default mode for cli should be unsafe' do
    invoker = invoke_cli_to_buffer %w(-o /dev/null)
    doc = invoker.document
    assert_equal Asciidoctor::SafeMode::UNSAFE, doc.safe
  end

"#
    );

    let cli = Cli::parse_from(["adoc", "-o", "/dev/null", "doc.adoc"]);
    assert_eq!(
        resolve_safe_mode(&cli).expect("valid safe mode"),
        SafeMode::Unsafe
    );
}

#[test]
fn should_set_safe_mode_if_specified() {
    verifies!(
        r#"
  test 'should set safe mode if specified' do
    invoker = invoke_cli_to_buffer %w(--safe -o /dev/null)
    doc = invoker.document
    assert_equal Asciidoctor::SafeMode::SAFE, doc.safe
  end

"#
    );

    let cli = Cli::parse_from(["adoc", "--safe", "-o", "/dev/null", "doc.adoc"]);
    assert_eq!(
        resolve_safe_mode(&cli).expect("valid safe mode"),
        SafeMode::Safe
    );
}

#[test]
fn should_set_safe_mode_to_specified_level() {
    verifies!(
        r#"
  test 'should set safe mode to specified level' do
    levels = {
      'unsafe' => Asciidoctor::SafeMode::UNSAFE,
      'safe'   => Asciidoctor::SafeMode::SAFE,
      'server' => Asciidoctor::SafeMode::SERVER,
      'secure' => Asciidoctor::SafeMode::SECURE,
    }
    levels.each do |name, const|
      invoker = invoke_cli_to_buffer %W(-S #{name} -o /dev/null)
      doc = invoker.document
      assert_equal const, doc.safe
    end
  end

"#
    );

    for (name, mode) in [
        ("unsafe", SafeMode::Unsafe),
        ("safe", SafeMode::Safe),
        ("server", SafeMode::Server),
        ("secure", SafeMode::Secure),
    ] {
        let cli = Cli::parse_from(["adoc", "-S", name, "-o", "/dev/null", "doc.adoc"]);
        assert_eq!(resolve_safe_mode(&cli).expect("valid safe mode"), mode);
    }
}

// Not applicable: `--eruby erubi` selects the Ruby eRuby template engine used
// by Asciidoctor's converters. `adoc` has no eRuby, so the option is
// meaningless here.
non_normative!(
    r#"
  test 'should set eRuby impl if specified' do
    invoker = invoke_cli_to_buffer %w(--eruby erubi -o /dev/null)
    doc = invoker.document
    assert_equal 'erubi', doc.instance_variable_get('@options')[:eruby]
  end

"#
);

#[test]
fn should_force_default_external_encoding_to_utf_8() {
    verifies!(
        r#"
  test 'should force default external encoding to UTF-8' do
    input_path = fixture_path 'encoding.adoc'
    # using open3 to work around a bug in JRuby process_manager.rb,
    # which tries to run a gsub on stdout prematurely breaking the test
    # warnings may be issued, so don't assert on stderr
    stdout_lines = run_command(asciidoctor_cmd, '-o', '-', '--trace', input_path, env: { 'LANG' => 'US-ASCII' }) {|out| out.readlines }
    refute_empty stdout_lines
    # NOTE Ruby on Windows runs with a IBM437 encoding by default
    stdout_lines.each {|l| l.force_encoding Encoding::UTF_8 } unless Encoding.default_external == Encoding::UTF_8
    stdout_str = stdout_lines.join
    assert_includes stdout_str, 'Codierungen sind verrückt auf älteren Versionen von Ruby'
  end

"#
    );

    // Rust reads and writes UTF-8 natively, so there is no external encoding to
    // force; the German umlauted text passes through unchanged.
    let output = String::from_utf8(
        run_stdin(
            &["-o", "-", "-"],
            "= T\n\n== Überschrift\n\nCodierungen sind verrückt auf älteren Versionen von Ruby\n",
        )
        .expect("adoc converts"),
    )
    .expect("output is UTF-8");
    assert!(output.contains("Codierungen sind verrückt auf älteren Versionen von Ruby"));
}

// Not applicable: forces stdin/stdout to a non-UTF-8 encoding via `-E` and a
// Ruby require. Rust's I/O is UTF-8 native and `adoc` has neither `-E` nor
// `-r`, so there is no encoding to coerce.
non_normative!(
    r#"
  test 'should force stdio encoding to UTF-8' do
    cmd = asciidoctor_cmd ['-E', 'IBM866:IBM866']
    # NOTE configure-stdin.rb populates stdin
    result = run_command(cmd, '-r', (fixture_path 'configure-stdin.rb'), '-e', '-o', '-', '-') {|out| out.read }
    # NOTE Ruby on Windows runs with a IBM437 encoding by default
    result.force_encoding Encoding::UTF_8 unless Encoding.default_external == Encoding::UTF_8
    assert_equal Encoding::UTF_8, result.encoding
    assert_include '<p>é</p>', result
    assert_include '<p>IBM866:IBM866</p>', result
  end

"#
);

// Not applicable: a Ruby regression test that stubs `Dir.home` to raise via
// `-r`. It concerns the Ruby runtime, not a rendering rule, and `adoc` has no
// `-r`.
non_normative!(
    r#"
  test 'should not fail to load if call to Dir.home fails', unless: RUBY_ENGINE == 'truffleruby' do
    cmd = asciidoctor_cmd ['-r', (fixture_path 'undef-dir-home.rb')]
    result = run_command(cmd, '-e', '-o', '-', (fixture_path 'basic.adoc')) {|out| out.read }
    assert_include 'Body content', result
  end

"#
);

#[test]
fn should_print_timings_when_t_flag_is_specified() {
    verifies!(
        r#"
  test 'should print timings when -t flag is specified' do
    input = 'Sample *AsciiDoc*'
    invoker = nil
    error = nil
    redirect_streams do |_, err|
      invoker = invoke_cli(%w(-t -o /dev/null), '-') { input }
      error = err.string
    end
    refute_nil invoker
    refute_nil error
    assert_match(/Total time/, error)
  end

"#
    );

    // `-t`/`--timings` prints a timing report to standard error after the
    // conversion. The Ruby test discards the HTML with `-o /dev/null`; `adoc`
    // writes it to `-o -`/stdout, which this helper discards, and asserts on the
    // report that lands on stderr — its `Total time` line matching the Ruby
    // regex.
    let (_failed, stderr) = run_stdin_streams(&["-t", "-o", "-"], "Sample *AsciiDoc*");
    assert!(
        stderr.contains("Total time"),
        "expected a timing report on stderr, got: {stderr}",
    );
}

// Not implemented: renders the `doctime`/`localtime` attributes via `-d inline`
// to check timezone formatting. `adoc` rejects the `inline` doctype (an
// unsupported structural doctype, like `book`/`manpage`), so there is no way to
// emit the two bare attribute lines this test reads. The UTC case would in fact
// hold – an unpinned `adoc` clock reads as UTC – but the offset counterpart
// below cannot, as this toolchain carries no timezone database.
non_normative!(
    r#"
  test 'should show timezone as UTC if system TZ is set to UTC' do
    input_path = fixture_path 'doctime-localtime.adoc'
    output = run_command(asciidoctor_cmd, '-d', 'inline', '-o', '-', '-e', input_path, env: { 'TZ' => 'UTC', 'SOURCE_DATE_EPOCH' => nil, 'IGNORE_SOURCE_DATE_EPOCH' => '1' }) {|out| out.read }
    doctime, localtime = output.lines.map(&:chomp)
    assert doctime.end_with?(' UTC')
    assert localtime.end_with?(' UTC')
  end

"#
);

// Not implemented: the offset-timezone counterpart of the previous test. It
// needs the same `-d inline` mode `adoc` lacks, and additionally a local
// timezone offset derived from the system `TZ` – which this toolchain cannot
// compute, having no timezone database (an unpinned clock reads as UTC).
non_normative!(
    r#"
  test 'should show timezone as offset if system TZ is not set to UTC' do
    input_path = fixture_path 'doctime-localtime.adoc'
    output = run_command(asciidoctor_cmd, '-d', 'inline', '-o', '-', '-e', input_path, env: { 'TZ' => 'EST+5', 'SOURCE_DATE_EPOCH' => nil, 'IGNORE_SOURCE_DATE_EPOCH' => '1' }) {|out| out.read }
    doctime, localtime = output.lines.map(&:chomp)
    assert doctime.end_with?(' -0500')
    assert localtime.end_with?(' -0500')
  end

"#
);

// `SOURCE_DATE_EPOCH` seeds the `doc*` and `local*` date/time attributes for
// reproducible builds – `adoc` honors it, and the derived `docdatetime`
// surfaces in the footer's "Last updated" stamp. This is verified without
// mutating the shared process environment (which every concurrent `adoc` run
// reads on startup, so a stray value would race the whole test binary): the
// CLI's parse of the raw value and the rendering of the resulting instant are
// checked separately.
#[test]
fn should_use_source_date_epoch_as_modified_time_of_input_file_and_local_time() {
    verifies!(
        r#"
  test 'should use SOURCE_DATE_EPOCH as modified time of input file and local time' do
    old_source_date_epoch = ENV.delete 'SOURCE_DATE_EPOCH'
    begin
      ENV['SOURCE_DATE_EPOCH'] = '1234123412'
      sample_filepath = fixture_path 'sample.adoc'
      invoker = invoke_cli_to_buffer %w(-o /dev/null), sample_filepath
      doc = invoker.document
      assert_equal '2009-02-08', (doc.attr 'docdate')
      assert_equal '2009', (doc.attr 'docyear')
      assert_match(/2009-02-08 20:03:32 UTC/, (doc.attr 'docdatetime'))
      assert_equal '2009-02-08', (doc.attr 'localdate')
      assert_equal '2009', (doc.attr 'localyear')
      assert_match(/2009-02-08 20:03:32 UTC/, (doc.attr 'localdatetime'))
    ensure
      if old_source_date_epoch
        ENV['SOURCE_DATE_EPOCH'] = old_source_date_epoch
      else
        ENV.delete 'SOURCE_DATE_EPOCH'
      end
    end
  end

"#
    );

    // `1234123412` is 2009-02-08T20:03:32Z. The CLI parses the raw value into
    // exactly that instant.
    let epoch = crate::parse_source_date_epoch("1234123412").expect("valid epoch");
    assert_eq!(
        epoch,
        Some(ReferenceTime::from_unix_timestamp(1_234_123_412))
    );

    // Pinning the clock there (with no input mtime) drives both the `doc*` and
    // `local*` families, so the footer's docdatetime reads
    // "2009-02-08 20:03:32 UTC" – matching the Ruby assertions on both
    // docdatetime and localdatetime.
    let options = Options::new()
        .standalone(true)
        .reference_time(ReferenceTime::from_unix_timestamp(1_234_123_412));

    let output = asciidoc_html5::convert_with("= Sample\n\nBody.\n", &options);

    assert!(
        output.contains("Last updated 2009-02-08 20:03:32 UTC"),
        "footer should show the SOURCE_DATE_EPOCH instant: {output}"
    );
}

// An empty (or all-whitespace) `SOURCE_DATE_EPOCH` is ignored – the clock falls
// back to the wall clock rather than being pinned. Verified against the CLI's
// parse directly, for the same process-environment reason as above.
#[test]
fn should_ignore_source_date_epoch_if_value_is_empty() {
    verifies!(
        r#"
  test 'should ignore SOURCE_DATE_EPOCH is value is empty' do
    old_source_date_epoch = ENV.delete 'SOURCE_DATE_EPOCH'
    begin
      ENV['SOURCE_DATE_EPOCH'] = ''
      sample_filepath = fixture_path 'sample.adoc'
      invoker = invoke_cli_to_buffer %w(-o /dev/null), sample_filepath
      doc = invoker.document
      current_year = Time.now.strftime '%F'
      assert (doc.attr 'localyear').to_i >= (current_year.to_i - 1)
    ensure
      if old_source_date_epoch
        ENV['SOURCE_DATE_EPOCH'] = old_source_date_epoch
      else
        ENV.delete 'SOURCE_DATE_EPOCH'
      end
    end
  end

"#
    );

    // An empty or whitespace-only value parses to `None` (ignored), so the clock
    // is left unpinned and the dates fall back to the current time.
    assert!(matches!(crate::parse_source_date_epoch(""), Ok(None)));
    assert!(matches!(crate::parse_source_date_epoch("   "), Ok(None)));
}

// A malformed `SOURCE_DATE_EPOCH` fails the run rather than silently falling
// back to the wall clock. Verified against the CLI's parse directly (the same
// process-environment reason as above); `main` turns the returned error into
// the non-zero exit the Ruby test asserts. The trailing line closes the Ruby
// `context` block.
#[test]
fn should_fail_if_source_date_epoch_is_malformed() {
    verifies!(
        r#"
  test 'should fail if SOURCE_DATE_EPOCH is malformed' do
    old_source_date_epoch = ENV.delete 'SOURCE_DATE_EPOCH'
    begin
      ENV['SOURCE_DATE_EPOCH'] = 'aaaaaaaa'
      sample_filepath = fixture_path 'sample.adoc'
      assert_equal 1, (invoke_cli_to_buffer %w(-o /dev/null), sample_filepath).code
    ensure
      if old_source_date_epoch
        ENV['SOURCE_DATE_EPOCH'] = old_source_date_epoch
      else
        ENV.delete 'SOURCE_DATE_EPOCH'
      end
    end
  end
end
"#
    );

    // A non-integer value is rejected with an `InvalidInput` error, which
    // `run_with_streams` propagates and `main` maps to a non-zero exit code.
    let result = crate::parse_source_date_epoch("aaaaaaaa");
    assert!(result.is_err(), "a malformed value must fail the run");
    assert_eq!(
        result.expect_err("malformed epoch errors").kind(),
        std::io::ErrorKind::InvalidInput
    );
}
