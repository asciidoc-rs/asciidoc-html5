//! Coverage of the AsciiDoc language description's *Inline Passthroughs* page.
//!
//! The page documents every inline passthrough form: the single and double
//! plus shorthands (which apply only the special-characters substitution), the
//! triple plus shorthand (which applies none), and the `pass:[]` macro (which
//! lets the author name the substitutions to apply). This crate renders each
//! form, so every section's described behavior — and the example source it
//! shows, whether inline in the page or pulled in from
//! `ref/asciidoc-lang/docs/modules/pass/examples/pass.adoc` via
//! `include::example$pass.adoc[tag=…]` — is verified through `convert`, driving
//! the exact source the page displays. The section headings, the opening
//! warnings, the substitution-values reference table, and the illustrative
//! source-display asides are tracked as non-normative.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/pass/pages/pass-macro.adoc");

non_normative!(
    r#"
= Inline Passthroughs

AsciiDoc supports several inline passthrough macros and shorthands.
Inline passthroughs are designed to prevent subsitutions for regions of text, or to give you more fine-grained control over which substitutions are applied.

WARNING: Due to the fact that inline syntax in AsciiDoc is processed using substitutions rather than a descending grammar, it's possible to create invalid interleaving of inline elements, or other adverse interactions, that leads to invalid or illogical output.
The inline passthrough provides a bailout option to mitigate these entanglements.
This problem is expected to be resolved properly by the AsciiDoc Language Specification, which will mandate that inline syntax is parsed as a tree rather than through substitutions (to the degree possible).

== Inline passthrough macros

"#
);

// The single and double plus shorthands: only the special-characters
// substitution is applied, and it cannot be modified. Markup that quotes,
// macros, or replacements would otherwise transform is left literal — only the
// angle brackets are escaped.
#[test]
fn single_and_double_plus_apply_only_special_characters() {
    verifies!(
        r#"
[[def-plus]]single and double plus:: A special syntax for preventing inline text from being formatted.
Only xref:subs:special-characters.adoc[special characters] are replaced in the output format.
The substitutions can't be modified for this type of passthrough.

"#
    );

    let output = convert("Escaped: +<b>*x*</b>+ end.\n");

    // Special characters are escaped (the angle brackets become entities) ...
    assert!(output.contains("Escaped: &lt;b&gt;*x*&lt;/b&gt; end."));
    // ... but quotes did not run (`*x*` stays literal) and no element was made.
    assert_css(&output, "strong", 0);
    assert_css(&output, "b", 0);
}

// The triple plus shorthand: no substitutions at all are applied, not even
// special characters, so raw markup passes straight through.
#[test]
fn triple_plus_applies_no_substitutions() {
    verifies!(
        r#"
triple plus:: A special inline syntax for designating passthrough content.
No substitutions are applied nor can they be added using the step and group substitution values.

"#
    );

    let output = convert("Raw: +++<b>*x*</b>+++ end.\n");

    // The angle brackets are not even escaped: the `<b>` markup passes through
    // as a real element, and the enclosed `*x*` keeps its literal asterisks
    // (quotes never ran).
    assert!(output.contains("Raw: <b>*x*</b> end."));
    assert_css(&output, "b", 1);
    assert_css(&output, "strong", 0);
}

// The `pass:[]` macro: it passes its target through, and a target with no
// substitution values applies none (so `#{variable}` stays literal). A target
// naming `c,a` applies the special-characters and attributes substitutions
// selectively — resolving `{email}` and escaping `<`/`>` — while leaving the
// `__…__` quotes markup untouched.
#[test]
fn inline_pass_macro_applies_named_substitutions() {
    verifies!(
        r#"
inline pass macro:: An inline macro named `pass` that can be used to passthrough content.
You can apply specific substitutions to the macro's target using substitution types and groups.
+
[source]
----
pass:[content like #{variable} passed directly to the output] followed by normal content.

content with only select substitutions applied: pass:c,a[__<{email}>__]
----

"#
    );

    // With no substitution values, the macro passes its target through raw.
    let raw = convert(
        "pass:[content like #{variable} passed directly to the output] followed by normal content.\n",
    );
    assert!(raw.contains(
        "content like #{variable} passed directly to the output followed by normal content."
    ));

    // With `c,a`, only special characters and attributes are applied: `{email}`
    // is resolved and `<`/`>` are escaped, but `__` (quotes) stays literal.
    let select = convert(
        ":email: me@example.com\n\ncontent with only select substitutions applied: pass:c,a[__<{email}>__]\n",
    );
    assert!(select.contains("__&lt;me@example.com&gt;__"));
    assert_css(&select, "em", 0);
}

non_normative!(
    r#"
TIP: When you need to prevent or control the substitutions on one or more blocks of content, use a xref:pass-block.adoc[delimited passthrough block or the pass block style].

[#single-double-plus]
== Single and double plus

"#
);

// The single and double plus passthroughs suppress inline formatting: the
// pluses are consumed and the enclosed words are emitted without any quotes
// markup applied.
#[test]
fn single_and_double_plus_suppress_formatting() {
    verifies!(
        r#"
The single and double plus passthroughs prevent text enclosed in either a pair of single pluses (`{plus}`) or a pair of double pluses (`{pp}`) from being formatted.

----
A +word+, a +sequence of words+, or ++char++acters that are escaped from formatting.
----

"#
    );

    let output = convert(
        "A +word+, a +sequence of words+, or ++char++acters that are escaped from formatting.\n",
    );
    assert_xpath(
        &output,
        r#"//p[text()="A word, a sequence of words, or characters that are escaped from formatting."]"#,
        1,
    );
}

non_normative!(
    r#"
The single and double pluses represent the constrained and unconstrained passthrough, respectively.
They have boundaries that match the xref:text:index.adoc#formatting-marks-and-pairs[constrained and unconstrained formatting marks].
The main difference, however, is that they are applied first to suppress formatting.

"#
);

// The single plus around a word or phrase suppresses substitution of its
// contents while still escaping special characters: `+/document/{id}+` is not
// attribute-substituted, `+<+`/`+>+` are escaped, and a formatting mark like
// `+``+` is emitted literally.
#[test]
fn single_plus_suppresses_substitution_but_escapes_special_characters() {
    verifies!(
        r#"
This type of passthrough is intended to suppress any special meaning of the source text itself.
This passthrough type still ensures, however, that the content is properly escaped in the output.
That means the xref:subs:special-characters.adoc[special characters] substitution is still applied.

As with all constrained pairs, the single plus passthrough is designed to be used around a word or phrase.

----
A word or phrase between single pluses, such as +/document/{id}+, is not substituted.
However, the special characters +<+ and +>+ are still escaped in the output.

You can also escape formatting marks, like +``+.
----

"#
    );

    // The attribute reference is not resolved.
    let phrase = convert(
        "A word or phrase between single pluses, such as +/document/{id}+, is not substituted.\n",
    );
    assert!(phrase.contains("/document/{id}"));

    // The special characters are still escaped.
    let special =
        convert("However, the special characters +<+ and +>+ are still escaped in the output.\n");
    assert!(special.contains("&lt;"));
    assert!(special.contains("&gt;"));

    // A formatting mark is emitted literally, not turned into monospace.
    let mark = convert("You can also escape formatting marks, like +``+.\n");
    assert!(mark.contains("like ``."));
    assert_css(&mark, "code", 0);
}

non_normative!(
    r#"
Being an unconstrained pair, the double plus passthrough can be used anywhere in the text.

"#
);

// The double plus, being unconstrained, can be used mid-word and around a link
// target: the target's underscores are not read as formatting, a bare
// formatting mark is emitted literally, and an attribute reference embedded in
// a word is not resolved.
#[test]
fn double_plus_suppresses_formatting_anywhere() {
    verifies!(
        r#"
----
Text formatting is not applied to a link target if it is surrounded by double pluses.
For example, link:++https://example.org/now_this__link_works.html++[].

You can also escape formatting marks, like all-natural++*++.

An attribute reference within a word, such as dev++{conf}++, is not replaced.
----

"#
    );

    // The link target keeps its double underscores rather than forming an emphasis.
    let link = convert("For example, link:++https://example.org/now_this__link_works.html++[].\n");
    assert_xpath(
        &link,
        r#"//a[@href="https://example.org/now_this__link_works.html"]"#,
        1,
    );
    assert_css(&link, "a em", 0);

    // A bare formatting mark and an in-word attribute reference are left literal.
    let mark = convert("You can also escape formatting marks, like all-natural++*++.\n");
    assert_xpath(
        &mark,
        r#"//p[text()="You can also escape formatting marks, like all-natural*."]"#,
        1,
    );

    let attr =
        convert("An attribute reference within a word, such as dev++{conf}++, is not replaced.\n");
    assert_xpath(&attr, r#"//p[contains(text(), "dev{conf}")]"#, 1);
}

non_normative!(
    r#"
The single and plus passthroughs are a surefire alternative to backslash escaping.

"#
);

// The plus passthroughs only prevent substitutions; they do not render the text
// in monospace. A `+word+` produces no `<code>` element.
#[test]
fn plus_passthroughs_do_not_apply_monospace() {
    verifies!(
        r#"
Note that the single and plus passthroughs only prevent substitutions.
They do not format the text in monospace.
If you want to do both, you must enclose the pair in a monospace formatting pair, known as xref:text:literal-monospace.adoc[literal monospace].

"#
    );

    let output = convert("A +word+ is not monospace.\n");
    assert_xpath(&output, r#"//p[text()="A word is not monospace."]"#, 1);
    assert_css(&output, "code", 0);
}

non_normative!(
    r#"
[#triple-plus]
== Triple plus

"#
);

// The triple plus passthrough excludes its content from every substitution. An
// indented line is a literal block whose triple-plus content stays literal, and
// the macro form passes raw HTML (`<del>`) straight through.
#[test]
fn triple_plus_excludes_all_substitutions() {
    verifies!(
        r#"
The triple plus passthrough excludes content enclosed in a pair of triple pluses (pass:[+++]) from all substitutions.

 +++content passed directly to the output+++ followed by normal content.

The triple plus macro is often used to output custom HTML or XML.

[source]
----
include::example$pass.adoc[tag=3p]
----

====
include::example$pass.adoc[tag=3p]
====

"#
    );

    // An indented line is a literal block; its triple-plus text stays verbatim.
    let literal =
        convert(" +++content passed directly to the output+++ followed by normal content.\n");
    assert_xpath(
        &literal,
        r#"//div[@class="literalblock"]//pre[text()="+++content passed directly to the output+++ followed by normal content."]"#,
        1,
    );

    // tag=3p: the triple plus macro passes raw HTML through, so the `<del>`
    // markup reaches the output as a real element rather than escaped text.
    let output = convert("The text +++<del>strike this</del>+++ is marked as deleted.\n");
    assert!(output.contains("The text <del>strike this</del> is marked as deleted."));
    assert_css(&output, "del", 1);
}

non_normative!(
    r#"
[#inline-pass]
== Inline pass macro

"#
);

// The inline pass macro with no substitution values excludes its content from
// all substitutions, so raw HTML (`<del>`) is passed through unescaped.
#[test]
fn inline_pass_macro_excludes_all_substitutions() {
    verifies!(
        r#"
Like other inline passthroughs, the inline pass macro can be used to control the substitutions applied to a run of text.
To exclude inline content from all of the substitutions, enclose it in the inline pass macro.

Here's one way to format text as underline when generating HTML from AsciiDoc:

[source]
----
include::example$pass.adoc[tag=in-macro]
----

And here's the result.

====
include::example$pass.adoc[tag=in-macro]
====

"#
    );

    // tag=in-macro
    let output = convert("The text pass:[<del>strike this</del>] is marked as deleted.\n");
    assert!(output.contains("The text <del>strike this</del> is marked as deleted."));
    assert_css(&output, "del", 1);
}

non_normative!(
    r#"
WARNING: Using passthroughs to send content directly to the output can couple your content to a specific output format, such as HTML.
To avoid this risk, you should consider using conditional preprocessor directives to select content for different output formats based on the current backend.

What sets the inline pass macro apart from the alternatives is that it allows the substitutions to be customized.
The inline pass macro also plays a critical role in the document header.
In fact, it's the only macro that is processed in the document header by default as part of the xref:subs:index.adoc#header-group[header substitution group] (though it can be used to enable other substitutions, as demonstrated in this section).

Let's look at how to use the inline pass macro to hand select substitutions.

=== Custom substitutions

You can customize the substitutions applied to the content of an inline pass macro by specifying one or more substitution values in the target of the macro.
Multiple values must be separated by commas and may not contain any spaces.
The substitution value is either the formal name of a substitution type or group, or its shorthand.

The following table lists the allowable substitution values:

.Substitution values accepted by the inline pass macro
[cols="1m,3m",width=50%]
|===
|Shorthand | Substitution Type

|c
|specialchars

|q
|quotes

|a
|attributes

|r
|replacements

|m
|macros

|p
|post replacements

h|Shorthand
h| Substitution Group

|n
|normal

|v
|verbatim
|===

"#
);

// The quotes substitution named on the macro (`q`) applies inline formatting to
// the target — `*this*` becomes `<strong>` — while the special-characters
// substitution stays off, so `<del>` passes through raw.
#[test]
fn inline_pass_macro_with_quotes_value() {
    verifies!(
        r#"
For example, the quotes substitution (i.e., `q` or `quotes`) is enabled on the inline passthrough macro as follows:

[source]
----
include::example$pass.adoc[tag=in-macro-sub]
----

Here's the result.

====
include::example$pass.adoc[tag=in-macro-sub]
====

"#
    );

    // tag=in-macro-sub
    let output = convert("The text pass:q[<del>strike *this*</del>] is marked as deleted.\n");
    assert_xpath(&output, r#"//del/strong[text()="this"]"#, 1);
    // Special characters stayed off: the `<del>` came through as raw markup.
    assert_css(&output, "del", 1);
}

// Multiple substitution values (`q,a`) apply in combination: quotes formats
// `_…_` into `<em>` and attributes resolves the `{docname}` reference.
#[test]
fn inline_pass_macro_with_multiple_values() {
    verifies!(
        r#"
To enable multiple substitution groups, separate each value in the macro target by a comma:

[source]
----
include::example$pass.adoc[tag=in-macro-subs]
----

Here's the result.

====
include::example$pass.adoc[tag=in-macro-subs]
====

"#
    );

    // tag=in-macro-subs
    let output =
        convert("The text pass:q,a[<del>strike _{docname}_</del>] is marked as deleted.\n");
    assert_xpath(&output, r#"//del/em"#, 1);
    assert_css(&output, "del", 1);
}

non_normative!(
    r#"
== Nesting blocks and passthroughs

When you're using passthroughs inside literal and listing blocks, it can be easy to forget that the single plus and triple plus passthroughs are xref:subs:macros.adoc[macros substitutions].
To enable the passthroughs, assign the `macros` value to the `subs` attribute.

....
[source,java,subs="+quotes,+macros"]
----
protected void configure(HttpSecurity http) throws Exception {
    http
        .authorizeRequests()
            **.antMatchers("/resources/+++**+++").permitAll()**
            .anyRequest().authenticated()
            .and()
        .formLogin()
            .loginPage("/login")
            .permitAll();
----
....

To learn more about applying substitutions to blocks, see xref:subs:apply-subs-to-blocks.adoc[].

"#
);

// A listing block with `subs="+quotes,+macros"` runs both added substitutions:
// the `**…**` quotes markup becomes `<strong>`, and the triple-plus passthrough
// (a macros substitution) keeps the enclosed `**` literal instead of
// formatting.
#[test]
fn nested_passthrough_in_listing_with_added_subs() {
    verifies!(
        r#"
[source,java,subs="+quotes,+macros"]
----
protected void configure(HttpSecurity http) throws Exception {
    http
        .authorizeRequests()
            **.antMatchers("/resources/+++**+++").permitAll()**
            .anyRequest().authenticated()
            .and()
        .formLogin()
            .loginPage("/login")
            .permitAll();
----
"#
    );

    let output = convert(
        "[source,java,subs=\"+quotes,+macros\"]\n----\n\
         protected void configure(HttpSecurity http) throws Exception {\n    http\n        \
         .authorizeRequests()\n            \
         **.antMatchers(\"/resources/+++**+++\").permitAll()**\n            \
         .anyRequest().authenticated()\n            .and()\n        .formLogin()\n            \
         .loginPage(\"/login\")\n            .permitAll();\n----\n",
    );

    // The quotes substitution formatted the `**…**` run.
    assert_xpath(
        &output,
        r#"//code//strong[contains(text(), ".antMatchers")]"#,
        1,
    );
    // The triple-plus passthrough kept the inner `**` literal.
    assert_xpath(
        &output,
        r#"//code//strong[contains(text(), "/resources/**")]"#,
        1,
    );
}
