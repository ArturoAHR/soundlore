#[macro_export]
macro_rules! assert_matches {
    ($val:expr, $pat:pat $(,)?) => {
        match $val {
            $pat => {}
            ref other => panic!(
                "Pattern was not matched:\n\n    value: {other:?}\n    pattern: {}",
                stringify!($pat)
            ),
        }
    };
}
