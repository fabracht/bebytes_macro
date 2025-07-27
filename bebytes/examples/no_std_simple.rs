//! Simple example of using BeBytes in a no_std environment
//!
//! This example can be tested with:
//! ```bash
//! cargo test --example no_std_simple --no-default-features
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::vec::Vec;
use bebytes::BeBytes;
#[cfg(test)]
use bebytes::BeBytesError;
#[cfg(not(feature = "std"))]
use bebytes::ToOwned;
#[cfg(feature = "std")]
use std::borrow::ToOwned;

// Simple data structure
#[derive(BeBytes, Debug, PartialEq)]
struct SensorData {
    sensor_id: u16,
    temperature: i16, // Temperature in 0.1°C units
    humidity: u8,     // Humidity in percent
    status: Status,
}

#[derive(BeBytes, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
enum Status {
    Ok = 0,
    Warning = 1,
    ErrorStatus = 2,
}

// Example with bit fields for compact data representation
#[derive(BeBytes, Debug, PartialEq)]
struct CompactReading {
    #[bits(12)] // 12-bit sensor ID (0-4095)
    sensor_id: u16,
    #[bits(10)] // 10-bit temperature value (0-1023)
    temp_raw: u16,
    #[bits(7)] // 7-bit humidity (0-127)
    humidity: u8,
    #[bits(3)] // 3-bit status (0-7)
    status: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_data_serialization() {
        let data = SensorData {
            sensor_id: 0x1234,
            temperature: 235, // 23.5°C
            humidity: 65,
            status: Status::Ok,
        };

        // Serialize
        let bytes = data.to_be_bytes();
        assert_eq!(bytes.len(), 6); // 2 + 2 + 1 + 1

        // Deserialize
        let (decoded, consumed) = SensorData::try_from_be_bytes(&bytes).unwrap();
        assert_eq!(consumed, 6);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_compact_reading() {
        // First test with simple values
        let reading = CompactReading {
            sensor_id: 1, // 12 bits
            temp_raw: 1,  // 10 bits
            humidity: 1,  // 7 bits
            status: 1,    // 3 bits
        };

        let bytes = reading.to_be_bytes();
        assert_eq!(bytes.len(), 4); // 32 bits total

        // Now test with actual values
        let reading2 = CompactReading {
            sensor_id: 0x123, // 12 bits (291)
            temp_raw: 0x1FF,  // 10 bits (511)
            humidity: 65,     // 7 bits
            status: 0b010,    // 3 bits (2)
        };

        let bytes2 = reading2.to_be_bytes();
        let (decoded, _) = CompactReading::try_from_be_bytes(&bytes2).unwrap();
        assert_eq!(decoded, reading2);
    }

    #[test]
    fn test_error_handling() {
        // Test with insufficient data
        let short_data = vec![0x12, 0x34]; // Only 2 bytes

        match SensorData::try_from_be_bytes(&short_data) {
            Err(BeBytesError::InsufficientData { expected, actual }) => {
                assert_eq!(expected, 2); // Expecting 2 bytes for temperature
                assert_eq!(actual, 0); // But have 0 bytes left
            }
            _ => panic!("Expected InsufficientData error"),
        }
    }
}

// This example is designed to work in no_std environments
// The tests demonstrate the functionality
fn main() {
    // Run with: cargo test --example no_std_simple --no-default-features
}
