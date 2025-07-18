use bebytes::BeBytes;

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum Status {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
struct PacketWithAutoEnums {
    #[bits(4)]
    header: u8,
    #[bits()] // Auto-sized to Status::__BEBYTES_MIN_BITS (2 bits)
    status: Status,
    #[bits()] // Auto-sized to Priority::__BEBYTES_MIN_BITS (2 bits)
    priority: Priority,
}

#[test]
fn test_auto_sized_enum_bits() {
    // Test that constants are generated correctly
    assert_eq!(Status::__BEBYTES_MIN_BITS, 2); // 4 variants need 2 bits
    assert_eq!(Priority::__BEBYTES_MIN_BITS, 2); // 3 variants need 2 bits

    // Test TryFrom<u8> implementation
    assert_eq!(Status::try_from(0u8).unwrap(), Status::Idle);
    assert_eq!(Status::try_from(1u8).unwrap(), Status::Running);
    assert_eq!(Status::try_from(2u8).unwrap(), Status::Paused);
    assert_eq!(Status::try_from(3u8).unwrap(), Status::Stopped);
    assert!(Status::try_from(4u8).is_err());

    assert_eq!(Priority::try_from(0u8).unwrap(), Priority::Low);
    assert_eq!(Priority::try_from(1u8).unwrap(), Priority::Medium);
    assert_eq!(Priority::try_from(2u8).unwrap(), Priority::High);
    assert!(Priority::try_from(3u8).is_err());

    // Test serialization/deserialization
    let packet = PacketWithAutoEnums {
        header: 0b1010,
        status: Status::Running,
        priority: Priority::High,
    };

    // Big-endian test
    let be_bytes = packet.to_be_bytes();
    assert_eq!(be_bytes.len(), 1); // All fields fit in 1 byte (4+2+2 = 8 bits)

    // Verify bit layout: header(4) | status(2) | priority(2)
    // 1010 | 01 | 10 = 0b10100110 = 166
    assert_eq!(be_bytes[0], 0b10100110);

    let (deserialized, bytes_read) = PacketWithAutoEnums::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(bytes_read, 1);
    assert_eq!(deserialized.header, 0b1010);
    assert_eq!(deserialized.status, Status::Running);
    assert_eq!(deserialized.priority, Priority::High);

    // Little-endian test
    let le_bytes = packet.to_le_bytes();
    assert_eq!(le_bytes.len(), 1);

    // In little-endian, bits within a byte are still laid out the same way
    // But we read from LSB first: priority(2) | status(2) | header(4)
    // 10 | 01 | 1010 = 0b10011010 = 154
    assert_eq!(le_bytes[0], 0b10011010);

    let (deserialized, bytes_read) = PacketWithAutoEnums::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(bytes_read, 1);
    assert_eq!(deserialized.header, 0b1010);
    assert_eq!(deserialized.status, Status::Running);
    assert_eq!(deserialized.priority, Priority::High);
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
#[repr(u8)]
enum LargeEnum {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    V10 = 10,
    V11 = 11,
    V12 = 12,
    V13 = 13,
    V14 = 14,
    V15 = 15,
    V16 = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, BeBytes)]
struct PacketWithLargeEnum {
    #[bits(3)]
    prefix: u8,
    #[bits()] // Auto-sized to LargeEnum::__BEBYTES_MIN_BITS (5 bits)
    large: LargeEnum,
}

#[test]
fn test_large_enum_auto_bits() {
    // Test that 17 variants need 5 bits (2^4 = 16 < 17 <= 2^5 = 32)
    assert_eq!(LargeEnum::__BEBYTES_MIN_BITS, 5);

    let packet = PacketWithLargeEnum {
        prefix: 0b101,
        large: LargeEnum::V16,
    };

    let be_bytes = packet.to_be_bytes();
    assert_eq!(be_bytes.len(), 1); // 3+5 = 8 bits = 1 byte

    // Verify bit layout: prefix(3) | large(5)
    // 101 | 10000 = 0b10110000 = 176
    assert_eq!(be_bytes[0], 0b10110000);

    let (deserialized, _) = PacketWithLargeEnum::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(deserialized.prefix, 0b101);
    assert_eq!(deserialized.large, LargeEnum::V16);
}
