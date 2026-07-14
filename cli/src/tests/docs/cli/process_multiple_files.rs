use std::path::{Path, PathBuf};

use clap::Parser as _;

use crate::{resolve_inputs, run, tests::sdd::*, Cli};

track_file!("docs/modules/cli/pages/process-multiple-files.adoc");

// This crate's "Process Multiple Source Files" page. It documents how `adoc`
// converts several files in one invocation — each to its own derived `.html` —
// and how it expands a quoted glob pattern itself, portably and with Ruby-style
// rules including the `**` double-glob. Each invocation is verified through
// `adoc`'s own input resolution (`Cli` + the private `resolve_inputs`) and, for
// the multi-file conversion, end to end through `run`.
//
// The unquoted-glob section describes the shell expanding the pattern before
// `adoc` sees it, which is outside `adoc`'s control; it stays non-normative.

/// A throwaway on-disk project rooted at a unique temp directory, used to lay
/// out `.adoc` files and match them with glob patterns.
struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Creates a fresh, empty project directory named for `label`.
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "adoc-docs-process-multiple-files-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create project dir");
        Self { dir }
    }

    /// The absolute path of `relative` within the project.
    fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }

    /// Writes `contents` to `relative`, creating any parent directories.
    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().expect("has parent")).expect("create parent dir");
        std::fs::write(&path, contents).expect("write file");
        path
    }

    /// A glob pattern (or file name) rooted at the project directory, so tests
    /// do not depend on the process's current directory.
    fn pattern(&self, glob: &str) -> String {
        self.path(glob).to_str().expect("path is UTF-8").to_string()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The files `adoc` resolves the given input arguments to, in order — driving
/// the full `Cli` parse plus the private `resolve_inputs`, then keeping only
/// the file sources.
fn resolve(args: &[&str]) -> Vec<PathBuf> {
    let mut full: Vec<&str> = vec!["adoc"];
    full.extend_from_slice(args);
    let cli = Cli::parse_from(full);
    resolve_inputs(&cli.inputs)
        .expect("resolve inputs")
        .iter()
        .filter_map(|source| source.file().map(Path::to_path_buf))
        .collect()
}

non_normative!(
    r#"
= Process Multiple Source Files
:navtitle: Process Multiple Files
:description: How to convert several AsciiDoc files in one adoc invocation, and how adoc expands quoted glob patterns portably.

The `adoc` command can convert several AsciiDoc files in a single invocation. Pass
more than one source file, or a glob pattern, and `adoc` converts each file in
turn.

[NOTE]
====
The prose on this page is non-normative documentation. The `adoc` invocations it
shows are normative: they are verified against the implementation, so the
documented behavior is guaranteed.
====

"#
);

// Several files convert in one invocation, each to its own output whose name is
// derived by swapping the extension for `.html`, written alongside its input.
#[test]
fn several_files_each_convert_to_their_own_output() {
    verifies!(
        r#"
== Convert several files at once

Suppose the current directory holds two AsciiDoc files, [.path]_a.adoc_ and
[.path]_b.adoc_. Pass both to `adoc` in one command:

 $ adoc a.adoc b.adoc

`adoc` converts each in turn, writing [.path]_a.adoc_ to [.path]_a.html_ and
[.path]_b.adoc_ to [.path]_b.html_. As when converting a single file, each output
name is derived from its input by swapping the extension for `.html`, and the
file is written alongside its input.

"#
    );

    let project = TempProject::new("multi");
    let a = project.write("a.adoc", "= A\n\nAlpha.\n");
    let b = project.write("b.adoc", "= B\n\nBravo.\n");

    let a_str = a.to_str().expect("path is UTF-8");
    let b_str = b.to_str().expect("path is UTF-8");
    let cli = Cli::parse_from(["adoc", a_str, b_str]);
    let mut stdout = Vec::new();
    run(&cli, &mut stdout).expect("adoc converts");

    // Nothing goes to standard output; each file lands in its own derived
    // `.html` next to the input.
    assert!(stdout.is_empty(), "adoc wrote to stdout instead of files");
    let a_html = std::fs::read_to_string(project.path("a.html")).expect("read a.html");
    let b_html = std::fs::read_to_string(project.path("b.html")).expect("read b.html");
    assert!(a_html.contains("<title>A</title>"));
    assert!(a_html.contains("<p>Alpha.</p>"));
    assert!(b_html.contains("<title>B</title>"));
    assert!(b_html.contains("<p>Bravo.</p>"));
}

non_normative!(
    r#"
== Match files with a glob pattern

To save typing, use the glob operator (`*`) to match every AsciiDoc file in the
current directory with a single argument:

 $ adoc *.adoc

Written this way, your shell expands the pattern before `adoc` runs, so the
command is really just the `adoc a.adoc b.adoc` from above. Shell globbing varies
from platform to platform, though, and most shells expand `*` only within a
single directory.

"#
);

// Quoting the pattern hands it to `adoc` unexpanded, and `adoc` performs the
// matching itself with portable, Ruby-style rules: `'*.adoc'` matches the
// current directory, `'*/*.adoc'` direct subfolders, and `'**/*.adoc'` the
// current folder plus subfolders at any depth.
#[test]
fn adoc_expands_quoted_globs_portably() {
    verifies!(
        r#"
== Let adoc expand the glob

To make the command portable, let `adoc` expand the glob itself. Quote the
pattern so the shell passes it through untouched:

 $ adoc '*.adoc'

`adoc` then performs the matching with the same portable, Ruby-style rules on
every platform. Two more patterns follow from that:

 $ adoc '*/*.adoc'

matches AsciiDoc files in direct subfolders, and the double-glob operator (`**`)
matches the current folder and subfolders at any depth -- something most shells
will not expand for you:

 $ adoc '**/*.adoc'

We always recommend quoting a glob pattern, so `adoc` expands it the same way
everywhere.

"#
    );

    let project = TempProject::new("globs");
    let a = project.write("a.adoc", "= A\n");
    let b = project.write("b.adoc", "= B\n");
    let c = project.write("sub/c.adoc", "= C\n");
    let d = project.write("sub/deep/d.adoc", "= D\n");

    // A quoted pattern reaches `adoc` verbatim, not as a shell-expanded list.
    let star = project.pattern("*.adoc");
    assert_eq!(
        Cli::parse_from(["adoc", &star]).inputs,
        vec![PathBuf::from(&star)]
    );

    // `*.adoc` matches the current directory only; `*/*.adoc` direct subfolders
    // only; `**/*.adoc` the current folder and every depth below it.
    assert_eq!(resolve(&[&star]), vec![a.clone(), b.clone()]);
    assert_eq!(resolve(&[&project.pattern("*/*.adoc")]), vec![c.clone()]);
    assert_eq!(resolve(&[&project.pattern("**/*.adoc")]), vec![a, b, c, d]);
}

// `adoc` globs an argument only when it names no existing file; a pattern (or
// name) that matches nothing is kept as-is and surfaces as a missing-file
// error.
#[test]
fn a_pattern_matching_nothing_is_a_missing_file() {
    verifies!(
        r#"
[NOTE]
.Known limitations
====
An argument is treated as a glob pattern only when it does not name an existing
file, mirroring Asciidoctor. A pattern that matches nothing is left as-is, so it
surfaces as a missing-file error, exactly as a misspelled filename would.
====

"#
    );

    let project = TempProject::new("missing");

    // An argument that names an existing file resolves to that file directly.
    let real = project.write("real.adoc", "= Real\n");
    assert_eq!(resolve(&[real.to_str().expect("UTF-8")]), vec![real]);

    // A pattern that matches nothing is kept verbatim...
    let nomatch = project.pattern("no-such-*.adoc");
    assert_eq!(resolve(&[&nomatch]), vec![PathBuf::from(&nomatch)]);

    // ...and converting it fails, exactly as a missing plain filename would.
    let cli = Cli::parse_from(["adoc", &nomatch]);
    let mut stdout = Vec::new();
    assert!(
        run(&cli, &mut stdout).is_err(),
        "adoc should fail on a pattern that matches no file"
    );
}

non_normative!(
    r#"
You can also set the xref:cli:set-safe-mode.adoc[safe mode from the CLI], which
governs how far each converted document may reach when resolving includes.
"#
);
