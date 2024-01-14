use trybuild::TestCases;

#[test]
fn ui_tests() {
    let t = TestCases::new();
    t.compile_fail("tests/compile_time/unsupported_structure.rs");
    t.compile_fail("tests/compile_time/unsupported_f64.rs");
    t.compile_fail("tests/compile_time/unsupported_char.rs");
    t.compile_fail("tests/compile_time/unsupported_isize.rs");
    t.pass("tests/compile_time/unnamed_fields.rs");
    t.pass("tests/compile_time/test_u8s.rs");
    t.pass("tests/compile_time/test_u16s.rs");
    t.pass("tests/compile_time/test_u32s.rs");
    t.pass("tests/compile_time/nested_struct.rs");
    t.pass("tests/compile_time/arrayed.rs");
}
