use trybuild::TestCases;

#[test]
fn ui_tests() {
    let t = TestCases::new();

    // ===== ATTRIBUTE TESTS =====
    t.compile_fail("tests/compile_time/attributes/bits_and_size_conflict.rs");
    t.compile_fail("tests/compile_time/attributes/fromfield_and_with_conflict.rs");
    t.compile_fail("tests/compile_time/attributes/multiple_endian_attrs.rs");

    // ===== BIT FIELD TESTS =====
    t.compile_fail("tests/compile_time/bit_fields/incomplete_byte.rs");
    t.compile_fail("tests/compile_time/bit_fields/zero_bits.rs");
    t.compile_fail("tests/compile_time/bit_fields/exceeds_type_size.rs");
    t.compile_fail("tests/compile_time/bit_fields/bits_on_non_numeric.rs");
    t.compile_fail("tests/compile_time/bit_fields/multiple_bits_attributes.rs");

    // ===== ENUM TESTS =====
    t.compile_fail("tests/compile_time/enums/duplicate_discriminants.rs");
    t.compile_fail("tests/compile_time/enums/data_variants.rs");
    #[cfg(feature = "std")]
    t.compile_fail("tests/compile_time/enums/invalid_flag_enum.rs");
    #[cfg(feature = "std")]
    t.compile_fail("tests/compile_time/enums/explicit_type_too_small.rs");

    // ===== SIZE EXPRESSION TESTS =====
    t.compile_fail("tests/compile_time/size_expressions/nonexistent_field.rs");
    t.compile_fail("tests/compile_time/size_expressions/circular_dependency.rs");
    t.compile_fail("tests/compile_time/size_expressions/division_by_zero.rs");
    t.compile_fail("tests/compile_time/size_expressions/invalid_operator.rs");

    // ===== TYPE TESTS =====
    t.compile_fail("tests/compile_time/types/unsupported_structure.rs");
    t.compile_fail("tests/compile_time/types/unsupported_isize.rs");
    t.compile_fail("tests/compile_time/types/bits_on_f32.rs");
    t.compile_fail("tests/compile_time/types/bits_on_f64.rs");
    t.compile_fail("tests/compile_time/types/bits_on_bool.rs");

    // ===== VECTOR TESTS =====
    t.compile_fail("tests/compile_time/vectors/fromfield_nonexistent.rs");
    t.compile_fail("tests/compile_time/vectors/fromfield_non_numeric.rs");
    t.compile_fail("tests/compile_time/vectors/multiple_vecs_no_size.rs");
    t.compile_fail("tests/compile_time/vectors/vec_not_last_no_size.rs");

    // ===== MARKER TESTS =====
    t.compile_fail("tests/compile_time/markers/non_ascii_char.rs");

    // ===== PASSING TESTS =====
    // These tests should compile successfully
    t.pass("tests/compile_time/unnamed_fields.rs");
    t.pass("tests/compile_time/test_u8s.rs");
    t.pass("tests/compile_time/test_u16s.rs");
    t.pass("tests/compile_time/test_u32s.rs");
    t.pass("tests/compile_time/test_chars.rs");
    t.pass("tests/compile_time/nested_struct.rs");
    t.pass("tests/compile_time/arrayed.rs");
    t.pass("tests/compile_time/vectors/safe_nested_vector.rs");
    #[cfg(feature = "std")]
    t.pass("tests/compile_time/enums/zero_value_flag_enum.rs");
    #[cfg(feature = "std")]
    t.pass("tests/compile_time/custom_result_alias.rs");
    t.pass("tests/compile_time/option.rs");
}
