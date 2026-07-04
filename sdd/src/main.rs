// Quick and dirty tool to generate spec coverage for asciidoc-html5. Not
// intended at this time to generalize to any other use case.

// This is adapted from the equivalent tool in asciidoc-parser. Two differences:
//
//   * This workspace has two crates that may verify parts of the AsciiDoc
//     language specification (`asciidoc-html5` and the `adoc` CLI), so we scan
//     the test modules of both.
//   * In addition to the AsciiDoc language description (`.adoc`), we treat the
//     Asciidoctor reference test suite (`.rb`) as spec sources to cover, since
//     this renderer is validated against Asciidoctor's own behavior.
//
// For now, please excuse the hard-coded settings and other shortcuts taken.

use std::{collections::HashMap, fs, io::BufRead, path::Path};

use walkdir::{DirEntry, WalkDir};

// Test module roots scanned for spec markers, one per workspace crate. A root
// that doesn't exist yet is simply skipped, so this produces empty (zero) spec
// coverage until tests with spec markers are added.
const TEST_ROOTS: &[&str] = &["../html5/src/tests", "../cli/src/tests"];

// Spec sources whose lines we measure coverage against, as `(root, extension)`
// pairs: the AsciiDoc language description (`.adoc`) and the Asciidoctor
// reference test suite (`.rb`) this renderer is validated against.
const SPEC_SOURCES: &[(&str, &str)] = &[
    ("../ref/asciidoc-lang/docs/modules", ".adoc"),
    ("../ref/asciidoctor", ".rb"),
];

fn main() {
    let mut spec_coverage: HashMap<String, Vec<(String, bool)>> = HashMap::new();

    for root in TEST_ROOTS {
        for entry in collect_files(root, ".rs") {
            let path = entry.path();
            if let Some((spec_path, cov)) = parse_rs_file(path) {
                spec_coverage.insert(spec_path, cov);
            }
        }
    }

    println!("{{\n    \"coverage\": {{");

    let mut spec_files: Vec<DirEntry> = vec![];
    for (root, extension) in SPEC_SOURCES {
        spec_files.extend(collect_files(root, extension));
    }

    let last_index = spec_files.len() - 1;

    for (count, entry) in spec_files.into_iter().enumerate() {
        let path = entry.path().to_str().unwrap().trim_start_matches("../");
        // (unwrap: Should have been filtered out above.)

        // if !path.contains("/revision-line.adoc") {
        //     continue;
        // }
        println!("        {path:?}: {{");

        emit_coverage(path, spec_coverage.get(path));

        if count < last_index {
            println!("        }},");
        } else {
            println!("        }}");
        }
    }

    println!("    }}\n}}");
}

// Collect every file ending in `extension` under `root`, skipping dotfiles.
// Returns an empty vec if the root doesn't exist, so crates without a test
// module yet simply contribute no coverage.
fn collect_files(root: &str, extension: &str) -> Vec<DirEntry> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if let Some(file_name) = e.file_name().to_str() {
                !file_name.starts_with('.')
            } else {
                false
            }
        })
        .filter_map(|e| {
            let e = e.ok()?;

            if !e.file_type().is_file() {
                return None;
            }

            if let Some(file_name) = e.file_name().to_str()
                && file_name.ends_with(extension)
            {
                Some(e)
            } else {
                None
            }
        })
        .collect()
}

fn parse_rs_file(path: &Path) -> Option<(String, Vec<(String, bool)>)> {
    // if !path.ends_with("revision_line.rs") {
    //     return None;
    // }

    let rs_file = fs::read(path).unwrap();

    let mut tracked_file: Option<String> = None;
    let mut lines: Vec<(String, bool)> = vec![];
    let mut in_non_normative_block = false;
    let mut in_verifies_block = false;
    let mut expect_track_file_path = false;

    for line in rs_file.lines() {
        let line = line.unwrap();

        // Single-line form: `track_file!("path");`.
        if let Some(tf) = line.strip_prefix("track_file!(\"")
            && let Some(tf) = tf.strip_suffix("\");")
        {
            if tracked_file.is_some() {
                panic!("ERROR: {path:?} contains multiple track_file! macros");
            }
            tracked_file = Some(tf.to_string());
            continue;
        }

        // Multi-line form (produced by rustfmt when the path is too long to fit
        // on one line):
        //
        //     track_file!(
        //         "path"
        //     );
        if line.trim_end() == "track_file!(" {
            expect_track_file_path = true;
            continue;
        }

        if expect_track_file_path {
            expect_track_file_path = false;
            if let Some(tf) = line.trim().strip_prefix('"')
                && let Some(tf) = tf.strip_suffix('"')
            {
                if tracked_file.is_some() {
                    panic!("ERROR: {path:?} contains multiple track_file! macros");
                }
                tracked_file = Some(tf.to_string());
            }
            continue;
        }

        if line.contains("non_normative!(") {
            // println!("NN+");
            in_non_normative_block = true;
            in_verifies_block = false;
            continue;
        }

        if line.contains("verifies!(") {
            // println!("VF+");
            in_non_normative_block = false;
            in_verifies_block = true;
            continue;
        }

        if line.starts_with("\"#") {
            // println!("QQQ");
            in_non_normative_block = false;
            in_verifies_block = false;
            continue;
        }

        if line.ends_with("r#\"") || line.ends_with("r##\"") {
            // println!("<<<");
            continue;
        }

        if in_non_normative_block {
            // println!("NN  {line}");
            lines.push((line, false));
        } else if in_verifies_block {
            // println!("VF  {line}");
            lines.push((line, true));
        } else {
            // println!("--  {line}");
        }
    }

    tracked_file.map(|tracked_file| (tracked_file, lines))
}

fn emit_coverage(path: &str, coverage: Option<&Vec<(String, bool)>>) {
    // if !path.contains("/id.adoc") {
    //     return;
    // }

    let path = format!("../{path}");
    let adoc_file = fs::read(path).unwrap();

    let empty_coverage: Vec<(String, bool)> = vec![];
    let coverage = if let Some(coverage) = coverage.as_ref() {
        coverage
    } else {
        &empty_coverage
    };

    let mut coverage_lines = coverage.iter();

    let mut output_lines: Vec<String> = vec![];

    for (count, line) in adoc_file.lines().enumerate() {
        let line = line.unwrap();
        let count = count + 1;

        // println!("\n\n{count:4}: {line}");

        let coverage_line = coverage_lines.next();

        if line.is_empty() {
            continue;
        }

        if let Some((cov_line, is_normative)) = coverage_line {
            // println!("      {cov_line}");
            if cov_line == &line && *is_normative {
                output_lines.push(format!("            \"{count}\": 1"));
            }
        } else {
            output_lines.push(format!("            \"{count}\": 0"));
        }
    }

    if output_lines.is_empty() {
        return;
    }

    let last_output_line_index = output_lines.len() - 1;

    for (count, line) in output_lines.iter().enumerate() {
        if count < last_output_line_index {
            println!("{line},");
        } else {
            println!("{line}");
        }
    }
}
