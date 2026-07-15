use std::path::{Path, PathBuf};

use clap::Parser as _;

use crate::{resolve_inputs, run, tests::sdd::*, Cli};

track_file!("ref/asciidoctor/docs/modules/cli/pages/process-multiple-files.adoc");

// Asciidoctor's "Process Multiple Source Files" page, tracked from the CLI
// crate. `adoc` mirrors this part of the interface: several input files convert
// in a single invocation, each to its own derived `.html`, and an argument that
// names no existing file is expanded as a portable, Ruby-style glob by `adoc`
// itself — including the `**` double-glob that most shells do not honor. Each
// claim drives `adoc`'s own input resolution (`Cli` + the private
// `resolve_inputs`), and the multi-file conversion is confirmed end to end
// through `run`.
//
// The unquoted-glob passages describe the shell expanding the pattern before
// `adoc` ever sees it, which is outside `adoc`'s control (and, as the page
// notes, not portable); they stay non-normative. The quoted forms `adoc` does
// expand itself are verified.

/// A throwaway on-disk project rooted at a unique temp directory, used to lay
/// out `.adoc` files and match them with glob patterns.
struct TempProject {
    dir: PathBuf,
}

impl TempProject {
    /// Creates a fresh, empty project directory named for `label`.
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "adoc-cli-process-multiple-files-{label}-{}",
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

    /// A glob pattern rooted at the project directory (so tests do not depend
    /// on the process's current directory).
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
/// the file sources (every argument here names or matches a file, never stdin).
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

"#
);

// Several input files convert in one invocation, each to its own output whose
// name is derived by swapping the extension for `.html` — so `adoc a.adoc
// b.adoc` writes `a.html` and `b.html` alongside their inputs.
#[test]
fn multiple_files_each_convert_to_their_own_output() {
    verifies!(
        r#"
The Asciidoctor CLI can convert multiple files in a single invocation.
If you pass multiple source filenames or a filename pattern to the CLI, Asciidoctor will convert each file in turn.

Let's assume there exist two AsciiDoc files in the current directory, [.path]_a.adoc_ and [.path]_b.adoc_.
You can pass both files to Asciidoctor using a single command, as follows:

 $ asciidoctor a.adoc b.adoc

Asciidoctor will convert both files, transforming [.path]_a.adoc_ to [.path]_a.html_ and [.path]_b.adoc_ to [.path]_b.html_.

"#
    );

    let project = TempProject::new("multi");
    let a = project.write("a.adoc", "= A\n\nAlpha.\n");
    let b = project.write("b.adoc", "= B\n\nBravo.\n");

    // Both files pass through a single invocation: `adoc a.adoc b.adoc`.
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
To save some typing, you can use the glob operator (`+*+`) to match all AsciiDoc files in the current directory using a single argument:

 $ asciidoctor *.adoc

Your shell will automatically expand the pattern and interpret the command exactly as you had typed it above:

 $ asciidoctor a.adoc b.adoc

You can pass all AsciiDoc files inside direct subfolders using the glob operator (`+*+`) in place of the directory name:

 $ asciidoctor */*.adoc

To match all files in the current directory and direct subfolders, combine both glob patterns:

 $ asciidoctor *.adoc */*.adoc

"#
);

// Quoting the pattern hands it to `adoc` unexpanded, and `adoc` performs the
// glob matching itself, the same way on every platform: `'*.adoc'` matches the
// current directory and `'*/*.adoc'` matches direct subfolders.
#[test]
fn quoted_globs_are_expanded_by_adoc_portably() {
    verifies!(
        r#"
Since the globs in this command rely on shell expansion, the command is not portable across platforms.
To make it portable, you can allow the Asciidoctor CLI to expand the globs.
To do so, instruct the shell to not expand the glob by quoting the pattern, as shown here:

 $ asciidoctor '*.adoc' '*/*.adoc'

This time, the arguments `+*.adoc+` and `+*/*.adoc+` are passed directly to Asciidoctor instead of being expanded.
Asciidoctor handles the glob matching in a manner that is portable across platforms.

"#
    );

    let project = TempProject::new("quoted");
    let a = project.write("a.adoc", "= A\n");
    let b = project.write("b.adoc", "= B\n");
    let c = project.write("sub/c.adoc", "= C\n");

    // A quoted pattern reaches `adoc` verbatim: the argument is the glob itself,
    // not a shell-expanded list of file names.
    let star = project.pattern("*.adoc");
    let sub_star = project.pattern("*/*.adoc");
    assert_eq!(
        Cli::parse_from(["adoc", &star, &sub_star]).inputs,
        vec![PathBuf::from(&star), PathBuf::from(&sub_star)]
    );

    // `adoc` then expands the patterns itself: `*.adoc` matches the two files in
    // the directory, `*/*.adoc` the one in the direct subfolder.
    assert_eq!(resolve(&[&star]), vec![a.clone(), b.clone()]);
    assert_eq!(resolve(&[&sub_star]), vec![c.clone()]);

    // Combining both patterns in one invocation matches all three.
    assert_eq!(resolve(&[&star, &sub_star]), vec![a, b, c]);
}

// `adoc`'s glob handling follows Ruby's file-globbing rules, so the `**`
// double-glob matches files in the current folder and in subfolders at any
// depth — matching more than most shells expand.
#[test]
fn the_double_glob_matches_any_depth() {
    verifies!(
        r#"
But it gets better.
The glob handling in Asciidoctor (which matches the rules of file globbing in Ruby) is likely more powerful than what your shell offers.
For example, you can match AsciiDoc files in the current folder and in folders of any depth using the double glob operator (`+**+`).

 $ asciidoctor '**/*.adoc'

Most shells do not honor this double glob pattern.

"#
    );

    let project = TempProject::new("double");
    let a = project.write("a.adoc", "= A\n");
    let c = project.write("sub/c.adoc", "= C\n");
    let d = project.write("sub/deep/d.adoc", "= D\n");

    // `**/*.adoc` reaches the current folder (a.adoc) and every depth below it
    // (sub/c.adoc, sub/deep/d.adoc).
    assert_eq!(resolve(&[&project.pattern("**/*.adoc")]), vec![a, c, d]);
}

non_normative!(
    r#"
In conclusion, when specifying a glob pattern, we always recommend enclosing the argument in quotes.
"#
);
