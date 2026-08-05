//! A minimal, dependency-free Base64 encoder for embedding binary assets as
//! `data:` URIs.
//!
//! This crate depends only on `asciidoc-parser` plus `std` (see `CLAUDE.md`),
//! so it hand-rolls the small amount of Base64 it needs rather than pulling in
//! a crate. [`encode`] produces standard Base64 (RFC 4648 alphabet) with `=`
//! padding and **no** line breaks, matching Ruby's `Array#pack('m0')` — the
//! encoding Asciidoctor uses when it builds an image data URI (its comment
//! notes `pack 'm0'` is equivalent to `Base64.strict_encode64`).

/// The standard Base64 alphabet (RFC 4648): `A–Z`, `a–z`, `0–9`, `+`, `/`.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes `bytes` as standard Base64 with `=` padding and no line breaks.
///
/// Every three input bytes become four output characters; a trailing group of
/// one or two bytes is padded to four characters with `=`. The result is thus
/// always a multiple of four characters long, and empty input yields an empty
/// string — the form a `data:…;base64,` URI expects.
pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        // Pack the (up to three) bytes into a 24-bit big-endian group; the
        // missing bytes of a short final chunk are treated as zero.
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let group = (b0 << 16) | (b1 << 8) | b2;

        // Emit four 6-bit sextets, replacing the ones that decode only padding
        // bytes with `=`: one `=` when the chunk had two bytes, two when it had
        // one.
        out.push(ALPHABET[(group >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(group >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(group >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[group as usize & 0x3f] as char
        } else {
            '='
        });
    }

    out
}

#[cfg(test)]
mod tests {
    use super::encode;

    // Empty input yields empty output — the body of a `data:…;base64,` URI for an
    // unreadable image.
    #[test]
    fn empty_input_is_empty() {
        assert_eq!(encode(b""), "");
    }

    // The classic RFC 4648 vectors, covering every padding case (none, one `=`,
    // two `=`).
    #[test]
    fn rfc4648_test_vectors() {
        assert_eq!(encode(b"f"), "Zg==");
        assert_eq!(encode(b"fo"), "Zm8=");
        assert_eq!(encode(b"foo"), "Zm9v");
        assert_eq!(encode(b"foob"), "Zm9vYg==");
        assert_eq!(encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    // Bytes across the full 0x00..=0xFF range, so the `+`/`/` alphabet slots and
    // the high bits are exercised (matches Ruby's `[bytes].pack 'm0'`).
    #[test]
    fn full_byte_range_uses_the_whole_alphabet() {
        assert_eq!(encode(&[0x00, 0x10, 0x83]), "ABCD");
        assert_eq!(encode(&[0xfb, 0xff, 0xbf]), "+/+/");
        // A one-pixel transparent GIF, the kind of payload a data URI carries.
        let gif = [
            0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00,
            0x00, 0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00,
            0x3b,
        ];
        assert_eq!(
            encode(&gif),
            "R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAICRAEAOw=="
        );
    }
}
