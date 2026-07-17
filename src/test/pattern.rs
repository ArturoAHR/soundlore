#[macro_export]
macro_rules! assert_matches {
    ($expression:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {
        match $expression {
            $pattern $(if $guard)? => {}
            ref other => panic!(
                "Pattern was not matched:\n\n    value: {other:?}\n    pattern: {}",
                stringify!($pattern)
            ),
        }
    };
}
