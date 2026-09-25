#[test]
fn test_types() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/types/fail/*.rs");
    t.pass("tests/types/pass/*.rs");
    #[cfg(feature = "basecoatui")]
    {
        t.compile_fail("tests/types/basecoatui/fail/*.rs");
        t.pass("tests/types/basecoatui/pass/*.rs");
    }
}
