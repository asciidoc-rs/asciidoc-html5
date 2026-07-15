//! HTML-output assertion helpers for porting Asciidoctor's Ruby test suite.
//!
//! Asciidoctor's tests assert on rendered HTML with two Nokogiri-backed
//! helpers: `assert_css` (a CSS selector) and `assert_xpath` (an XPath
//! expression), each optionally with an expected match count. This module is
//! the `asciidoc-html5` counterpart, mirroring the `assert_dom` harness in
//! `asciidoc-parser` but querying the *rendered HTML string* rather than the
//! parsed `Document`.
//!
//! # Design
//!
//! - **Parsing** uses [`scraper`] (a real html5ever tree builder), so the DOM
//!   the assertions see is built with the same HTML5 tree-construction rules a
//!   browser (and Nokogiri) applies. [`parse`] chooses a full-document or
//!   fragment parse by sniffing the string, matching how Asciidoctor's tests
//!   pick a Nokogiri parser: `convert` output is an embedded fragment, while
//!   `convert_with(…standalone(true))` output is a full document.
//! - **`assert_css`** uses `scraper`'s native selector engine (Servo's
//!   `selectors`), which already covers every selector idiom the Ruby suite
//!   uses (`>`, descendant, `:nth-child`, `:last-of-type`, `[attr*=…]`, `:not`,
//!   `:empty`, …). There is no reason to reimplement CSS.
//! - **`assert_xpath`** runs a small [`xpath`] subset over a [`dom`] projection
//!   of the parsed tree. `scraper` has no XPath support, and a faithful XPath
//!   engine over real HTML would be a large dependency (libxml2) or a large
//!   amount of code; the Ruby suite only exercises a narrow, well-understood
//!   slice of XPath, so we implement exactly that slice. See [`xpath`] for the
//!   supported grammar and the deliberate exclusions.
//!
//! # Count semantics
//!
//! Both helpers take an explicit expected count, matching the most common form
//! in the Ruby suite (`assert_xpath expr, output, N`). Asciidoctor also allows
//! omitting the count (meaning "at least one match") and passing a boolean for
//! `count(...)`-style expressions; those forms are not needed yet and can be
//! added as sibling helpers when a ported page first requires them.

mod dom;
mod xpath;

use scraper::{Html, Selector};

/// Parses `html` into a `scraper` tree, choosing a document or fragment parse
/// by sniffing for a leading doctype / `<html>` (the standalone case) versus an
/// embedded body fragment.
fn parse(html: &str) -> Html {
    let head = html.trim_start();
    if head.len() >= 5 && head[..5].eq_ignore_ascii_case("<html")
        || head.len() >= 9 && head[..9].eq_ignore_ascii_case("<!doctype")
    {
        Html::parse_document(html)
    } else {
        Html::parse_fragment(html)
    }
}

/// Asserts that `selector` matches exactly `expected` elements in `html`.
///
/// # Panics
///
/// Panics if the match count differs from `expected`, or if `selector` is not a
/// valid CSS selector.
#[track_caller]
pub(crate) fn assert_css(html: &str, selector: &str, expected: usize) {
    let document = parse(html);
    let compiled = Selector::parse(selector)
        .unwrap_or_else(|e| panic!("invalid CSS selector `{selector}`: {e:?}"));
    let count = document.select(&compiled).count();

    assert_eq!(
        count, expected,
        "CSS `{selector}` matched {count} element(s), expected {expected}, in:\n{html}"
    );
}

/// Asserts that `xpath` matches exactly `expected` nodes in `html`.
///
/// `xpath` must fall within the supported subset documented in [`xpath`].
///
/// # Panics
///
/// Panics if the match count differs from `expected`.
#[track_caller]
pub(crate) fn assert_xpath(html: &str, xpath: &str, expected: usize) {
    let document = parse(html);
    let root = dom::from_html(&document);
    let count = xpath::query(&root, xpath).len();

    assert_eq!(
        count, expected,
        "XPath `{xpath}` matched {count} node(s), expected {expected}, in:\n{html}"
    );
}

#[cfg(test)]
mod tests {
    use super::{assert_css, assert_xpath};

    const FRAGMENT: &str = r#"<div id="content">
<div id="preamble">
<div class="sectionbody">
<div class="paragraph"><p>Preamble.</p></div>
</div>
</div>
<div class="sect1">
<h2 id="_first_section">First Section</h2>
<div class="sectionbody">
<div class="paragraph"><p>Body.</p></div>
</div>
</div>
</div>"#;

    #[test]
    fn css_counts_and_structure() {
        assert_css(FRAGMENT, "p", 2);
        assert_css(FRAGMENT, "#content > .sect1 > h2", 1);
        assert_css(FRAGMENT, "div.paragraph > p", 2);
        assert_css(FRAGMENT, "h2#_first_section", 1);
        assert_css(FRAGMENT, "#preamble", 1);
    }

    #[test]
    fn xpath_descendant_child_and_predicates() {
        assert_xpath(FRAGMENT, "//p", 2);
        assert_xpath(FRAGMENT, r#"//*[@id="preamble"]"#, 1);
        assert_xpath(FRAGMENT, r#"//*[@id="preamble"]//p"#, 1);
        assert_xpath(FRAGMENT, r#"//h2[@id="_first_section"]"#, 1);
        assert_xpath(FRAGMENT, r#"//*[@id="content"]/*[@class="sect1"]"#, 1);
    }

    #[test]
    fn xpath_following_sibling_axis() {
        // The preamble's following sibling is the section, which contains the h2.
        assert_xpath(
            FRAGMENT,
            r#"//*[@id="preamble"]/following-sibling::*//h2[@id="_first_section"]"#,
            1,
        );
        // The section has no following sibling.
        assert_xpath(FRAGMENT, r#"//*[@class="sect1"]/following-sibling::*"#, 0);
    }

    #[test]
    fn xpath_general_preceding_and_following_axes() {
        // `preceding::` sees earlier elements but not the context's ancestors.
        assert_xpath(FRAGMENT, r#"//h2[@id="_first_section"]/preceding::p"#, 1);
        // The first paragraph's only earlier elements are its ancestors.
        assert_xpath(FRAGMENT, r#"//p[text()="Preamble."]/preceding::*"#, 0);

        // `following::` sees later elements but not the context's descendants.
        assert_xpath(FRAGMENT, r#"//*[@id="preamble"]/following::p"#, 1);
        // `#content` is the outermost element, so nothing follows it.
        assert_xpath(FRAGMENT, r#"//*[@id="content"]/following::*"#, 0);
    }

    #[test]
    fn xpath_positional_and_text() {
        assert_xpath(FRAGMENT, r#"//div[@class="paragraph"]/p"#, 2);
        assert_xpath(FRAGMENT, r#"//p[text()="Preamble."]"#, 1);
        // `[1]` is per-context: the first `p` of each of the two paragraph divs.
        assert_xpath(FRAGMENT, r#"//div[@class="paragraph"]/p[1]"#, 2);
    }

    #[test]
    fn standalone_document_is_parsed() {
        let doc = "<!DOCTYPE html>\n<html><head><title>t</title></head><body>\
                   <div id=\"content\"><div class=\"paragraph\"><p>Hi.</p></div></div>\
                   </body></html>";
        assert_css(doc, "#content > .paragraph > p", 1);
        assert_xpath(doc, r#"//*[@id="content"]/*[@class="paragraph"]/p"#, 1);
        assert_xpath(doc, "//title", 1);
    }
}
