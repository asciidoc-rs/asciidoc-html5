//! Coverage of the AsciiDoc language description's *Use an Include File
//! Multiple Times* page.
//!
//! The page's rule is that including the same file more than once would produce
//! duplicate IDs, and that referencing a document attribute in the ID (reset
//! between the includes) keeps each generated ID unique. This is verified end
//! to end through `convert`: the same fragment is included twice with
//! `:chapter:` set to a different value each time, and the two copies emit
//! distinct IDs and matching cross-references.

use super::convert_including;
use crate::tests::{assert_html::assert_css, sdd::*};

track_file!(
    "ref/asciidoc-lang/docs/modules/directives/pages/include-multiple-times-in-same-document.adoc"
);

non_normative!(
    r#"
= Use an Include File Multiple Times
//Include a File Multiple Times in the Same Document
//[#include-multiple]
// Title and anchor are from user-manual.adoc
// Section content from multiple-include.adoc

A document can include the same file any number of times.
The problem comes if there are IDs in the included file; the output document (HTML or DocBook) will then have duplicate IDs which will make it not well-formed.
To fix this, you can reference a dynamic variable from the primary document in the ID.

For example, let's say you want to include the same subsection describing a bike chain in both the operation and maintenance chapters:

"#
);

// Including the same fragment twice, with `:chapter:` set to a different value
// before each include, resolves the fragment's attribute-referencing ID to a
// distinct value each time — so the two copies carry unique IDs and their
// cross-references resolve to the matching section.
#[test]
fn a_dynamic_attribute_in_the_id_keeps_repeated_includes_unique() {
    verifies!(
        r#"
----
= Bike Manual

:chapter: operation
== Operation

\include::fragment-chain.adoc[]

:chapter: maintenance
== Maintenance

\include::fragment-chain.adoc[]
----

Write [.path]_fragment-chain.adoc_ as:

----
[id=chain-{chapter}]
=== Chain

See xref:chain-{chapter}[].
----

The first time the [.path]_fragment-chain.adoc_ file is included, the ID of the included section resolves to `chain-operation`.
The second time the file included, the ID resolves to `chain-maintenance`.

"#
    );

    let output = convert_including(
        "= Bike Manual\n\n\
         :chapter: operation\n== Operation\n\ninclude::fragment-chain.adoc[]\n\n\
         :chapter: maintenance\n== Maintenance\n\ninclude::fragment-chain.adoc[]\n",
    );

    // Each include resolves the section ID to a distinct value ...
    assert_css(&output, "#chain-operation", 1);
    assert_css(&output, "#chain-maintenance", 1);

    // ... and each cross-reference points at its own copy.
    assert_css(&output, r##"a[href="#chain-operation"]"##, 1);
    assert_css(&output, r##"a[href="#chain-maintenance"]"##, 1);
}

non_normative!(
    r#"
In order for this to work, you must use the long-hand forms of both the ID assignment and the cross reference.
The single quotes around the variable name in the assignment are required to force variable substitution (aka interpolation).
"#
);
