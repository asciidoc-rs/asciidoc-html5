//! Coverage of the AsciiDoc language description's *ifeval Directive* page.
//!
//! The verifiable rules are that `ifeval` includes its enclosed lines when the
//! bracketed expression evaluates to true; that an expression is a left value,
//! an operator, and a right value; that quoted operands compare as strings and
//! unquoted numeric operands as numbers; that mixing value types makes the
//! comparison fail (and so evaluate to false); and the behavior of each
//! comparison operator. All are driven through `convert`. The prose about the
//! absence of single-line/long-form variants, the complement-terminator
//! restriction, and the Ruby-style coercion taxonomy is non-normative.

use super::convert_with_attrs;
use crate::tests::sdd::*;

track_file!("ref/asciidoc-lang/docs/modules/directives/pages/ifeval.adoc");

non_normative!(
    r#"
= ifeval Directive

"#
);

// `ifeval` includes the lines it encloses when the bracketed expression
// evaluates to true, and drops them when it evaluates to false.
#[test]
fn ifeval_includes_enclosed_lines_when_the_expression_is_true() {
    verifies!(
        r#"
Lines enclosed by an `ifeval` directive (i.e., between the `ifeval` and `endif` directives) are included if the expression inside the square brackets of the `ifeval` directive evaluates to true.

.ifeval example
----
\ifeval::[{sectnumlevels} == 3]
If the `sectnumlevels` attribute has the value 3, this sentence is included.
\endif::[]
----

"#
    );

    let sentence = "If the sectnumlevels attribute has the value 3, this sentence is included.";

    // `sectnumlevels` defaults to 3, so the expression is true and the sentence
    // is included.
    let truthy = convert_with_attrs(
        "ifeval::[{sectnumlevels} == 3]\nIf the sectnumlevels attribute has the value 3, this sentence is included.\nendif::[]",
        &[],
    );
    assert!(truthy.contains(sentence), "{truthy}");

    // A false expression drops the enclosed line.
    let falsy = convert_with_attrs(
        "ifeval::[{sectnumlevels} == 5]\nIf the sectnumlevels attribute has the value 3, this sentence is included.\nendif::[]",
        &[],
    );
    assert!(!falsy.contains(sentence), "{falsy}");
}

// Non-normative: `ifeval` has no single-line or long-form variant, and cannot
// be terminated by a complement `endif` — only an anonymous `endif::[]`. These
// examples use `<condition>` placeholders that carry no executable expression.
non_normative!(
    r#"
The `ifeval` directive does not have a single-line or long-form variant like `ifdef` and `ifndef`.

Unlike `ifdef` and `ifndef`, you cannot terminate a specific `ifeval` directive using its complement.
For example, the following `ifeval` block is not valid:

.Invalid ifeval terminator
----
\ifeval::[<condition>]
conditional content
\endif::[<condition>]
----

You can only terminate the previous `ifeval` directive using an anonymous `endif::[]` directive, as shown here:

.Valid ineval terminator
----
\ifeval::[<condition>]
conditional content
\endif::[]
----

If you're mixing `ifeval` directives with `ifdef` or `ifndef` directives, you should always close multiline `ifdef` and `ifndef` directives by name (`endif::name-of-attribute[]`) so the `ifeval` directive does not end prematurely.

== Anatomy

"#
);

// An expression is a left-hand value, an operator, and a right-hand value.
// Verified with a bare numeric comparison and a quoted string comparison
// against a resolved attribute.
#[test]
fn an_expression_is_a_left_value_an_operator_and_a_right_value() {
    verifies!(
        r#"
The expression of an `ifeval` directive consists of a left-hand value and a right-hand value with an operator in between.
It's customary to include a single space on either side of the operator.

.ifeval expression examples
----
\ifeval::[2 > 1]
...
\endif::[]

\ifeval::["{backend}" == "html5"]
...
\endif::[]

\ifeval::[{sectnumlevels} == 3]
...
\endif::[]

// the value of outfilesuffix includes a leading period (e.g., .html)
\ifeval::["{docname}{outfilesuffix}" == "main.html"]
...
\endif::[]
----

"#
    );

    let numeric = convert_with_attrs("ifeval::[2 > 1]\ntwo exceeds one\nendif::[]", &[]);
    assert!(numeric.contains("two exceeds one"), "{numeric}");

    // `backend` resolves to `html5`, so the quoted-string comparison holds.
    let string = convert_with_attrs(
        "ifeval::[\"{backend}\" == \"html5\"]\nthe backend is html5\nendif::[]",
        &[],
    );
    assert!(string.contains("the backend is html5"), "{string}");
}

non_normative!(
    r#"
== Values

"#
);

// Attribute references are resolved first, then each side is coerced to a type:
// a quoted operand is a string, an unquoted numeric operand is a number, and
// mixing types makes the comparison fail (so the condition is false).
#[test]
fn quoted_operands_are_strings_and_unquoted_numeric_operands_are_numbers() {
    verifies!(
        r#"
Each expression value can reference the name of zero or more AsciiDoc attributes using the attribute reference syntax (for example, `+{backend}+`).

Attribute references are resolved (i.e., substituted) first.
Once attributes references have been resolved, each value is coerced to a recognized type.

When you expect the attribute reference to resolve to a string, that is, a sequence of characters, enclose that side of the expression in quotes.
For example:

.ifeval that compares two string expressions
----
\ifeval::["{backend}" == "html5"]
----

If you expect the attribute to resolve to a number, you do not need to enclose the expression in quotes.
In this case, the values will be compared as numbers.
The same rule applies to boolean values.

You should not attempt to mix value types in a comparison.
For example, the following expression is not valid:

.Invalid ifeval expression
----
\ifeval::["{sectnumlevels}" > 3]
----

"#
    );

    // A quoted operand compares as a string.
    let string_match = convert_with_attrs(
        "ifeval::[\"{backend}\" == \"html5\"]\nstring compared\nendif::[]",
        &[],
    );
    assert!(string_match.contains("string compared"), "{string_match}");

    let string_miss = convert_with_attrs(
        "ifeval::[\"{backend}\" == \"docbook5\"]\nstring compared\nendif::[]",
        &[],
    );
    assert!(!string_miss.contains("string compared"), "{string_miss}");

    // An unquoted numeric operand compares as a number.
    let number = convert_with_attrs(
        "ifeval::[{sectnumlevels} == 3]\nnumber compared\nendif::[]",
        &[],
    );
    assert!(number.contains("number compared"), "{number}");

    // Mixing a quoted (string) side with an unquoted number is invalid, so the
    // comparison fails and the content is skipped.
    let mixed = convert_with_attrs(
        "ifeval::[\"{sectnumlevels}\" > 3]\nmixed types\nendif::[]",
        &[],
    );
    assert!(!mixed.contains("mixed types"), "{mixed}");
}

// Non-normative: the taxonomy of recognized value types and the Ruby-style
// coercion rules for unquoted values (nil, boolean, whitespace, float, integer)
// — internal coercion detail observable only indirectly through comparisons.
non_normative!(
    r#"
The following values types are recognized:

number:: Either an integer or floating-point value.
quoted string:: Enclosed in either single (`'`) or double (`"`) quotes.
boolean:: Literal value of `true` or `false`.

=== How value type coercion works

If a value is enclosed in quotes, the characters between the quotes is used and always coerced to a string.

If a value is *not* enclosed in quotes, it's subject to the following type coercion rules:

* an empty value becomes nil (aka null) (and thus safe for use in a comparison).
* a value of `true` or `false` becomes a boolean value.
* a value of only repeating whitespace becomes a single whitespace string.
* a value containing a period becomes a floating-point number.
* any other value is coerced to an integer value.

== Operators

"#
);

// Each comparison operator behaves as documented, and a comparison between
// operands of different types fails — so the condition evaluates to false and
// the enclosed content is skipped.
#[test]
fn each_operator_compares_and_a_type_mismatch_evaluates_to_false() {
    verifies!(
        r#"
The value on each side is compared using the operator to derive an outcome.

`==`::
Checks if the two values are equal.
`!=`::
Checks if the two values are not equal.
`<`::
Checks whether the left-hand side is less than the right-hand side.
`+<=+`::
Checks whether the left-hand side is less than or equal to the right-hand side.
`>`::
Checks whether the left-hand side is greater than the right-hand side.
`+>=+`::
Checks whether the left-hand side is greater than or equal to the right-hand side.

Both sides should be of the same value type.
If they are not, the comparison will fail.
If the comparison fails, the condition will evaluate to false (i.e., the content inside the directive will be skipped).

"#
    );

    // Each operator, given a true comparison, keeps its content.
    for expr in ["3 == 3", "2 != 3", "1 < 2", "2 <= 2", "3 > 2", "3 >= 3"] {
        let src = format!("ifeval::[{expr}]\nkept\nendif::[]");
        let out = convert_with_attrs(&src, &[]);
        assert!(out.contains("kept"), "expr `{expr}`:\n{out}");
    }

    // A false comparison drops the content.
    let dropped = convert_with_attrs("ifeval::[1 > 2]\nkept\nendif::[]", &[]);
    assert!(!dropped.contains("kept"), "{dropped}");

    // Comparing a string operand to a number is a type mismatch: the comparison
    // fails and the content is skipped.
    let mismatch = convert_with_attrs("ifeval::[\"x\" > 1]\nkept\nendif::[]", &[]);
    assert!(!mismatch.contains("kept"), "{mismatch}");
}

non_normative!(
    r#"
The operators follow the same rules as operators in Ruby.
"#
);
