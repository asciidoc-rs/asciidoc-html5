//! Coverage of the AsciiDoc language description's *Horizontal Description
//! List* page.
//!
//! The `horizontal` style renders a description list as a two-column table
//! (`<div class="hdlist"><table>`) so each term sits beside its description.
//! The optional `labelwidth`/`itemwidth` attributes become `<col>` widths in
//! the table's `<colgroup>`. This crate renders both forms, so the page's two
//! examples and the claims about them are verified through `convert`; only the
//! title and the closing CSS aside are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/lists/pages/horizontal.adoc");

non_normative!(
    r#"
= Horizontal Description List

"#
);

// The `horizontal` style lays each term beside its description by rendering the
// list as a two-column table: the wrapper is `<div class="hdlist">`, each term
// a `<td class="hdlist1">` and each description a `<td class="hdlist2">`.
#[test]
fn horizontal_style_renders_a_two_column_table() {
    verifies!(
        r#"
If you want the first term and description of an item to start on the same line (i.e., horizontal arrangement), add the horizontal style to the description list.

[.wrap]
----
include::example$description.adoc[tag=base-horz]
----

====
include::example$description.adoc[tag=base-horz]
====

"#
    );

    let output = convert(
        "[horizontal]\n\
         CPU:: The brain of the computer.\n\
         Hard drive:: Permanent storage for operating system and/or user files.\n\
         RAM:: Temporarily stores information the CPU uses during operation.\n",
    );

    assert_css(&output, "div.hdlist > table", 1);
    assert_css(&output, "div.hdlist table tr", 3);
    assert_css(&output, "td.hdlist1", 3);
    assert_css(&output, "td.hdlist2 > p", 3);
    assert_xpath(
        &output,
        r#"(//tr)[1]/td[@class="hdlist1"][normalize-space(text())="CPU"]"#,
        1,
    );
    assert_xpath(
        &output,
        r#"(//tr)[1]/td[@class="hdlist2"]/p[text()="The brain of the computer."]"#,
        1,
    );
}

non_normative!(
    r#"
By default, the term and description columns will be sized automatically.
If the content is not arranged in the way that you want, you need to adjust the width distribution.

"#
);

// The `labelwidth` and `itemwidth` attributes set the term and description
// column widths. Each becomes a `<col>` in the table's `<colgroup>` with an
// inline percentage `width` style.
#[test]
fn labelwidth_and_itemwidth_set_the_column_widths() {
    verifies!(
        r#"
You can control the width of the term and description columns using the (improperly named) `labelwidth` and `itemwidth` attributes on the list, respectively.
Both attributes are optional.
The value of each attribute is a number from 0 to 100 (a unitless percentage).
If both attributes are specified, their values should add up to 100.

[.wrap]
----
include::example$description.adoc[tag=widths]
----

====
include::example$description.adoc[tag=widths]
====

"#
    );

    let output = convert(
        "[horizontal,labelwidth=25,itemwidth=75]\n\
         A short term:: The term for this item likely fits inside the column's width.\n\
         A long term that wraps across multiple lines:: The term for this item wraps \
         since the width of the term column is restricted using the `labelwidth` attribute.\n",
    );

    assert_css(&output, "div.hdlist table > colgroup > col", 2);
    assert_xpath(&output, r#"(//colgroup/col)[1][@style="width: 25%;"]"#, 1);
    assert_xpath(&output, r#"(//colgroup/col)[2][@style="width: 75%;"]"#, 1);
}

non_normative!(
    r#"
When converting to HTML, you can assign a role to the description list instead to control the column widths using CSS.
"#
);
