use uc_tc::guard;

#[test]
fn guard_returns_the_result_when_there_is_no_panic() {
    assert_eq!(guard(-1, || 42), 42);
}

#[test]
fn guard_returns_the_fallback_when_the_code_panics() {
    let result = guard(-1, || -> i32 { panic!("test panic") });

    assert_eq!(result, -1);
}
