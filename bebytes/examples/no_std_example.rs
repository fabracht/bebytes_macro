//! Example of using BeBytes in a no_std environment
//!
//! This example demonstrates how to use BeBytes without the standard library,
//! which is useful for embedded systems and other constrained environments.
//!
//! To run this example with no_std:
//! ```bash
//! cargo run --example no_std_example --no-default-features
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;
use alloc::vec::Vec;
use bebytes::BeBytes;
#[cfg(not(feature = "std"))]
use bebytes::ToOwned;
#[cfg(feature = "std")]
use std::borrow::ToOwned;

// Panic handler required for no_std binaries
#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In a real embedded application, you might want to reset the system
    // or write to a debug console here
    loop {}
}

// Define a simple packet structure
#[derive(BeBytes, Debug, PartialEq)]
struct Packet {
    // Header fields
    version: u8,
    packet_type: PacketType,
    length: u16,

    // Payload
    #[With(size(8))]
    payload: Vec<u8>,

    // Checksum
    checksum: u16,
}

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum PacketType {
    Data = 0,
    Control = 1,
    Ack = 2,
}

// Example of a bitfield structure for flags
#[derive(BeBytes, Debug, PartialEq)]
struct Flags {
    #[bits(1)]
    urgent: u8,
    #[bits(1)]
    ack: u8,
    #[bits(1)]
    push: u8,
    #[bits(5)]
    reserved: u8,
}

// Example of nested structures
#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    header: Header,
    #[FromField(header.body_length)]
    body: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct Header {
    msg_id: u32,
    body_length: u16,
    flags: u8,
}

// Entry point for no_std binary
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Example 1: Simple packet serialization
    let packet = Packet {
        version: 1,
        packet_type: PacketType::Data,
        length: 8,
        payload: alloc::vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
        checksum: 0x1234,
    };

    // Serialize to bytes
    let bytes = packet.to_be_bytes();

    // Deserialize back
    match Packet::try_from_be_bytes(&bytes) {
        Ok((decoded, consumed)) => {
            assert_eq!(decoded, packet);
            assert_eq!(consumed, bytes.len());
        }
        Err(_) => {
            // Handle error in embedded environment
        }
    }

    // Example 2: Bitfield flags
    let flags = Flags {
        urgent: 1,
        ack: 0,
        push: 1,
        reserved: 0b10101,
    };

    let flag_bytes = flags.to_be_bytes();
    assert_eq!(flag_bytes.len(), 1); // All flags fit in a single byte

    // Example 3: Variable length data with nested fields
    let message = Message {
        header: Header {
            msg_id: 0xDEADBEEF,
            body_length: 5,
            flags: 0xFF,
        },
        body: alloc::vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE],
    };

    let msg_bytes = message.to_be_bytes();

    // The body length is determined by header.body_length field
    match Message::try_from_be_bytes(&msg_bytes) {
        Ok((decoded, _)) => {
            assert_eq!(decoded.body.len(), 5);
            assert_eq!(decoded.header.body_length, 5);
        }
        Err(_) => {
            // Handle error
        }
    }

    // Example 4: Error handling in no_std
    let insufficient_data = [0x01, 0x02]; // Not enough bytes for a Packet
    match Packet::try_from_be_bytes(&insufficient_data) {
        Err(bebytes::BeBytesError::InsufficientData { expected, actual }) => {
            // In embedded, you might log this or set an error flag
            let _ = (expected, actual); // Avoid unused variable warning
        }
        _ => {
            // Unexpected result
        }
    }

    // Loop forever (typical for embedded applications)
    loop {}
}

// Memory allocator for no_std
// In a real embedded application, you would configure this based on your platform
#[cfg(not(feature = "std"))]
extern crate libc;

#[cfg(not(feature = "std"))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(feature = "std"))]
struct Allocator;

#[cfg(not(feature = "std"))]
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size()) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void);
    }
}

#[cfg(not(feature = "std"))]
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

// Main function for std builds
#[cfg(feature = "std")]
fn main() {
    println!("This example is designed for no_std environments.");
    println!("Run with: cargo run --example no_std_example --no-default-features");
}
