//! Port of Asciidoctor's `attributes_test.rb`.
//!
//! Document attribute *assignment* and *interpolation* are the parser's job:
//! `asciidoc-parser` resolves attribute entries, continuation folding, header
//! substitutions, counters, and the intrinsic attributes at parse time, and
//! this crate re-exports the parsed [`Document`](crate::Document). So the many
//! tests that assert on `doc.attributes[...]` port directly against
//! [`Document::attribute_value`](crate::Document::attribute_value) /
//! [`is_attribute_set`](crate::Document::is_attribute_set) via [`load`] /
//! [`load_with`] (the convert-over-parse carve-out for *document state*), and
//! the interpolation/block-attribute tests port through `convert` (embedded) /
//! `convert_with(..standalone(true)..)` /
//! `convert_with(..doctype("inline")..)`, the counterparts to Asciidoctor's
//! `convert_string_to_embedded` / `convert_string` / `convert_inline_string`.
//!
//! What stays `non_normative!` here:
//! - **`asciidoc-parser` model / mutation APIs with no rendered or document-
//!   state counterpart**: `Asciidoctor::Block.new` + `node.attr`/`attr?`
//!   fallback-name lookups, `set_attr`/`remove_attr`/`set_attribute`,
//!   `roles=`/`add_role`/`remove_role`, and the raw block attribute-name map
//!   (`block.attr('foo')` on a normal paragraph, which renders nothing). These
//!   exercise the layer below this renderer.
//! - the **`{set:name:value}` / `{set:name!}` inline attribute-entry macro**,
//!   which `asciidoc-parser` deliberately does not support (documented in its
//!   README as discouraged AsciiDoc syntax); the reference leaves the macro
//!   text verbatim, so the drop-line/assignment behavior these tests expect
//!   does not occur.
//! - **compat mode** (`compat-mode` set/unset mid-document, and the compat
//!   `+...+` passthrough in a block title): permanently out of scope for this
//!   crate, like the compat-mode tests in `links_test`.
//! - the **DocBook backend** and **book doctype** (custom backend/doctype
//!   attribute matrices, the docbook special-section test): this crate targets
//!   only the `html5` backend, and non-article doctypes are out of scope for
//!   1.0.
//! - **setext** (two-line/underlined) titles, intentionally out of scope for
//!   this project (the "collapses spaces in attribute names" input frames its
//!   header that way).
//! - the **intrinsic-attribute enumeration** test, which iterates
//!   `Asciidoctor::INTRINSIC_ATTRIBUTES` (a parser-internal table with no
//!   public equivalent here); representative intrinsics are still checked
//!   individually.
//! - the **`toc` attribute matrix**: this crate renders every TOC placement
//!   byte-for-byte (covered in `sections_test`), but `asciidoc-parser` stores
//!   the raw `toc` value (`left`, `macro`, `auto`, …) rather than normalizing
//!   it to `''` the way the Ruby matrix asserts, so the matrix's `attr?('toc',
//!   '')` check has no faithful re-expression.
//! - the **missing-attribute drop-line diagnostic**: this crate drops/blanks
//!   the line exactly as Asciidoctor does (verified through the rendered
//!   output), but `asciidoc-parser` surfaces no `INFO`-level message for it, so
//!   a test whose *only* assertion is that log message cannot be verified
//!   (tracked upstream as
//!   <https://github.com/asciidoc-rs/asciidoc-parser/issues/1011>).

use asciidoc_parser::{document::InterpretedValue, warnings::WarningType};

use crate::{
    convert, convert_with, load, load_with,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
    Options, SafeMode,
};

track_file!("ref/asciidoctor/test/attributes_test.rb");

/// Wraps `s` as a resolved attribute [`InterpretedValue::Value`] — the common
/// shape asserted against `doc.attributes['name']`.
fn val(s: &str) -> InterpretedValue {
    InterpretedValue::Value(s.to_string())
}

/// Loads `src` and returns the resolved value of document attribute `name` —
/// the counterpart to Asciidoctor's
/// `document_from_string(src).attributes[name]`.
fn value_of(src: &str, name: &str) -> InterpretedValue {
    load(src).attribute_value(name)
}

/// Renders `input` as a standalone document — the counterpart to Asciidoctor's
/// `convert_string` (the embedded `convert` maps to
/// `convert_string_to_embedded`).
fn convert_standalone(input: &str) -> String {
    convert_with(input, &Options::new().standalone(true))
}

/// Renders `input` with the inline doctype and returns it trimmed of the single
/// trailing newline — the counterpart to Asciidoctor's `convert_inline_string`.
fn convert_inline(input: &str, options: Options) -> String {
    convert_with(input, &options.doctype("inline"))
        .trim_end_matches('\n')
        .to_string()
}

/// Asserts that `html` contains `needle` (the counterpart to the Ruby suite's
/// `assert_includes` / `assert_match` on a literal substring).
#[track_caller]
fn assert_includes(html: &str, needle: &str) {
    assert!(
        html.contains(needle),
        "expected output to contain:\n{needle}\n\nbut it was:\n{html}"
    );
}

/// Asserts that `html` does not contain `needle` (the counterpart to
/// `refute_includes` / `refute_match`).
#[track_caller]
fn refute_includes(html: &str, needle: &str) {
    assert!(
        !html.contains(needle),
        "expected output NOT to contain:\n{needle}\n\nbut it was:\n{html}"
    );
}

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

"#
);

non_normative!(
    r#"
context 'Attributes' do
  default_logger = Asciidoctor::LoggerManager.logger

  setup do
    Asciidoctor::LoggerManager.logger = (@logger = Asciidoctor::MemoryLogger.new)
  end

  teardown do
    Asciidoctor::LoggerManager.logger = default_logger
  end

"#
);

mod assignment {
    use super::*;
    non_normative!(
        r#"
  context 'Assignment' do
"#
    );

    #[test]
    fn creates_an_attribute() {
        verifies!(
            r#"
    test 'creates an attribute' do
      doc = document_from_string(':frog: Tanglefoot')
      assert_equal 'Tanglefoot', doc.attributes['frog']
    end

"#
        );

        assert_eq!(value_of(":frog: Tanglefoot", "frog"), val("Tanglefoot"));
    }

    #[test]
    fn requires_a_space_after_colon_following_attribute_name() {
        verifies!(
            r#"
    test 'requires a space after colon following attribute name' do
      doc = document_from_string 'foo:bar'
      assert_nil doc.attributes['foo']
    end

"#
        );

        assert!(!load("foo:bar").is_attribute_set("foo"));
    }

    #[test]
    fn does_not_recognize_attribute_entry_if_name_contains_colon() {
        verifies!(
            r#"
    # NOTE AsciiDoc.py recognizes this entry
    test 'does not recognize attribute entry if name contains colon' do
      input = ':foo:bar: baz'
      doc = document_from_string input
      refute doc.attr?('foo:bar')
      assert_equal 1, doc.blocks.size
      assert_equal :paragraph, doc.blocks[0].context
    end

"#
        );

        let input = ":foo:bar: baz";
        assert!(!load(input).is_attribute_set("foo:bar"));
        assert_xpath(&convert(input), r#"//p[text()=":foo:bar: baz"]"#, 1);
    }

    #[test]
    fn does_not_recognize_attribute_entry_if_name_ends_with_colon() {
        verifies!(
            r#"
    # NOTE AsciiDoc.py recognizes this entry
    test 'does not recognize attribute entry if name ends with colon' do
      input = ':foo:: bar'
      doc = document_from_string input
      refute doc.attr?('foo:')
      assert_equal 1, doc.blocks.size
      assert_equal :dlist, doc.blocks[0].context
    end

"#
        );

        let input = ":foo:: bar";
        assert!(!load(input).is_attribute_set("foo:"));
        assert_css(&convert(input), ".dlist", 1);
    }

    #[test]
    fn allows_any_word_character_defined_by_unicode_in_an_attribute_name() {
        verifies!(
            r#"
    # NOTE AsciiDoc.py does not recognize this entry
    test 'allows any word character defined by Unicode in an attribute name' do
      [['café', 'a coffee shop'], ['سمن', %(سازمان مردمنهاد)]].each do |(name, value)|
        str = <<~EOS
        :#{name}: #{value}

        {#{name}}
        EOS
        result = convert_string_to_embedded str
        assert_includes result, %(<p>#{value}</p>)
      end
    end

"#
        );

        for (name, value) in [("café", "a coffee shop"), ("سمن", "سازمان مردمنهاد")]
        {
            let src = format!(":{name}: {value}\n\n{{{name}}}\n");
            assert_includes(&convert(&src), &format!("<p>{value}</p>"));
        }
    }

    #[test]
    fn creates_an_attribute_by_fusing_a_legacy_multi_line_value() {
        verifies!(
            r#"
    test 'creates an attribute by fusing a legacy multi-line value' do
      str = <<~'EOS'
      :description: This is the first      +
                    Ruby implementation of +
                    AsciiDoc.
      EOS
      doc = document_from_string(str)
      assert_equal 'This is the first Ruby implementation of AsciiDoc.', doc.attributes['description']
    end

"#
        );

        let src = ":description: This is the first      +\n              Ruby implementation of +\n              AsciiDoc.\n";
        assert_eq!(
            value_of(src, "description"),
            val("This is the first Ruby implementation of AsciiDoc."),
        );
    }

    #[test]
    fn creates_an_attribute_by_fusing_a_multi_line_value() {
        verifies!(
            r#"
    test 'creates an attribute by fusing a multi-line value' do
      str = <<~'EOS'
      :description: This is the first \
                    Ruby implementation of \
                    AsciiDoc.
      EOS
      doc = document_from_string(str)
      assert_equal 'This is the first Ruby implementation of AsciiDoc.', doc.attributes['description']
    end

"#
        );

        let src = ":description: This is the first \\\n              Ruby implementation of \\\n              AsciiDoc.\n";
        assert_eq!(
            value_of(src, "description"),
            val("This is the first Ruby implementation of AsciiDoc."),
        );
    }

    #[test]
    fn honors_line_break_characters_in_multi_line_values() {
        verifies!(
            r#"
    test 'honors line break characters in multi-line values' do
      str = <<~'EOS'
      :signature: Linus Torvalds + \
      Linux Hacker + \
      linus.torvalds@example.com
      EOS
      doc = document_from_string(str)
      assert_equal %(Linus Torvalds +\nLinux Hacker +\nlinus.torvalds@example.com), doc.attributes['signature']
    end

"#
        );

        let src =
            ":signature: Linus Torvalds + \\\nLinux Hacker + \\\nlinus.torvalds@example.com\n";
        assert_eq!(
            value_of(src, "signature"),
            val("Linus Torvalds +\nLinux Hacker +\nlinus.torvalds@example.com"),
        );
    }

    #[test]
    fn should_allow_pass_macro_to_surround_a_multi_line_value_that_contains_line_breaks() {
        verifies!(
            r#"
    test 'should allow pass macro to surround a multi-line value that contains line breaks' do
      str = <<~'EOS'
      :signature: pass:a[{author} + \
      {title} + \
      {email}]
      EOS
      doc = document_from_string str, attributes: { 'author' => 'Linus Torvalds', 'title' => 'Linux Hacker', 'email' => 'linus.torvalds@example.com' }
      assert_equal %(Linus Torvalds +\nLinux Hacker +\nlinus.torvalds@example.com), (doc.attr 'signature')
    end

"#
        );

        let doc = load_with(
            ":signature: pass:a[{author} + \\\n{title} + \\\n{email}]\n",
            &Options::new()
                .attribute("author", "Linus Torvalds")
                .attribute("title", "Linux Hacker")
                .attribute("email", "linus.torvalds@example.com"),
        );
        assert_eq!(
            doc.attribute_value("signature"),
            val("Linus Torvalds +\nLinux Hacker +\nlinus.torvalds@example.com"),
        );
    }

    #[test]
    fn should_delete_an_attribute_that_ends_with() {
        verifies!(
            r#"
    test 'should delete an attribute that ends with !' do
      doc = document_from_string(":frog: Tanglefoot\n:frog!:")
      assert_nil doc.attributes['frog']
    end

"#
        );

        assert_eq!(
            value_of(":frog: Tanglefoot\n:frog!:", "frog"),
            InterpretedValue::Unset
        );
    }

    #[test]
    fn should_delete_an_attribute_that_ends_with_set_via_api() {
        verifies!(
            r#"
    test 'should delete an attribute that ends with ! set via API' do
      doc = document_from_string(":frog: Tanglefoot", attributes: { 'frog!' => '' })
      assert_nil doc.attributes['frog']
    end

"#
        );

        let doc = load_with(":frog: Tanglefoot", &Options::new().unset("frog"));
        assert_eq!(doc.attribute_value("frog"), InterpretedValue::Unset);
    }

    #[test]
    fn should_delete_an_attribute_that_begins_with() {
        verifies!(
            r#"
    test 'should delete an attribute that begins with !' do
      doc = document_from_string(":frog: Tanglefoot\n:!frog:")
      assert_nil doc.attributes['frog']
    end

"#
        );

        assert_eq!(
            value_of(":frog: Tanglefoot\n:!frog:", "frog"),
            InterpretedValue::Unset
        );
    }

    #[test]
    fn should_delete_an_attribute_that_begins_with_set_via_api() {
        verifies!(
            r#"
    test 'should delete an attribute that begins with ! set via API' do
      doc = document_from_string(":frog: Tanglefoot", attributes: { '!frog' => '' })
      assert_nil doc.attributes['frog']
    end

"#
        );

        let doc = load_with(":frog: Tanglefoot", &Options::new().unset("frog"));
        assert_eq!(doc.attribute_value("frog"), InterpretedValue::Unset);
    }

    #[test]
    fn should_delete_an_attribute_set_via_api_to_nil_value() {
        verifies!(
            r#"
    test 'should delete an attribute set via API to nil value' do
      doc = document_from_string(":frog: Tanglefoot", attributes: { 'frog' => nil })
      assert_nil doc.attributes['frog']
    end

"#
        );

        // Ruby passes { 'frog' => nil }; this crate's `unset` expresses the same
        // delete.
        let doc = load_with(":frog: Tanglefoot", &Options::new().unset("frog"));
        assert_eq!(doc.attribute_value("frog"), InterpretedValue::Unset);
    }

    #[test]
    fn doesn_t_choke_when_deleting_a_non_existing_attribute() {
        verifies!(
            r#"
    test "doesn't choke when deleting a non-existing attribute" do
      doc = document_from_string(':frog!:')
      assert_nil doc.attributes['frog']
    end

"#
        );

        assert_eq!(value_of(":frog!:", "frog"), InterpretedValue::Unset);
    }

    #[test]
    fn replaces_special_characters_in_attribute_value() {
        verifies!(
            r#"
    test "replaces special characters in attribute value" do
      doc = document_from_string(":xml-busters: <>&")
      assert_equal '&lt;&gt;&amp;', doc.attributes['xml-busters']
    end

"#
        );

        assert_eq!(
            value_of(":xml-busters: <>&", "xml-busters"),
            val("&lt;&gt;&amp;")
        );
    }

    #[test]
    fn performs_attribute_substitution_on_attribute_value() {
        verifies!(
            r#"
    test "performs attribute substitution on attribute value" do
      doc = document_from_string(":version: 1.0\n:release: Asciidoctor {version}")
      assert_equal 'Asciidoctor 1.0', doc.attributes['release']
    end

"#
        );

        assert_eq!(
            value_of(":version: 1.0\n:release: Asciidoctor {version}", "release"),
            val("Asciidoctor 1.0"),
        );
    }

    non_normative!(
        r#"
    test 'assigns attribute to empty string if substitution fails to resolve attribute' do
      input = ':release: Asciidoctor {version}'
      document_from_string input, attributes: { 'attribute-missing' => 'drop-line' }
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: version'
    end

"#
    );

    #[test]
    fn assigns_multi_line_attribute_to_empty_string_if_substitution_fails_to_resolve_attribute() {
        verifies!(
            r#"
    test 'assigns multi-line attribute to empty string if substitution fails to resolve attribute' do
      input = <<~'EOS'
      :release: Asciidoctor +
                {version}
      EOS
      doc = document_from_string input, attributes: { 'attribute-missing' => 'drop-line' }
      assert_equal '', doc.attributes['release']
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: version'
    end

"#
        );

        let src = ":release: Asciidoctor +\n          {version}\n";
        let doc = load_with(
            src,
            &Options::new().attribute("attribute-missing", "drop-line"),
        );
        // The unresolved reference blanks the value (Asciidoctor also logs an INFO
        // message, which asciidoc-parser does not surface: asciidoc-parser#1011).
        assert_eq!(doc.attribute_value("release"), val(""));
    }

    #[test]
    fn resolves_attributes_inside_attribute_value_within_header() {
        verifies!(
            r#"
    test 'resolves attributes inside attribute value within header' do
      input = <<~'EOS'
      = Document Title
      :big: big
      :bigfoot: {big}foot

      {bigfoot}
      EOS

      result = convert_string_to_embedded input
      assert_includes result, 'bigfoot'
    end

"#
        );

        let input = "= Document Title\n:big: big\n:bigfoot: {big}foot\n\n{bigfoot}\n";
        assert_includes(&convert(input), "bigfoot");
    }

    #[test]
    fn resolves_attributes_and_pass_macro_inside_attribute_value_outside_header() {
        verifies!(
            r#"
    test 'resolves attributes and pass macro inside attribute value outside header' do
      input = <<~'EOS'
      = Document Title

      content

      :big: pass:a,q[_big_]
      :bigfoot: {big}foot
      {bigfoot}
      EOS

      result = convert_string_to_embedded input
      assert_includes result, '<em>big</em>foot'
    end

"#
        );

        let input = "= Document Title\n\ncontent\n\n:big: pass:a,q[_big_]\n:bigfoot: {big}foot\n{bigfoot}\n";
        assert_includes(&convert(input), "<em>big</em>foot");
    }

    #[test]
    fn should_limit_maximum_size_of_attribute_value_if_safe_mode_is_secure() {
        verifies!(
            r#"
    test 'should limit maximum size of attribute value if safe mode is SECURE' do
      expected = 'a' * 4096
      input = <<~EOS
      :name: #{'a' * 5000}

      {name}
      EOS

      result = convert_inline_string input
      assert_equal expected, result
      assert_equal 4096, result.bytesize
    end

"#
        );

        let big = "a".repeat(5000);
        let src = format!(":name: {big}\n\n{{name}}\n");
        let doc = load_with(&src, &Options::new().safe_mode(SafeMode::Secure));
        assert_eq!(doc.attribute_value("name"), val(&"a".repeat(4096)));
    }

    #[test]
    fn should_handle_multibyte_characters_when_limiting_attribute_value_size() {
        verifies!(
            r#"
    test 'should handle multibyte characters when limiting attribute value size' do
      expected = '日本'
      input = <<~'EOS'
      :name: 日本語

      {name}
      EOS

      result = convert_inline_string input, attributes: { 'max-attribute-value-size' => 6 }
      assert_equal expected, result
      assert_equal 6, result.bytesize
    end

"#
        );

        let doc = load_with(
            ":name: 日本語\n\n{name}\n",
            &Options::new().attribute("max-attribute-value-size", "6"),
        );
        assert_eq!(doc.attribute_value("name"), val("日本"));
    }

    #[test]
    fn should_not_mangle_multibyte_characters_when_limiting_attribute_value_size() {
        verifies!(
            r#"
    test 'should not mangle multibyte characters when limiting attribute value size' do
      expected = '日本'
      input = <<~'EOS'
      :name: 日本語

      {name}
      EOS

      result = convert_inline_string input, attributes: { 'max-attribute-value-size' => 8 }
      assert_equal expected, result
      assert_equal 6, result.bytesize
    end

"#
        );

        let doc = load_with(
            ":name: 日本語\n\n{name}\n",
            &Options::new().attribute("max-attribute-value-size", "8"),
        );
        assert_eq!(doc.attribute_value("name"), val("日本"));
    }

    #[test]
    fn should_allow_maximize_size_of_attribute_value_to_be_disabled() {
        verifies!(
            r#"
    test 'should allow maximize size of attribute value to be disabled' do
      expected = 'a' * 5000
      input = <<~EOS
      :name: #{'a' * 5000}

      {name}
      EOS

      result = convert_inline_string input, attributes: { 'max-attribute-value-size' => nil }
      assert_equal expected, result
      assert_equal 5000, result.bytesize
    end

"#
        );

        let big = "a".repeat(5000);
        let src = format!(":name: {big}\n\n{{name}}\n");
        // Ruby passes max-attribute-value-size => nil; an empty value disables the cap.
        let doc = load_with(
            &src,
            &Options::new().attribute("max-attribute-value-size", ""),
        );
        assert_eq!(doc.attribute_value("name"), val(&big));
    }

    #[test]
    fn resolves_user_home_attribute_if_safe_mode_is_less_than_server() {
        verifies!(
            r#"
    test 'resolves user-home attribute if safe mode is less than SERVER' do
      input = <<~'EOS'
      :imagesdir: {user-home}/etc/images

      {imagesdir}
      EOS
      output = convert_inline_string input, safe: :safe
      assert_equal %(#{Asciidoctor::USER_HOME}/etc/images), output
    end

"#
        );

        let input = ":imagesdir: {user-home}/etc/images\n\n{imagesdir}\n";
        let output = convert_inline(input, Options::new().safe_mode(SafeMode::Safe));
        refute_includes(&output, "{user-home}");
        assert!(output.ends_with("/etc/images"), "output was: {output}");
        assert!(
            !output.starts_with("./"),
            "user-home should resolve to an absolute path, was: {output}"
        );
    }

    #[test]
    fn user_home_attribute_resolves_to_if_safe_mode_is_server_or_greater() {
        verifies!(
            r#"
    test 'user-home attribute resolves to . if safe mode is SERVER or greater' do
      input = <<~'EOS'
      :imagesdir: {user-home}/etc/images

      {imagesdir}
      EOS
      output = convert_inline_string input, safe: :server
      assert_equal './etc/images', output
    end

"#
        );

        let input = ":imagesdir: {user-home}/etc/images\n\n{imagesdir}\n";
        assert_eq!(
            convert_inline(input, Options::new().safe_mode(SafeMode::Server)),
            "./etc/images"
        );
    }

    #[test]
    fn user_home_attribute_can_be_overridden_by_api_if_safe_mode_is_less_than_server() {
        verifies!(
            r#"
    test 'user-home attribute can be overridden by API if safe mode is less than SERVER' do
      input = <<~'EOS'
      Go {user-home}!
      EOS
      output = convert_inline_string input, attributes: { 'user-home' => '/home' }
      assert_equal 'Go /home!', output
    end

"#
        );

        let output = convert_inline(
            "Go {user-home}!\n",
            Options::new().attribute("user-home", "/home"),
        );
        assert_eq!(output, "Go /home!");
    }

    #[test]
    fn user_home_attribute_can_be_overridden_by_api_if_safe_mode_is_server_or_greater() {
        verifies!(
            r#"
    test 'user-home attribute can be overridden by API if safe mode is SERVER or greater' do
      input = <<~'EOS'
      Go {user-home}!
      EOS
      output = convert_inline_string input, safe: :server, attributes: { 'user-home' => '/home' }
      assert_equal 'Go /home!', output
    end

"#
        );

        let output = convert_inline(
            "Go {user-home}!\n",
            Options::new()
                .safe_mode(SafeMode::Server)
                .attribute("user-home", "/home"),
        );
        assert_eq!(output, "Go /home!");
    }

    #[test]
    fn apply_custom_substitutions_to_text_in_passthrough_macro_and_assign_to_attribute() {
        verifies!(
            r#"
    test "apply custom substitutions to text in passthrough macro and assign to attribute" do
      doc = document_from_string(":xml-busters: pass:[<>&]")
      assert_equal '<>&', doc.attributes['xml-busters']
      doc = document_from_string(":xml-busters: pass:none[<>&]")
      assert_equal '<>&', doc.attributes['xml-busters']
      doc = document_from_string(":xml-busters: pass:specialcharacters[<>&]")
      assert_equal '&lt;&gt;&amp;', doc.attributes['xml-busters']
      doc = document_from_string(":xml-busters: pass:n,-c[<(C)>]")
      assert_equal '<&#169;>', doc.attributes['xml-busters']
    end

"#
        );

        assert_eq!(
            value_of(":xml-busters: pass:[<>&]", "xml-busters"),
            val("<>&")
        );
        assert_eq!(
            value_of(":xml-busters: pass:none[<>&]", "xml-busters"),
            val("<>&")
        );
        assert_eq!(
            value_of(":xml-busters: pass:specialcharacters[<>&]", "xml-busters"),
            val("&lt;&gt;&amp;"),
        );
        assert_eq!(
            value_of(":xml-busters: pass:n,-c[<(C)>]", "xml-busters"),
            val("<&#169;>")
        );
    }

    #[test]
    fn should_not_recognize_pass_macro_with_invalid_substitution_list_in_attribute_value() {
        verifies!(
            r#"
    test 'should not recognize pass macro with invalid substitution list in attribute value' do
      [',', '42', 'a,'].each do |subs|
        doc = document_from_string %(:pass-fail: pass:#{subs}[whale])
        assert_equal %(pass:#{subs}[whale]), doc.attributes['pass-fail']
      end
    end

"#
        );

        for subs in [",", "42", "a,"] {
            let src = format!(":pass-fail: pass:{subs}[whale]");
            assert_eq!(
                value_of(&src, "pass-fail"),
                val(&format!("pass:{subs}[whale]"))
            );
        }
    }

    #[test]
    fn attribute_is_treated_as_defined_until_it_s_not() {
        verifies!(
            r#"
    test "attribute is treated as defined until it's not" do
      input = <<~'EOS'
      :holygrail:
      ifdef::holygrail[]
      The holy grail has been found!
      endif::holygrail[]

      :holygrail!:
      ifndef::holygrail[]
      Buggers! What happened to the grail?
      endif::holygrail[]
      EOS
      output = convert_string input
      assert_xpath '//p', output, 2
      assert_xpath '(//p)[1][text() = "The holy grail has been found!"]', output, 1
      assert_xpath '(//p)[2][text() = "Buggers! What happened to the grail?"]', output, 1
    end

"#
        );

        let input = concat!(
            ":holygrail:\n",
            "ifdef::holygrail[]\n",
            "The holy grail has been found!\n",
            "endif::holygrail[]\n",
            "\n",
            ":holygrail!:\n",
            "ifndef::holygrail[]\n",
            "Buggers! What happened to the grail?\n",
            "endif::holygrail[]\n",
        );
        let output = convert_standalone(input);
        assert_xpath(&output, "//p", 2);
        assert_xpath(
            &output,
            r#"(//p)[1][text() = "The holy grail has been found!"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"(//p)[2][text() = "Buggers! What happened to the grail?"]"#,
            1,
        );
    }

    #[test]
    fn attribute_set_via_api_overrides_attribute_set_in_document() {
        verifies!(
            r#"
    test 'attribute set via API overrides attribute set in document' do
      doc = document_from_string(':cash: money', attributes: { 'cash' => 'heroes' })
      assert_equal 'heroes', doc.attributes['cash']
    end

"#
        );

        let doc = load_with(":cash: money", &Options::new().attribute("cash", "heroes"));
        assert_eq!(doc.attribute_value("cash"), val("heroes"));
    }

    #[test]
    fn attribute_set_via_api_cannot_be_unset_by_document() {
        verifies!(
            r#"
    test 'attribute set via API cannot be unset by document' do
      doc = document_from_string(':cash!:', attributes: { 'cash' => 'heroes' })
      assert_equal 'heroes', doc.attributes['cash']
    end

"#
        );

        let doc = load_with(":cash!:", &Options::new().attribute("cash", "heroes"));
        assert_eq!(doc.attribute_value("cash"), val("heroes"));
    }

    #[test]
    fn attribute_soft_set_via_api_using_modifier_on_name_can_be_overridden_by_document() {
        verifies!(
            r#"
    test 'attribute soft set via API using modifier on name can be overridden by document' do
      doc = document_from_string(':cash: money', attributes: { 'cash@' => 'heroes' })
      assert_equal 'money', doc.attributes['cash']
    end

"#
        );

        let doc = load_with(
            ":cash: money",
            &Options::new().attribute_default("cash", "heroes"),
        );
        assert_eq!(doc.attribute_value("cash"), val("money"));
    }

    #[test]
    fn attribute_soft_set_via_api_using_modifier_on_value_can_be_overridden_by_document() {
        verifies!(
            r#"
    test 'attribute soft set via API using modifier on value can be overridden by document' do
      doc = document_from_string(':cash: money', attributes: { 'cash' => 'heroes@' })
      assert_equal 'money', doc.attributes['cash']
    end

"#
        );

        // Ruby spells the soft default with a value-side `@` (`heroes@`); this crate's
        // soft-default option expresses the same override-me-in-document semantics.
        let doc = load_with(
            ":cash: money",
            &Options::new().attribute_default("cash", "heroes"),
        );
        assert_eq!(doc.attribute_value("cash"), val("money"));
    }

    #[test]
    fn attribute_soft_set_via_api_using_modifier_on_name_can_be_unset_by_document() {
        verifies!(
            r#"
    test 'attribute soft set via API using modifier on name can be unset by document' do
      doc = document_from_string(':cash!:', attributes: { 'cash@' => 'heroes' })
      assert_nil doc.attributes['cash']
      doc = document_from_string(':cash!:', attributes: { 'cash@' => true })
      assert_nil doc.attributes['cash']
    end

"#
        );

        let doc = load_with(
            ":cash!:",
            &Options::new().attribute_default("cash", "heroes"),
        );
        assert_eq!(doc.attribute_value("cash"), InterpretedValue::Unset);
        let doc = load_with(":cash!:", &Options::new().set_default("cash"));
        assert_eq!(doc.attribute_value("cash"), InterpretedValue::Unset);
    }

    #[test]
    fn attribute_soft_set_via_api_using_modifier_on_value_can_be_unset_by_document() {
        verifies!(
            r#"
    test 'attribute soft set via API using modifier on value can be unset by document' do
      doc = document_from_string(':cash!:', attributes: { 'cash' => 'heroes@' })
      assert_nil doc.attributes['cash']
    end

"#
        );

        let doc = load_with(
            ":cash!:",
            &Options::new().attribute_default("cash", "heroes"),
        );
        assert_eq!(doc.attribute_value("cash"), InterpretedValue::Unset);
    }

    #[test]
    fn attribute_unset_via_api_cannot_be_set_by_document() {
        verifies!(
            r#"
    test 'attribute unset via API cannot be set by document' do
      [
        { 'cash!' => '' },
        { '!cash' => '' },
        { 'cash' => nil },
      ].each do |attributes|
        doc = document_from_string(':cash: money', attributes: attributes)
        assert_nil doc.attributes['cash']
      end
    end

"#
        );

        let doc = load_with(":cash: money", &Options::new().unset("cash"));
        assert_eq!(doc.attribute_value("cash"), InterpretedValue::Unset);
    }

    #[test]
    fn attribute_soft_unset_via_api_can_be_set_by_document() {
        verifies!(
            r#"
    test 'attribute soft unset via API can be set by document' do
      [
        { 'cash!@' => '' },
        { '!cash@' => '' },
        { 'cash!' => '@' },
        { '!cash' => '@' },
        { 'cash' => false },
      ].each do |attributes|
        doc = document_from_string(':cash: money', attributes: attributes)
        assert_equal 'money', doc.attributes['cash']
      end
    end

"#
        );

        let doc = load_with(":cash: money", &Options::new().unset_default("cash"));
        assert_eq!(doc.attribute_value("cash"), val("money"));
    }

    #[test]
    fn can_soft_unset_built_in_attribute_from_api_and_still_override_in_document() {
        verifies!(
            r#"
    test 'can soft unset built-in attribute from API and still override in document' do
      [
        { 'sectids!@' => '' },
        { '!sectids@' => '' },
        { 'sectids!' => '@' },
        { '!sectids' => '@' },
        { 'sectids' => false },
      ].each do |attributes|
        doc = document_from_string '== Heading', attributes: attributes
        refute doc.attr?('sectids')
        assert_css '#_heading', (doc.convert standalone: false), 0
        doc = document_from_string %(:sectids:\n\n== Heading), attributes: attributes
        assert doc.attr?('sectids')
        assert_css '#_heading', (doc.convert standalone: false), 1
      end
    end

"#
        );

        let doc = load_with("== Heading", &Options::new().unset_default("sectids"));
        assert!(!doc.is_attribute_set("sectids"));
        assert_css(
            &convert_with("== Heading", &Options::new().unset_default("sectids")),
            "#_heading",
            0,
        );
        let doc = load_with(
            ":sectids:\n\n== Heading",
            &Options::new().unset_default("sectids"),
        );
        assert!(doc.is_attribute_set("sectids"));
        assert_css(
            &convert_with(
                ":sectids:\n\n== Heading",
                &Options::new().unset_default("sectids"),
            ),
            "#_heading",
            1,
        );
    }

    #[test]
    fn backend_and_doctype_attributes_are_set_by_default_in_default_configuration() {
        verifies!(
            r#"
    test 'backend and doctype attributes are set by default in default configuration' do
      input = <<~'EOS'
      = Document Title
      Author Name

      content
      EOS

      doc = document_from_string input
      expect = {
        'backend' => 'html5',
        'backend-html5' => '',
        'backend-html5-doctype-article' => '',
        'outfilesuffix' => '.html',
        'basebackend' => 'html',
        'basebackend-html' => '',
        'basebackend-html-doctype-article' => '',
        'doctype' => 'article',
        'doctype-article' => '',
        'filetype' => 'html',
        'filetype-html' => '',
      }
      expect.each do |key, val|
        assert doc.attributes.key? key
        assert_equal val, doc.attributes[key]
      end
    end

"#
        );

        let input = "= Document Title\nAuthor Name\n\ncontent\n";
        let doc = load(input);
        for (key, v) in [
            ("backend", "html5"),
            ("backend-html5", ""),
            ("backend-html5-doctype-article", ""),
            ("outfilesuffix", ".html"),
            ("basebackend", "html"),
            ("basebackend-html", ""),
            ("basebackend-html-doctype-article", ""),
            ("doctype", "article"),
            ("doctype-article", ""),
            ("filetype", "html"),
            ("filetype-html", ""),
        ] {
            assert!(doc.has_attribute(key), "missing attribute {key}");
            assert_eq!(doc.attribute_value(key), val(v), "attribute {key}");
        }
    }

    non_normative!(
        r#"
    test 'backend and doctype attributes are set by default in custom configuration' do
      input = <<~'EOS'
      = Document Title
      Author Name

      content
      EOS

      doc = document_from_string input, doctype: 'book', backend: 'docbook'
      expect = {
        'backend' => 'docbook5',
        'backend-docbook5' => '',
        'backend-docbook5-doctype-book' => '',
        'outfilesuffix' => '.xml',
        'basebackend' => 'docbook',
        'basebackend-docbook' => '',
        'basebackend-docbook-doctype-book' => '',
        'doctype' => 'book',
        'doctype-book' => '',
        'filetype' => 'xml',
        'filetype-xml' => '',
      }
      expect.each do |key, val|
        assert doc.attributes.key? key
        assert_equal val, doc.attributes[key]
      end
    end

    test 'backend attributes are updated if backend attribute is defined in document and safe mode is less than SERVER' do
      input = <<~'EOS'
      = Document Title
      Author Name
      :backend: docbook
      :doctype: book

      content
      EOS

      doc = document_from_string input, safe: Asciidoctor::SafeMode::SAFE
      expect = {
        'backend' => 'docbook5',
        'backend-docbook5' => '',
        'backend-docbook5-doctype-book' => '',
        'outfilesuffix' => '.xml',
        'basebackend' => 'docbook',
        'basebackend-docbook' => '',
        'basebackend-docbook-doctype-book' => '',
        'doctype' => 'book',
        'doctype-book' => '',
        'filetype' => 'xml',
        'filetype-xml' => '',
      }
      expect.each do |key, val|
        assert doc.attributes.key?(key)
        assert_equal val, doc.attributes[key]
      end

      refute doc.attributes.key?('backend-html5')
      refute doc.attributes.key?('backend-html5-doctype-article')
      refute doc.attributes.key?('basebackend-html')
      refute doc.attributes.key?('basebackend-html-doctype-article')
      refute doc.attributes.key?('doctype-article')
      refute doc.attributes.key?('filetype-html')
    end

"#
    );

    #[test]
    fn backend_attributes_defined_in_document_options_overrides_backend_attribute_in_document() {
        verifies!(
            r#"
    test 'backend attributes defined in document options overrides backend attribute in document' do
      doc = document_from_string(':backend: docbook5', safe: Asciidoctor::SafeMode::SAFE, attributes: { 'backend' => 'html5' })
      assert_equal 'html5', doc.attributes['backend']
      assert doc.attributes.key? 'backend-html5'
      assert_equal 'html', doc.attributes['basebackend']
      assert doc.attributes.key? 'basebackend-html'
    end

"#
        );

        let doc = load_with(
            ":backend: docbook5",
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute("backend", "html5"),
        );
        assert_eq!(doc.attribute_value("backend"), val("html5"));
        assert!(doc.has_attribute("backend-html5"));
        assert_eq!(doc.attribute_value("basebackend"), val("html"));
        assert!(doc.has_attribute("basebackend-html"));
    }

    non_normative!(
        r#"
    test 'can only access a positional attribute from the attributes hash' do
      node = Asciidoctor::Block.new nil, :paragraph, attributes: { 1 => 'position 1' }
      assert_nil node.attr(1)
      refute node.attr?(1)
      assert_equal 'position 1', node.attributes[1]
    end

    test 'attr should not retrieve attribute from document if not set on block' do
      doc = document_from_string 'paragraph', attributes: { 'name' => 'value' }
      para = doc.blocks[0]
      assert_nil para.attr 'name'
    end

    test 'attr looks for attribute on document if fallback name is true' do
      doc = document_from_string 'paragraph', attributes: { 'name' => 'value' }
      para = doc.blocks[0]
      assert_equal 'value', (para.attr 'name', nil, true)
    end

    test 'attr uses fallback name when looking for attribute on document' do
      doc = document_from_string 'paragraph', attributes: { 'alt-name' => 'value' }
      para = doc.blocks[0]
      assert_equal 'value', (para.attr 'name', nil, 'alt-name')
    end

    test 'attr? should not check for attribute on document if not set on block' do
      doc = document_from_string 'paragraph', attributes: { 'name' => 'value' }
      para = doc.blocks[0]
      refute para.attr? 'name'
    end

    test 'attr? checks for attribute on document if fallback name is true' do
      doc = document_from_string 'paragraph', attributes: { 'name' => 'value' }
      para = doc.blocks[0]
      assert para.attr? 'name', nil, true
    end

    test 'attr? checks for fallback name when looking for attribute on document' do
      doc = document_from_string 'paragraph', attributes: { 'alt-name' => 'value' }
      para = doc.blocks[0]
      assert para.attr? 'name', nil, 'alt-name'
    end

    test 'set_attr should set value to empty string if no value is specified' do
      node = Asciidoctor::Block.new nil, :paragraph, attributes: {}
      node.set_attr 'foo'
      assert_equal '', (node.attr 'foo')
    end

    test 'remove_attr should remove attribute and return previous value' do
      doc = empty_document
      node = Asciidoctor::Block.new doc, :paragraph, attributes: { 'foo' => 'bar' }
      assert_equal 'bar', (node.remove_attr 'foo')
      assert_nil node.attr('foo')
    end

    test 'set_attr should not overwrite existing key if overwrite is false' do
      node = Asciidoctor::Block.new nil, :paragraph, attributes: { 'foo' => 'bar' }
      assert_equal 'bar', (node.attr 'foo')
      node.set_attr 'foo', 'baz', false
      assert_equal 'bar', (node.attr 'foo')
    end

    test 'set_attr should overwrite existing key by default' do
      node = Asciidoctor::Block.new nil, :paragraph, attributes: { 'foo' => 'bar' }
      assert_equal 'bar', (node.attr 'foo')
      node.set_attr 'foo', 'baz'
      assert_equal 'baz', (node.attr 'foo')
    end

    test 'set_attr should set header attribute in loaded document' do
      input = <<~'EOS'
      :uri: http://example.org

      {uri}
      EOS

      doc = Asciidoctor.load input, attributes: { 'uri' => 'https://github.com' }
      doc.set_attr 'uri', 'https://google.com'
      output = doc.convert
      assert_xpath '//a[@href="https://google.com"]', output, 1
    end

    test 'set_attribute should set attribute if key is not locked' do
      doc = empty_document
      refute doc.attr? 'foo'
      res = doc.set_attribute 'foo', 'baz'
      assert res
      assert_equal 'baz', (doc.attr 'foo')
    end

    test 'set_attribute should not set key if key is locked' do
      doc = empty_document attributes: { 'foo' => 'bar' }
      assert_equal 'bar', (doc.attr 'foo')
      res = doc.set_attribute 'foo', 'baz'
      refute res
      assert_equal 'bar', (doc.attr 'foo')
    end

    test 'set_attribute should update backend attributes' do
      doc = empty_document attributes: { 'backend' => 'html5@' }
      assert_equal '', (doc.attr 'backend-html5')
      res = doc.set_attribute 'backend', 'docbook5'
      assert res
      refute doc.attr? 'backend-html5'
      assert_equal '', (doc.attr 'backend-docbook5')
    end

    test 'verify toc attribute matrix' do
      expected_data = <<~'EOS'
      #attributes                               |toc|toc-position|toc-placement|toc-class
      toc                                       |   |nil         |auto         |nil
      toc=header                                |   |nil         |auto         |nil
      toc=beeboo                                |   |nil         |auto         |nil
      toc=left                                  |   |left        |auto         |toc2
      toc2                                      |   |left        |auto         |toc2
      toc=right                                 |   |right       |auto         |toc2
      toc=preamble                              |   |content     |preamble     |nil
      toc=macro                                 |   |content     |macro        |nil
      toc toc-placement=macro toc-position=left |   |content     |macro        |nil
      toc toc-placement!                        |   |content     |macro        |nil
      EOS

      expected = expected_data.lines.map do |l|
        next if l.start_with? '#'
        l.split('|').map {|e| (e = e.strip) == 'nil' ? nil : e }
      end.compact

      expected.each do |expect|
        raw_attrs, toc, toc_position, toc_placement, toc_class = expect
        attrs = Hash[*raw_attrs.split.map {|e| e.include?('=') ? e.split('=', 2) : [e, ''] }.flatten]
        doc = document_from_string '', attributes: attrs
        toc ? (assert doc.attr?('toc', toc)) : (refute doc.attr?('toc'))
        toc_position ? (assert doc.attr?('toc-position', toc_position)) : (refute doc.attr?('toc-position'))
        toc_placement ? (assert doc.attr?('toc-placement', toc_placement)) : (refute doc.attr?('toc-placement'))
        toc_class ? (assert doc.attr?('toc-class', toc_class)) : (refute doc.attr?('toc-class'))
      end
    end
"#
    );

    non_normative!(
        r#"
  end

"#
    );
}

mod interpolation {
    use super::*;
    non_normative!(
        r#"
  context 'Interpolation' do

"#
    );

    #[test]
    fn convert_properly_with_simple_names() {
        verifies!(
            r#"
    test "convert properly with simple names" do
      html = convert_string(":frog: Tanglefoot\n:my_super-hero: Spiderman\n\nYo, {frog}!\nBeat {my_super-hero}!")
      assert_xpath %(//p[text()="Yo, Tanglefoot!\nBeat Spiderman!"]), html, 1
    end

"#
        );

        let html = convert_standalone(
            ":frog: Tanglefoot\n:my_super-hero: Spiderman\n\nYo, {frog}!\nBeat {my_super-hero}!",
        );
        assert_xpath(&html, "//p[text()=\"Yo, Tanglefoot!\nBeat Spiderman!\"]", 1);
    }

    #[test]
    fn attribute_lookup_is_not_case_sensitive() {
        verifies!(
            r#"
    test 'attribute lookup is not case sensitive' do
      input = <<~'EOS'
      :He-Man: The most powerful man in the universe

      He-Man: {He-Man}

      She-Ra: {She-Ra}
      EOS
      result = convert_string_to_embedded input, attributes: { 'She-Ra' => 'The Princess of Power' }
      assert_xpath '//p[text()="He-Man: The most powerful man in the universe"]', result, 1
      assert_xpath '//p[text()="She-Ra: The Princess of Power"]', result, 1
    end

"#
        );

        let input = ":He-Man: The most powerful man in the universe\n\nHe-Man: {He-Man}\n\nShe-Ra: {She-Ra}\n";
        let result = convert_with(
            input,
            &Options::new().attribute("She-Ra", "The Princess of Power"),
        );
        assert_xpath(
            &result,
            r#"//p[text()="He-Man: The most powerful man in the universe"]"#,
            1,
        );
        assert_xpath(&result, r#"//p[text()="She-Ra: The Princess of Power"]"#, 1);
    }

    #[test]
    fn convert_properly_with_single_character_name() {
        verifies!(
            r#"
    test "convert properly with single character name" do
      html = convert_string(":r: Ruby\n\nR is for {r}!")
      assert_xpath %(//p[text()="R is for Ruby!"]), html, 1
    end

"#
        );

        let html = convert_standalone(":r: Ruby\n\nR is for {r}!");
        assert_xpath(&html, r#"//p[text()="R is for Ruby!"]"#, 1);
    }

    non_normative!(
        r#"
    test "collapses spaces in attribute names" do
      input = <<~'EOS'
      Main Header
      ===========
      :My frog: Tanglefoot

      Yo, {myfrog}!
      EOS
      output = convert_string input
      assert_xpath '(//p)[1][text()="Yo, Tanglefoot!"]', output, 1
    end

"#
    );

    #[test]
    fn ignores_lines_with_bad_attributes_if_attribute_missing_is_drop_line() {
        verifies!(
            r#"
    test 'ignores lines with bad attributes if attribute-missing is drop-line' do
      input = <<~'EOS'
      :attribute-missing: drop-line

      This is
      blah blah {foobarbaz}
      all there is.
      EOS
      output = convert_string_to_embedded input
      para = xmlnodes_at_css 'p', output, 1
      refute_includes 'blah blah', para.content
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: foobarbaz'
    end

"#
        );

        let input =
            ":attribute-missing: drop-line\n\nThis is\nblah blah {foobarbaz}\nall there is.\n";
        let output = convert(input);
        // The line referencing the missing attribute is dropped (Asciidoctor also logs
        // an INFO message, which asciidoc-parser does not surface:
        // asciidoc-parser#1011).
        refute_includes(&output, "blah blah");
    }

    #[test]
    fn attribute_value_gets_interpreted_when_converting() {
        verifies!(
            r#"
    test 'attribute value gets interpreted when converting' do
      doc = document_from_string(":google: http://google.com[Google]\n\n{google}")
      assert_equal 'http://google.com[Google]', doc.attributes['google']
      output = doc.convert
      assert_xpath '//a[@href="http://google.com"][text() = "Google"]', output, 1
    end

"#
        );

        let input = ":google: http://google.com[Google]\n\n{google}";
        assert_eq!(value_of(input, "google"), val("http://google.com[Google]"));
        let output = convert_standalone(input);
        assert_xpath(
            &output,
            r#"//a[@href="http://google.com"][text() = "Google"]"#,
            1,
        );
    }

    #[test]
    fn should_drop_line_with_reference_to_missing_attribute_if_attribute_missing_attribute_is_drop_line(
    ) {
        verifies!(
            r#"
    test 'should drop line with reference to missing attribute if attribute-missing attribute is drop-line' do
      input = <<~'EOS'
      :attribute-missing: drop-line

      Line 1: This line should appear in the output.
      Line 2: Oh no, a {bogus-attribute}! This line should not appear in the output.
      EOS

      output = convert_string_to_embedded input
      assert_match(/Line 1/, output)
      refute_match(/Line 2/, output)
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: bogus-attribute'
    end

"#
        );

        let input = ":attribute-missing: drop-line\n\nLine 1: This line should appear in the output.\nLine 2: Oh no, a {bogus-attribute}! This line should not appear in the output.\n";
        let output = convert(input);
        assert_includes(&output, "Line 1");
        refute_includes(&output, "Line 2");
    }

    #[test]
    fn should_not_drop_line_with_reference_to_missing_attribute_by_default() {
        verifies!(
            r#"
    test 'should not drop line with reference to missing attribute by default' do
      input = <<~'EOS'
      Line 1: This line should appear in the output.
      Line 2: A {bogus-attribute}! This time, this line should appear in the output.
      EOS

      output = convert_string_to_embedded input
      assert_match(/Line 1/, output)
      assert_match(/Line 2/, output)
      assert_match(/\{bogus-attribute\}/, output)
    end

"#
        );

        let input = "Line 1: This line should appear in the output.\nLine 2: A {bogus-attribute}! This time, this line should appear in the output.\n";
        let output = convert(input);
        assert_includes(&output, "Line 1");
        assert_includes(&output, "Line 2");
        assert_includes(&output, "{bogus-attribute}");
    }

    non_normative!(
        r#"
    test 'should drop line with attribute unassignment by default' do
      input = <<~'EOS'
      :a:

      Line 1: This line should appear in the output.
      Line 2: {set:a!}This line should not appear in the output.
      EOS

      output = convert_string_to_embedded input
      assert_match(/Line 1/, output)
      refute_match(/Line 2/, output)
    end

    test 'should not drop line with attribute unassignment if attribute-undefined is drop' do
      input = <<~'EOS'
      :attribute-undefined: drop
      :a:

      Line 1: This line should appear in the output.
      Line 2: {set:a!}This line should appear in the output.
      EOS

      output = convert_string_to_embedded input
      assert_match(/Line 1/, output)
      assert_match(/Line 2/, output)
      refute_match(/\{set:a!\}/, output)
    end

    test 'should drop line that only contains attribute assignment' do
      input = <<~'EOS'
      Line 1
      {set:a}
      Line 2
      EOS

      output = convert_string_to_embedded input
      assert_xpath %(//p[text()="Line 1\nLine 2"]), output, 1
    end

"#
    );

    #[test]
    fn should_drop_line_that_only_contains_unresolved_attribute_when_attribute_missing_is_drop() {
        verifies!(
            r#"
    test 'should drop line that only contains unresolved attribute when attribute-missing is drop' do
      input = <<~'EOS'
      Line 1
      {unresolved}
      Line 2
      EOS

      output = convert_string_to_embedded input, attributes: { 'attribute-missing' => 'drop' }
      assert_xpath %(//p[text()="Line 1\nLine 2"]), output, 1
    end

"#
        );

        let input = "Line 1\n{unresolved}\nLine 2\n";
        let output = convert_with(
            input,
            &Options::new().attribute("attribute-missing", "drop"),
        );
        assert_xpath(&output, "//p[text()=\"Line 1\nLine 2\"]", 1);
    }

    #[test]
    fn substitutes_inside_unordered_list_items() {
        verifies!(
            r#"
    test "substitutes inside unordered list items" do
      html = convert_string(":foo: bar\n* snort at the {foo}\n* yawn")
      assert_xpath %(//li/p[text()="snort at the bar"]), html, 1
    end

"#
        );

        let html = convert_standalone(":foo: bar\n* snort at the {foo}\n* yawn");
        assert_xpath(&html, r#"//li/p[text()="snort at the bar"]"#, 1);
    }

    #[test]
    fn substitutes_inside_section_title() {
        verifies!(
            r#"
    test 'substitutes inside section title' do
      output = convert_string(":prefix: Cool\n\n== {prefix} Title\n\ncontent")
      assert_xpath '//h2[text()="Cool Title"]', output, 1
      assert_css 'h2#_cool_title', output, 1
    end

"#
        );

        let output = convert_standalone(":prefix: Cool\n\n== {prefix} Title\n\ncontent");
        assert_xpath(&output, r#"//h2[text()="Cool Title"]"#, 1);
        assert_css(&output, "h2#_cool_title", 1);
    }

    #[test]
    fn interpolates_attribute_defined_in_header_inside_attribute_entry_in_header() {
        verifies!(
            r#"
    test 'interpolates attribute defined in header inside attribute entry in header' do
      input = <<~'EOS'
      = Title
      Author Name
      :attribute-a: value
      :attribute-b: {attribute-a}

      preamble
      EOS
      doc = document_from_string(input, parse_header_only: true)
      assert_equal 'value', doc.attributes['attribute-b']
    end

"#
        );

        let input =
            "= Title\nAuthor Name\n:attribute-a: value\n:attribute-b: {attribute-a}\n\npreamble\n";
        assert_eq!(value_of(input, "attribute-b"), val("value"));
    }

    #[test]
    fn interpolates_author_attribute_inside_attribute_entry_in_header() {
        verifies!(
            r#"
    test 'interpolates author attribute inside attribute entry in header' do
      input = <<~'EOS'
      = Title
      Author Name
      :name: {author}

      preamble
      EOS
      doc = document_from_string(input, parse_header_only: true)
      assert_equal 'Author Name', doc.attributes['name']
    end

"#
        );

        let input = "= Title\nAuthor Name\n:name: {author}\n\npreamble\n";
        assert_eq!(value_of(input, "name"), val("Author Name"));
    }

    #[test]
    fn interpolates_revinfo_attribute_inside_attribute_entry_in_header() {
        verifies!(
            r#"
    test 'interpolates revinfo attribute inside attribute entry in header' do
      input = <<~'EOS'
      = Title
      Author Name
      2013-01-01
      :date: {revdate}

      preamble
      EOS
      doc = document_from_string(input, parse_header_only: true)
      assert_equal '2013-01-01', doc.attributes['date']
    end

"#
        );

        let input = "= Title\nAuthor Name\n2013-01-01\n:date: {revdate}\n\npreamble\n";
        assert_eq!(value_of(input, "date"), val("2013-01-01"));
    }

    #[test]
    fn attribute_entries_can_resolve_previously_defined_attributes() {
        verifies!(
            r#"
    test 'attribute entries can resolve previously defined attributes' do
      input = <<~'EOS'
      = Title
      Author Name
      v1.0, 2010-01-01: First release!
      :a: value
      :a2: {a}
      :revdate2: {revdate}

      {a} == {a2}

      {revdate} == {revdate2}
      EOS

      doc = document_from_string input
      assert_equal '2010-01-01', doc.attr('revdate')
      assert_equal '2010-01-01', doc.attr('revdate2')
      assert_equal 'value', doc.attr('a')
      assert_equal 'value', doc.attr('a2')

      output = doc.convert
      assert_includes output, 'value == value'
      assert_includes output, '2010-01-01 == 2010-01-01'
    end

"#
        );

        let input = "= Title\nAuthor Name\nv1.0, 2010-01-01: First release!\n:a: value\n:a2: {a}\n:revdate2: {revdate}\n\n{a} == {a2}\n\n{revdate} == {revdate2}\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("revdate"), val("2010-01-01"));
        assert_eq!(doc.attribute_value("revdate2"), val("2010-01-01"));
        assert_eq!(doc.attribute_value("a"), val("value"));
        assert_eq!(doc.attribute_value("a2"), val("value"));
        let output = convert_standalone(input);
        assert_includes(&output, "value == value");
        assert_includes(&output, "2010-01-01 == 2010-01-01");
    }

    #[test]
    fn should_warn_if_unterminated_block_comment_is_detected_in_document_header() {
        verifies!(
            r#"
    test 'should warn if unterminated block comment is detected in document header' do
      input = <<~'EOS'
      = Document Title
      :foo: bar
      ////
      :hey: there

      content
      EOS
      doc = document_from_string input
      assert_nil doc.attr('hey')
      assert_message @logger, :WARN, '<stdin>: line 3: unterminated comment block', Hash
    end

"#
        );

        let input = "= Document Title\n:foo: bar\n////\n:hey: there\n\ncontent\n";
        let doc = load(input);
        assert!(!doc.is_attribute_set("hey"));
        // Asciidoctor logs "unterminated comment block"; asciidoc-parser reports the
        // same condition as an UnterminatedDelimitedBlock warning.
        assert!(doc
            .warnings()
            .any(|w| w.warning == WarningType::UnterminatedDelimitedBlock));
    }

    non_normative!(
        r#"
    test 'substitutes inside block title' do
      input = <<~'EOS'
      :gem_name: asciidoctor

      .Require the +{gem_name}+ gem
      To use {gem_name}, the first thing to do is to import it in your Ruby source file.
      EOS
      output = convert_string_to_embedded input, attributes: { 'compat-mode' => '' }
      assert_xpath '//*[@class="title"]/code[text()="asciidoctor"]', output, 1

      input = <<~'EOS'
      :gem_name: asciidoctor

      .Require the `{gem_name}` gem
      To use {gem_name}, the first thing to do is to import it in your Ruby source file.
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//*[@class="title"]/code[text()="asciidoctor"]', output, 1
    end

"#
    );

    #[test]
    fn sets_attribute_until_it_is_deleted() {
        verifies!(
            r#"
    test 'sets attribute until it is deleted' do
      input = <<~'EOS'
      :foo: bar

      Crossing the {foo}.

      :foo!:

      Belly up to the {foo}.
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//p[text()="Crossing the bar."]', output, 1
      assert_xpath '//p[text()="Belly up to the bar."]', output, 0
    end

"#
        );

        let input = ":foo: bar\n\nCrossing the {foo}.\n\n:foo!:\n\nBelly up to the {foo}.\n";
        let output = convert(input);
        assert_xpath(&output, r#"//p[text()="Crossing the bar."]"#, 1);
        assert_xpath(&output, r#"//p[text()="Belly up to the bar."]"#, 0);
    }

    non_normative!(
        r#"
    test 'should allow compat-mode to be set and unset in middle of document' do
      input = <<~'EOS'
      :foo: bar

      [[paragraph-a]]
      `{foo}`

      :compat-mode!:

      [[paragraph-b]]
      `{foo}`

      :compat-mode:

      [[paragraph-c]]
      `{foo}`
      EOS

      result = convert_string_to_embedded input, attributes: { 'compat-mode' => '@' }
      assert_xpath '/*[@id="paragraph-a"]//code[text()="{foo}"]', result, 1
      assert_xpath '/*[@id="paragraph-b"]//code[text()="bar"]', result, 1
      assert_xpath '/*[@id="paragraph-c"]//code[text()="{foo}"]', result, 1
    end

"#
    );

    #[test]
    fn does_not_disturb_attribute_looking_things_escaped_with_backslash() {
        verifies!(
            r#"
    test 'does not disturb attribute-looking things escaped with backslash' do
      html = convert_string(":foo: bar\nThis is a \\{foo} day.")
      assert_xpath '//p[text()="This is a {foo} day."]', html, 1
    end

"#
        );

        let html = convert_standalone(":foo: bar\nThis is a \\{foo} day.");
        assert_xpath(&html, r#"//p[text()="This is a {foo} day."]"#, 1);
    }

    #[test]
    fn does_not_disturb_attribute_looking_things_escaped_with_literals() {
        verifies!(
            r#"
    test 'does not disturb attribute-looking things escaped with literals' do
      html = convert_string(":foo: bar\nThis is a +++{foo}+++ day.")
      assert_xpath '//p[text()="This is a {foo} day."]', html, 1
    end

"#
        );

        let html = convert_standalone(":foo: bar\nThis is a +++{foo}+++ day.");
        assert_xpath(&html, r#"//p[text()="This is a {foo} day."]"#, 1);
    }

    #[test]
    fn does_not_substitute_attributes_inside_listing_blocks() {
        verifies!(
            r#"
    test 'does not substitute attributes inside listing blocks' do
      input = <<~'EOS'
      :forecast: snow

      ----
      puts 'The forecast for today is {forecast}'
      ----
      EOS
      output = convert_string(input)
      assert_match(/\{forecast\}/, output)
    end

"#
        );

        let input = ":forecast: snow\n\n----\nputs 'The forecast for today is {forecast}'\n----\n";
        assert_includes(&convert_standalone(input), "{forecast}");
    }

    #[test]
    fn does_not_substitute_attributes_inside_literal_blocks() {
        verifies!(
            r#"
    test 'does not substitute attributes inside literal blocks' do
      input = <<~'EOS'
      :foo: bar

      ....
      You insert the text {foo} to expand the value
      of the attribute named foo in your document.
      ....
      EOS
      output = convert_string(input)
      assert_match(/\{foo\}/, output)
    end

"#
        );

        let input = ":foo: bar\n\n....\nYou insert the text {foo} to expand the value\nof the attribute named foo in your document.\n....\n";
        assert_includes(&convert_standalone(input), "{foo}");
    }

    #[test]
    fn does_not_show_docdir_and_shows_relative_docfile_if_safe_mode_is_server_or_greater() {
        verifies!(
            r#"
    test 'does not show docdir and shows relative docfile if safe mode is SERVER or greater' do
      input = <<~'EOS'
      * docdir: {docdir}
      * docfile: {docfile}
      EOS

      docdir = Dir.pwd
      docfile = File.join(docdir, 'sample.adoc')
      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SERVER, attributes: { 'docdir' => docdir, 'docfile' => docfile }
      assert_xpath '//li[1]/p[text()="docdir: "]', output, 1
      assert_xpath '//li[2]/p[text()="docfile: sample.adoc"]', output, 1
    end

"#
        );

        let input = "* docdir: {docdir}\n* docfile: {docfile}\n";
        let output = convert_with(
            input,
            &Options::new()
                .safe_mode(SafeMode::Server)
                .attribute("docdir", "/path/to/docs")
                .attribute("docfile", "/path/to/docs/sample.adoc"),
        );
        assert_xpath(&output, r#"//li[1]/p[text()="docdir: "]"#, 1);
        assert_xpath(&output, r#"//li[2]/p[text()="docfile: sample.adoc"]"#, 1);
    }

    #[test]
    fn shows_absolute_docdir_and_docfile_paths_if_safe_mode_is_less_than_server() {
        verifies!(
            r#"
    test 'shows absolute docdir and docfile paths if safe mode is less than SERVER' do
      input = <<~'EOS'
      * docdir: {docdir}
      * docfile: {docfile}
      EOS

      docdir = Dir.pwd
      docfile = File.join(docdir, 'sample.adoc')
      output = convert_string_to_embedded input, safe: Asciidoctor::SafeMode::SAFE, attributes: { 'docdir' => docdir, 'docfile' => docfile }
      assert_xpath %(//li[1]/p[text()="docdir: #{docdir}"]), output, 1
      assert_xpath %(//li[2]/p[text()="docfile: #{docfile}"]), output, 1
    end

"#
        );

        let input = "* docdir: {docdir}\n* docfile: {docfile}\n";
        let docdir = "/path/to/docs";
        let docfile = "/path/to/docs/sample.adoc";
        let output = convert_with(
            input,
            &Options::new()
                .safe_mode(SafeMode::Safe)
                .attribute("docdir", docdir)
                .attribute("docfile", docfile),
        );
        assert_xpath(
            &output,
            &format!(r#"//li[1]/p[text()="docdir: {docdir}"]"#),
            1,
        );
        assert_xpath(
            &output,
            &format!(r#"//li[2]/p[text()="docfile: {docfile}"]"#),
            1,
        );
    }

    non_normative!(
        r#"
    test 'assigns attribute defined in attribute reference with set prefix and value' do
      input = '{set:foo:bar}{foo}'
      output = convert_string_to_embedded input
      assert_xpath '//p', output, 1
      assert_xpath '//p[text()="bar"]', output, 1
    end

    test 'assigns attribute defined in attribute reference with set prefix and no value' do
      input = "{set:foo}\n{foo}yes"
      output = convert_string_to_embedded input
      assert_xpath '//p', output, 1
      assert_xpath '//p[normalize-space(text())="yes"]', output, 1
    end

    test 'assigns attribute defined in attribute reference with set prefix and empty value' do
      input = "{set:foo:}\n{foo}yes"
      output = convert_string_to_embedded input
      assert_xpath '//p', output, 1
      assert_xpath '//p[normalize-space(text())="yes"]', output, 1
    end

    test 'unassigns attribute defined in attribute reference with set prefix' do
      input = <<~'EOS'
      :attribute-missing: drop-line
      :foo:

      {set:foo!}
      {foo}yes
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//p', output, 1
      assert_xpath '//p/child::text()', output, 0
      assert_message @logger, :INFO, 'dropping line containing reference to missing attribute: foo'
    end
"#
    );

    non_normative!(
        r#"
  end

"#
    );
}

mod intrinsic_attributes {
    use super::*;
    non_normative!(
        r#"
  context "Intrinsic attributes" do

"#
    );

    non_normative!(
        r#"
    test "substitute intrinsics" do
      Asciidoctor::INTRINSIC_ATTRIBUTES.each_pair do |key, value|
        html = convert_string("Look, a {#{key}} is here")
        # can't use Nokogiri because it interprets the HTML entities and we can't match them
        assert_match(/Look, a #{Regexp.escape(value)} is here/, html)
      end
    end

"#
    );

    #[test]
    fn don_t_escape_intrinsic_substitutions() {
        verifies!(
            r#"
    test "don't escape intrinsic substitutions" do
      html = convert_string('happy{nbsp}together')
      assert_match(/happy&#160;together/, html)
    end

"#
        );

        let html = convert_standalone("happy{nbsp}together");
        assert_includes(&html, "happy&#160;together");
    }

    #[test]
    fn escape_special_characters() {
        verifies!(
            r#"
    test "escape special characters" do
      html = convert_string('<node>&</node>')
      assert_match(/&lt;node&gt;&amp;&lt;\/node&gt;/, html)
    end

"#
        );

        let html = convert_standalone("<node>&</node>");
        assert_includes(&html, "&lt;node&gt;&amp;&lt;/node&gt;");
    }

    #[test]
    fn creates_counter() {
        verifies!(
            r#"
    test 'creates counter' do
      input = '{counter:mycounter}'

      doc = document_from_string input
      output = doc.convert
      assert_equal 1, doc.attributes['mycounter']
      assert_xpath '//p[text()="1"]', output, 1
    end

"#
        );

        let input = "{counter:mycounter}";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("1"));
        assert_xpath(&convert(input), r#"//p[text()="1"]"#, 1);
    }

    #[test]
    fn creates_counter_silently() {
        verifies!(
            r#"
    test 'creates counter silently' do
      input = '{counter2:mycounter}'

      doc = document_from_string input
      output = doc.convert
      assert_equal 1, doc.attributes['mycounter']
      assert_xpath '//p[text()="1"]', output, 0
    end

"#
        );

        let input = "{counter2:mycounter}";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("1"));
        assert_xpath(&convert(input), r#"//p[text()="1"]"#, 0);
    }

    #[test]
    fn creates_counter_with_numeric_seed_value() {
        verifies!(
            r#"
    test 'creates counter with numeric seed value' do
      input = '{counter2:mycounter:10}'

      doc = document_from_string input
      doc.convert
      assert_equal 10, doc.attributes['mycounter']
    end

"#
        );

        assert_eq!(value_of("{counter2:mycounter:10}", "mycounter"), val("10"));
    }

    #[test]
    fn creates_counter_with_character_seed_value() {
        verifies!(
            r#"
    test 'creates counter with character seed value' do
      input = '{counter2:mycounter:A}'

      doc = document_from_string input
      doc.convert
      assert_equal 'A', doc.attributes['mycounter']
    end

"#
        );

        assert_eq!(value_of("{counter2:mycounter:A}", "mycounter"), val("A"));
    }

    #[test]
    fn can_seed_counter_to_start_at_1() {
        verifies!(
            r#"
    test 'can seed counter to start at 1' do
      input = <<~'EOS'
      :mycounter: 0

      {counter:mycounter}
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p[text()="1"]', output, 1
    end

"#
        );

        let output = convert(":mycounter: 0\n\n{counter:mycounter}\n");
        assert_xpath(&output, r#"//p[text()="1"]"#, 1);
    }

    #[test]
    fn can_seed_counter_to_start_at_a() {
        verifies!(
            r#"
    test 'can seed counter to start at A' do
      input = <<~'EOS'
      :mycounter: @

      {counter:mycounter}
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//p[text()="A"]', output, 1
    end

"#
        );

        let output = convert(":mycounter: @\n\n{counter:mycounter}\n");
        assert_xpath(&output, r#"//p[text()="A"]"#, 1);
    }

    #[test]
    fn increments_counter_with_positive_numeric_value() {
        verifies!(
            r#"
    test 'increments counter with positive numeric value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:mycounter:1}
      {counter:mycounter}
      {counter:mycounter}
      {mycounter}
      ++++
      EOS

      doc = document_from_string input, standalone: false
      output = doc.convert
      assert_equal 3, doc.attributes['mycounter']
      assert_equal %w(1 2 3 3), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:mycounter:1}\n{counter:mycounter}\n{counter:mycounter}\n{mycounter}\n++++\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("3"));
        assert_includes(&convert(input), "1\n2\n3\n3");
    }

    #[test]
    fn increments_counter_with_negative_numeric_value() {
        verifies!(
            r#"
    test 'increments counter with negative numeric value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:mycounter:-2}
      {counter:mycounter}
      {counter:mycounter}
      {mycounter}
      ++++
      EOS

      doc = document_from_string input, standalone: false
      output = doc.convert
      assert_equal 0, doc.attributes['mycounter']
      assert_equal %w(-2 -1 0 0), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:mycounter:-2}\n{counter:mycounter}\n{counter:mycounter}\n{mycounter}\n++++\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("0"));
        assert_includes(&convert(input), "-2\n-1\n0\n0");
    }

    #[test]
    fn increments_counter_with_ascii_character_value() {
        verifies!(
            r#"
    test 'increments counter with ASCII character value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:mycounter:A}
      {counter:mycounter}
      {counter:mycounter}
      {mycounter}
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_equal %w(A B C C), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:mycounter:A}\n{counter:mycounter}\n{counter:mycounter}\n{mycounter}\n++++\n";
        assert_includes(&convert(input), "A\nB\nC\nC");
    }

    #[test]
    fn increments_counter_with_non_ascii_character_value() {
        verifies!(
            r#"
    test 'increments counter with non-ASCII character value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:mycounter:é}
      {counter:mycounter}
      {counter:mycounter}
      {mycounter}
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_equal %w(é ê ë ë), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:mycounter:é}\n{counter:mycounter}\n{counter:mycounter}\n{mycounter}\n++++\n";
        assert_includes(&convert(input), "é\nê\në\në");
    }

    #[test]
    fn increments_counter_with_emoji_character_value() {
        verifies!(
            r#"
    test 'increments counter with emoji character value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:smiley:😋}
      {counter:smiley}
      {counter:smiley}
      {smiley}
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_equal %w(😋 😌 😍 😍), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:smiley:😋}\n{counter:smiley}\n{counter:smiley}\n{smiley}\n++++\n";
        assert_includes(&convert(input), "😋\n😌\n😍\n😍");
    }

    #[test]
    fn increments_counter_with_multi_character_value() {
        verifies!(
            r#"
    test 'increments counter with multi-character value' do
      input = <<~'EOS'
      [subs=attributes]
      ++++
      {counter:math:1x}
      {counter:math}
      {counter:math}
      {math}
      ++++
      EOS

      output = convert_string_to_embedded input
      assert_equal %w(1x 1y 1z 1z), output.lines.map {|l| l.rstrip }
    end

"#
        );

        let input = "[subs=attributes]\n++++\n{counter:math:1x}\n{counter:math}\n{counter:math}\n{math}\n++++\n";
        assert_includes(&convert(input), "1x\n1y\n1z\n1z");
    }

    #[test]
    fn counter_uses_0_as_seed_value_if_seed_attribute_is_nil() {
        verifies!(
            r#"
    test 'counter uses 0 as seed value if seed attribute is nil' do
      input = <<~'EOS'
      :mycounter:

      {counter:mycounter}

      {mycounter}
      EOS

      doc = document_from_string input
      output = doc.convert standalone: false
      assert_equal 1, doc.attributes['mycounter']
      assert_xpath '//p[text()="1"]', output, 2
    end

"#
        );

        let input = ":mycounter:\n\n{counter:mycounter}\n\n{mycounter}\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("1"));
        assert_xpath(&convert(input), r#"//p[text()="1"]"#, 2);
    }

    #[test]
    fn counter_value_can_be_reset_by_attribute_entry() {
        verifies!(
            r#"
    test 'counter value can be reset by attribute entry' do
      input = <<~'EOS'
      :mycounter:

      before: {counter:mycounter} {counter:mycounter} {counter:mycounter}

      :mycounter!:

      after: {counter:mycounter}
      EOS

      doc = document_from_string input
      output = doc.convert standalone: false
      assert_equal 1, doc.attributes['mycounter']
      assert_xpath '//p[text()="before: 1 2 3"]', output, 1
      assert_xpath '//p[text()="after: 1"]', output, 1
    end

"#
        );

        let input = ":mycounter:\n\nbefore: {counter:mycounter} {counter:mycounter} {counter:mycounter}\n\n:mycounter!:\n\nafter: {counter:mycounter}\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("1"));
        let output = convert(input);
        assert_xpath(&output, r#"//p[text()="before: 1 2 3"]"#, 1);
        assert_xpath(&output, r#"//p[text()="after: 1"]"#, 1);
    }

    #[test]
    fn counter_value_can_be_advanced_by_attribute_entry() {
        verifies!(
            r#"
    test 'counter value can be advanced by attribute entry' do
      input = <<~'EOS'
      before: {counter:mycounter}

      :mycounter: 10

      after: {counter:mycounter}
      EOS

      doc = document_from_string input
      output = doc.convert standalone: false
      assert_equal 11, doc.attributes['mycounter']
      assert_xpath '//p[text()="before: 1"]', output, 1
      assert_xpath '//p[text()="after: 11"]', output, 1
    end

"#
        );

        let input = "before: {counter:mycounter}\n\n:mycounter: 10\n\nafter: {counter:mycounter}\n";
        let doc = load(input);
        assert_eq!(doc.attribute_value("mycounter"), val("11"));
        let output = convert(input);
        assert_xpath(&output, r#"//p[text()="before: 1"]"#, 1);
        assert_xpath(&output, r#"//p[text()="after: 11"]"#, 1);
    }

    #[test]
    fn nested_document_should_use_counter_from_parent_document() {
        verifies!(
            r#"
    test 'nested document should use counter from parent document' do
      input = <<~'EOS'
      .Title for Foo
      image::foo.jpg[]

      [cols="2*a"]
      |===
      |
      .Title for Bar
      image::bar.jpg[]

      |
      .Title for Baz
      image::baz.jpg[]
      |===

      .Title for Qux
      image::qux.jpg[]
      EOS

      output = convert_string_to_embedded input
      assert_xpath '//div[@class="title"]', output, 4
      assert_xpath '//div[@class="title"][text() = "Figure 1. Title for Foo"]', output, 1
      assert_xpath '//div[@class="title"][text() = "Figure 2. Title for Bar"]', output, 1
      assert_xpath '//div[@class="title"][text() = "Figure 3. Title for Baz"]', output, 1
      assert_xpath '//div[@class="title"][text() = "Figure 4. Title for Qux"]', output, 1
    end

"#
        );

        let input = ".Title for Foo\nimage::foo.jpg[]\n\n[cols=\"2*a\"]\n|===\n|\n.Title for Bar\nimage::bar.jpg[]\n\n|\n.Title for Baz\nimage::baz.jpg[]\n|===\n\n.Title for Qux\nimage::qux.jpg[]\n";
        let output = convert(input);
        assert_xpath(&output, r#"//div[@class="title"]"#, 4);
        assert_xpath(
            &output,
            r#"//div[@class="title"][text() = "Figure 1. Title for Foo"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//div[@class="title"][text() = "Figure 2. Title for Bar"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//div[@class="title"][text() = "Figure 3. Title for Baz"]"#,
            1,
        );
        assert_xpath(
            &output,
            r#"//div[@class="title"][text() = "Figure 4. Title for Qux"]"#,
            1,
        );
    }

    #[test]
    fn should_not_allow_counter_to_modify_locked_attribute() {
        verifies!(
            r#"
    test 'should not allow counter to modify locked attribute' do
      input = <<~'EOS'
      {counter:foo:ignored} is not {foo}
      EOS

      output = convert_string_to_embedded input, attributes: { 'foo' => 'bar' }
      assert_xpath '//p[text()="bas is not bar"]', output, 1
    end

"#
        );

        let output = convert_with(
            "{counter:foo:ignored} is not {foo}\n",
            &Options::new().attribute("foo", "bar"),
        );
        assert_xpath(&output, r#"//p[text()="bas is not bar"]"#, 1);
    }

    #[test]
    fn should_not_allow_counter2_to_modify_locked_attribute() {
        verifies!(
            r#"
    test 'should not allow counter2 to modify locked attribute' do
      input = <<~'EOS'
      {counter2:foo:ignored}{foo}
      EOS

      output = convert_string_to_embedded input, attributes: { 'foo' => 'bar' }
      assert_xpath '//p[text()="bar"]', output, 1
    end

"#
        );

        let output = convert_with(
            "{counter2:foo:ignored}{foo}\n",
            &Options::new().attribute("foo", "bar"),
        );
        assert_xpath(&output, r#"//p[text()="bar"]"#, 1);
    }

    #[test]
    fn should_not_allow_counter_to_modify_built_in_locked_attribute() {
        verifies!(
            r#"
    test 'should not allow counter to modify built-in locked attribute' do
      input = <<~'EOS'
      {counter:max-include-depth:128} is one more than {max-include-depth}
      EOS

      doc = document_from_string input, standalone: false
      output = doc.convert
      assert_xpath '//p[text()="65 is one more than 64"]', output, 1
      assert_equal 64, doc.attributes['max-include-depth']
    end

"#
        );

        let input = "{counter:max-include-depth:128} is one more than {max-include-depth}\n";
        let doc = load(input);
        assert_xpath(
            &convert(input),
            r#"//p[text()="65 is one more than 64"]"#,
            1,
        );
        assert_eq!(doc.attribute_value("max-include-depth"), val("64"));
    }

    #[test]
    fn should_not_allow_counter2_to_modify_built_in_locked_attribute() {
        verifies!(
            r#"
    test 'should not allow counter2 to modify built-in locked attribute' do
      input = <<~'EOS'
      {counter2:max-include-depth:128}{max-include-depth}
      EOS

      doc = document_from_string input, standalone: false
      output = doc.convert
      assert_xpath '//p[text()="64"]', output, 1
      assert_equal 64, doc.attributes['max-include-depth']
    end
"#
        );

        let input = "{counter2:max-include-depth:128}{max-include-depth}\n";
        let doc = load(input);
        assert_xpath(&convert(input), r#"//p[text()="64"]"#, 1);
        assert_eq!(doc.attribute_value("max-include-depth"), val("64"));
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod block_attributes {
    use super::*;
    non_normative!(
        r#"
  context 'Block attributes' do
"#
    );

    non_normative!(
        r#"
    test 'parses attribute names as name token' do
      input = <<~'EOS'
      [normal,foo="bar",_foo="_bar",foo1="bar1",foo-foo="bar-bar",foo.foo="bar.bar"]
      content
      EOS

      block = block_from_string input
      assert_equal 'bar', block.attr('foo')
      assert_equal '_bar', block.attr('_foo')
      assert_equal 'bar1', block.attr('foo1')
      assert_equal 'bar-bar', block.attr('foo-foo')
      assert_equal 'bar.bar', block.attr('foo.foo')
    end

"#
    );

    #[test]
    fn positional_attributes_assigned_to_block() {
        verifies!(
            r#"
    test 'positional attributes assigned to block' do
      input = <<~'EOS'
      [quote, author, source]
      ____
      A famous quote.
      ____
      EOS
      doc = document_from_string(input)
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attr('attribution')
      assert_equal 'author', qb.attr(:attribution)
      assert_equal 'author', qb.attributes['attribution']
      assert_equal 'source', qb.attributes['citetitle']
    end

"#
        );

        let input = "[quote, author, source]\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_css(&output, ".quoteblock", 1);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite[text()="source"]"#,
            1,
        );
        assert_includes(&output, "author");
    }

    #[test]
    fn normal_substitutions_are_performed_on_single_quoted_positional_attribute() {
        verifies!(
            r#"
    test 'normal substitutions are performed on single-quoted positional attribute' do
      input = <<~'EOS'
      [quote, author, 'http://wikipedia.org[source]']
      ____
      A famous quote.
      ____
      EOS
      doc = document_from_string(input)
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attr('attribution')
      assert_equal 'author', qb.attr(:attribution)
      assert_equal 'author', qb.attributes['attribution']
      assert_equal '<a href="http://wikipedia.org">source</a>', qb.attributes['citetitle']
    end

"#
        );

        let input =
            "[quote, author, 'http://wikipedia.org[source]']\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite/a[@href="http://wikipedia.org"][text()="source"]"#,
            1,
        );
    }

    #[test]
    fn normal_substitutions_are_performed_on_single_quoted_named_attribute() {
        verifies!(
            r#"
    test 'normal substitutions are performed on single-quoted named attribute' do
      input = <<~'EOS'
      [quote, author, citetitle='http://wikipedia.org[source]']
      ____
      A famous quote.
      ____
      EOS
      doc = document_from_string(input)
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attr('attribution')
      assert_equal 'author', qb.attr(:attribution)
      assert_equal 'author', qb.attributes['attribution']
      assert_equal '<a href="http://wikipedia.org">source</a>', qb.attributes['citetitle']
    end

"#
        );

        let input = "[quote, author, citetitle='http://wikipedia.org[source]']\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite/a[@href="http://wikipedia.org"][text()="source"]"#,
            1,
        );
    }

    #[test]
    fn normal_substitutions_are_performed_once_on_single_quoted_named_title_attribute() {
        verifies!(
            r#"
    test 'normal substitutions are performed once on single-quoted named title attribute' do
      input = <<~'EOS'
      [title='*title*']
      content
      EOS
      output = convert_string_to_embedded input
      assert_xpath '//*[@class="title"]/strong[text()="title"]', output, 1
    end

"#
        );

        let output = convert("[title='*title*']\ncontent\n");
        assert_xpath(&output, r#"//*[@class="title"]/strong[text()="title"]"#, 1);
    }

    #[test]
    fn attribute_list_may_not_begin_with_space() {
        verifies!(
            r#"
    test 'attribute list may not begin with space' do
      input = <<~'EOS'
      [ quote]
      ____
      A famous quote.
      ____
      EOS

      doc = document_from_string input
      b1 = doc.blocks.first
      assert_equal ['[ quote]'], b1.lines
    end

"#
        );

        let input = "[ quote]\n____\nA famous quote.\n____\n";
        assert_xpath(&convert(input), r#"//p[text()="[ quote]"]"#, 1);
    }

    #[test]
    fn attribute_list_may_begin_with_comma() {
        verifies!(
            r#"
    test 'attribute list may begin with comma' do
      input = <<~'EOS'
      [, author, source]
      ____
      A famous quote.
      ____
      EOS

      doc = document_from_string input
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attributes['attribution']
      assert_equal 'source', qb.attributes['citetitle']
    end

"#
        );

        let input = "[, author, source]\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_css(&output, ".quoteblock", 1);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite[text()="source"]"#,
            1,
        );
        assert_includes(&output, "author");
    }

    #[test]
    fn first_attribute_in_list_may_be_double_quoted() {
        verifies!(
            r#"
    test 'first attribute in list may be double quoted' do
      input = <<~'EOS'
      ["quote", "author", "source", role="famous"]
      ____
      A famous quote.
      ____
      EOS

      doc = document_from_string input
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attributes['attribution']
      assert_equal 'source', qb.attributes['citetitle']
      assert_equal 'famous', qb.attributes['role']
    end

"#
        );

        let input =
            "[\"quote\", \"author\", \"source\", role=\"famous\"]\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_css(&output, ".quoteblock.famous", 1);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite[text()="source"]"#,
            1,
        );
    }

    #[test]
    fn first_attribute_in_list_may_be_single_quoted() {
        verifies!(
            r#"
    test 'first attribute in list may be single quoted' do
      input = <<~'EOS'
      ['quote', 'author', 'source', role='famous']
      ____
      A famous quote.
      ____
      EOS

      doc = document_from_string input
      qb = doc.blocks.first
      assert_equal 'quote', qb.style
      assert_equal 'author', qb.attributes['attribution']
      assert_equal 'source', qb.attributes['citetitle']
      assert_equal 'famous', qb.attributes['role']
    end

"#
        );

        let input = "['quote', 'author', 'source', role='famous']\n____\nA famous quote.\n____\n";
        let output = convert(input);
        assert_css(&output, ".quoteblock.famous", 1);
        assert_xpath(
            &output,
            r#"//div[@class="attribution"]/cite[text()="source"]"#,
            1,
        );
    }

    #[test]
    fn attribute_with_value_none_without_quotes_is_ignored() {
        verifies!(
            r#"
    test 'attribute with value None without quotes is ignored' do
      input = <<~'EOS'
      [id=None]
      paragraph
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      refute para.attributes.key?('id')
    end

"#
        );

        let output = convert("[id=None]\nparagraph\n");
        assert_css(&output, "#None", 0);
        assert_xpath(&output, r#"//div[@class="paragraph"][not(@id)]"#, 1);
    }

    #[test]
    fn role_returns_true_if_role_is_assigned() {
        verifies!(
            r#"
    test 'role? returns true if role is assigned' do
      input = <<~'EOS'
      [role="lead"]
      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      assert p.role?
    end

"#
        );

        assert_css(
            &convert("[role=\"lead\"]\nA paragraph\n"),
            ".paragraph.lead",
            1,
        );
    }

    #[test]
    fn role_does_not_return_true_if_role_attribute_is_set_on_document() {
        verifies!(
            r#"
    test 'role? does not return true if role attribute is set on document' do
      input = <<~'EOS'
      :role: lead

      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      refute p.role?
    end

"#
        );

        assert_css(
            &convert(":role: lead\n\nA paragraph\n"),
            ".paragraph.lead",
            0,
        );
    }

    #[test]
    fn role_can_check_for_exact_role_name_match() {
        verifies!(
            r#"
    test 'role? can check for exact role name match' do
      input = <<~'EOS'
      [role="lead"]
      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      assert p.role?('lead')
      p2 = doc.blocks.last
      refute p2.role?('final')
    end

"#
        );

        let output = convert("[role=\"lead\"]\nA paragraph\n");
        assert_css(&output, ".paragraph.lead", 1);
        assert_css(&output, ".paragraph.final", 0);
    }

    #[test]
    fn has_role_can_check_for_presence_of_role_name() {
        verifies!(
            r#"
    test 'has_role? can check for presence of role name' do
      input = <<~'EOS'
      [role="lead abstract"]
      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      refute p.role?('lead')
      assert p.has_role?('lead')
    end

"#
        );

        assert_css(
            &convert("[role=\"lead abstract\"]\nA paragraph\n"),
            ".paragraph.lead.abstract",
            1,
        );
    }

    #[test]
    fn has_role_does_not_look_for_role_defined_as_document_attribute() {
        verifies!(
            r#"
    test 'has_role? does not look for role defined as document attribute' do
      input = <<~'EOS'
      :role: lead abstract

      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      refute p.has_role?('lead')
    end

"#
        );

        assert_css(
            &convert(":role: lead abstract\n\nA paragraph\n"),
            ".lead",
            0,
        );
    }

    #[test]
    fn roles_returns_array_of_role_names() {
        verifies!(
            r#"
    test 'roles returns array of role names' do
      input = <<~'EOS'
      [role="story lead"]
      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      assert_equal ['story', 'lead'], p.roles
    end

"#
        );

        assert_css(
            &convert("[role=\"story lead\"]\nA paragraph\n"),
            ".paragraph.story.lead",
            1,
        );
    }

    #[test]
    fn roles_returns_empty_array_if_role_attribute_is_not_set() {
        verifies!(
            r#"
    test 'roles returns empty array if role attribute is not set' do
      input = 'a paragraph'

      doc = document_from_string input
      p = doc.blocks.first
      assert_equal [], p.roles
    end

"#
        );

        assert_xpath(&convert("a paragraph"), r#"//div[@class="paragraph"]"#, 1);
    }

    #[test]
    fn roles_does_not_return_value_of_roles_document_attribute() {
        verifies!(
            r#"
    test 'roles does not return value of roles document attribute' do
      input = <<~'EOS'
      :role: story lead

      A paragraph
      EOS

      doc = document_from_string input
      p = doc.blocks.first
      assert_equal [], p.roles
    end

"#
        );

        assert_xpath(
            &convert(":role: story lead\n\nA paragraph\n"),
            r#"//div[@class="paragraph"]"#,
            1,
        );
    }

    non_normative!(
        r#"
    test 'roles= sets the role attribute on the node' do
      doc = document_from_string 'a paragraph'
      p = doc.blocks.first
      p.role = 'foobar'
      assert_equal 'foobar', (p.attr 'role')
    end

    test 'roles= coerces array value to a space-separated string' do
      doc = document_from_string 'a paragraph'
      p = doc.blocks.first
      p.role = %w(foo bar)
      assert_equal 'foo bar', (p.attr 'role')
    end

"#
    );

    #[test]
    fn attribute_substitutions_are_performed_on_attribute_list_before_parsing_attributes() {
        verifies!(
            r#"
    test "Attribute substitutions are performed on attribute list before parsing attributes" do
      input = <<~'EOS'
      :lead: role="lead"

      [{lead}]
      A paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      assert_equal 'lead', para.attributes['role']
    end

"#
        );

        let input = ":lead: role=\"lead\"\n\n[{lead}]\nA paragraph\n";
        assert_css(&convert(input), ".paragraph.lead", 1);
    }

    #[test]
    fn id_role_and_options_attributes_can_be_specified_on_block_style_using_shorthand_syntax() {
        verifies!(
            r#"
    test 'id, role and options attributes can be specified on block style using shorthand syntax' do
      input = <<~'EOS'
      [literal#first.lead%step]
      A literal paragraph.
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      assert_equal :literal, para.context
      assert_equal 'first', para.attributes['id']
      assert_equal 'lead', para.attributes['role']
      assert para.attributes.key?('step-option')
      refute para.attributes.key?('options')
    end

"#
        );

        let output = convert("[literal#first.lead%step]\nA literal paragraph.\n");
        assert_css(&output, "#first.literalblock.lead", 1);
    }

    #[test]
    fn id_role_and_options_attributes_can_be_specified_using_shorthand_syntax_on_block_style_using_multiple_block_attribute_lines(
    ) {
        verifies!(
            r#"
    test 'id, role and options attributes can be specified using shorthand syntax on block style using multiple block attribute lines' do
      input = <<~'EOS'
      [literal]
      [#first]
      [.lead]
      [%step]
      A literal paragraph.
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      assert_equal :literal, para.context
      assert_equal 'first', para.attributes['id']
      assert_equal 'lead', para.attributes['role']
      assert para.attributes.key?('step-option')
      refute para.attributes.key?('options')
    end

"#
        );

        let output = convert("[literal]\n[#first]\n[.lead]\n[%step]\nA literal paragraph.\n");
        assert_css(&output, "#first.literalblock.lead", 1);
    }

    #[test]
    fn multiple_roles_and_options_can_be_specified_in_block_style_using_shorthand_syntax() {
        verifies!(
            r#"
    test 'multiple roles and options can be specified in block style using shorthand syntax' do
      input = <<~'EOS'
      [.role1%option1.role2%option2]
      Text
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      assert_equal 'role1 role2', para.attributes['role']
      assert para.attributes.key?('option1-option')
      assert para.attributes.key?('option2-option')
      refute para.attributes.key?('options')
    end

"#
        );

        assert_css(
            &convert("[.role1%option1.role2%option2]\nText\n"),
            ".paragraph.role1.role2",
            1,
        );
    }

    non_normative!(
        r#"
    test 'options specified using shorthand syntax on block style across multiple lines should be additive' do
      input = <<~'EOS'
      [%option1]
      [%option2]
      Text
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      assert para.attributes.key?('option1-option')
      assert para.attributes.key?('option2-option')
      refute para.attributes.key?('options')
    end

"#
    );

    #[test]
    fn roles_specified_using_shorthand_syntax_on_block_style_across_multiple_lines_should_be_additive(
    ) {
        verifies!(
            r#"
    test 'roles specified using shorthand syntax on block style across multiple lines should be additive' do
      input = <<~'EOS'
      [.role1]
      [.role2.role3]
      Text
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      assert_equal 'role1 role2 role3', para.attributes['role']
    end

"#
        );

        assert_css(
            &convert("[.role1]\n[.role2.role3]\nText\n"),
            ".paragraph.role1.role2.role3",
            1,
        );
    }

    #[test]
    fn setting_a_role_using_the_role_attribute_replaces_any_existing_roles() {
        verifies!(
            r#"
    test 'setting a role using the role attribute replaces any existing roles' do
      input = <<~'EOS'
      [.role1]
      [role=role2]
      [.role3]
      Text
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      assert_equal 'role2 role3', para.attributes['role']
    end

"#
        );

        let output = convert("[.role1]\n[role=role2]\n[.role3]\nText\n");
        assert_css(&output, ".paragraph.role2.role3", 1);
        assert_css(&output, ".role1", 0);
    }

    #[test]
    fn setting_a_role_using_the_shorthand_syntax_on_block_style_should_not_clear_the_id() {
        verifies!(
            r#"
    test 'setting a role using the shorthand syntax on block style should not clear the ID' do
      input = <<~'EOS'
      [#id]
      [.role]
      Text
      EOS

      doc = document_from_string input
      para = doc.blocks.first
      assert_equal 'id', para.id
      assert_equal 'role', para.role
    end

"#
        );

        assert_css(&convert("[#id]\n[.role]\nText\n"), "#id.paragraph.role", 1);
    }

    non_normative!(
        r#"
    test 'a role can be added using add_role when the node has no roles' do
      input = 'A normal paragraph'
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.add_role 'role1'
      assert res
      assert_equal 'role1', para.attributes['role']
      assert para.has_role? 'role1'
    end

    test 'a role can be added using add_role when the node already has a role' do
      input = <<~'EOS'
      [.role1]
      A normal paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.add_role 'role2'
      assert res
      assert_equal 'role1 role2', para.attributes['role']
      assert para.has_role? 'role1'
      assert para.has_role? 'role2'
    end

    test 'a role is not added using add_role if the node already has that role' do
      input = <<~'EOS'
      [.role1]
      A normal paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.add_role 'role1'
      refute res
      assert_equal 'role1', para.attributes['role']
      assert para.has_role? 'role1'
    end

    test 'an existing role can be removed using remove_role' do
      input = <<~'EOS'
      [.role1.role2]
      A normal paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.remove_role 'role1'
      assert res
      assert_equal 'role2', para.attributes['role']
      assert para.has_role? 'role2'
      refute para.has_role?('role1')
    end

    test 'roles are removed when last role is removed using remove_role' do
      input = <<~'EOS'
      [.role1]
      A normal paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.remove_role 'role1'
      assert res
      refute para.role?
      assert_nil para.attributes['role']
      refute para.has_role? 'role1'
    end

    test 'roles are not changed when a non-existent role is removed using remove_role' do
      input = <<~'EOS'
      [.role1]
      A normal paragraph
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.remove_role 'role2'
      refute res
      assert_equal 'role1', para.attributes['role']
      assert para.has_role? 'role1'
      refute para.has_role?('role2')
    end

    test 'roles are not changed when using remove_role if the node has no roles' do
      input = 'A normal paragraph'
      doc = document_from_string(input)
      para = doc.blocks.first
      res = para.remove_role 'role1'
      refute res
      assert_nil para.attributes['role']
      refute para.has_role?('role1')
    end

"#
    );

    #[test]
    fn option_can_be_specified_in_first_position_of_block_style_using_shorthand_syntax() {
        verifies!(
            r#"
    test 'option can be specified in first position of block style using shorthand syntax' do
      input = <<~'EOS'
      [%interactive]
      - [x] checked
      EOS

      doc = document_from_string input
      list = doc.blocks.first
      assert list.attributes.key? 'interactive-option'
      refute list.attributes.key? 'options'
    end

"#
        );

        assert_css(
            &convert("[%interactive]\n- [x] checked\n"),
            "input[type=checkbox]",
            1,
        );
    }

    #[test]
    fn id_and_role_attributes_can_be_specified_on_section_style_using_shorthand_syntax() {
        verifies!(
            r#"
    test 'id and role attributes can be specified on section style using shorthand syntax' do
      input = <<~'EOS'
      [dedication#dedication.small]
      == Section
      Content.
      EOS
      output = convert_string_to_embedded input
      assert_xpath '/div[@class="sect1 small"]', output, 1
      assert_xpath '/div[@class="sect1 small"]/h2[@id="dedication"]', output, 1
    end

"#
        );

        let output = convert("[dedication#dedication.small]\n== Section\nContent.\n");
        assert_xpath(&output, r#"/div[@class="sect1 small"]"#, 1);
        assert_xpath(
            &output,
            r#"/div[@class="sect1 small"]/h2[@id="dedication"]"#,
            1,
        );
    }

    non_normative!(
        r#"
    test 'id attribute specified using shorthand syntax should not create a special section' do
      input = <<~'EOS'
      [#idname]
      == Section

      content
      EOS

      doc = document_from_string input, backend: 'docbook'
      section = doc.blocks[0]
      refute_nil section
      assert_equal :section, section.context
      refute section.special
      output = doc.convert
      assert_css 'article:root > section', output, 1
      assert_css 'article:root > section[xml|id="idname"]', output, 1
    end

"#
    );

    #[test]
    fn block_attributes_are_additive() {
        verifies!(
            r#"
    test "Block attributes are additive" do
      input = <<~'EOS'
      [id='foo']
      [role='lead']
      A paragraph.
      EOS
      doc = document_from_string(input)
      para = doc.blocks.first
      assert_equal 'foo', para.id
      assert_equal 'lead', para.attributes['role']
    end

"#
        );

        assert_css(
            &convert("[id='foo']\n[role='lead']\nA paragraph.\n"),
            "#foo.paragraph.lead",
            1,
        );
    }

    #[test]
    fn last_wins_for_id_attribute() {
        verifies!(
            r#"
    test "Last wins for id attribute" do
      input = <<~'EOS'
      [[bar]]
      [[foo]]
      == Section

      paragraph

      [[baz]]
      [id='coolio']
      === Section
      EOS
      doc = document_from_string(input)
      sec = doc.first_section
      assert_equal 'foo', sec.id
      subsec = sec.blocks.last
      assert_equal 'coolio', subsec.id
    end

"#
        );

        let input =
            "[[bar]]\n[[foo]]\n== Section\n\nparagraph\n\n[[baz]]\n[id='coolio']\n=== Section\n";
        let output = convert(input);
        assert_xpath(&output, r#"//h2[@id="foo"]"#, 1);
        assert_xpath(&output, r#"//h3[@id="coolio"]"#, 1);
    }

    #[test]
    fn trailing_block_attributes_transfer_to_the_following_section() {
        verifies!(
            r#"
    test "trailing block attributes transfer to the following section" do
      input = <<~'EOS'
      [[one]]

      == Section One

      paragraph

      [[sub]]
      // try to mess this up!

      === Sub-section

      paragraph

      [role='classy']

      ////
      block comment
      ////

      == Section Two

      content
      EOS
      doc = document_from_string(input)
      section_one = doc.blocks.first
      assert_equal 'one', section_one.id
      subsection = section_one.blocks.last
      assert_equal 'sub', subsection.id
      section_two = doc.blocks.last
      assert_equal 'classy', section_two.attr(:role)
    end
"#
        );

        let input = "[[one]]\n\n== Section One\n\nparagraph\n\n[[sub]]\n// try to mess this up!\n\n=== Sub-section\n\nparagraph\n\n[role='classy']\n\n////\nblock comment\n////\n\n== Section Two\n\ncontent\n";
        let output = convert(input);
        assert_xpath(&output, r#"//h2[@id="one"]"#, 1);
        assert_xpath(&output, r#"//h3[@id="sub"]"#, 1);
        assert_css(&output, ".sect1.classy", 1);
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
