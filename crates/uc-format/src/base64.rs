//! Base64 decoder with the rules of UE `FBase64` in the standard mode. The
//! rules are in `docs/formats/world-partition.md`, "Base64".

use crate::error::Error;

const FIELD: &str = "Base64 text";

/// Decodes Base64 text: the alphabet of RFC 4648, table 1 (`A–Z`, `a–z`,
/// `0–9`, `+`, `/`).
///
/// As UE, the function removes all `=` at the end, and accepts text
/// without padding. A length (without the `=` at the end) of 4n + 1 is a
/// bad value at the end of the text. A character that is not in the
/// alphabet is a bad value at its byte offset. The unused bits of the
/// last character are not checked.
pub fn decode(text: &str) -> Result<Vec<u8>, Error> {
    let input = text.trim_end_matches('=').as_bytes();
    if input.len() % 4 == 1 {
        return Err(Error::BadValue {
            offset: input.len(),
            field: FIELD,
        });
    }
    // At most 3 bytes for each 4 characters, so the size has a limit.
    let mut bytes = Vec::with_capacity(input.len() / 4 * 3 + 2);
    for (index, chunk) in input.chunks(4).enumerate() {
        let mut group = 0u32;
        for (position, &character) in chunk.iter().enumerate() {
            let value = sextet(character).ok_or(Error::BadValue {
                offset: index * 4 + position,
                field: FIELD,
            })?;
            group |= u32::from(value) << (18 - 6 * position);
        }
        let [_, first, second, third] = group.to_be_bytes();
        // 2 characters give 1 byte, 3 give 2 bytes, 4 give 3 bytes.
        bytes.extend_from_slice(&[first, second, third][..chunk.len() - 1]);
    }
    Ok(bytes)
}

/// Gives the 6-bit value of a character of the alphabet.
fn sextet(character: u8) -> Option<u8> {
    match character {
        b'A'..=b'Z' => Some(character - b'A'),
        b'a'..=b'z' => Some(character - b'a' + 26),
        b'0'..=b'9' => Some(character - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
