# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.2...asciidoc-html5-v0.1.3)
_17 July 2026_

### Added

* Render sidebar and example blocks and the inline doctype ([#101](https://github.com/asciidoc-rs/asciidoc-html5/pull/101))
* *(html5)* Render source/open/quote/verse/admonition blocks and port paragraphs_test.rb ([#93](https://github.com/asciidoc-rs/asciidoc-html5/pull/93))
* Honor an explicit docdir attribute as a piped include base directory ([#90](https://github.com/asciidoc-rs/asciidoc-html5/pull/90))
* Honor notitle as the inverse of showtitle for the embedded title ([#88](https://github.com/asciidoc-rs/asciidoc-html5/pull/88))
* Match Asciidoctor's embedded/standalone output defaults and add -e/--embedded ([#76](https://github.com/asciidoc-rs/asciidoc-html5/pull/76))
* Add convert_outline API and port the generate-html-toc page ([#72](https://github.com/asciidoc-rs/asciidoc-html5/pull/72))
* *(html5)* Honor the safe mode for the doctype attribute ([#67](https://github.com/asciidoc-rs/asciidoc-html5/pull/67))
* Implement the docfile/docdir/docname/docfilesuffix attributes ([#65](https://github.com/asciidoc-rs/asciidoc-html5/pull/65))
* Add native load/load_file API and port the convert-files page ([#64](https://github.com/asciidoc-rs/asciidoc-html5/pull/64))
* *(html5)* Honor the safe mode for the backend attribute ([#63](https://github.com/asciidoc-rs/asciidoc-html5/pull/63))
* *(html5)* Implement copycss stylesheet copying ([#57](https://github.com/asciidoc-rs/asciidoc-html5/pull/57))
* *(cli)* Convert multiple files and expand globs in one invocation ([#83](https://github.com/asciidoc-rs/asciidoc-html5/pull/83))

### Documented

* Port the API Options page ([#100](https://github.com/asciidoc-rs/asciidoc-html5/pull/100))
* Port the sourcemap API page ([#81](https://github.com/asciidoc-rs/asciidoc-html5/pull/81))
* Port the reference-safe-mode page ([#84](https://github.com/asciidoc-rs/asciidoc-html5/pull/84))
* *(safe-modes)* Drop docinfo from the not-yet-surfaced list ([#80](https://github.com/asciidoc-rs/asciidoc-html5/pull/80))
* Port the docinfo relocation stub and add a Docinfo Files page ([#78](https://github.com/asciidoc-rs/asciidoc-html5/pull/78))
* Port the find-blocks API page ([#75](https://github.com/asciidoc-rs/asciidoc-html5/pull/75))
* Port the convert-strings API page ([#69](https://github.com/asciidoc-rs/asciidoc-html5/pull/69))
* Port the CLI Options page ([#92](https://github.com/asciidoc-rs/asciidoc-html5/pull/92))
* Port the output-file CLI page and add -D/--destination-dir ([#82](https://github.com/asciidoc-rs/asciidoc-html5/pull/82))
* Port the io-piping CLI page ([#70](https://github.com/asciidoc-rs/asciidoc-html5/pull/70))

### Other

* *(html5)* Track the migrate module pages as non-normative ([#98](https://github.com/asciidoc-rs/asciidoc-html5/pull/98))
* *(html5)* Add HTML-output assertion harness and port preamble_test.rb ([#85](https://github.com/asciidoc-rs/asciidoc-html5/pull/85))
* *(cli)* Track the asciidoctor(1) man page as non-normative ([#91](https://github.com/asciidoc-rs/asciidoc-html5/pull/91))

### Updated dependencies

* *(deps)* Bump asciidoc-parser from 0.19.2 to 0.20.0 ([#62](https://github.com/asciidoc-rs/asciidoc-html5/pull/62))

## [0.1.2](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.1...asciidoc-html5-v0.1.2)
_11 July 2026_

### Added

* *(html5)* Support linking and embedding custom stylesheets ([#53](https://github.com/asciidoc-rs/asciidoc-html5/pull/53))
* Support docinfo files (head/header/footer injection) ([#55](https://github.com/asciidoc-rs/asciidoc-html5/pull/55))
* Resolve includes and add the `-B`/`--base-dir` CLI option ([#54](https://github.com/asciidoc-rs/asciidoc-html5/pull/54))
* Implement safe mode and gate default-stylesheet embedding ([#43](https://github.com/asciidoc-rs/asciidoc-html5/pull/43))
* Pass document attributes into the API and CLI ([#41](https://github.com/asciidoc-rs/asciidoc-html5/pull/41))
* *(html5)* Embed Asciidoctor's default stylesheet and web fonts ([#35](https://github.com/asciidoc-rs/asciidoc-html5/pull/35))
* *(html5)* Sketch baseline renderer architecture ([#17](https://github.com/asciidoc-rs/asciidoc-html5/pull/17))

### Documented

* Port the Asciidoctor HTML backend page ([#34](https://github.com/asciidoc-rs/asciidoc-html5/pull/34))
* Track Asciidoctor tooling index page as non-normative ([#33](https://github.com/asciidoc-rs/asciidoc-html5/pull/33))
* Port the Asciidoctor API index page ([#28](https://github.com/asciidoc-rs/asciidoc-html5/pull/28))
* Port the Asciidoctor get-started page ([#25](https://github.com/asciidoc-rs/asciidoc-html5/pull/25))
* Add crate introduction page and verify baseline conversion ([#21](https://github.com/asciidoc-rs/asciidoc-html5/pull/21))
* Port the Asciidoctor CLI overview page ([#29](https://github.com/asciidoc-rs/asciidoc-html5/pull/29))

## [0.1.1](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.0...asciidoc-html5-v0.1.1)
_04 July 2026_

### Added

* Add placeholder READMEs for the html5 and cli crates

## [0.1.0](https://github.com/asciidoc-rs/asciidoc-html5/releases/tag/asciidoc-html5-v0.1.0)
_04 July 2026_

### Added

* Initial placeholder projects for HTML5 renderer and CLI wrapper
