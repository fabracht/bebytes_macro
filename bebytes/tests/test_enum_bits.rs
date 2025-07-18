use bebytes::BeBytes;

// Simple test enum with a few variants
#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
pub enum TestStatus {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
    ErrorState = 4,
}

// Another enum to test different value ranges
#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

// Test struct using integer bit fields to represent enum values
#[derive(BeBytes, Debug, PartialEq)]
pub struct StatusPacket {
    #[bits(3)] // 3 bits for status (0-4)
    status_value: u8,
    #[bits(2)] // 2 bits for priority (0-3)
    priority_value: u8,
    #[bits(3)] // Fill remaining bits to complete the byte
    reserved: u8,
}

impl StatusPacket {
    pub fn new_with_enums(status: TestStatus, priority: Priority, reserved: u8) -> Self {
        Self {
            status_value: status as u8,
            priority_value: priority as u8,
            reserved,
        }
    }

    pub fn status(&self) -> TestStatus {
        match self.status_value {
            0 => TestStatus::Idle,
            1 => TestStatus::Running,
            2 => TestStatus::Paused,
            3 => TestStatus::Stopped,
            4 => TestStatus::ErrorState,
            _ => TestStatus::Idle, // Default
        }
    }

    pub fn priority(&self) -> Priority {
        match self.priority_value {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            3 => Priority::Critical,
            _ => Priority::Low, // Default
        }
    }
}

// Test struct with enum as regular field
#[derive(BeBytes, Debug, PartialEq)]
pub struct SimpleEnumPacket {
    header: u8,
    status: TestStatus,
    priority: Priority,
    footer: u16,
}

// Test struct demonstrating enum size calculation
#[derive(BeBytes, Debug, PartialEq)]
pub struct MixedPacket {
    #[bits(1)]
    enabled: u8,
    #[bits(3)] // status bits
    status_bits: u8,
    #[bits(2)] // priority bits
    priority_bits: u8,
    #[bits(2)] // padding
    padding: u8,
    payload: u16,
}

impl MixedPacket {
    pub fn new_with_enums(
        enabled: u8,
        status: TestStatus,
        priority: Priority,
        padding: u8,
        payload: u16,
    ) -> Self {
        Self {
            enabled,
            status_bits: status as u8,
            priority_bits: priority as u8,
            padding,
            payload,
        }
    }

    pub fn status(&self) -> TestStatus {
        match self.status_bits {
            0 => TestStatus::Idle,
            1 => TestStatus::Running,
            2 => TestStatus::Paused,
            3 => TestStatus::Stopped,
            4 => TestStatus::ErrorState,
            _ => TestStatus::Idle,
        }
    }

    pub fn priority(&self) -> Priority {
        match self.priority_bits {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            3 => Priority::Critical,
            _ => Priority::Low,
        }
    }
}

#[test]
fn test_enum_bits_basic() {
    // Test using helper constructor
    let packet = StatusPacket::new_with_enums(TestStatus::Running, Priority::High, 0);

    // Test big-endian
    let be_bytes = packet.to_be_bytes();
    println!("BE bytes: {:?}", be_bytes);
    for byte in &be_bytes {
        print!("{:08b} ", byte);
    }
    println!();

    let (decoded, len) = StatusPacket::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(decoded.status_value, TestStatus::Running as u8);
    assert_eq!(decoded.priority_value, Priority::High as u8);
    assert_eq!(decoded.reserved, 0);
    assert_eq!(len, be_bytes.len());

    // Test using accessor methods
    assert_eq!(decoded.status(), TestStatus::Running);
    assert_eq!(decoded.priority(), Priority::High);

    // Test little-endian
    let le_bytes = packet.to_le_bytes();
    println!("LE bytes: {:?}", le_bytes);

    let (decoded_le, len_le) = StatusPacket::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(decoded_le.status(), TestStatus::Running);
    assert_eq!(decoded_le.priority(), Priority::High);
    assert_eq!(len_le, le_bytes.len());
}

#[test]
fn test_enum_bits_all_variants() {
    // Test all status variants
    let statuses = [
        TestStatus::Idle,
        TestStatus::Running,
        TestStatus::Paused,
        TestStatus::Stopped,
        TestStatus::ErrorState,
    ];

    let priorities = [
        Priority::Low,
        Priority::Normal,
        Priority::High,
        Priority::Critical,
    ];

    for status in &statuses {
        for priority in &priorities {
            let packet = StatusPacket::new_with_enums(*status, *priority, 0);

            // Test round trip big-endian
            let be_bytes = packet.to_be_bytes();
            let (decoded, _) = StatusPacket::try_from_be_bytes(&be_bytes).unwrap();
            assert_eq!(decoded.status(), *status);
            assert_eq!(decoded.priority(), *priority);

            // Test round trip little-endian
            let le_bytes = packet.to_le_bytes();
            let (decoded_le, _) = StatusPacket::try_from_le_bytes(&le_bytes).unwrap();
            assert_eq!(decoded_le.status(), *status);
            assert_eq!(decoded_le.priority(), *priority);
        }
    }
}

#[test]
fn test_simple_enum_packet() {
    let packet = SimpleEnumPacket {
        header: 0xFF,
        status: TestStatus::Paused,
        priority: Priority::Normal,
        footer: 0x1234,
    };

    // Test big-endian
    let be_bytes = packet.to_be_bytes();
    println!("Simple enum packet BE bytes: {:?}", be_bytes);

    let (decoded, len) = SimpleEnumPacket::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(decoded, packet);
    assert_eq!(len, be_bytes.len());

    // Test little-endian
    let le_bytes = packet.to_le_bytes();
    let (decoded_le, _) = SimpleEnumPacket::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(decoded_le, packet);
}

#[test]
fn test_mixed_packet() {
    let packet = MixedPacket::new_with_enums(1, TestStatus::Paused, Priority::Normal, 0, 0x1234);

    // Test big-endian
    let be_bytes = packet.to_be_bytes();
    println!("Mixed packet BE bytes: {:?}", be_bytes);

    let (decoded, len) = MixedPacket::try_from_be_bytes(&be_bytes).unwrap();
    assert_eq!(decoded.enabled, 1);
    assert_eq!(decoded.status(), TestStatus::Paused);
    assert_eq!(decoded.priority(), Priority::Normal);
    assert_eq!(decoded.payload, 0x1234);
    assert_eq!(len, be_bytes.len());

    // Test little-endian
    let le_bytes = packet.to_le_bytes();
    println!("Mixed packet LE bytes: {:?}", le_bytes);

    let (decoded_le, len_le) = MixedPacket::try_from_le_bytes(&le_bytes).unwrap();
    assert_eq!(decoded_le.enabled, 1);
    assert_eq!(decoded_le.status(), TestStatus::Paused);
    assert_eq!(decoded_le.priority(), Priority::Normal);
    assert_eq!(decoded_le.payload, 0x1234);
    assert_eq!(len_le, le_bytes.len());
}

#[test]
fn test_enum_constructor() {
    // Test direct construction with new()
    let packet = StatusPacket::new(TestStatus::ErrorState as u8, Priority::Critical as u8, 0);
    assert_eq!(packet.status_value, TestStatus::ErrorState as u8);
    assert_eq!(packet.priority_value, Priority::Critical as u8);
    assert_eq!(packet.reserved, 0);

    // Test using accessor methods
    assert_eq!(packet.status(), TestStatus::ErrorState);
    assert_eq!(packet.priority(), Priority::Critical);

    let mixed = MixedPacket::new(1, TestStatus::Running as u8, Priority::Low as u8, 0, 0xABCD);
    assert_eq!(mixed.enabled, 1);
    assert_eq!(mixed.status(), TestStatus::Running);
    assert_eq!(mixed.priority(), Priority::Low);
    assert_eq!(mixed.padding, 0);
    assert_eq!(mixed.payload, 0xABCD);
}

#[test]
fn test_endianness_difference() {
    let packet =
        MixedPacket::new_with_enums(1, TestStatus::ErrorState, Priority::Critical, 0, 0x1234);

    let be_bytes = packet.to_be_bytes();
    let le_bytes = packet.to_le_bytes();

    // The byte representations should be different for multi-byte values
    assert_ne!(be_bytes, le_bytes);

    // But decoding with the correct endianness should yield the same result
    let (from_be, _) = MixedPacket::try_from_be_bytes(&be_bytes).unwrap();
    let (from_le, _) = MixedPacket::try_from_le_bytes(&le_bytes).unwrap();

    assert_eq!(from_be.enabled, packet.enabled);
    assert_eq!(from_be.status(), packet.status());
    assert_eq!(from_be.priority(), packet.priority());
    assert_eq!(from_be.payload, packet.payload);

    assert_eq!(from_le.enabled, packet.enabled);
    assert_eq!(from_le.status(), packet.status());
    assert_eq!(from_le.priority(), packet.priority());
    assert_eq!(from_le.payload, packet.payload);
}

#[test]
fn test_enum_min_bits() {
    // Test that enums expose their minimum bit requirements
    assert_eq!(TestStatus::__BEBYTES_MIN_BITS, 3); // Need 3 bits for values 0-4
    assert_eq!(Priority::__BEBYTES_MIN_BITS, 2); // Need 2 bits for values 0-3
}
