//! Example demonstrating TTT (Type-Trait-Trait) pattern improvements in BeBytes
//!
//! This example shows:
//! 1. Improved error handling with specific error types
//! 2. Type-safe builder pattern for constructing byte sequences

use bebytes::builder::BytesBuilder;
use bebytes::{BeBytes, BeBytesError};

#[derive(Debug, BeBytes, PartialEq)]
struct NetworkPacket {
    version: u8,
    flags: u16,
    payload_len: u8,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}

/// Demonstrates improved error handling with specific error types
fn demonstrate_error_handling() {
    println!("=== Error Handling Demo ===\n");

    // Case 1: Insufficient data
    let short_data = vec![0x01, 0x00]; // Only 2 bytes, need at least 4
    match NetworkPacket::try_from_be_bytes(&short_data) {
        Ok(_) => println!("Unexpected success"),
        Err(BeBytesError::InsufficientData { expected, actual }) => {
            println!(
                "✓ Caught insufficient data error: expected {} bytes, got {}",
                expected, actual
            );
        }
        Err(e) => println!("Different error: {:?}", e),
    }

    // Case 2: Empty buffer
    let empty_data = vec![];
    match NetworkPacket::try_from_be_bytes(&empty_data) {
        Ok(_) => println!("Unexpected success"),
        Err(BeBytesError::EmptyBuffer) => {
            println!("✓ Caught empty buffer error");
        }
        Err(e) => println!("Different error: {:?}", e),
    }

    // Case 3: Success case
    let valid_data = vec![
        0x01, // version
        0x00, 0x42, // flags (big-endian)
        0x05, // payload_len
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, // payload
    ];

    match NetworkPacket::try_from_be_bytes(&valid_data) {
        Ok((packet, bytes_read)) => {
            println!("✓ Successfully parsed packet:");
            println!("  Version: {}", packet.version);
            println!("  Flags: 0x{:04X}", packet.flags);
            println!("  Payload: {:?}", packet.payload);
            println!("  Bytes consumed: {}", bytes_read);
        }
        Err(e) => println!("Unexpected error: {:?}", e),
    }
}

/// Demonstrates the type-safe builder pattern
fn demonstrate_builder_pattern() {
    println!("\n=== Builder Pattern Demo ===\n");

    // Build a simple packet
    let packet_bytes = BytesBuilder::new()
        .u8(0x01) // version
        .u16_be(0x0042) // flags
        .u8(0x05) // payload length
        .with_size(5) // Specify size for next field
        .bytes(vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE])
        .build_variable();

    println!("Built packet bytes: {:02X?}", packet_bytes);

    // Parse it back
    let (packet, _) = NetworkPacket::try_from_be_bytes(&packet_bytes).unwrap();
    println!("Parsed back: {:?}", packet);

    // Demonstrate padding with sized builder
    let padded_bytes = BytesBuilder::new()
        .u8(0x02) // version
        .u16_be(0x1234) // flags
        .u8(0x08) // payload length
        .with_size(8) // Specify 8 bytes
        .bytes(vec![0x11, 0x22, 0x33]) // Only provide 3, will be padded
        .build_variable();

    println!("\nBuilt padded packet: {:02X?}", padded_bytes);
    println!("Note: The last 5 bytes are padded with zeros");

    // Complex example with multiple variable fields
    let complex_bytes = BytesBuilder::new()
        .u32_be(0xDEADBEEF) // Magic number
        .u16_le(0x1234) // Little-endian field
        .fixed_bytes([0xFF; 4]) // Fixed array
        .remaining_bytes(vec![0x01, 0x02, 0x03]) // Variable tail
        .append_bytes(vec![0x04, 0x05]) // Append more
        .build_variable();

    println!("\nComplex structure: {:02X?}", complex_bytes);
}

/// Demonstrates type safety of the builder
fn demonstrate_type_safety() {
    println!("\n=== Type Safety Demo ===\n");

    // This compiles - correct order
    let _correct = BytesBuilder::new()
        .u8(0x42) // Fixed size first
        .u32_be(0x12345678) // More fixed size
        .remaining_bytes(vec![0x01, 0x02]) // Then variable
        .build_variable();

    println!("✓ Correct order compiles");

    // This would NOT compile (commented out):
    // let _wrong = BytesBuilder::new()
    //     .remaining_bytes(vec![0x01, 0x02])  // Variable first
    //     .u8(0x42)                            // ERROR: Can't add fixed after variable!
    //     .build();

    println!("✓ Type system prevents incorrect field ordering at compile time");

    // Demonstrates the state transitions
    let builder_empty = BytesBuilder::new();
    let builder_fixed = builder_empty.u8(0x01);
    let builder_sized = builder_fixed.with_size(10);
    let builder_variable = builder_sized.bytes(vec![0x02; 10]);
    let final_bytes = builder_variable.build_variable();

    println!("✓ State transitions: Empty → HasFixed → Sized → HasVariable");
    println!("  Final bytes: {:02X?}", &final_bytes[..5]); // Show first 5 bytes
}

fn main() {
    println!("BeBytes TTT Pattern Improvements Demo\n");
    println!("=====================================\n");

    demonstrate_error_handling();
    demonstrate_builder_pattern();
    demonstrate_type_safety();

    println!("\n=====================================");
    println!("All demonstrations completed successfully!");
}
