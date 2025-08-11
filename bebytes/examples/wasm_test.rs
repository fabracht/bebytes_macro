#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
pub struct WasmPacket {
    version: u8,
    #[bits(4)]
    msg_type: u8,
    #[bits(4)]
    flags: u8,
    payload_len: u16,
    #[FromField(payload_len)]
    payload: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn encode_packet(
    version: u8,
    msg_type: u8,
    flags: u8,
    payload_ptr: *const u8,
    payload_len: u16,
) -> *mut u8 {
    unsafe {
        let payload = core::slice::from_raw_parts(payload_ptr, payload_len as usize).to_vec();
        let packet = WasmPacket {
            version,
            msg_type,
            flags,
            payload_len,
            payload,
        };

        let bytes = packet.to_be_bytes();
        let mut boxed = bytes.into_boxed_slice();
        let ptr = boxed.as_mut_ptr();
        core::mem::forget(boxed);
        ptr
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn decode_packet(data_ptr: *const u8, data_len: usize) -> u8 {
    unsafe {
        let data = core::slice::from_raw_parts(data_ptr, data_len);
        match WasmPacket::try_from_be_bytes(data) {
            Ok((packet, _)) => packet.version,
            Err(_) => 0,
        }
    }
}

fn main() {
    // Test in native environment
    let packet = WasmPacket {
        version: 1,
        msg_type: 5,
        flags: 3,
        payload_len: 4,
        payload: vec![1, 2, 3, 4],
    };

    let bytes = packet.to_be_bytes();
    println!("Encoded bytes: {:?}", bytes);

    let (decoded, _) = WasmPacket::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(packet, decoded);
    println!("Successfully decoded packet!");
}
