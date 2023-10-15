use trybuild::TestCases;

#[test]
fn ui_tests() {
    let t = TestCases::new();
    t.compile_fail("tests/compile_time/unsupported_structure.rs");
    t.pass("tests/compile_time/unnamed_fields.rs");
}
