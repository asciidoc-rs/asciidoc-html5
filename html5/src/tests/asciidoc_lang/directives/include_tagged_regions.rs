//! Coverage of the AsciiDoc language description's *Include Content by Tagged
//! Regions* page.
//!
//! The `tag`/`tags` attribute selects the lines between `tag::name[]` and
//! `end::name[]` directives in the target. Every concrete selection the page
//! demonstrates is verified through `convert` against Ruby-comment fixtures: a
//! single tag, multiple tags, fine-grained regions nested inside larger ones,
//! tags placed behind (and not behind) line comments, the wildcard/negation
//! ordering rules, and every row of the tag-filtering permutation table (`*`,
//! `**`, `!*`, `foo`, `foo;!bar`, `foo;!*`, `*;!foo`, `!foo`, `!foo;!bar`,
//! `!*;foo`). Only the prose on tag-directive placement and the sentence that
//! sets up the permutation table are non-normative.

use super::convert_including;
use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/include-tagged-regions.adoc");

non_normative!(
    r#"
= Include Content by Tagged Regions
// aka Select Portions of a Document to Include

The include directive enables you to select portions of a file to include instead of including the whole file.
Use the `lines` attribute to include individual lines or a range of lines (by line number).
Use the `tags` attribute (or `tag` attribute for the singular case) to select lines that fall between regions marked with user-defined tags.

When including multiple line ranges or multiple tags, each entry in the list must be separated by either a comma or a semi-colon.
If commas are used, the entire value must be enclosed in quotes (per attribute rules).
You can eliminate the need for quotes by using the semi-colon as the data separator instead.

== Tagging regions

Tags are useful when you want to identify specific regions of a file to include.
You can then select the lines between the boundaries of the include tag/end directives to include using the `tags` attribute.

In the include file, the tag directives (e.g., `tag::name[]` and `end::name[]`) must follow a word boundary and precede a space character or the end of line.
The tag name must not be empty and must consist exclusively of non-space characters.

Typically, the tag directives will be placed after a line comment as defined by the language of the source file.
This is especially important when using tags to include portions of an AsciiDoc document which may itself be converted.
If the tag is not behind a line comment, the tag will be treated as regular content, which could disrupt the block structure, such as a table.

For languages that only support circumfix comments, such as XML, you can enclose the tag directives between the circumfix comment markers, offset by a space on either side.
For example, in XML files, you can use `+<!-- tag::name[] -->+` and `+<!-- end::name[] -->+`.

Including by tag includes all regions marked with that tag.
This makes it possible to include a group of lines from different regions of the document using a single tag.

TIP: If the target file has tagged lines, and you just want to ignore those lines, use the `tags` attribute to filter them out.
See <<tag-filtering>> for details.

The example below shows how you tag a region of content inside a file containing multiple code examples.
The tag directives are preceded by a hash (`#`) because that's the start of a line comment in Ruby.

.Tagged code snippets in a file named core.rb
[source,ruby,subs=attributes+]
----
include::example$include.adoc[tag=tag-co]
----
<.> To indicate the start of a tagged region, insert a comment line in the code.
<.> Assign a name to the `tag` directive. In this example, the tag is named _timings_.
<.> Insert another comment line where you want the tagged region to end.
<.> Assign the name of the region you want to terminate to the `end` directive.
<.> This is the start of a tagged snippet named _parse_.
<.> This is the end of the tagged snippet named _parse_.

"#
);

// Selecting a single tag includes only the lines between that tag's
// `tag::`/`end::` directives — here the `parse` region of `core.rb`, not the
// `timings` region.
#[test]
fn a_single_tag_selects_its_region() {
    verifies!(
        r#"
In the next example, the tagged region named _parse_ is selected by the `include` directive.

.Selecting the _parse_ code snippet from a document
[source]
....
include::example$include.adoc[tag=target-co]
....
<.> In the directive's brackets, set the `tag` attribute and assign it the unique name of the code snippet you tagged in your code file.

"#
    );

    let output = convert_including("....\ninclude::core.rb[tag=parse]\n....\n");
    assert!(output.contains("Document.new lines, options"), "{output}");
    assert!(!output.contains("timings.record :read"), "{output}");
}

// Multiple tags may be selected from the same file, separated by a semicolon
// (so no quotes are needed) — here both the `timings` and `parse` regions.
#[test]
fn multiple_tags_select_several_regions() {
    verifies!(
        r#"
You can include multiple tags from the same file.

.Selecting the _timings_ and the _parse_ code snippets from a document
[source]
....
include::example$include.adoc[tag=target-co-multiple]
....

"#
    );

    let output = convert_including("....\ninclude::core.rb[tags=timings;parse]\n....\n");
    assert!(output.contains("timings.record :read"), "{output}");
    assert!(output.contains("Document.new lines, options"), "{output}");
}

// A tag can enclose finer-grained tags. Selecting the outer tag includes its
// nested regions, and because these tag directives sit behind line comments,
// including the whole file (no tag filter) renders the content without the tag
// lines.
#[test]
fn tags_nest_and_commented_tag_lines_are_never_rendered() {
    verifies!(
        r#"
It's also possible to have fine-grained tagged regions inside larger tagged regions.

In the next example, tagged regions are defined behind line comments.
By putting each tag behind a line comment, regardless of how the content is included, you don't have to worry about those lines appearing in the rendered document.

----
// tag::snippets[]
// tag::snippet-a[]
snippet a
// end::snippet-a[]

// tag::snippet-b[]
snippet b
// end::snippet-b[]
// end::snippets[]
----

Let's assume you include this file using the following include directive:

----
\include::file-with-snippets.adoc[tag=snippets]
----

The following lines will be selected and displayed:

....
snippet a

snippet b
....

You could also include the whole file without worry that the tags will be rendered:

----
\include::file-with-snippets.adoc[]
----

"#
    );

    // Selecting the outer `snippets` tag includes both nested regions.
    let selected = convert_including("....\ninclude::nested-snippets.adoc[tag=snippets]\n....\n");
    assert!(selected.contains("snippet a"), "{selected}");
    assert!(selected.contains("snippet b"), "{selected}");

    // Including the whole file renders the content but not the commented tag
    // lines.
    let whole = convert_including("include::nested-snippets.adoc[]\n");
    assert!(whole.contains("snippet a"), "{whole}");
    assert!(whole.contains("snippet b"), "{whole}");
    assert!(!whole.contains("tag::"), "{whole}");
}

// When tags are *not* behind line comments, selecting a specific tag still
// includes only that region.
#[test]
fn a_tag_not_behind_a_comment_still_selects_its_region() {
    verifies!(
        r#"
Now let's consider the case when the tags are not placed behind line comments.
In this case, you need to ensure that xref:tag-filtering[tag filtering] is being used or else those tags will be visible in the rendered document.

----
text a

tag::snippet-b[]
snippet b
end::snippet-b[]

text c
----

If you only want to include a specific tagged region of the file, use the following include directive:

----
\include::file-with-snippets.adoc[tag=snippet-b]
----

The following lines will be selected and displayed:

....
snippet b
....

"#
    );

    let output =
        convert_including("....\ninclude::uncommented-snippets.adoc[tag=snippet-b]\n....\n");
    assert!(output.contains("snippet b"), "{output}");
    assert!(!output.contains("text a"), "{output}");
    assert!(!output.contains("text c"), "{output}");
}

// The double wildcard `**` includes the whole file but discards every line that
// contains a tag directive — so an uncommented tag's directive lines do not
// appear in the output.
#[test]
fn the_double_wildcard_keeps_all_lines_but_drops_tag_directives() {
    verifies!(
        r#"
If you want to include the whole file, but also filter out any include tags, use the following include directive:

----
\include::file-with-snippets.adoc[tag=**]
----

The following lines will be selected and displayed:

....
text a
snippet b
text c
....

If you did not specify tag filtering, tag directives that aren't behind a line comment (e.g., `tag::snippet-b[]`) will also be printed too.
Tag filtering is explained in more detail in the next section.

"#
    );

    let output = convert_including("....\ninclude::uncommented-snippets.adoc[tag=**]\n....\n");
    assert!(output.contains("text a"), "{output}");
    assert!(output.contains("snippet b"), "{output}");
    assert!(output.contains("text c"), "{output}");
    assert!(!output.contains("tag::snippet-b"), "{output}");
}

// The filtering modifiers: the single wildcard `*` selects all tagged regions;
// the double wildcard `**` selects every line except tag-directive lines; and
// `!` negates a wildcard or tag (so `!*` selects only the non-tagged regions).
#[test]
fn the_filtering_wildcards_select_in_bulk() {
    verifies!(
        r#"
[#tag-filtering]
== Tag filtering

The previous section showed how to select tagged regions explicitly, but you can also use wildcards and exclusions.
These expressions give you the ability to include or exclude tags in bulk.
For example, here's how to include all lines that are not enclosed in a tag:

----
\include::file-with-snippets.adoc[tag=!*]
----

When tag filtering is used, lines that contain a tag directive _are always discarded_ (like a line comment).
Even if you're not including content by tags, you can specify the double wildcard (`+**+`) to filter out all lines in the include file that contain a tag directive.

The modifiers you can use for filtering are as follows:

`*`::
The single wildcard.
Select all tagged regions.
May only be specified once, negated or not.

`**`::
The double wildcard.
Select all the lines in the document *except for lines that contain a tag directive*.
Use this symbol if you want to include a file that has tag directives, but you want to discard the lines that contain a tag directive.
May only be specified once, negated or not.

`!`::
Negate the wildcard or tag.

"#
    );

    // `*` selects all tagged regions, not the surrounding lines.
    let star = convert_including("....\ninclude::foo-bar.rb[tag=*]\n....\n");
    assert!(star.contains("foo start"), "{star}");
    assert!(star.contains("bar body"), "{star}");
    assert!(!star.contains("outside before"), "{star}");

    // `**` selects every line but drops the tag-directive lines.
    let double = convert_including("....\ninclude::foo-bar.rb[tag=**]\n....\n");
    assert!(double.contains("outside before"), "{double}");
    assert!(double.contains("foo start"), "{double}");
    assert!(!double.contains("tag::foo"), "{double}");

    // `!*` selects only the regions outside any tag.
    let not_star = convert_including("....\ninclude::foo-bar.rb[tag=!*]\n....\n");
    assert!(not_star.contains("outside before"), "{not_star}");
    assert!(not_star.contains("outside after"), "{not_star}");
    assert!(!not_star.contains("foo start"), "{not_star}");
}

// The ordering and negation rules for the wildcards: the double wildcard is
// applied first regardless of where it sits in the list; a negated double
// wildcard selects no lines; and a negated single wildcard means different
// things before a tag name (`!*;foo`) versus after one (`foo;!*`).
#[test]
fn the_wildcard_ordering_and_negation_rules_hold() {
    verifies!(
        r#"
The double wildcard is always applied first, regardless of where it appears in the list.
If the double wildcard is not negated (i.e., `+**+`), it should only be combined with exclusions (e.g., `+**;!foo+`).
A negated double wildcard (i.e., `+!**+`), which selects no lines, is usually implied as the starting point.
A negated single wildcard has different meaning depending on whether it comes before tag names (e.g., `+!*;foo+`) or after at least one tag name (e.g., `+foo;!*+`).

"#
    );

    // The double wildcard is applied first regardless of position: `!foo;**` and
    // `**;!foo` select the same lines (all lines but the `foo` region, minus the
    // tag-directive lines).
    let leading = convert_including("....\ninclude::foo-bar.rb[tags=!foo;**]\n....\n");
    let trailing = convert_including("....\ninclude::foo-bar.rb[tags=**;!foo]\n....\n");
    for out in [&leading, &trailing] {
        assert!(out.contains("outside before"), "{out}");
        assert!(out.contains("outside after"), "{out}");
        assert!(!out.contains("foo start"), "{out}");
        assert!(!out.contains("tag::foo"), "{out}");
    }

    // A negated double wildcard selects no lines.
    let negated = convert_including("....\ninclude::foo-bar.rb[tag=!**]\n....\n");
    assert!(!negated.contains("outside before"), "{negated}");
    assert!(!negated.contains("foo start"), "{negated}");

    // A negated single wildcard means different things by position: before a tag
    // name (`!*;foo`) it keeps the non-tagged regions and the `foo` region;
    // after one (`foo;!*`) it keeps only `foo` minus its nested tagged regions.
    let before = convert_including("....\ninclude::foo-bar.rb[tags=!*;foo]\n....\n");
    assert!(before.contains("outside before"), "{before}");
    assert!(before.contains("foo start"), "{before}");

    let after = convert_including("....\ninclude::foo-bar.rb[tags=foo;!*]\n....\n");
    assert!(!after.contains("outside before"), "{after}");
    assert!(after.contains("foo start"), "{after}");
    assert!(!after.contains("bar body"), "{after}");
}

// Non-normative: the sentence that sets up the permutation table (a `foo`
// region with a nested `bar` region — the shape of the `foo-bar.rb` fixture
// used below).
non_normative!(
    r#"
Let's assume we have a region tagged `foo` with a nested region tagged `bar`.
Here are some of the permutations you can use (along with their implied long-hand forms):

"#
);

// Every row of the permutation table, verified against a `foo` region with a
// nested `bar` region and surrounding non-tagged lines (the `foo-bar.rb`
// fixture).
#[test]
fn the_filter_permutations_select_the_documented_lines() {
    verifies!(
        r#"
`+**+`:: Selects all the lines in the document (except for lines that contain a tag directive).
_(implies `+**;*+`)_

`+*+`:: Selects all tagged regions in the document.
Does not select lines outside of tagged regions.
_(implies `+!**;*+`)_

`+!*+`:: Selects only the regions in the document outside of tags (i.e., non-tagged regions).
_(implies `+**;!*+`)_

`foo`:: Selects only regions tagged _foo_.
_(implies `+!**;foo+`)_

`foo;!bar`:: Selects only regions tagged _foo_, but excludes any nested regions tagged _bar_.
_(implies `+!**;foo;!bar+`)_

`+foo;!*+`:: Selects only regions tagged _foo_, but excludes any nested tagged regions.
_(implies `+!**;foo;!*+`)_

`+*;!foo+`:: Selects all tagged regions, but excludes any regions tagged _foo_ (nested or otherwise).
_(implies `+!**;*;!foo+`)_

`!foo`:: Selects all the lines in the document except for regions tagged _foo_.
_(implies `+**;!foo+`)_

`!foo;!bar`:: Selects all the lines in the document except for regions tagged _foo_ or _bar_.
_(implies `+**;!foo;!bar+`)_

`+!*;foo+`:: Selects the regions in the document outside of tags (i.e., non-tagged regions) and inside regions tagged _foo_, excluding any nested tagged regions.
To include nested tagged regions, they each must be named explicitly.
_(implies `+**;!*;foo+`)_

"#
    );

    // Selects `foo-bar.rb` through the given tag filter and returns the output.
    let select =
        |spec: &str| convert_including(&format!("....\ninclude::foo-bar.rb[{spec}]\n....\n"));

    // `**`: all lines except the tag-directive lines.
    let double = select("tag=**");
    assert!(double.contains("outside before"), "{double}");
    assert!(double.contains("foo start"), "{double}");
    assert!(double.contains("bar body"), "{double}");
    assert!(double.contains("outside after"), "{double}");
    assert!(!double.contains("tag::foo"), "{double}");

    // `*`: all tagged regions, none of the surrounding lines.
    let star = select("tag=*");
    assert!(star.contains("foo start"), "{star}");
    assert!(star.contains("bar body"), "{star}");
    assert!(!star.contains("outside before"), "{star}");

    // `!*`: only the non-tagged regions.
    let not_star = select("tag=!*");
    assert!(not_star.contains("outside before"), "{not_star}");
    assert!(not_star.contains("outside after"), "{not_star}");
    assert!(!not_star.contains("foo start"), "{not_star}");

    // `foo`: the whole `foo` region, including its nested `bar` region.
    let foo = select("tag=foo");
    assert!(foo.contains("foo start"), "{foo}");
    assert!(foo.contains("bar body"), "{foo}");
    assert!(foo.contains("foo end"), "{foo}");
    assert!(!foo.contains("outside before"), "{foo}");

    // `foo;!bar`: the `foo` region, excluding the nested `bar` region.
    let foo_not_bar = select("tags=foo;!bar");
    assert!(foo_not_bar.contains("foo start"), "{foo_not_bar}");
    assert!(foo_not_bar.contains("foo end"), "{foo_not_bar}");
    assert!(!foo_not_bar.contains("bar body"), "{foo_not_bar}");

    // `foo;!*`: the `foo` region, excluding every nested tagged region.
    let foo_not_star = select("tags=foo;!*");
    assert!(foo_not_star.contains("foo start"), "{foo_not_star}");
    assert!(foo_not_star.contains("foo end"), "{foo_not_star}");
    assert!(!foo_not_star.contains("bar body"), "{foo_not_star}");

    // `*;!foo`: all tagged regions except `foo` (nested or otherwise) — nothing
    // here, since `bar` is nested inside `foo`.
    let star_not_foo = select("tags=*;!foo");
    assert!(!star_not_foo.contains("foo start"), "{star_not_foo}");
    assert!(!star_not_foo.contains("bar body"), "{star_not_foo}");
    assert!(!star_not_foo.contains("outside before"), "{star_not_foo}");

    // `!foo`: all lines except the `foo` region.
    let not_foo = select("tag=!foo");
    assert!(not_foo.contains("outside before"), "{not_foo}");
    assert!(not_foo.contains("outside after"), "{not_foo}");
    assert!(!not_foo.contains("foo start"), "{not_foo}");
    assert!(!not_foo.contains("bar body"), "{not_foo}");

    // `!foo;!bar`: all lines except the `foo` or `bar` regions.
    let not_foo_bar = select("tags=!foo;!bar");
    assert!(not_foo_bar.contains("outside before"), "{not_foo_bar}");
    assert!(not_foo_bar.contains("outside after"), "{not_foo_bar}");
    assert!(!not_foo_bar.contains("foo start"), "{not_foo_bar}");

    // `!*;foo`: the non-tagged regions plus the `foo` region, excluding nested
    // tagged regions (so `bar` is not included).
    let not_star_foo = select("tags=!*;foo");
    assert!(not_star_foo.contains("outside before"), "{not_star_foo}");
    assert!(not_star_foo.contains("foo start"), "{not_star_foo}");
    assert!(not_star_foo.contains("outside after"), "{not_star_foo}");
    assert!(!not_star_foo.contains("bar body"), "{not_star_foo}");
}

// How a filter's leading modifier fixes its implied starting set: a leading
// negated tag or single wildcard implies the pattern begins with `**` (all
// non-tag-directive lines), while a leading inclusion implies it begins with
// `!**` (no lines).
#[test]
fn the_leading_modifier_sets_the_implied_starting_set() {
    verifies!(
        r#"
If the filter begins with a negated tag or single wildcard, it implies that the pattern begins with `+**+`.
An exclusion not preceded by an inclusion implicitly starts by selecting all the lines that do not contain a tag directive.
Otherwise, it implies that the pattern begins with `+!**+`.
A leading inclusion implicitly starts by selecting no lines.
"#
    );

    // A leading exclusion (`!foo`) starts from all lines, so the non-tagged lines
    // outside `foo` survive.
    let leading_exclusion = convert_including("....\ninclude::foo-bar.rb[tag=!foo]\n....\n");
    assert!(
        leading_exclusion.contains("outside before"),
        "{leading_exclusion}"
    );
    assert!(
        leading_exclusion.contains("outside after"),
        "{leading_exclusion}"
    );

    // A leading inclusion (`foo`) starts from no lines, so only the named region
    // is selected — the non-tagged lines are not.
    let leading_inclusion = convert_including("....\ninclude::foo-bar.rb[tag=foo]\n....\n");
    assert!(
        leading_inclusion.contains("foo start"),
        "{leading_inclusion}"
    );
    assert!(
        !leading_inclusion.contains("outside before"),
        "{leading_inclusion}"
    );
}
