//! Port of Asciidoctor's `helpers_test.rb`.
//!
//! Like [`paths_test`](super::paths_test), this file exercises internal utility
//! methods on `Asciidoctor::Helpers` (and the `Asciidoctor::UriSniffRx` regex),
//! not rendered behavior. This crate is an HTML5 renderer: it neither exposes
//! nor reimplements `Helpers`, so almost every claim here is tracked
//! `non_normative!` – its subject is a helper's isolated return value, which
//! has no `convert`-driven form to verify (see the "test rendered output, not
//! internals" rule in this directory's README).
//!
//! The one claim that *does* have a faithful rendered form is verified. The
//! `encode_uri_component` helper surfaces in the renderer when a `mailto:`
//! macro carries a subject (and/or body): those positional attributes are
//! URI-encoded into the link's query string. So the "should not URI encode
//! select non-word characters" claim (`-.` passes through unchanged) is checked
//! through a rendered mailto link rather than deferred.
//!
//! Its sibling – "should URI encode non-word characters generally" – stays
//! `non_normative!`, because the only rendered surface cannot reproduce the
//! helper's raw claim. A mailto subject runs the special-characters
//! substitution first, so the test's `&` becomes `&amp;` *before* encoding: our
//! output is `…%26amp%3B…` (matching Asciidoctor 2.0.26 exactly), not the raw
//! `…%26…` the isolated `encode_uri_component` unit test asserts. That
//! composite mailto-encoding parity belongs with the link tests, not to a
//! re-expression of this helper unit test.
//!
//! The remaining contexts have no rendered form here at all:
//!
//! - **URIs and Paths** – `rootname`, `extname?`, `uriish?`, and matching
//!   against `UriSniffRx` are internal string helpers. Where the renderer needs
//!   the same logic it has its own port covered by its own tests:
//!   [`crate::renderer::looks_like_uri`] mirrors `UriSniffRx`, and
//!   Asciidoctor's `Helpers.extname` is mirrored by the output-filename
//!   derivation in `options.rs` (`file_extension_matches_asciidoctor_extname`).
//!   Neither is reachable through `convert`, so these tests are not
//!   re-expressed here.
//! - **Type Resolution** – `class_for_name` / `resolve_class` resolve a Ruby
//!   class from a `String` name via reflection. This statically-typed Rust
//!   crate has no equivalent, so the whole context is scaffolding.

use crate::{
    convert,
    tests::{assert_html::assert_xpath, sdd::*},
};

track_file!("ref/asciidoctor/test/helpers_test.rb");

non_normative!(
    r#"
# frozen_string_literal: true
require_relative 'test_helper'

context 'Helpers' do
"#
);

mod uri_encoding {
    use super::*;

    non_normative!(
        r#"
  context 'URI Encoding' do
"#
    );

    // The only rendered surface for `encode_uri_component` is a `mailto:`
    // subject/body, and that text is special-characters-escaped before it is
    // encoded. So this test's `&` renders as `&amp;` → `%26amp%3B` (parity with
    // Asciidoctor 2.0.26 confirmed), never the raw `%26` the helper unit test
    // asserts – there is no faithful rendered form of this claim.
    non_normative!(
        r#"
    test 'should URI encode non-word characters generally' do
      given = ' !*/%&?\\='
      expect = '%20%21%2A%2F%25%26%3F%5C%3D'
      assert_equal expect, (Asciidoctor::Helpers.encode_uri_component given)
    end

"#
    );

    #[test]
    fn should_not_uri_encode_select_non_word_characters() {
        verifies!(
            r#"
    test 'should not URI encode select non-word characters' do
      # NOTE Ruby 2.5 and up stopped encoding ~
      given = '-.'
      expect = given
      assert_equal expect, (Asciidoctor::Helpers.encode_uri_component given)
    end
"#
        );

        // A `mailto:` macro's second positional attribute (the subject) is
        // URI-encoded via `encode_uri_component`. The characters `-` and `.` are
        // among the select non-word characters it leaves untouched, so they
        // survive verbatim in the rendered query string.
        let html = convert("mailto:doc@example.org[Doc,-.]");

        assert_xpath(
            &html,
            r#"//a[@href="mailto:doc@example.org?subject=-."]"#,
            1,
        );
    }

    non_normative!(
        r#"
  end

"#
    );
}

mod uris_and_paths {
    use super::*;

    non_normative!(
        r#"
  context 'URIs and Paths' do
    test 'rootname should return file name without extension' do
      assert_equal 'main', Asciidoctor::Helpers.rootname('main.adoc')
      assert_equal 'docs/main', Asciidoctor::Helpers.rootname('docs/main.adoc')
    end

    test 'rootname should file name if it has no extension' do
      assert_equal 'main', Asciidoctor::Helpers.rootname('main')
      assert_equal 'docs/main', Asciidoctor::Helpers.rootname('docs/main')
    end

    test 'rootname should ignore dot not in last segment' do
      assert_equal 'include.d/main', Asciidoctor::Helpers.rootname('include.d/main')
      assert_equal 'include.d/main', Asciidoctor::Helpers.rootname('include.d/main.adoc')
    end

    test 'extname? should return whether path contains an extname' do
      assert Asciidoctor::Helpers.extname?('document.adoc')
      assert Asciidoctor::Helpers.extname?('path/to/document.adoc')
      assert_nil Asciidoctor::Helpers.extname?('basename')
      refute Asciidoctor::Helpers.extname?('include.d/basename')
    end

    test 'UriSniffRx should detect URIs' do
      assert_match Asciidoctor::UriSniffRx, 'http://example.com'
      assert_match Asciidoctor::UriSniffRx, 'https://example.com'
      assert_match Asciidoctor::UriSniffRx, 'data:image/gif;base64,R0lGODlhAQABAIAAAAUEBAAAACwAAAAAAQABAAACAkQBADs='
    end

    test 'UriSniffRx should not detect an absolute Windows path as a URI' do
      refute_match Asciidoctor::UriSniffRx, 'c:/sample.adoc'
      refute_match Asciidoctor::UriSniffRx, 'c:\\sample.adoc'
    end

    test 'uriish? should not detect a classloader path as a URI on JRuby' do
      input = 'uri:classloader:/sample.png'
      assert_match Asciidoctor::UriSniffRx, input
      if jruby?
        refute Asciidoctor::Helpers.uriish? input
      else
        assert Asciidoctor::Helpers.uriish? input
      end
    end

    test 'UriSniffRx should not detect URI that does not start on first line' do
      refute_match Asciidoctor::UriSniffRx, %(text\nhttps://example.org)
    end
  end

"#
    );
}

mod type_resolution {
    use super::*;

    non_normative!(
        r#"
  context 'Type Resolution' do
    test 'should get class for top-level class name' do
      clazz = Asciidoctor::Helpers.class_for_name 'String'
      refute_nil clazz
      assert_equal String, clazz
    end

    test 'should get class for class name in module' do
      clazz = Asciidoctor::Helpers.class_for_name 'Asciidoctor::Document'
      refute_nil clazz
      assert_equal Asciidoctor::Document, clazz
    end

    test 'should get class for class name resolved from root' do
      clazz = Asciidoctor::Helpers.class_for_name '::Asciidoctor::Document'
      refute_nil clazz
      assert_equal Asciidoctor::Document, clazz
    end

    test 'should raise exception if cannot find class for name' do
      begin
        Asciidoctor::Helpers.class_for_name 'InvalidModule::InvalidClass'
        flunk 'Expecting RuntimeError to be raised'
      rescue NameError => e
        assert_match %r/^Could not resolve class for name: InvalidModule::InvalidClass$/, e.message
      end
    end

    test 'should raise exception if constant name is invalid' do
      begin
        Asciidoctor::Helpers.class_for_name 'foobar'
        flunk 'Expecting RuntimeError to be raised'
      rescue NameError => e
        assert_match %r/^Could not resolve class for name: foobar$/, e.message
      end
    end

    test 'should raise exception if class not found in scope' do
      begin
        Asciidoctor::Helpers.class_for_name 'Asciidoctor::Extensions::String'
        flunk 'Expecting RuntimeError to be raised'
      rescue NameError => e
        assert_match %r/^Could not resolve class for name: Asciidoctor::Extensions::String$/, e.message
      end
    end

    test 'should raise exception if name resolves to module' do
      begin
        Asciidoctor::Helpers.class_for_name 'Asciidoctor::Extensions'
        flunk 'Expecting RuntimeError to be raised'
      rescue NameError => e
        assert_match %r/^Could not resolve class for name: Asciidoctor::Extensions$/, e.message
      end
    end

    test 'should resolve class if class is given' do
      clazz = Asciidoctor::Helpers.resolve_class Asciidoctor::Document
      refute_nil clazz
      assert_equal Asciidoctor::Document, clazz
    end

    test 'should resolve class if class from string' do
      clazz = Asciidoctor::Helpers.resolve_class 'Asciidoctor::Document'
      refute_nil clazz
      assert_equal Asciidoctor::Document, clazz
    end

    test 'should not resolve class if not in scope' do
      begin
        Asciidoctor::Helpers.resolve_class 'Asciidoctor::Extensions::String'
        flunk 'Expecting RuntimeError to be raised'
      rescue NameError => e
        assert_match %r/^Could not resolve class for name: Asciidoctor::Extensions::String$/, e.message
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
