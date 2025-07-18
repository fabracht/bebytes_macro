use bebytes::BeBytes;

// Test edge cases for enum bit width calculation
#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum SingleVariant {
    Only = 0,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum TwoVariants {
    First = 0,
    Second = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum ExactPowerOfTwo {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum JustOverPowerOfTwo {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
}

#[test]
fn test_minimum_bits_calculation() {
    // 1 variant needs 0 bits (but we use 1 bit minimum)
    assert_eq!(SingleVariant::__BEBYTES_MIN_BITS, 1);

    // 2 variants need 1 bit
    assert_eq!(TwoVariants::__BEBYTES_MIN_BITS, 1);

    // 8 variants need exactly 3 bits
    assert_eq!(ExactPowerOfTwo::__BEBYTES_MIN_BITS, 3);

    // 9 variants need 4 bits
    assert_eq!(JustOverPowerOfTwo::__BEBYTES_MIN_BITS, 4);
}

// Test multiple auto-sized enums in a single struct
#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
struct MultiAutoEnums {
    #[bits()]
    single: SingleVariant,
    #[bits()]
    two: TwoVariants,
    #[bits()]
    eight: ExactPowerOfTwo,
    #[bits()]
    nine: JustOverPowerOfTwo,
}

#[test]
fn test_multiple_auto_enums() {
    let packet = MultiAutoEnums {
        single: SingleVariant::Only,
        two: TwoVariants::Second,
        eight: ExactPowerOfTwo::V7,
        nine: JustOverPowerOfTwo::V8,
    };

    // Total bits: 1 + 1 + 3 + 4 = 9 bits = 2 bytes
    let be_bytes = packet.to_be_bytes();
    println!(
        "Bytes generated: {:?}, length: {}",
        be_bytes,
        be_bytes.len()
    );
    println!("Expected total bits: 1 + 1 + 3 + 4 = 9");
    assert_eq!(be_bytes.len(), 2);

    let (deserialized, bytes_read) = MultiAutoEnums::try_from_be_bytes(&be_bytes).unwrap();
    println!("Bytes read: {}", bytes_read);
    // With 9 bits total, we need to read 2 bytes (ceiling of 9/8)
    assert_eq!(bytes_read, 2);
    assert_eq!(deserialized.single, SingleVariant::Only);
    assert_eq!(deserialized.two, TwoVariants::Second);
    assert_eq!(deserialized.eight, ExactPowerOfTwo::V7);
    assert_eq!(deserialized.nine, JustOverPowerOfTwo::V8);
}

// Test auto-sized enums mixed with explicit-sized fields
#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
struct MixedBitFields {
    #[bits(5)]
    explicit_u8: u8,
    #[bits()]
    auto_enum: TwoVariants,
    #[bits(2)]
    another_explicit: u8,
    // Total: 5 + 1 + 2 = 8 bits
}

#[test]
fn test_mixed_auto_and_explicit_bits() {
    let packet = MixedBitFields {
        explicit_u8: 0b11010,
        auto_enum: TwoVariants::First,
        another_explicit: 0b11,
    };

    let be_bytes = packet.to_be_bytes();
    assert_eq!(be_bytes.len(), 1);

    // Big-endian bit layout: explicit_u8(5) | auto_enum(1) | another_explicit(2)
    // 11010 | 0 | 11 = 0b11010011 = 211
    assert_eq!(be_bytes[0], 0b11010011);

    let (deserialized, _) = MixedBitFields::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(deserialized.explicit_u8, 0b11010);
    assert_eq!(deserialized.auto_enum, TwoVariants::First);
    assert_eq!(deserialized.another_explicit, 0b11);
}

// Test error handling for invalid discriminant values
#[test]
fn test_try_from_errors() {
    // SingleVariant only has value 0
    assert!(SingleVariant::try_from(1u8).is_err());

    // TwoVariants has values 0-1
    assert!(TwoVariants::try_from(2u8).is_err());
    assert!(TwoVariants::try_from(255u8).is_err());

    // ExactPowerOfTwo has values 0-7
    assert!(ExactPowerOfTwo::try_from(8u8).is_err());

    // JustOverPowerOfTwo has values 0-8
    assert!(JustOverPowerOfTwo::try_from(9u8).is_err());
}

// Test with non-contiguous discriminant values
#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum NonContiguous {
    A = 0,
    B = 2,
    C = 5,
    D = 7,
}

#[test]
fn test_non_contiguous_enum() {
    // Max discriminant is 7, which needs 3 bits
    assert_eq!(NonContiguous::__BEBYTES_MIN_BITS, 3);

    // Test TryFrom for valid values
    assert_eq!(NonContiguous::try_from(0u8).unwrap(), NonContiguous::A);
    assert_eq!(NonContiguous::try_from(2u8).unwrap(), NonContiguous::B);
    assert_eq!(NonContiguous::try_from(5u8).unwrap(), NonContiguous::C);
    assert_eq!(NonContiguous::try_from(7u8).unwrap(), NonContiguous::D);

    // Test TryFrom for invalid values (gaps in discriminants)
    assert!(NonContiguous::try_from(1u8).is_err());
    assert!(NonContiguous::try_from(3u8).is_err());
    assert!(NonContiguous::try_from(4u8).is_err());
    assert!(NonContiguous::try_from(6u8).is_err());
}

// Test auto-sized enum at the end of multi-byte bit fields
#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
struct CrossByteBoundary {
    #[bits(7)]
    large_field: u8,
    #[bits()] // 3 bits for ExactPowerOfTwo
    enum_field: ExactPowerOfTwo,
    #[bits(6)]
    trailing: u8,
    // Total: 7 + 3 + 6 = 16 bits = 2 bytes
}

#[test]
fn test_cross_byte_boundary() {
    let packet = CrossByteBoundary {
        large_field: 0b1111111,          // All bits set
        enum_field: ExactPowerOfTwo::V5, // Value 5 = 0b101
        trailing: 0b101010,
    };

    let be_bytes = packet.to_be_bytes();
    assert_eq!(be_bytes.len(), 2);

    let (deserialized, _) = CrossByteBoundary::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(deserialized.large_field, 0b1111111);
    assert_eq!(deserialized.enum_field, ExactPowerOfTwo::V5);
    assert_eq!(deserialized.trailing, 0b101010);
}

// Test little-endian behavior with auto-sized enums
#[test]
fn test_little_endian_auto_enums() {
    let packet = MixedBitFields {
        explicit_u8: 0b10101,
        auto_enum: TwoVariants::Second,
        another_explicit: 0b10,
    };

    let le_bytes = packet.to_le_bytes();
    assert_eq!(le_bytes.len(), 1);

    // Little-endian bit layout within byte: another_explicit(2) | auto_enum(1) | explicit_u8(5)
    // 10 | 1 | 10101 = 0b10110101 = 181
    assert_eq!(le_bytes[0], 0b10110101);

    let (deserialized, _) = MixedBitFields::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(deserialized.explicit_u8, 0b10101);
    assert_eq!(deserialized.auto_enum, TwoVariants::Second);
    assert_eq!(deserialized.another_explicit, 0b10);
}
