//! Tests for the string interpreter functionality

use bebytes::{BeBytes, StringInterpreter, Utf8};

#[test]
fn test_direct_interpreter_usage() {
    // Test using the interpreter directly
    let text = "Hello, BeBytes!";
    let bytes = Utf8::to_bytes(text);
    let result = Utf8::from_bytes(bytes).unwrap();
    assert_eq!(result, text);
}

#[test]
fn test_struct_with_interpreter() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct Message {
        id: u32,
        len: u8,
        #[FromField(len)]
        content: String,
    }

    let msg = Message {
        id: 42,
        len: 13,
        content: "Hello, world!".to_string(),
    };

    // Serialize
    let bytes = msg.to_be_bytes();
    
    // Deserialize
    let (decoded, _) = Message::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded, msg);
}

// Example of a custom interpreter that could be used in the future
struct Base64Interpreter;

impl StringInterpreter for Base64Interpreter {
    fn from_bytes(bytes: &[u8]) -> Result<String, bebytes::BeBytesError> {
        // This is just a placeholder - in real implementation you'd decode base64
        // For now, just treat it as UTF-8
        Utf8::from_bytes(bytes)
    }
    
    fn to_bytes(s: &str) -> &[u8] {
        // This is just a placeholder - in real implementation you'd encode to base64
        // For now, just return raw bytes
        s.as_bytes()
    }
}

#[test]
fn test_custom_interpreter_concept() {
    // This demonstrates how a custom interpreter would work
    let text = "Custom encoding test";
    let bytes = Base64Interpreter::to_bytes(text);
    let result = Base64Interpreter::from_bytes(bytes).unwrap();
    assert_eq!(result, text);
}