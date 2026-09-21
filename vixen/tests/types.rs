#[test]
fn test_types() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/types/fail/*.rs");
    t.pass("tests/types/pass/*.rs");
}
