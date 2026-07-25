//! Tracks Asciidoctor's test fixture `fixtures/undef-dir-home.rb`.
//!
//! Tracked entirely as `non_normative!`: it is test-suite infrastructure /
//! fixture data, not a statement of rendered behavior, so it carries no claim
//! for a `convert`-driven test to verify. Reproduced verbatim so it is honestly
//! accounted for as a tracked spec surface rather than counted as uncovered.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/test/fixtures/undef-dir-home.rb");

non_normative!(
    r#"
# undef_method wasn't public until 2.5
Dir.singleton_class.send :undef_method, :home
"#
);
