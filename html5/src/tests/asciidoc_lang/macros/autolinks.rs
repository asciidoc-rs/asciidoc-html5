//! Coverage of the AsciiDoc language description's *Autolinks* page.
//!
//! The macros substitution detects bare URLs (of recognized schemes) and email
//! addresses and converts them to links: a URL autolink carries the `bare`
//! role, an email autolink becomes a `mailto:` link. Angle brackets around a
//! URL are discarded but mark its boundary; a leading backslash (or disabling
//! the macros substitution) prevents autolinking. Each behavior is verified
//! through `convert`, matching Asciidoctor 2.0.26.
//!
//! Two examples are pulled in with `include::example$url.adoc[tag=…]`; the
//! tests reproduce those tagged snippets inline.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/autolinks.adoc");

non_normative!(
    r#"
= Autolinks

The AsciiDoc processor will detect common URLs (unless escaped) wherever the macros substitution step is applied and automatically convert them into links.
This page documents the recognized URL schemes and how to disable this behavior on a case-by-case basis.

"#
);

// Each recognized scheme is autolinked without markup. A URL autolink carries
// the `bare` role.
#[test]
fn url_schemes() {
    verifies!(
        r#"
== URL schemes for autolinks

AsciiDoc recognizes the following common URL schemes without the help of any markup:

[#schemes]
* http
* https
* ftp
* irc
* mailto

"#
    );

    // http, https, ftp, and irc URLs each autolink with the `bare` role. (The
    // `mailto` scheme is exercised by `email_autolink` below.)
    for scheme in ["http", "https", "ftp", "irc"] {
        let url = format!("{scheme}://example.org");
        let output = convert(&format!("Visit {url} now."));
        assert!(
            output.contains(&format!(r#"<a href="{url}" class="bare">{url}</a>"#)),
            "{scheme} URL should autolink with the bare role; got:\n{output}"
        );
    }
}

// A bare URL is both the target and the link text, and a trailing period is not
// captured into the link.
#[test]
fn https_example() {
    verifies!(
        r#"
The URL in the following example begins with a recognized protocol (i.e., https), so the AsciiDoc processor will automatically turn it into a hyperlink.

[source]
----
include::example$url.adoc[tag=base-co]
----
<.> The trailing period will not get caught up in the link.

The URL is also used as the link text.
If you want to use xref:url-macro.adoc#link-text[custom link text], you must use the xref:url-macro.adoc[URL macro].

"#
    );

    // The `base-co` snippet: the trailing period sits outside the link.
    let output =
        convert("The homepage for the Asciidoctor Project is https://www.asciidoctor.org.");
    assert!(output.contains(
        r#"<a href="https://www.asciidoctor.org" class="bare">https://www.asciidoctor.org</a>."#
    ));
}

// Angle brackets around a bare URL are recognized but discarded from the
// output.
#[test]
fn angle_brackets_are_discarded() {
    verifies!(
        r#"
In plain text documents, a bare URL is often enclosed in angle brackets.

[source]
----
You'll often see <https://example.org> used in examples.
----

To accommodate this convention, the AsciiDoc processor will still recognize the URL as an autolink, but will discard the angle brackets in the output (as they are not deemed significant).

"#
    );

    let output = convert("You'll often see <https://example.org> used in examples.");
    assert!(
        output.contains(r#"<a href="https://example.org" class="bare">https://example.org</a>"#)
    );
    // The angle brackets do not appear around the link.
    assert!(!output.contains("&lt;https://example.org"));
    assert!(!output.contains("example.org&gt;"));
}

// Angle brackets mark the URL boundary, so trailing content (here an em dash)
// is not captured into the link.
#[test]
fn angle_brackets_mark_the_boundary() {
    verifies!(
        r#"
The angled brackets are also a convenient way to delinate the boundaries of the URL.
If the bare URL is capturing more characters than it should, you can mark the boundaries explicitly using the angle brackets.

[source]
----
<https://google.com> -- where you find stuff
----

Without the angle brackets delinating the boundaries of the URL, the emdash gets caught up in the URL.
This happens because the replacements substitution is applied before the macros substitution.
So instead of the URL being followed by a space, it's followed directly by the emdash.
The angle brackets ensures it does get included in the URL.

"#
    );

    let output = convert("<https://google.com> -- where you find stuff");
    // The link target ends at `google.com`; the em dash is outside it.
    assert!(output.contains(r#"<a href="https://google.com" class="bare">https://google.com</a>"#));
}

// A URL autolink automatically gets the `bare` role.
#[test]
fn autolink_gets_bare_role() {
    verifies!(
        r#"
Any link created from a bare URL (i.e., an autolink) automatically gets assigned the "bare" role.
This allows the theming system (e.g., CSS) to recognize autolinks (and other bare URLs) and style them distinctly.

"#
    );

    let output = convert("Visit https://example.org now.");
    assert!(output.contains(r#"<a href="https://example.org" class="bare">"#));
}

non_normative!(
    r#"
== Email autolinks

"#
);

// An email address is autolinked to a `mailto:` link.
#[test]
fn email_autolink() {
    verifies!(
        r#"
AsciiDoc also detects and autolinks most email addresses.

[source]
----
include::example$url.adoc[tag=bare-email]
----

"#
    );

    // The `bare-email` snippet.
    let output = convert("Email us at hello@example.com to say hello.");
    assert!(output.contains(r#"<a href="mailto:hello@example.com">hello@example.com</a>"#));
}

// The email-detection constraints (domain-suffix length, permitted symbols) and
// the fallback to the mailto macro are descriptive.
non_normative!(
    r#"
In order for this to work, the domain suffix must be between 2 and 5 characters (e.g., .com) and only common symbols like period (`.`), hyphen (`-`), and plus (`+`) are permitted.
For email address which do not conform to these restriction, you can use the xref:mailto-macro.adoc[email macro].

"#
);

non_normative!(
    r#"
== Escaping URLs and email addresses

"#
);

// A leading backslash prevents autolinking; the backslash is removed and the
// URL or email is shown as plain text.
#[test]
fn escaping_with_a_backslash() {
    verifies!(
        r#"
To prevent automatic linking of a URL or email address, you can add a single backslash (`\`) in front of it.

[source]
----
Once launched, the site will be available at \https://example.org.

If you cannot access the site, email \help@example.org for assistance.
----

The backslash in front of the URL and email address will not appear in the output.
The URL and email address will both be shown in plain text.

"#
    );

    let output = convert(
        "Once launched, the site will be available at \\https://example.org.\n\n\
         If you cannot access the site, email \\help@example.org for assistance.",
    );
    // No links are produced, and the escaping backslash is gone.
    assert!(!output.contains("<a "));
    assert!(output.contains("Once launched, the site will be available at https://example.org."));
    assert!(output.contains("email help@example.org for assistance."));
}

// Turning off the macros substitution (`[subs=-macros]`) also prevents
// autolinking.
#[test]
fn disable_via_incremental_subs() {
    verifies!(
        r#"
Since autolinks are a feature of the xref:subs:macros.adoc[macros substitution], another way to prevent automatic linking of a URL or email address is to turn off the macros substitution using xref:subs:apply-subs-to-blocks.adoc#incremental[incremental subs].

[source]
----
[subs=-macros]
Once launched, the site will be available at https://example.org.
----

"#
    );

    let output = convert(
        "[subs=-macros]\nOnce launched, the site will be available at https://example.org.",
    );
    assert!(!output.contains("<a "));
    assert!(output.contains("https://example.org."));
}

non_normative!(
    r#"
The `subs` attribute is only recognized on a leaf block, such as a paragraph.
"#
);
