# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.4...asciidoc-html5-v0.1.5)
_02 August 2026_

### Added

* *(html5)* Transcode a non-UTF-8 include file per the encoding attribute ([#138](https://github.com/asciidoc-rs/asciidoc-html5/pull/138)) ([#246](https://github.com/asciidoc-rs/asciidoc-html5/pull/246))
* *(html5)* Copy and link the CodeRay stylesheet when highlighting source ([#247](https://github.com/asciidoc-rs/asciidoc-html5/pull/247))
* *(html5)* Render the cellbgcolor document attribute on table cells ([#245](https://github.com/asciidoc-rs/asciidoc-html5/pull/245))
* Wire document date/time attributes and SOURCE_DATE_EPOCH through adoc ([#248](https://github.com/asciidoc-rs/asciidoc-html5/pull/248))
* *(html5)* Render the cell-local #footnotes block for AsciiDoc cells ([#235](https://github.com/asciidoc-rs/asciidoc-html5/pull/235))

### Documented

* *(html5)* Document that :parse has no counterpart ([#244](https://github.com/asciidoc-rs/asciidoc-html5/pull/244))

### Fixed

* Port the AsciiDoc lists module pages (reference coverage + docs) ([#263](https://github.com/asciidoc-rs/asciidoc-html5/pull/263))
* Port asciidoc-lang directives pages for SDD coverage ([#262](https://github.com/asciidoc-rs/asciidoc-html5/pull/262))
* Port 19 attributes-module spec pages for SDD coverage ([#261](https://github.com/asciidoc-rs/asciidoc-html5/pull/261))
* Port the asciidoc-lang pass module pages and fix related issues ([#260](https://github.com/asciidoc-rs/asciidoc-html5/pull/260))
* *(html5)* Chomp the trailing newline from the copied asciidoctor.css ([#249](https://github.com/asciidoc-rs/asciidoc-html5/pull/249))
* *(html5)* Preserve a list-attached literal paragraph's leading indent ([#168](https://github.com/asciidoc-rs/asciidoc-html5/pull/168)) ([#236](https://github.com/asciidoc-rs/asciidoc-html5/pull/236))
* Close the CLI output-collision TOCTOU window by binding the check to the opened file ([#243](https://github.com/asciidoc-rs/asciidoc-html5/pull/243))

### Other

* *(html5)* Add SDD coverage for five asciidoc-lang ROOT pages ([#258](https://github.com/asciidoc-rs/asciidoc-html5/pull/258))
* *(html5)* Port Asciidoctor api_test.rb for SDD coverage ([#255](https://github.com/asciidoc-rs/asciidoc-html5/pull/255))
* *(html5)* Port Asciidoctor attributes_test.rb (SDD) ([#241](https://github.com/asciidoc-rs/asciidoc-html5/pull/241))

## [0.1.4](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.3...asciidoc-html5-v0.1.4)
_30 July 2026_

### Added

* *(html5)* Distinguish an unreadable include file from a missing one ([#146](https://github.com/asciidoc-rs/asciidoc-html5/pull/146)) ([#234](https://github.com/asciidoc-rs/asciidoc-html5/pull/234))
* *(html5)* Render block images (image:: block macro) ([#224](https://github.com/asciidoc-rs/asciidoc-html5/pull/224))
* *(html5)* Add renderer performance benchmarks with CodSpeed CI ([#228](https://github.com/asciidoc-rs/asciidoc-html5/pull/228))
* *(html5)* Render source blocks with a client-side syntax highlighter ([#225](https://github.com/asciidoc-rs/asciidoc-html5/pull/225))
* *(html5)* Render video and audio blocks (video::, audio::) ([#226](https://github.com/asciidoc-rs/asciidoc-html5/pull/226))
* *(html5)* Render STEM blocks ([stem]/[latexmath]/[asciimath]) ([#222](https://github.com/asciidoc-rs/asciidoc-html5/pull/222))
* *(html5)* Render the table of contents (:toc:), all placements ([#220](https://github.com/asciidoc-rs/asciidoc-html5/pull/220))
* *(html5)* Render AsciiDoc callout lists (colist) ([#214](https://github.com/asciidoc-rs/asciidoc-html5/pull/214))
* *(cli)* Add `--help syntax` topic printing an AsciiDoc crib sheet ([#232](https://github.com/asciidoc-rs/asciidoc-html5/pull/232))
* *(cli)* Print usage at a terminal when no input file is given ([#212](https://github.com/asciidoc-rs/asciidoc-html5/pull/212))
* *(cli)* Add -t/--timings to report conversion time ([#209](https://github.com/asciidoc-rs/asciidoc-html5/pull/209))

### Documented

* *(html5)* Point stale footnote-cell markers at asciidoc-parser#975 ([#210](https://github.com/asciidoc-rs/asciidoc-html5/pull/210))

### Fixed

* *(html5)* Pass a non-numbering ordered-list style through, matching Asciidoctor ([#227](https://github.com/asciidoc-rs/asciidoc-html5/pull/227))
* *(html5)* Render [,lang] source-block shorthand with the <code> wrapper ([#221](https://github.com/asciidoc-rs/asciidoc-html5/pull/221))
* *(html5)* Render an empty inline passthrough paragraph as <p></p> ([#203](https://github.com/asciidoc-rs/asciidoc-html5/pull/203))

### Other

* *(html5)* Verify example block caption and counter model ([#113](https://github.com/asciidoc-rs/asciidoc-html5/pull/113)) ([#219](https://github.com/asciidoc-rs/asciidoc-html5/pull/219))
* *(html5)* Verify the remote include:: link fallback under a non-secure safe mode ([#136](https://github.com/asciidoc-rs/asciidoc-html5/pull/136)) ([#218](https://github.com/asciidoc-rs/asciidoc-html5/pull/218))
* *(html5)* Cover delimited source block <code> wrapper ([#159](https://github.com/asciidoc-rs/asciidoc-html5/pull/159)) ([#208](https://github.com/asciidoc-rs/asciidoc-html5/pull/208))
* *(cli)* Port Asciidoctor options_test.rb ([#233](https://github.com/asciidoc-rs/asciidoc-html5/pull/233))

## [0.1.3](https://github.com/asciidoc-rs/asciidoc-html5/compare/asciidoc-html5-v0.1.2...asciidoc-html5-v0.1.3)
_26 July 2026_

### Added

* *(html5)* Render section-heading anchors for sectanchors and sectlinks ([#197](https://github.com/asciidoc-rs/asciidoc-html5/pull/197))
* *(html5)* Assign document-title id and roles to standalone <body> ([#196](https://github.com/asciidoc-rs/asciidoc-html5/pull/196))
* *(html5)* Render description lists (dlist) ([#192](https://github.com/asciidoc-rs/asciidoc-html5/pull/192))
* *(cli)* Surface parser warnings to stderr with -q/-v/-w/--failure-level ([#185](https://github.com/asciidoc-rs/asciidoc-html5/pull/185))
* *(html5)* Render collapsible example blocks and port the spec page ([#173](https://github.com/asciidoc-rs/asciidoc-html5/pull/173))
* *(html5)* Render unordered and ordered lists ([#158](https://github.com/asciidoc-rs/asciidoc-html5/pull/158))
* *(html5)* Caption titled listing blocks via listing-caption ([#172](https://github.com/asciidoc-rs/asciidoc-html5/pull/172))
* *(html5)* Render AsciiDoc tables ([#165](https://github.com/asciidoc-rs/asciidoc-html5/pull/165))
* *(html5)* Normalize verbatim block content and render passthrough blocks ([#153](https://github.com/asciidoc-rs/asciidoc-html5/pull/153))
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
* *(cli)* Add -n/--section-numbers to set the sectnums attribute ([#195](https://github.com/asciidoc-rs/asciidoc-html5/pull/195))
* *(cli)* Accept -d/--doctype for compatibility, error on non-article ([#190](https://github.com/asciidoc-rs/asciidoc-html5/pull/190))
* *(cli)* Accept -b/--backend for compatibility, error on non-html5 ([#184](https://github.com/asciidoc-rs/asciidoc-html5/pull/184))
* *(cli)* Add -R/--source-dir to preserve input structure under -D ([#182](https://github.com/asciidoc-rs/asciidoc-html5/pull/182))
* *(cli)* Convert multiple files and expand globs in one invocation ([#83](https://github.com/asciidoc-rs/asciidoc-html5/pull/83))

### Documented

* *(html5)* Clean up dangling "tables not rendered" references ([#174](https://github.com/asciidoc-rs/asciidoc-html5/pull/174))
* *(html5)* Mark compat-mode as permanently out of scope (won't-do) ([#171](https://github.com/asciidoc-rs/asciidoc-html5/pull/171))
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

* *(html5)* Port Asciidoctor's substitutions_test.rb ([#199](https://github.com/asciidoc-rs/asciidoc-html5/pull/199))
* *(html5)* Port sections_test.rb and render section numbers/appendix captions ([#194](https://github.com/asciidoc-rs/asciidoc-html5/pull/194))
* *(html5)* Verify ulist/olist assertions inside AsciiDoc table cells ([#161](https://github.com/asciidoc-rs/asciidoc-html5/pull/161)) ([#181](https://github.com/asciidoc-rs/asciidoc-html5/pull/181))
* *(html5)* Port helpers_test.rb Ruby test suite ([#180](https://github.com/asciidoc-rs/asciidoc-html5/pull/180))
* *(html5)* Cover passthrough blocks and raw-block blank-line stripping ([#118](https://github.com/asciidoc-rs/asciidoc-html5/pull/118)) ([#176](https://github.com/asciidoc-rs/asciidoc-html5/pull/176))
* *(html5)* Verify utf8 encoding html-backend tests now that lists render ([#175](https://github.com/asciidoc-rs/asciidoc-html5/pull/175))
* *(html5)* Port reader_test.rb, verifying document-visible preprocessor behavior ([#135](https://github.com/asciidoc-rs/asciidoc-html5/pull/135))
* *(html5)* Verify verse escaped-brace subs after asciidoc-parser 0.27.1 ([#143](https://github.com/asciidoc-rs/asciidoc-html5/pull/143))
* *(html5)* Verify leading-period block title now that parser recognizes it ([#144](https://github.com/asciidoc-rs/asciidoc-html5/pull/144))
* *(html5)* Port links_test.rb ([#123](https://github.com/asciidoc-rs/asciidoc-html5/pull/123))
* *(html5)* Port the front half of blocks_test.rb ([#122](https://github.com/asciidoc-rs/asciidoc-html5/pull/122))
* *(html5)* Port text_test.rb inline-substitution suite ([#119](https://github.com/asciidoc-rs/asciidoc-html5/pull/119))
* *(html5)* Track paths_test.rb as non-normative ([#109](https://github.com/asciidoc-rs/asciidoc-html5/pull/109))
* *(html5)* Track parser-internal and out-of-scope Asciidoctor suites as non-normative ([#108](https://github.com/asciidoc-rs/asciidoc-html5/pull/108))
* *(html5)* Track the migrate module pages as non-normative ([#98](https://github.com/asciidoc-rs/asciidoc-html5/pull/98))
* *(html5)* Add HTML-output assertion harness and port preamble_test.rb ([#85](https://github.com/asciidoc-rs/asciidoc-html5/pull/85))
* *(cli)* Verify -b html5 backend selection in invoker port ([#193](https://github.com/asciidoc-rs/asciidoc-html5/pull/193))
* *(cli)* Port invoker_test.rb ([#145](https://github.com/asciidoc-rs/asciidoc-html5/pull/145))
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
