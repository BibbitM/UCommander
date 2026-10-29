//! Types and safe FFI helpers for the Total Commander plugin API.

pub mod buffers;
pub mod content;

use std::panic::{AssertUnwindSafe, catch_unwind};

/// Runs `f`, and returns `on_panic` if `f` panics.
///
/// Use it in each exported function. A panic must not cross the FFI boundary,
/// because that closes Total Commander.
pub fn guard<R>(on_panic: R, f: impl FnOnce() -> R) -> R {
    // The plugin state after a panic is not used again for this call, so
    // AssertUnwindSafe is acceptable.
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(on_panic)
}

#[cfg(test)]
mod tests {
    #[test]
    fn package_name_is_uc_tc() {
        assert_eq!(env!("CARGO_PKG_NAME"), "uc-tc");
    }
}
