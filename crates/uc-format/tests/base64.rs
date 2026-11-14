//! Base64 decoder with the rules of UE `FBase64` in the standard mode.

use uc_format::Error;
use uc_format::base64::decode;

#[test]
fn decodes_the_rfc_4648_test_vectors() {
    let vectors: [(&str, &[u8]); 7] = [
        ("", b""),
        ("Zg==", b"f"),
        ("Zm8=", b"fo"),
        ("Zm9v", b"foo"),
        ("Zm9vYg==", b"foob"),
        ("Zm9vYmE=", b"fooba"),
        ("Zm9vYmFy", b"foobar"),
    ];
    for (text, bytes) in vectors {
        assert_eq!(decode(text).unwrap(), bytes, "{text}");
    }
}

#[test]
fn uses_plus_and_slash_for_62_and_63() {
    assert_eq!(decode("+/+/").unwrap(), [0xFB, 0xFF, 0xBF]);
}

#[test]
fn accepts_text_without_padding_and_with_more_padding() {
    assert_eq!(decode("Zg").unwrap(), b"f");
    assert_eq!(decode("Zm8").unwrap(), b"fo");
    assert_eq!(decode("Zg=====").unwrap(), b"f");
}

#[test]
fn a_character_out_of_the_alphabet_gives_a_bad_value() {
    for (text, offset) in [("Zm9v!A==", 4), ("Zm-v", 2), ("Zm_v", 2), ("Zżv=", 1)] {
        assert_eq!(
            decode(text),
            Err(Error::BadValue {
                offset,
                field: "Base64 text"
            }),
            "{text}"
        );
    }
}

#[test]
fn padding_before_the_end_gives_a_bad_value() {
    assert_eq!(
        decode("Zg==Zg=="),
        Err(Error::BadValue {
            offset: 2,
            field: "Base64 text"
        })
    );
}

#[test]
fn a_length_of_4n_plus_1_gives_a_bad_value() {
    assert_eq!(
        decode("Zm9vY"),
        Err(Error::BadValue {
            offset: 5,
            field: "Base64 text"
        })
    );
    assert_eq!(
        decode("Z==="),
        Err(Error::BadValue {
            offset: 1,
            field: "Base64 text"
        })
    );
}
