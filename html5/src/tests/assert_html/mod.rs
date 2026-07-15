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
/// embedded body fragment. The returned flag is `true` for a fragment parse —
/// the callers need it because scraper wraps a fragment in a synthetic `<html>`
/// that shifts what "the root" means (see [`dom::from_html`] and
/// [`rewrite_root_for_fragment`]).
fn parse(html: &str) -> (Html, bool) {
    let head = html.trim_start();
    if head.len() >= 5 && head[..5].eq_ignore_ascii_case("<html")
        || head.len() >= 9 && head[..9].eq_ignore_ascii_case("<!doctype")
    {
        (Html::parse_document(html), false)
    } else {
        (Html::parse_fragment(html), true)
    }
}

/// Rewrites a `:root` selector for a fragment parse.
///
/// The Ruby suite uses `:root` to pin an assertion to a *top-level* element of
/// the embedded fragment (e.g. `.paragraph:root`). Nokogiri — the oracle —
/// models a fragment with no wrapper element, so those top-level elements are
/// the document roots and match `:root`. `scraper` instead wraps every fragment
/// in a synthetic `<html>`, so the same elements are that wrapper's direct
/// children and `:root` matches nothing. Anchor the selector under the wrapper
/// (`html > …`) and drop `:root` to recover Nokogiri's meaning.
///
/// Only a `:root` on the leading compound is supported — the sole form the
/// suite uses; anything else fails loudly rather than silently matching the
/// wrong set.
fn rewrite_root_for_fragment(selector: &str) -> String {
    let idx = selector
        .find(":root")
        .expect("selector must contain `:root`");
    assert!(
        !selector[..idx].contains([' ', '>', '+', '~', ',']),
        "assert_css supports `:root` only on the leading compound selector (got `{selector}`)"
    );
    let stripped = selector.replacen(":root", "", 1);

    format!("html > {stripped}")
}

/// Asserts that `selector` matches exactly `expected` elements in `html`.
///
/// # Panics
///
/// Panics if the match count differs from `expected`, or if `selector` is not a
/// valid CSS selector.
#[track_caller]
pub(crate) fn assert_css(html: &str, selector: &str, expected: usize) {
    let (document, is_fragment) = parse(html);

    // `scraper`'s selector engine treats a fragment's synthetic `<html>` wrapper
    // as the root, so `:root` never matches a fragment's top-level elements the
    // way Nokogiri does; rewrite it to the equivalent wrapper-anchored selector.
    let rewritten;
    let selector = if is_fragment && selector.contains(":root") {
        rewritten = rewrite_root_for_fragment(selector);
        &rewritten
    } else {
        selector
    };

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
    let (document, is_fragment) = parse(html);
    let root = dom::from_html(&document, is_fragment);
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

    // Two top-level sibling paragraphs, the first with a block title — the
    // shape the paragraphs suite asserts against with grouped and `:root`
    // selectors.
    const SIBLINGS: &str = r#"<div class="paragraph">
<div class="title">Titled</div>
<p>Paragraph.</p>
</div>
<div class="paragraph">
<p>Winning.</p>
</div>"#;

    #[test]
    fn xpath_leading_slash_matches_fragment_top_level() {
        // A leading `/` is a child step from the (wrapperless) fragment root, so
        // it matches the fragment's own top-level elements — not scraper's
        // synthetic `<html>`.
        assert_xpath(SIBLINGS, r#"/*[@class="paragraph"]"#, 2);
        assert_xpath(SIBLINGS, r#"/*[@class="paragraph"]/p"#, 2);
        assert_xpath(FRAGMENT, r#"/*[@id="content"]"#, 1);
    }

    #[test]
    fn xpath_grouped_positional_is_global() {
        // `(//p)[N]` picks the Nth paragraph across the whole document, unlike a
        // per-context `//p[N]`.
        assert_xpath(SIBLINGS, r#"(//p)[1][text()="Paragraph."]"#, 1);
        assert_xpath(SIBLINGS, r#"(//p)[2][text()="Winning."]"#, 1);
        assert_xpath(SIBLINGS, r#"(//p)[2][text()="Paragraph."]"#, 0);

        // A trailing relative path runs from the group's positional result.
        assert_xpath(
            SIBLINGS,
            r#"(//p)[1]/preceding-sibling::*[@class="title"]"#,
            1,
        );
        assert_xpath(
            SIBLINGS,
            r#"(//p)[1]/preceding-sibling::*[@class="title"][text()="Titled"]"#,
            1,
        );
        // The second paragraph has no title sibling.
        assert_xpath(
            SIBLINGS,
            r#"(//p)[2]/preceding-sibling::*[@class="title"]"#,
            0,
        );

        // A grouped child path, then a positional pick, then a further step.
        assert_xpath(
            SIBLINGS,
            r#"(/*[@class="paragraph"])[1]/p[text()="Paragraph."]"#,
            1,
        );
        assert_xpath(
            SIBLINGS,
            r#"(/*[@class="paragraph"])[2]/p[text()="Winning."]"#,
            1,
        );
    }

    #[test]
    fn xpath_text_value_may_contain_brackets() {
        // A `]` inside a quoted predicate value must not be read as the end of
        // the predicate (verbatim blocks routinely contain `[]`).
        let html =
            r#"<div class="literalblock"><div class="content"><pre>image::x[]</pre></div></div>"#;
        assert_xpath(
            html,
            r#"/*[@class="literalblock"]//pre[text()="image::x[]"]"#,
            1,
        );
        assert_xpath(html, r#"//pre[text()="image::x[]"]"#, 1);
    }

    #[test]
    fn xpath_contains_and_normalize_space_text() {
        let html = r#"<div class="quoteblock"><blockquote>
Famous quote.
</blockquote></div>
<div class="verseblock"><pre class="content">Famous   verse.</pre></div>"#;
        // `contains(text(), …)` matches an element whose direct text includes
        // the substring (the blockquote's text has surrounding newlines).
        assert_xpath(
            html,
            r#"//*[@class="quoteblock"]//*[contains(text(), "Famous quote.")]"#,
            1,
        );
        assert_xpath(html, r#"//blockquote[contains(text(), "nope")]"#, 0);
        // `normalize-space(text())` collapses internal whitespace before the
        // comparison.
        assert_xpath(
            html,
            r#"//pre[normalize-space(text()) = "Famous verse."]"#,
            1,
        );
        assert_xpath(html, r#"//pre[text() = "Famous verse."]"#, 0);
    }

    #[test]
    fn css_root_matches_fragment_top_level() {
        // `:root` pins to a fragment's top-level elements, mirroring Nokogiri —
        // `scraper`'s wrapper `<html>` would otherwise make it match nothing.
        assert_css(SIBLINGS, ".paragraph:root", 2);
        assert_css(SIBLINGS, ".paragraph:root > p", 2);
        // A nested paragraph is not a root, so it does not match.
        assert_css(FRAGMENT, ".paragraph:root", 0);
        assert_css(FRAGMENT, ".sect1:root", 0);
        assert_css(FRAGMENT, "#content:root", 1);
    }

    #[test]
    #[should_panic(expected = "leading compound")]
    fn css_root_on_inner_compound_panics() {
        // `:root` on anything but the leading compound is unsupported; it must
        // fail loudly rather than anchor the wrong element.
        assert_css(FRAGMENT, "#content .paragraph:root", 0);
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
    #[should_panic(expected = "unsupported XPath predicate")]
    fn xpath_unsupported_predicate_panics() {
        // `starts-with(...)` is not implemented; it must fail loudly, not be
        // silently ignored (which would drop the predicate and over-match).
        assert_xpath(FRAGMENT, r#"//p[starts-with(text(),"Pre")]"#, 1);
    }

    #[test]
    #[should_panic(expected = "unsupported XPath axis")]
    fn xpath_unsupported_axis_panics() {
        assert_xpath(FRAGMENT, "//p/ancestor::div", 1);
    }

    #[test]
    #[should_panic(expected = "unterminated XPath predicate")]
    fn xpath_unterminated_predicate_panics() {
        // A typo — `[` with no closing `]` — must fail, not silently evaluate
        // as `//p` without the predicate.
        assert_xpath(FRAGMENT, r#"//p[text()="Preamble."#, 1);
    }

    #[test]
    #[should_panic(expected = "malformed XPath node test")]
    fn xpath_stray_closing_bracket_panics() {
        // `p]` must not be accepted as a tag named "p]" (a silent zero-match).
        assert_xpath(FRAGMENT, "//p]", 0);
    }

    #[test]
    #[should_panic(expected = "malformed XPath node test")]
    fn xpath_trailing_text_after_predicate_panics() {
        // Junk after the predicate must fail, not be dropped.
        assert_xpath(FRAGMENT, r#"//p[@id="x"]oops"#, 0);
    }

    #[test]
    fn xpath_class_predicate_is_exact_not_token() {
        // XPath `@class="v"` is exact equality, so a multi-class element is not
        // matched by a single-token value (unlike CSS `.paragraph`).
        let html = r#"<div class="paragraph lead"><p>x</p></div>
<div class="paragraph"><p>y</p></div>"#;
        assert_xpath(html, r#"//div[@class="paragraph"]"#, 1);
        assert_xpath(html, r#"//div[@class="paragraph lead"]"#, 1);
        assert_xpath(html, r#"//div[@class="lead"]"#, 0);
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
