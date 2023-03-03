#[test]
fn try_build() {
    let t = trybuild::TestCases::new();

    #[cfg(feature = "any-ptr")]
    t.pass("tests/build/*.rs");
}
