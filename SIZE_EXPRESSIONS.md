# Size Expressions in BeBytes

BeBytes now supports dynamic field sizing using mathematical expressions and field references. This enables efficient binary protocol implementations where field sizes depend on other fields in the struct.

## Overview

Size expressions allow you to specify the size of `Vec<u8>` and `String` fields using:
- **Mathematical operations**: `+`, `-`, `*`, `/`, `%`
- **Field references**: Reference other fields in the same struct
- **Literal values**: Constant integers
- **Parentheses**: For grouping expressions

## Syntax

Use the `#[With(size(expression))]` attribute to specify dynamic field sizes:

```rust
use bebytes_derive::BeBytes;

#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    count: u8,
    #[With(size(count * 4))]  // Size = count × 4 bytes
    data: Vec<u8>,
}
```

## Supported Expressions

### Mathematical Operations

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct MathOperations {
    base: u8,
    multiplier: u8,
    
    #[With(size(base + 10))]          // Addition
    add_data: Vec<u8>,
    
    #[With(size(base * multiplier))]  // Multiplication
    mult_data: Vec<u8>,
    
    #[With(size(100 / base))]         // Division
    div_data: Vec<u8>,
    
    #[With(size(base % 4))]           // Modulo
    mod_data: Vec<u8>,
}
```

### Field References

Reference any previously defined field in the struct:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct FieldReferences {
    header_length: u16,
    payload_size: u32,
    
    #[With(size(header_length))]
    header: Vec<u8>,
    
    #[With(size(payload_size))]
    payload: Vec<u8>,
}
```

### Complex Expressions

Combine operations with parentheses for complex calculations:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct ComplexExpressions {
    width: u8,
    height: u8,
    bytes_per_pixel: u8,
    
    // Calculate image buffer size: width × height × bytes_per_pixel
    #[With(size((width * height) * bytes_per_pixel))]
    image_data: Vec<u8>,
    
    // Header size with padding
    #[With(size(width + height + 16))]
    header_data: Vec<u8>,
}
```

### String Fields

Size expressions work with both `Vec<u8>` and `String` fields:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct StringMessage {
    name_length: u8,
    message_length: u16,
    
    #[With(size(name_length))]
    name: String,
    
    #[With(size(message_length))]
    message: String,
}
```

## Protocol Examples

### IPv4 Packet

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Ipv4Packet {
    version: u8,
    header_length: u8,
    type_of_service: u8,
    total_length: u16,
    identification: u16,
    flags_and_fragment: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    
    // IPv4 addresses are always 4 bytes
    #[With(size(4))]
    source_address: Vec<u8>,
    
    #[With(size(4))]
    dest_address: Vec<u8>,
}
```

### DNS Message

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct DnsMessage {
    id: u16,
    flags: u16,
    question_count: u16,
    answer_count: u16,
    authority_count: u16,
    additional_count: u16,
    
    // Variable-length sections based on counts
    #[With(size(question_count * 5))]  // ~5 bytes per question
    questions: Vec<u8>,
    
    #[With(size(answer_count * 12))]   // ~12 bytes per answer
    answers: Vec<u8>,
}
```

### MQTT Packet

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct MqttPacket {
    fixed_header: u8,
    remaining_length: u8,
    
    // Payload size determined by remaining length
    #[With(size(remaining_length))]
    payload: Vec<u8>,
}
```

## Usage Example

```rust
use bebytes_derive::BeBytes;
use bebytes::BeBytes as _; // Import trait

#[derive(BeBytes, Debug, PartialEq)]
struct NetworkMessage {
    header_size: u8,
    payload_count: u16,
    
    #[With(size(header_size))]
    header: Vec<u8>,
    
    #[With(size(payload_count * 8))]
    payload: Vec<u8>,
}

fn main() {
    let msg = NetworkMessage {
        header_size: 16,
        payload_count: 3,
        header: vec![0; 16],           // 16 bytes
        payload: vec![1; 24],          // 3 × 8 = 24 bytes
    };
    
    // Serialize to bytes
    let bytes = msg.to_be_bytes();
    
    // Deserialize back
    let (parsed, _) = NetworkMessage::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(msg, parsed);
    
    println!(\"Serialized {} bytes\", bytes.len());
}
```

## Important Notes

### Field Ordering
- Fields referenced in size expressions must be defined **before** the fields that use them
- Variable-size fields (`Vec<u8>`, `String`) should generally be placed toward the end of structs

### Validation
- Size mismatches during serialization will panic with descriptive error messages
- Insufficient data during deserialization returns `BeBytesError::InsufficientData`

### Performance
- Size expressions are evaluated at runtime during serialization/deserialization
- The compile-time `field_size()` returns 0 for variable-size fields
- No significant performance overhead compared to manual size calculations

### Error Handling
```rust
// Size mismatch will panic during serialization
let bad_msg = NetworkMessage {
    header_size: 16,
    payload_count: 3,
    header: vec![0; 10],  // Wrong size! Expected 16 bytes
    payload: vec![1; 24],
};

// This will panic:
// let bytes = bad_msg.to_be_bytes(); // panic: Vector size 10 does not match expected size 16

// Insufficient data returns error during deserialization
let short_bytes = vec![1, 2, 3]; // Not enough data
let result = NetworkMessage::try_from_be_bytes(&short_bytes);
assert!(result.is_err());
```

## Current Limitations

- **Conditional expressions**: `if-else` expressions are not yet fully supported due to token parsing complexity
- **Nested field access**: Currently limited to direct field references (no `header.length` style access)
- **Type restrictions**: Only supported for `Vec<u8>` and `String` fields

## Future Enhancements

- Support for conditional expressions: `#[With(size(if version == 4 { 4 } else { 16 }))]`
- Nested field access: `#[With(size(header.length))]`
- More complex expressions and built-in functions
- Compile-time size validation where possible

## Migration from Fixed Sizes

Old fixed-size syntax still works:
```rust
#[With(size(16))]  // Fixed 16 bytes
data: Vec<u8>,
```

New expression syntax enables dynamic sizing:
```rust
#[With(size(length_field))]  // Dynamic size based on field
data: Vec<u8>,
```