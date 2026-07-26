//! Tracks Asciidoctor's test fixture `fixtures/include-asciidoctor.rb`.
//!
//! Tracked entirely as `non_normative!`: it is test-suite infrastructure /
//! fixture data, not a statement of rendered behavior, so it carries no claim
//! for a `convert`-driven test to verify. Reproduced verbatim so it is honestly
//! accounted for as a tracked spec surface rather than counted as uncovered.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/test/fixtures/include-asciidoctor.rb");

non_normative!(
    r#"
include Asciidoctor
"#
);
