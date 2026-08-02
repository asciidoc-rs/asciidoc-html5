//! Coverage of the AsciiDoc language description's *Highlight PHP Source Code*
//! page.
//!
//! PHP can be written in a pure form or mixed with HTML, and the page explains
//! which language tag (`php`, `html+php`, or `php` plus the `mixed` option) to
//! use for each. The actual disambiguation happens inside the syntax
//! highlighter, which this crate does not run server-side; what this crate can
//! verify is that each language tag is accepted and carried onto the source
//! block's `code` element, and that the page's syntax displays render verbatim.

use crate::{
    convert,
    tests::{
        assert_html::{assert_css, assert_xpath},
        sdd::*,
    },
};

track_file!("ref/asciidoc-lang/docs/modules/verbatim/pages/highlight-php.adoc");

// Conceptual introduction: PHP's two modes and the challenge they pose to the
// syntax highlighter. The highlighter behavior itself is not performed by this
// crate; the observable language-tag handling is verified below.
non_normative!(
    r#"
= Highlight PHP Source Code

The PHP language has two modes.
It can either be used as a standalone language (pure mode) or it can be mixed with HTML (mixed mode) by putting it inside PHP tags (a form that resembles an XML processing instruction).
This presents some challenges for the syntax highlighter.

"#
);

// Pure PHP uses the `php` language tag. The source block carries the language
// onto its `code` element (`language-php`, `data-lang="php"`), and the page's
// surrounding `[listing]` block displays the source verbatim.
#[test]
fn pure_php_uses_the_php_language_tag() {
    verifies!(
        r#"
If the code in the source block is pure PHP, you should use the language tag `php`.
For example:

[listing]
....
[source,php]
----
echo "Hello, World!";
----
....

"#
    );

    // The `php` language tag is carried onto the source block's `code` element.
    let source = convert("[source,php]\n----\necho \"Hello, World!\";\n----\n");
    assert_css(&source, "pre.highlight > code.language-php", 1);
    assert_xpath(&source, r#"//code[@data-lang="php"]"#, 1);

    // The page's `[listing]` block displays that source verbatim.
    let display =
        convert("[listing]\n....\n[source,php]\n----\necho \"Hello, World!\";\n----\n....\n");
    assert_css(&display, "div.listingblock > div.content > pre", 1);
    assert!(display.contains("[source,php]\n----\necho \"Hello, World!\";\n----"));
}

// Mixed PHP/HTML can use the `html+php` language tag, which is carried onto the
// `code` element unchanged (`language-html+php`).
#[test]
fn mixed_php_can_use_the_html_php_language_tag() {
    verifies!(
        r#"
If the PHP source is mixed with HTML, you should either use the language tag `html+php`, as shown here:

[listing]
....
[source,html+php]
----
<p>
<?php echo "Hello, World!"; ?>
</p>
----
....

"#
    );

    let source =
        convert("[source,html+php]\n----\n<p>\n<?php echo \"Hello, World!\"; ?>\n</p>\n----\n");
    assert_css(&source, "div.listingblock > div.content > pre.highlight", 1);

    // The `+` in the language class makes it awkward as a CSS token, so the
    // class and `data-lang` are checked by exact-match XPath.
    assert_xpath(&source, r#"//code[@class="language-html+php"]"#, 1);
    assert_xpath(&source, r#"//code[@data-lang="html+php"]"#, 1);
}

// Alternatively, the `php` language tag with the `mixed` option (the `%mixed`
// shorthand) selects mixed mode. The option is accepted and the block still
// renders as a `php` source block.
#[test]
fn mixed_php_can_use_the_mixed_option() {
    verifies!(
        r#"
Or you should use the language tag `php` and set the `mixed` option on the source block, as shown here:

[listing]
....
[source%mixed,php]
----
<p>
<?php echo "Hello, World!"; ?>
</p>
----
....

"#
    );

    let source =
        convert("[source%mixed,php]\n----\n<p>\n<?php echo \"Hello, World!\"; ?>\n</p>\n----\n");
    assert_css(&source, "pre.highlight > code.language-php", 1);
    assert_xpath(&source, r#"//code[@data-lang="php"]"#, 1);
}

// The disambiguation described here — assuming an implicit start PHP tag for
// the `php` tag, disabled by `mixed` or `html+php` — is internal to the syntax
// highlighter, which this crate does not run server-side. Not observable in the
// rendered output.
non_normative!(
    r#"
Under the covers, the syntax highlighter is configured to assume an implicit start PHP tag is present when the language tag is `php`.
Both the `mixed` option and the language tag `html+php` disable this setting.
"#
);
