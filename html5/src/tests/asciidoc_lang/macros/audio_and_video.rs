//! Coverage of the AsciiDoc language description's *Audio and Video* page.
//!
//! The block `audio::` and `video::` macros embed self-hosted media as HTML5
//! `<audio>`/`<video>` elements (with `start`/`end` time fragments, autoplay,
//! and a title caption), while a `video::` with a `vimeo`/`youtube` service
//! renders an `<iframe>` embed (including YouTube `list`/`playlist` playlists,
//! specified via attributes or in the target). All verified through `convert`,
//! matching Asciidoctor 2.0.26.
//!
//! The examples are pulled in with `include::example$audio.adoc` /
//! `example$video.adoc`; the tests reproduce those tagged snippets inline. The
//! two attribute reference tables are non-normative.

use crate::{convert, tests::sdd::*};

track_file!("ref/asciidoc-lang/docs/modules/macros/pages/audio-and-video.adoc");

non_normative!(
    r#"
= Audio and Video
:url-video-element: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video
:url-audio-element: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio
:url-media-formats: https://developer.mozilla.org/en-US/docs/Web/HTML/Supported_media_formats#Browser_compatibility

== Audio macro syntax

The block audio macro enables you to embed audio streams into your documentation.
You can embed self-hosted audio files that are supported by the browser.

The audio formats AsciiDoc supports is dictated by the output format, such as the formats supported by the browser when generating HTML.
While this was once a precarious ordeal, HTML 5 has brought sanity to audio support in the browser by adding a dedicated {url-audio-element}[`<audio>`^] element and by introducing several standard audio formats.
Those formats are now widely supported across browsers and systems.

For a canonical list of supported web audio formats and their interaction with modern browsers, see the {url-media-formats}[Mozilla Developer Supported Media Formats^] documentation.

"#
);

// A basic audio macro embeds an `<audio>` element (controls enabled by
// default).
#[test]
fn audio_basic() {
    verifies!(
        r#"
.Basic audio file include
----
include::example$audio.adoc[tag=basic]
----

"#
    );

    let output = convert("audio::ocean-waves.wav[]");
    assert!(output.contains(r#"<div class="audioblock">"#));
    assert!(output.contains(r#"<audio src="ocean-waves.wav" controls>"#));
}

// The `start` attribute becomes a `#t=` media fragment, and `opts=autoplay`
// adds the `autoplay` boolean attribute.
#[test]
fn audio_attributes() {
    verifies!(
        r#"
You can control the audio settings using additional attributes on the macro.
For instance, you can offset the start time of playback using the `start` attribute and enable autoplay using the `autoplay` option.

.Set attributes for local audio playback
----
include::example$audio.adoc[tag=attrs]
----
"#
    );

    assert!(convert("audio::ocean-waves.wav[start=60,opts=autoplay]")
        .contains(r#"<audio src="ocean-waves.wav#t=60" autoplay controls>"#));
}

// A block title on the audio macro renders a caption above it.
#[test]
fn audio_caption() {
    verifies!(
        r#"

You can include a caption above the audio using the title attribute.

.Add a caption to the audio
[source]
----
include::example$audio.adoc[tag=caption]
----

"#
    );

    let output = convert(".Take a zen moment\naudio::ocean-waves.wav[]");
    assert!(output.contains(r#"<div class="title">Take a zen moment</div>"#));
    assert!(output.contains(r#"<audio src="ocean-waves.wav" controls>"#));
}

non_normative!(
    r#"
== Video macro syntax

The block video macro enables you to embed videos into your documentation.
You can embed self-hosted videos or videos shared on popular video hosting sites such as Vimeo and YouTube.

The video formats AsciiDoc supports is dictated by the output format, such as the formats supported by the browser when generating HTML.
While this was once a precarious ordeal, HTML 5 has brought sanity to video support in the browser by adding a dedicated {url-video-element}[`<video>`^] element and by introducing several standard video formats.
Those formats are now widely supported across browsers and systems.

For a canonical list of supported web video formats and their interaction with modern browsers, see the {url-media-formats}[Mozilla Developer Supported Media Formats^] documentation.

.A recommendation for serving video to browsers
****
Where appropriate, we recommend using a video hosting service like Vimeo or YouTube to serve videos in online documentation.
These services specialize in streaming optimized video to the browser, with the lowest latency possible given hardware, software, and network capabilities of the device viewing the video.

Vimeo even offers a white label mode so users aren't made aware that the video is being served through its service.

See <<Vimeo and YouTube videos>> for details about how to serve videos from these services.
****

"#
);

// A basic video macro embeds a `<video>` element (controls enabled by default).
#[test]
fn video_base() {
    verifies!(
        r#"
.Basic video file include
[source]
----
include::example$video.adoc[tag=base]
----

"#
    );

    let output = convert("video::video-file.mp4[]");
    assert!(output.contains(r#"<div class="videoblock">"#));
    assert!(output.contains(r#"<video src="video-file.mp4" controls>"#));
}

// The `width`, `start`, and autoplay attributes apply to the `<video>` element.
#[test]
fn video_attributes() {
    verifies!(
        r#"
You can control the video settings using additional attributes on the macro.
For instance, you can offset the start time of playback using the `start` attribute and enable autoplay using the `autoplay` option.

.Set attributes for local video playback
[source]
----
include::example$video.adoc[tag=attr]
----

"#
    );

    assert!(
        convert("video::video-file.mp4[width=640,start=60,opts=autoplay]")
            .contains(r#"<video src="video-file.mp4#t=60" width="640" autoplay controls>"#)
    );
}

// A block title on the video macro renders a caption.
#[test]
fn video_caption() {
    verifies!(
        r#"
You can include a caption on the video using the title attribute.

.Add a caption to a video
[source]
----
include::example$video.adoc[tag=caption]
----

"#
    );

    let output = convert(".A walkthrough of the product\nvideo::video-file.mp4[]");
    assert!(output.contains(r#"<div class="title">A walkthrough of the product</div>"#));
    assert!(output.contains(r#"<video src="video-file.mp4" controls>"#));
}

non_normative!(
    r#"
=== Vimeo and YouTube videos

The video macro supports embedding videos from external video hosting services like Vimeo and YouTube.
The AsciiDoc processor, specifically the converter, automatically generates the correct code to embed the video in the HTML output.

IMPORTANT: In order for an embedded YouTube video to work in Firefox when viewing the generated HTML document through the file: protocol, you must set `security.fileuri.strict_origin_policy` on the about:config settings page to `false`.

To use this feature, put the video ID in the macro target and the name of the hosting service in the first positional attribute.

"#
);

// A `vimeo` or `youtube` service renders an `<iframe>` embed for the video ID.
#[test]
fn vimeo_and_youtube() {
    verifies!(
        r#"
.Embed a Vimeo video
[source]
----
include::example$video.adoc[tag=vimeo]
----

.Embed a YouTube video
[source]
----
include::example$video.adoc[tag=youtube]
----

"#
    );

    assert!(convert("video::67480300[vimeo]").contains(
        r#"<iframe src="https://player.vimeo.com/video/67480300" frameborder="0" allowfullscreen>"#
    ));

    assert!(convert("video::RvRhUHTV_8k[youtube]").contains(
        r#"<iframe src="https://www.youtube.com/embed/RvRhUHTV_8k?rel=0" frameborder="0" allowfullscreen>"#
    ));
}

// The `list` attribute associates a YouTube playlist with the video.
#[test]
fn youtube_with_list() {
    verifies!(
        r#"
When embedding a YouTube video, you can specify a playlist to associate with the video using the `list` attribute.
The playlist must be specified by its ID.

.Embed a YouTube video with a playlist
[source]
----
include::example$video.adoc[tag=youtube-with-list]
----

"#
    );

    assert!(
        convert("video::RvRhUHTV_8k[youtube,list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP]").contains(
            r#"<iframe src="https://www.youtube.com/embed/RvRhUHTV_8k?rel=0&amp;list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP" frameborder="0" allowfullscreen>"#
        )
    );
}

// The playlist ID can instead follow the video ID in the target, separated by a
// slash.
#[test]
fn youtube_with_list_in_target() {
    verifies!(
        r#"
Instead of using the `list` attribute, you can specify the ID of the playlist after the video ID in the target, separated by a slash.

.Embed a YouTube video with a playlist in the target
[source]
----
include::example$video.adoc[tag=youtube-with-list-in-target]
----

"#
    );

    assert!(
        convert("video::RvRhUHTV_8k/PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP[youtube]").contains(
            r#"<iframe src="https://www.youtube.com/embed/RvRhUHTV_8k?rel=0&amp;list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP" frameborder="0" allowfullscreen>"#
        )
    );
}

// The `playlist` attribute builds a dynamic, unnamed playlist from extra IDs.
#[test]
fn youtube_with_playlist() {
    verifies!(
        r#"
Alternatively, you can create a dynamic, unnamed playlist by listing several additional video IDs in the `playlist` attribute.

.Embed a YouTube video with a dynamic playlist
[source]
----
include::example$video.adoc[tag=youtube-with-playlist]
----

"#
    );

    assert!(
        convert(r#"video::RvRhUHTV_8k[youtube,playlist="_SvwdK_HibQ,SGqg_ZzThDU"]"#).contains(
            r#"<iframe src="https://www.youtube.com/embed/RvRhUHTV_8k?rel=0&amp;playlist=RvRhUHTV_8k,_SvwdK_HibQ,SGqg_ZzThDU" frameborder="0" allowfullscreen>"#
        )
    );
}

// The dynamic playlist IDs can instead be listed in the target after the video
// ID, separated by commas.
#[test]
fn youtube_with_playlist_in_target() {
    verifies!(
        r#"
Instead of using the `playlist` attribute, you can create a dynamic, unnamed playlist by listing several video IDs in the target separated by a comma.

.Embed a YouTube video with a dynamic playlist in the target
[source]
----
include::example$video.adoc[tag=youtube-with-playlist-in-target]
----

"#
    );

    assert!(
        convert("video::RvRhUHTV_8k,_SvwdK_HibQ,SGqg_ZzThDU[youtube]").contains(
            r#"<iframe src="https://www.youtube.com/embed/RvRhUHTV_8k?rel=0&amp;playlist=RvRhUHTV_8k,_SvwdK_HibQ,SGqg_ZzThDU" frameborder="0" allowfullscreen>"#
        )
    );
}

// The audio and video attribute reference tables (many entries are
// YouTube/Vimeo-, DocBook-, or PDF-specific).
non_normative!(
    r#"
== Audio and video attributes and options

.Audio attributes and values
[%autowidth]
|===
|Attribute |Value(s) |Example Syntax |Notes

|`title`
|User defined text
|`.Ocean waves`
|

|`start`
|User-defined playback start time in seconds.
|`start=30`
|

|`end`
|User-defined playback end time in seconds.
|`end=90`
|

|`options` (`opts`)
|`autoplay`, `loop`, `controls`, `nocontrols`
|`opts="autoplay,loop"`
|The controls value is enabled by default
|===

.Video attributes and values
[%autowidth]
|===
|Attribute |Value(s) |Example Syntax |Notes

|`title`
|User defined text
|`.An ocean sunset`
|

|`poster`
|A URL to an image to show until the user plays or seeks.
|`poster=sunset.jpg`
|Can be specified as the first positional (unnamed) attribute.
Also used to specify the service when referring to a video hosted on Vimeo (`vimeo`) or YouTube (`youtube`).

|`width`
|User-defined size in pixels.
|`width=640`
|Can be specified as the second positional (unnamed) attribute.

|`height`
|User-defined size in pixels.
|`height=480`
|Can be specified as the third positional (unnamed) attribute.

|`start`
|User-defined playback start time in seconds.
|`start=30`
|

|`end`
|User-defined playback end time in seconds.
|`end=90`
|

|`theme`
|The YouTube theme to use for the frame.
|`theme=light`
|Valid values are `dark` (the default) and `light`.

|`lang`
|The language used in the YouTube frame.
|`lang=fr`
|A BCP 47 language tag (typically a two-letter language code, like `en`).

|`list`
|The ID of a playlist to associate with a YouTube video.
|`list=PLabc123`
|Only applies to YouTube videos.

|`playlist`
|Additional video IDs to create a dynamic YouTube playlist.
|`playlist="video-abc,video-xyz"`
|IDs must be separated by commas.
Therefore, the value must be enclosed in double quotes.
Only applies to YouTube videos.

|`align`
|`left`, `center`, `right`
|`align=center`
|Follows the same alignment rules as a block image.

|`options` (`opts`)
|`autoplay`, `loop`, `modest`, `nocontrols`, `nofullscreen`, `muted`
|`opts="autoplay,loop"`
|The controls are enabled by default.
The `modest` option enables modest branding for a YouTube video.
|===
"#
);
