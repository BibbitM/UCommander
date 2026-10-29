//! The numeric values must match the Content-Plugin Guide 2.12 (`contplug.h` 2.10).

use uc_tc::content::{FieldType, NO_MORE_FIELDS, ValueError};

#[test]
fn field_types_have_the_guide_values() {
    let expected = [
        (FieldType::Numeric32, 1),
        (FieldType::Numeric64, 2),
        (FieldType::NumericFloating, 3),
        (FieldType::Date, 4),
        (FieldType::Time, 5),
        (FieldType::Boolean, 6),
        (FieldType::MultipleChoice, 7),
        (FieldType::String, 8),
        (FieldType::FullText, 9),
        (FieldType::DateTime, 10),
        (FieldType::StringW, 11),
        (FieldType::FullTextW, 12),
    ];
    for (field_type, value) in expected {
        assert_eq!(field_type.code(), value, "{field_type:?}");
    }
}

#[test]
fn no_more_fields_is_zero() {
    assert_eq!(NO_MORE_FIELDS, 0);
}

#[test]
fn value_errors_have_the_guide_values() {
    let expected = [
        (ValueError::NoSuchField, -1),
        (ValueError::FileError, -2),
        (ValueError::FieldEmpty, -3),
        (ValueError::OnDemand, -4),
        (ValueError::NotSupported, -5),
        (ValueError::SetCancel, -6),
    ];
    for (error, value) in expected {
        assert_eq!(error.code(), value, "{error:?}");
    }
}
