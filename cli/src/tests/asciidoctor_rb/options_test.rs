//! Port of Asciidoctor's `options_test.rb`, tracked from the CLI crate.
//!
//! `options_test.rb` exercises `Asciidoctor::Cli::Options.parse!` — the Ruby
//! CLI's own option parser — which turns an `argv` array into an options `Hash`
//! (or prints usage / an error and returns an exit code). `adoc` has no such
//! object: it parses `argv` straight into a clap `Cli` struct, then validates
//! and applies the result in `run`. So each ported `#[test]` drives `adoc`'s
//! real machinery — `Cli::parse_from`/`try_parse_from`, the `check_doctype` /
//! `resolve_safe_mode` / `parse_failure_level` helpers, the `syntax_help_topic`
//! crib-sheet path, and (for attribute effects) an end-to-end `run_with_input`
//! conversion — and re-expresses the Ruby assertions against what `adoc`
//! exposes.
//!
//! A few Ruby exit codes differ by design: clap reports usage errors with exit
//! code 2 where the Ruby CLI uses 1, so those tests verify that the error is
//! raised and names the offending option rather than pinning the Ruby code.
//!
//! Kept `non_normative!` are the tests for behavior `adoc` does not have,
//! grouped by why:
//!
//! - Ruby CLI features with no `adoc` analog: the `-h manpage` help topic (and
//!   its `ASCIIDOCTOR_MANPAGE_PATH` lookup), the `-r`/`--require` library
//!   loader, and the `-I`/`--load-path` `$LOAD_PATH` manipulation.
//! - Out of scope for this html5-only, article-only renderer: other backends
//!   (`-b docbook`, `-b my_custom_backend`), the non-`article` doctypes
//!   (`book`, `inline`) that `-d` rejects at runtime, and the custom template
//!   engine (`-E`) and template directories (`-T`).
//! - Documented divergences from the oracle: an empty `-a`/`-a =` spec, which
//!   Asciidoctor silently ignores but `adoc` rejects as a missing attribute
//!   name; and `-q`/`-v` together, which Asciidoctor resolves last-wins but
//!   `adoc` deliberately rejects as a conflicting combination (its `--help`
//!   states "Cannot be combined").

use std::{ffi::OsString, path::PathBuf};

use asciidoc_html5::SafeMode;
use clap::{error::ErrorKind, Parser as _};

use crate::{
    check_doctype, parse_failure_level, resolve_safe_mode, run_with_input, syntax_help_topic,
    tests::sdd::*, Cli, LogLevel, SYNTAX_CRIB_SHEET,
};

track_file!("ref/asciidoctor/test/options_test.rb");

/// Converts `source` (fed on standard input) with `args`, returning the
/// rendered HTML. Used by the attribute tests, whose Ruby counterparts read the
/// parsed value back off the options `Hash`; `adoc` exposes an attribute's
/// effect only through the rendered output, so these drive a real conversion.
fn render(args: &[&str], source: &str) -> String {
    let cli = Cli::parse_from(std::iter::once("adoc").chain(args.iter().copied()));
    let mut stdin = source.as_bytes();
    let mut stdout = Vec::new();
    run_with_input(&cli, &mut stdin, &mut stdout).expect("adoc converts");
    String::from_utf8(stdout).expect("output is UTF-8")
}

/// The help text `adoc` prints for `-h`, captured from the clap "display help"
/// error the flag raises (which carries exit code 0).
fn help_text() -> String {
    Cli::try_parse_from(["adoc", "-h"])
        .expect_err("`-h` short-circuits to the help display")
        .render()
        .to_string()
}

/// Builds an `argv` of `OsString`s for the raw-args helpers such as
/// [`syntax_help_topic`], which inspect the command line before clap parses it.
fn argv<const N: usize>(args: [&str; N]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'
require File.join Asciidoctor::LIB_DIR, 'asciidoctor/cli/options'

context 'Options' do
"#
);

#[test]
fn help_flag_prints_usage_and_returns_zero() {
    verifies!(
        r#"
  test 'should print usage and return error code 0 when help flag is present' do
    redirect_streams do |stdout, stderr|
      exitval = Asciidoctor::Cli::Options.parse!(%w(-h))
      assert_equal 0, exitval
      assert_match(/^Usage:/, stdout.string)
    end
  end

"#
    );

    // `adoc`'s `-h` is a clap help flag: it raises the "display help" error,
    // whose exit code is 0 (matching Ruby's `exitval == 0`) and whose rendered
    // text carries a line beginning `Usage:` (matching `/^Usage:/`).
    let err = Cli::try_parse_from(["adoc", "-h"]).expect_err("`-h` displays help");
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    assert_eq!(err.exit_code(), 0);

    let help = err.render().to_string();
    assert!(
        help.lines().any(|line| line.starts_with("Usage:")),
        "help output should carry a Usage: line, got:\n{help}"
    );
}

#[test]
fn help_shows_safe_modes_in_severity_order() {
    verifies!(
        r#"
  test 'should show safe modes in severity order' do
    redirect_streams do |stdout, stderr|
      exitval = Asciidoctor::Cli::Options.parse!(%w(-h))
      assert_equal 0, exitval
      assert_match(/unsafe, safe, server, secure/, stdout.string)
    end
  end

"#
    );

    // The claim is that the help lists the safe modes from least to most
    // restrictive. `adoc`'s `--safe-mode` summary phrases the list "unsafe,
    // safe, server, or secure" (an Oxford "or" where Ruby writes a bare comma),
    // so that one substring encodes the same severity ordering.
    let help = help_text();
    assert!(
        help.contains("unsafe, safe, server, or secure"),
        "help should list safe modes in severity order, got:\n{help}"
    );
}

#[test]
fn help_flag_with_unknown_topic_prints_usage_and_returns_zero() {
    verifies!(
        r#"
  test 'should print usage and return error code 0 when help flag is unknown' do
    exitval, output = redirect_streams do |out, _|
      [Asciidoctor::Cli::Options.parse!(%w(-h unknown)), out.string]
    end
    assert_equal 0, exitval
    assert_match(/^Usage:/, output)
  end

"#
    );

    // An unrecognized help topic is not the `syntax` crib sheet, so `adoc` falls
    // through to clap, where the `-h` flag still wins over the trailing word and
    // displays usage with exit code 0 — matching Ruby's fallback to the usage
    // statement for an unknown topic.
    assert!(!syntax_help_topic(&argv(["adoc", "-h", "unknown"])));

    let err = Cli::try_parse_from(["adoc", "-h", "unknown"]).expect_err("`-h` displays help");
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    assert_eq!(err.exit_code(), 0);
    assert!(err
        .render()
        .to_string()
        .lines()
        .any(|line| line.starts_with("Usage:")));
}

// No analog: `-h manpage` dumps Asciidoctor's own troff man page, a Ruby CLI
// help topic `adoc` does not implement (its only topic is `syntax`, below), so
// there is no man-page output to assert against.
non_normative!(
    r#"
  test 'should dump man page and return error code 0 when help topic is manpage' do
    exitval, output = redirect_streams do |out, _|
      [Asciidoctor::Cli::Options.parse!(%w(-h manpage)), out.string]
    end
    assert_equal 0, exitval
    assert_includes output, 'Manual: Asciidoctor Manual'
    assert_includes output, '.TH "ASCIIDOCTOR"'
  end

"#
);

#[test]
fn help_topic_syntax_prints_syntax_overview() {
    verifies!(
        r#"
  test 'should an overview of the AsciiDoc syntax and return error code 0 when help topic is syntax' do
    exitval, output = redirect_streams do |out, _|
      [Asciidoctor::Cli::Options.parse!(%w(-h syntax)), out.string]
    end
    assert_equal 0, exitval
    assert_includes output, '= AsciiDoc Syntax'
    assert_includes output, '== Text Formatting'
  end

"#
    );

    // `adoc` implements the `syntax` help topic: `-h syntax` (like `--help
    // syntax`) prints an AsciiDoc crib sheet and exits 0. The crib sheet is
    // embedded as `SYNTAX_CRIB_SHEET`, and — as the Ruby test checks — it opens
    // with the `= AsciiDoc Syntax` title and carries a `== Text Formatting`
    // section.
    assert!(syntax_help_topic(&argv(["adoc", "-h", "syntax"])));
    assert!(syntax_help_topic(&argv(["adoc", "--help", "syntax"])));

    assert!(SYNTAX_CRIB_SHEET.contains("= AsciiDoc Syntax"));
    assert!(SYNTAX_CRIB_SHEET.contains("== Text Formatting"));
}

// No analog: the failure path of the `-h manpage` topic — looking the man page
// up via `ASCIIDOCTOR_MANPAGE_PATH` and failing when it is missing. `adoc` has
// no man-page help topic (see above), so there is no such lookup to fail.
non_normative!(
    r#"
  test 'should print message and return error code 1 when manpage is not found' do
    old_manpage_path = ENV['ASCIIDOCTOR_MANPAGE_PATH']
    begin
      ENV['ASCIIDOCTOR_MANPAGE_PATH'] = (manpage_path = fixture_path 'no-such-file.1')
      redirect_streams do |out, stderr|
        exitval = Asciidoctor::Cli::Options.parse!(%w(-h manpage))
        assert_equal 1, exitval
        assert_equal %(asciidoctor: FAILED: manual page not found: #{manpage_path}), stderr.string.chomp
      end
    ensure
      if old_manpage_path
        ENV['ASCIIDOCTOR_MANPAGE_PATH'] = old_manpage_path
      else
        ENV.delete 'ASCIIDOCTOR_MANPAGE_PATH'
      end
    end
  end

"#
);

#[test]
fn invalid_option_is_rejected() {
    verifies!(
        r#"
  test 'should return error code 1 when invalid option present' do
    redirect_streams do |stdout, stderr|
      exitval = Asciidoctor::Cli::Options.parse!(%w(--foobar))
      assert_equal 1, exitval
      assert_equal 'asciidoctor: invalid option: --foobar', stderr.string.chomp
    end
  end

"#
    );

    // clap rejects an unknown option, naming it in the error. clap's usage-error
    // exit code is 2 (where the Ruby CLI uses 1), so the assertion is on the
    // error kind and that it names `--foobar`, not on the numeric code.
    let err = Cli::try_parse_from(["adoc", "--foobar"]).expect_err("unknown option is rejected");
    assert_eq!(err.kind(), ErrorKind::UnknownArgument);
    assert_ne!(err.exit_code(), 0);
    assert!(err.render().to_string().contains("--foobar"));
}

#[test]
fn invalid_option_argument_is_rejected() {
    verifies!(
        r#"
  test 'should return error code 1 when option has invalid argument' do
    redirect_streams do |stdout, stderr|
      exitval = Asciidoctor::Cli::Options.parse!(%w(-d chapter input.ad)) # had to change for #320
      assert_equal 1, exitval
      assert_equal 'asciidoctor: invalid argument: -d chapter', stderr.string.chomp
    end
  end

"#
    );

    // clap accepts any string for `-d`, so `adoc` validates the doctype in
    // `check_doctype` when the run begins: `chapter` (invalid in Ruby too) is
    // rejected there with an error that names it, the same rejection the Ruby
    // parser reports up front.
    let cli = Cli::parse_from(["adoc", "-d", "chapter", "input.ad"]);
    let err = check_doctype(&cli).expect_err("an unknown doctype is rejected");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("chapter"));
}

#[test]
fn option_missing_required_argument_is_rejected() {
    verifies!(
        r#"
  test 'should return error code 1 when option is missing required argument' do
    redirect_streams do |stdout, stderr|
      exitval = Asciidoctor::Cli::Options.parse!(%w(-b))
      assert_equal 1, exitval
      assert_equal 'asciidoctor: option missing argument: -b', stderr.string.chomp
    end
  end

"#
    );

    // `-b`/`--backend` takes a value, so clap rejects it with none supplied,
    // naming the option in the error. As with the unknown-option case, clap's
    // usage-error exit code is 2 rather than the Ruby CLI's 1.
    let err = Cli::try_parse_from(["adoc", "-b"]).expect_err("a missing value is rejected");
    assert_ne!(err.exit_code(), 0);
    assert!(err.render().to_string().contains("--backend"));
}

// Out of scope / no analog: `-b docbook` selects a backend `adoc` does not
// produce (it renders only html5, rejecting other backends in `check_backend`),
// and the Ruby "extra arguments detected" warning for the trailing `- -` is a
// Ruby-CLI diagnostic; `adoc` has its own handling of an extra `-` argument,
// exercised in `invoker_test`.
non_normative!(
    r#"
  test 'should emit warning when unparsed options remain' do
    redirect_streams do |stdout, stderr|
      options = Asciidoctor::Cli::Options.parse!(%w(-b docbook - -))
      assert_kind_of Hash, options
      assert_match(/asciidoctor: WARNING: extra arguments .*/, stderr.string.chomp)
    end
  end

"#
);

#[test]
fn basic_argument_assignment() {
    verifies!(
        r#"
  test 'basic argument assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-w -v -e -d book test/fixtures/sample.adoc))

    assert_equal 2, options[:verbose]
    assert_equal false, options[:standalone]
    assert_equal 'book', options[:attributes]['doctype']
    assert_equal 1, options[:input_files].size
    assert_equal 'test/fixtures/sample.adoc', options[:input_files][0]
  end

"#
    );

    // The Ruby test reads the parsed options hash; the clap `Cli` is `adoc`'s
    // equivalent. `-v` (Ruby verbose 2) sets `verbose`, `-e` (Ruby standalone
    // false) sets `embedded`, `-w` sets `warnings`, and the lone input file is
    // recorded. `-d book` is captured verbatim at parse time — as the Ruby hash
    // captures `doctype => 'book'` — though `book` is later rejected at runtime
    // (only `article` is supported; see the non-normative book/inline tests).
    let cli = Cli::parse_from([
        "adoc",
        "-w",
        "-v",
        "-e",
        "-d",
        "book",
        "test/fixtures/sample.adoc",
    ]);

    assert!(cli.verbose);
    assert!(cli.embedded);
    assert!(cli.warnings);
    assert_eq!(cli.doctype.as_deref(), Some("book"));
    assert_eq!(cli.inputs, vec![PathBuf::from("test/fixtures/sample.adoc")]);
}

#[test]
fn supports_legacy_option_for_no_header_footer() {
    verifies!(
        r#"
  test 'supports legacy option for no header footer' do
    options = Asciidoctor::Cli::Options.parse!(%w(-s test/fixtures/sample.adoc))

    assert_equal false, options[:standalone]
    assert_equal 1, options[:input_files].size
    assert_equal 'test/fixtures/sample.adoc', options[:input_files][0]
  end

"#
    );

    // `adoc` accepts the legacy `-s`/`--no-header-footer` spelling as an alias
    // of `-e`/`--embedded`, so `-s` selects embedded output (Ruby standalone
    // false) and records the single input file.
    let cli = Cli::parse_from(["adoc", "-s", "test/fixtures/sample.adoc"]);

    assert!(cli.embedded);
    assert_eq!(cli.inputs, vec![PathBuf::from("test/fixtures/sample.adoc")]);
}

#[test]
fn standard_attribute_assignment() {
    verifies!(
        r#"
  test 'standard attribute assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-a docinfosubs=attributes,replacements -a icons test/fixtures/sample.adoc))

    assert_equal 'attributes,replacements', options[:attributes]['docinfosubs']
    assert_equal '', options[:attributes]['icons']
  end

"#
    );

    // The Ruby test reads the parsed attribute values; `adoc` surfaces them only
    // through the conversion. `-a docinfosubs=attributes,replacements` gives the
    // attribute that value, observed by referencing `{docinfosubs}`; the bare
    // `-a icons` sets `icons` to the empty string, observed by an `ifdef::icons`
    // whose body appears only when the attribute is set.
    let out = render(
        &[
            "-a",
            "docinfosubs=attributes,replacements",
            "-a",
            "icons",
            "-e",
            "-o",
            "-",
        ],
        "{docinfosubs}\n\nifdef::icons[icons-set]\n",
    );

    assert!(out.contains("attributes,replacements"));
    assert!(out.contains("icons-set"));
}

#[test]
fn multiple_attribute_arguments() {
    verifies!(
        r#"
  test 'multiple attribute arguments' do
    options = Asciidoctor::Cli::Options.parse!(%w(-a imagesdir=images -a icons test/fixtures/sample.adoc))

    assert_equal 'images', options[:attributes]['imagesdir']
    assert_equal '', options[:attributes]['icons']
  end

"#
    );

    // Two `-a` options apply together: `imagesdir=images` prefixes image targets
    // with `images/` (observed on a rendered `image::` target), and the bare `-a
    // icons` sets `icons` (observed through `ifdef::icons`).
    let out = render(
        &["-a", "imagesdir=images", "-a", "icons", "-e", "-o", "-"],
        "image::pic.png[]\n\nifdef::icons[icons-set]\n",
    );

    assert!(out.contains(r#"src="images/pic.png""#));
    assert!(out.contains("icons-set"));
}

#[test]
fn splits_attribute_pair_on_first_equal_sign() {
    verifies!(
        r#"
  test 'should only split attribute key/value pairs on first equal sign' do
    options = Asciidoctor::Cli::Options.parse!(%w(-a name=value=value test/fixtures/sample.adoc))

    assert_equal 'value=value', options[:attributes]['name']
  end

"#
    );

    // `adoc` splits an `-a` spec on the first `=` only, so `name=value=value`
    // assigns `name` the value `value=value`, seen by referencing `{name}`.
    let out = render(&["-a", "name=value=value", "-e", "-o", "-"], "{name}\n");
    assert!(out.contains("value=value"));
}

// Documented divergence: Asciidoctor silently ignores an empty `-a` spec
// (`attr.rstrip.empty?`), leaving the attributes hash absent (`assert_nil`).
// `adoc` instead rejects an empty attribute name in `apply_attribute_spec`
// ("invalid attribute: missing attribute name"), failing the run rather than
// ignoring it — a deliberately stricter choice kept over matching the oracle.
non_normative!(
    r#"
  test 'should not fail if value of attribute option is empty' do
    options = Asciidoctor::Cli::Options.parse!(['-a', '', 'test/fixtures/sample.adoc'])

    assert_nil options[:attributes]
  end

"#
);

// Documented divergence: as above, Asciidoctor ignores a bare `-a =` spec
// (`attr == '='`); `adoc` rejects it as a missing attribute name.
non_normative!(
    r#"
  test 'should not fail if value of attribute option is equal sign' do
    options = Asciidoctor::Cli::Options.parse!(['-a', '=', 'test/fixtures/sample.adoc'])

    assert_nil options[:attributes]
  end

"#
);

#[test]
fn attribute_value_preserves_utf8() {
    verifies!(
        r#"
  test 'should gracefully force encoding to UTF-8 if encoding on string is mislabeled' do
    args = ['-a', ((%w(platform-name 云平台).join '=').force_encoding Encoding::ASCII_8BIT), '-']
    options = Asciidoctor::Cli::Options.parse! args

    assert_equal '云平台', options[:attributes]['platform-name']
    assert_equal Encoding::UTF_8, options[:attributes]['platform-name'].encoding
  end

"#
    );

    // The Ruby test guards against a byte string mislabeled as ASCII-8BIT; Rust
    // `String`s are always UTF-8, so the encoding-object assertion has no analog
    // and the value simply round-trips. Passing `-a platform-name=云平台` and
    // referencing `{platform-name}` yields the multibyte value unchanged.
    let out = render(
        &["-a", "platform-name=云平台", "-e", "-o", "-"],
        "{platform-name}\n",
    );
    assert!(out.contains("云平台"));
}

#[test]
fn allows_safe_mode_to_be_specified() {
    verifies!(
        r#"
  test 'should allow safe mode to be specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-S safe test/fixtures/sample.adoc))
    assert_equal Asciidoctor::SafeMode::SAFE, options[:safe]
  end

"#
    );

    // `-S safe` resolves to `SafeMode::Safe`, the same level the Ruby test reads
    // off `options[:safe]`.
    let cli = Cli::parse_from(["adoc", "-S", "safe", "test/fixtures/sample.adoc"]);
    assert_eq!(
        resolve_safe_mode(&cli).expect("safe mode resolves"),
        SafeMode::Safe
    );
}

// Out of scope: Asciidoctor accepts any backend name and records it; `adoc`
// produces only the html5 backend and rejects every other in `check_backend`,
// so `my_custom_backend` has no supported behavior to verify.
non_normative!(
    r#"
  test 'should allow any backend to be specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-b my_custom_backend test/fixtures/sample.adoc))

    assert_equal 'my_custom_backend', options[:attributes]['backend']
  end

"#
);

#[test]
fn article_doctype_assignment() {
    verifies!(
        r#"
  test 'article doctype assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-d article test/fixtures/sample.adoc))
    assert_equal 'article', options[:attributes]['doctype']
  end

"#
    );

    // `article` is the doctype `adoc` supports: it is captured at parse time and
    // accepted by `check_doctype`.
    let cli = Cli::parse_from(["adoc", "-d", "article", "test/fixtures/sample.adoc"]);
    assert_eq!(cli.doctype.as_deref(), Some("article"));
    check_doctype(&cli).expect("article is a supported doctype");
}

// Out of scope: `adoc` models only the `article` doctype; `book` is rejected at
// runtime by `check_doctype`, so there is no supported behavior to verify.
non_normative!(
    r#"
  test 'book doctype assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-d book test/fixtures/sample.adoc))
    assert_equal 'book', options[:attributes]['doctype']
  end

"#
);

// Out of scope: same as `book` — the `inline` doctype is not modeled and is
// rejected at runtime.
non_normative!(
    r#"
  test 'inline doctype assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-d inline test/fixtures/sample.adoc))
    assert_equal 'inline', options[:attributes]['doctype']
  end

"#
);

// No analog: `-E`/`--template-engine` selects a custom-template engine (Haml,
// Slim, ...); `adoc` has only its built-in html5 converter and no custom
// templating, so it offers no such option.
non_normative!(
    r#"
  test 'template engine assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-E haml test/fixtures/sample.adoc))
    assert_equal 'haml', options[:template_engine]
  end

"#
);

// No analog: `-T`/`--template-dir` points at a directory of custom converter
// templates, which `adoc` does not support (see the template-engine test).
non_normative!(
    r#"
  test 'template directory assignment' do
    options = Asciidoctor::Cli::Options.parse!(%w(-T custom-backend test/fixtures/sample.adoc))
    assert_equal ['custom-backend'], options[:template_dirs]
  end

"#
);

// No analog: as above, repeated `-T` options accumulate template directories;
// `adoc` has no custom-template support.
non_normative!(
    r#"
  test 'multiple template directory assignments' do
    options = Asciidoctor::Cli::Options.parse!(%w(-T custom-backend -T custom-backend-hacks test/fixtures/sample.adoc))
    assert_equal ['custom-backend', 'custom-backend-hacks'], options[:template_dirs]
  end

"#
);

// No analog: `-r`/`--require` loads Ruby libraries before processing (via
// `require`); `adoc` is a native binary with no runtime library loading.
non_normative!(
    r#"
  test 'multiple -r flags requires specified libraries' do
    options = Asciidoctor::Cli::Options.new
    redirect_streams do |stdout, stderr|
      exitval = options.parse! %w(-r foobar -r foobaz test/fixtures/sample.adoc)
      assert_match(%(asciidoctor: FAILED: 'foobar' could not be loaded), stderr.string)
      assert_equal 1, exitval
      assert_equal ['foobar', 'foobaz'], options[:requires]
    end
  end

"#
);

// No analog: same as above — a comma-separated `-r` list of Ruby libraries.
non_normative!(
    r#"
  test '-r flag with multiple values requires specified libraries' do
    options = Asciidoctor::Cli::Options.new
    redirect_streams do |stdout, stderr|
      exitval = options.parse! %w(-r foobar,foobaz test/fixtures/sample.adoc)
      assert_match(%(asciidoctor: FAILED: 'foobar' could not be loaded), stderr.string)
      assert_equal 1, exitval
      assert_equal ['foobar', 'foobaz'], options[:requires]
    end
  end

"#
);

// No analog: `-I`/`--load-path` prepends directories to Ruby's `$LOAD_PATH`;
// `adoc` has no interpreter load path to manipulate.
non_normative!(
    r#"
  test '-I option appends paths to $LOAD_PATH' do
    options = Asciidoctor::Cli::Options.new
    old_load_path = $:.dup
    begin
      exitval = options.parse! %w(-I foobar -I foobaz test/fixtures/sample.adoc)
      refute_equal 1, exitval
      assert_equal old_load_path.size + 2, $:.size
      assert_equal File.expand_path('foobar'), $:[0]
      assert_equal File.expand_path('foobaz'), $:[1]
      assert_equal ['foobar', 'foobaz'], options[:load_paths]
    ensure
      ($:.size - old_load_path.size).times { $:.shift }
    end
  end

"#
);

// No analog: same as above — a `PATH_SEPARATOR`-joined `-I` value split into
// multiple `$LOAD_PATH` entries.
non_normative!(
    r#"
  test '-I option appends multiple paths to $LOAD_PATH' do
    options = Asciidoctor::Cli::Options.new
    old_load_path = $:.dup
    begin
      exitval = options.parse! %W(-I foobar#{File::PATH_SEPARATOR}foobaz test/fixtures/sample.adoc)
      refute_equal 1, exitval
      assert_equal old_load_path.size + 2, $:.size
      assert_equal File.expand_path('foobar'), $:[0]
      assert_equal File.expand_path('foobaz'), $:[1]
      assert_equal ['foobar', 'foobaz'], options[:load_paths]
    ensure
      ($:.size - old_load_path.size).times { $:.shift }
    end
  end

"#
);

#[test]
fn failure_level_defaults_to_fatal() {
    verifies!(
        r#"
  test 'should set failure level to FATAL by default' do
    options = Asciidoctor::Cli::Options.parse! %w(test/fixtures/sample.adoc)
    assert_equal ::Logger::Severity::FATAL, options[:failure_level]
  end

"#
    );

    // With no `--failure-level`, `adoc` defaults the option to `FATAL`, which
    // resolves to `LogLevel::Fatal`.
    let cli = Cli::parse_from(["adoc", "test/fixtures/sample.adoc"]);
    assert_eq!(
        parse_failure_level(&cli.failure_level).unwrap(),
        LogLevel::Fatal
    );
}

#[test]
fn failure_level_fatal_abbreviations() {
    verifies!(
        r#"
  test 'should allow failure level to be set to FATAL using any recognized abbreviation' do
    %w(f fatal FATAL).each do |val|
      options = Asciidoctor::Cli::Options.parse! %W(--failure-level=#{val} test/fixtures/sample.adoc)
      assert_equal ::Logger::Severity::FATAL, options[:failure_level]
    end
  end

"#
    );

    // Matching Asciidoctor's option completion, `adoc` accepts any unambiguous
    // abbreviation of `fatal` (case-insensitively), so `f`, `fatal`, and `FATAL`
    // all resolve to `LogLevel::Fatal`.
    for val in ["f", "fatal", "FATAL"] {
        let arg = format!("--failure-level={val}");
        let cli = Cli::parse_from(["adoc", &arg, "test/fixtures/sample.adoc"]);
        assert_eq!(
            parse_failure_level(&cli.failure_level).unwrap(),
            LogLevel::Fatal,
            "{val} should resolve to FATAL"
        );
    }
}

#[test]
fn failure_level_error_abbreviations() {
    verifies!(
        r#"
  test 'should allow failure level to be set to ERROR using any recognized abbreviation' do
    %w(e err ERR error ERROR).each do |val|
      options = Asciidoctor::Cli::Options.parse!(%W(--failure-level=#{val} test/fixtures/sample.adoc))
      assert_equal ::Logger::Severity::ERROR, options[:failure_level]
    end
  end

"#
    );

    // Any unambiguous abbreviation of `error` resolves to `LogLevel::Error`.
    for val in ["e", "err", "ERR", "error", "ERROR"] {
        let arg = format!("--failure-level={val}");
        let cli = Cli::parse_from(["adoc", &arg, "test/fixtures/sample.adoc"]);
        assert_eq!(
            parse_failure_level(&cli.failure_level).unwrap(),
            LogLevel::Error,
            "{val} should resolve to ERROR"
        );
    }
}

#[test]
fn failure_level_warn_abbreviations() {
    verifies!(
        r#"
  test 'should allow failure level to be set to WARN using any recognized abbreviation' do
    %w(w warn WARN warning WARNING).each do |val|
      options = Asciidoctor::Cli::Options.parse! %W(--failure-level=#{val} test/fixtures/sample.adoc)
      assert_equal ::Logger::Severity::WARN, options[:failure_level]
    end
  end

"#
    );

    // Any unambiguous abbreviation of `warning` (including the `warn` spelling)
    // resolves to `LogLevel::Warn`.
    for val in ["w", "warn", "WARN", "warning", "WARNING"] {
        let arg = format!("--failure-level={val}");
        let cli = Cli::parse_from(["adoc", &arg, "test/fixtures/sample.adoc"]);
        assert_eq!(
            parse_failure_level(&cli.failure_level).unwrap(),
            LogLevel::Warn,
            "{val} should resolve to WARN"
        );
    }
}

#[test]
fn failure_level_rejects_unknown_value() {
    verifies!(
        r#"
  test 'should not allow failure level to be set to unknown value' do
    exit_code, messages = redirect_streams do |_, err|
      [(Asciidoctor::Cli::Options.parse! %w(--failure-level=foobar test/fixtures/sample.adoc)), err.string]
    end
    assert_equal 1, exit_code
    assert_includes messages, 'invalid argument: --failure-level=foobar'
  end

"#
    );

    // clap accepts any string for `--failure-level`, so `adoc` validates it when
    // the run begins: `foobar` is no abbreviation of any level, so
    // `parse_failure_level` rejects it with an error naming the value.
    let cli = Cli::parse_from([
        "adoc",
        "--failure-level=foobar",
        "test/fixtures/sample.adoc",
    ]);
    let err = parse_failure_level(&cli.failure_level).expect_err("an unknown level is rejected");
    assert!(err.to_string().contains("foobar"));
}

#[test]
fn verbose_flag_enables_verbose() {
    verifies!(
        r#"
  test 'should set verbose to 2 when -v flag is specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-v test/fixtures/sample.adoc))
    assert_equal 2, options[:verbose]
  end

"#
    );

    // Ruby's verbosity level 2 (show info-level messages) corresponds to
    // `adoc`'s `-v`/`--verbose` flag being set.
    let cli = Cli::parse_from(["adoc", "-v", "test/fixtures/sample.adoc"]);
    assert!(cli.verbose);
    assert!(!cli.quiet);
}

#[test]
fn quiet_flag_enables_quiet() {
    verifies!(
        r#"
  test 'should set verbose to 0 when -q flag is specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-q test/fixtures/sample.adoc))
    assert_equal 0, options[:verbose]
  end

"#
    );

    // Ruby's verbosity level 0 (silence messages) corresponds to `adoc`'s
    // `-q`/`--quiet` flag being set.
    let cli = Cli::parse_from(["adoc", "-q", "test/fixtures/sample.adoc"]);
    assert!(cli.quiet);
    assert!(!cli.verbose);
}

// Documented divergence: Asciidoctor treats verbosity as a level where the last
// of `-q`/`-v` wins, so `-q -v` yields verbose (2). `adoc` models `-q` and `-v`
// as mutually exclusive flags (clap `conflicts_with`, and its `--help` states
// "Cannot be combined"), so it rejects the combination rather than resolving a
// winner.
non_normative!(
    r#"
  test 'should set verbose to 2 when -v flag is specified after -q flag' do
    options = Asciidoctor::Cli::Options.parse!(%w(-q -v test/fixtures/sample.adoc))
    assert_equal 2, options[:verbose]
  end

"#
);

// Documented divergence: as above, Asciidoctor's last-wins gives `-v -q` quiet
// (0); `adoc` rejects the conflicting combination.
non_normative!(
    r#"
  test 'should set verbose to 0 when -q flag is specified after -v flag' do
    options = Asciidoctor::Cli::Options.parse!(%w(-v -q test/fixtures/sample.adoc))
    assert_equal 0, options[:verbose]
  end

"#
);

#[test]
fn warnings_flag_is_accepted() {
    verifies!(
        r#"
  test 'should enable warnings when -w flag is specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-w test/fixtures/sample.adoc))
    assert options[:warnings]
  end

"#
    );

    // `-w`/`--warnings` is recorded (in Asciidoctor it toggles Ruby script
    // warnings; `adoc` accepts it for compatibility, as a native binary has no
    // such VM warnings to enable).
    let cli = Cli::parse_from(["adoc", "-w", "test/fixtures/sample.adoc"]);
    assert!(cli.warnings);
}

#[test]
fn timings_flag_enables_timings() {
    verifies!(
        r#"
  test 'should enable timings when -t flag is specified' do
    options = Asciidoctor::Cli::Options.parse!(%w(-t test/fixtures/sample.adoc))
    assert_equal true, options[:timings]
  end

"#
    );

    // `-t`/`--timings` turns on the timing report.
    let cli = Cli::parse_from(["adoc", "-t", "test/fixtures/sample.adoc"]);
    assert!(cli.timings);
}

#[test]
fn timings_disabled_by_default() {
    verifies!(
        r#"
  test 'timings option is disable by default' do
    options = Asciidoctor::Cli::Options.parse!(%w(test/fixtures/sample.adoc))
    assert_equal false, options[:timings]
  end

end
"#
    );

    // With no `-t`, the timing report stays off.
    let cli = Cli::parse_from(["adoc", "test/fixtures/sample.adoc"]);
    assert!(!cli.timings);
}
