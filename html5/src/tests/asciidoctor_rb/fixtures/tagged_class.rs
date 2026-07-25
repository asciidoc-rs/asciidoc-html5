//! Tracks Asciidoctor's test fixture `fixtures/tagged-class.rb`.
//!
//! Tracked entirely as `non_normative!`: it is test-suite infrastructure /
//! fixture data, not a statement of rendered behavior, so it carries no claim
//! for a `convert`-driven test to verify. Reproduced verbatim so it is honestly
//! accounted for as a tracked spec surface rather than counted as uncovered.

use crate::tests::sdd::*;

track_file!("ref/asciidoctor/test/fixtures/tagged-class.rb");

non_normative!(
    r#"
class Dog
  #tag::init[]
  def initialize breed
    @breed = breed
  end
  #end::init[]
  #tag::bark[]

  def bark
    #tag::bark-beagle[]
    if @breed == 'beagle'
      'woof woof woof woof woof'
    #end::bark-beagle[]
    #tag::bark-other[]
    else
      'woof woof'
    #end::bark-other[]
    #tag::bark-all[]
    end
    #end::bark-all[]
  end
  #end::bark[]
end
"#
);
