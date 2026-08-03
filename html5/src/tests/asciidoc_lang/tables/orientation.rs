//! Coverage of the AsciiDoc language description's *Table Orientation* page.
//!
//! The page describes displaying a table in landscape via the `rotate` option
//! (preferred) or the `orientation` attribute. As the page itself states, this
//! is only implemented by the DocBook backend (which sets `orient="land"`); the
//! HTML backend produces no distinct output for either form. The whole page is
//! therefore tracked as non-normative. A standalone test confirms that neither
//! form breaks HTML rendering: each still produces an ordinary `<table>`.

use crate::{
    convert,
    tests::{assert_html::assert_css, sdd::*},
};

track_file!("ref/asciidoc-lang/docs/modules/tables/pages/orientation.adoc");

// The entire page describes landscape orientation, which the closing paragraph
// states is only implemented by the DocBook backend. The HTML backend emits no
// distinct output for the `rotate` option or the `orientation` attribute, so
// there is no HTML rendering rule to verify here.
non_normative!(
    r#"
= Table Orientation

== Landscape

A table can be displayed in landscape (rotated 90 degrees counterclockwise) using the `rotate` option (preferred).

[source]
----
[%rotate]
include::example$table.adoc[tag=base]
----

A table can also be displayed in landscape using the `orientation` attribute.

[source]
----
[orientation=landscape]
include::example$table.adoc[tag=base]
----

Currently, this is only implemented by the DocBook backend (it sets the attribute `orient="land"`).
"#
);

// The `rotate` option and the `orientation` attribute have no effect on the
// HTML backend, but they must not break rendering: each still produces an
// ordinary table. The `base` table snippet is the one referenced by the page's
// `include::example$table.adoc[tag=base]` directives.
#[test]
fn landscape_forms_render_ordinary_html_table() {
    let base = "|===\n|Cell in column 1, row 1\n|Cell in column 2, row 1\n\n|Cell in column 1, row 2\n|Cell in column 2, row 2\n|===";

    let rotate = convert(&format!("[%rotate]\n{base}"));

    assert_css(&rotate, "table.tableblock", 1);

    let orientation = convert(&format!("[orientation=landscape]\n{base}"));

    assert_css(&orientation, "table.tableblock", 1);
}
