//! Tracks Asciidoctor's test fixture `fixtures/configure-stdin.rb`.
//!
//! Tracked entirely as `non_normative!`: it is test-suite infrastructure /
//! fixture data, not a statement of rendered behavior, so it carries no claim
//! for a `convert`-driven test to verify. Reproduced verbatim so it is honestly
//! accounted for as a tracked spec surface rather than counted as uncovered.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/test/fixtures/configure-stdin.rb");

non_normative!(
    r#"
require 'stringio'
io = StringIO.new String.new %(é\n\n#{Encoding.default_external}:#{Encoding.default_internal}), encoding: Encoding::UTF_8
io.set_encoding Encoding.default_external, Encoding.default_internal
$stdin = io
"#
);
