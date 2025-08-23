# Marker Attributes Documentation

This document describes the marker-based field delimiting features in BeBytes, which enable efficient handling of protocols that use sentinel bytes to separate variable-length data sections.

## Overview

BeBytes provides two marker attributes for handling delimited data:
- `#[UntilMarker(byte_or_char)]` - Reads bytes until a marker is encountered
- `#[AfterMarker(byte_or_char)]` - Skips bytes until finding a marker, then reads remaining data

Markers can be specified as:
- Byte values: `0xFF`, `255`, `0x00`
- ASCII character literals: `'\n'`, `'\0'`, `'\t'`, `'\r'`

## UntilMarker Attribute

### Basic Usage

The `#[UntilMarker(byte_or_char)]` attribute reads bytes into a `Vec<u8>` until the specified marker is found:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct Message {
    header: u32,
    #[UntilMarker(0xFF)]  // Using byte value
    content: Vec<u8>,
    #[UntilMarker('\n')]  // Using character literal
    line: Vec<u8>,
    footer: u16,
}
```

### Behavior

- **Reading**: Consumes bytes until the marker is found
- **Marker handling**: The marker byte is consumed but not included in the field
- **Missing marker**: If no marker is found, reads all remaining bytes
- **Writing**: Appends the marker byte after the field data

### Example: Protocol with Delimited Sections

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct NetworkPacket {
    packet_type: u8,
    #[UntilMarker('\0')]  // Null-terminated using character literal
    sender_name: Vec<u8>,
    #[UntilMarker('\0')]  // Null-terminated using character literal
    receiver_name: Vec<u8>,
    message_length: u16,
    #[FromField(message_length)]
    message: Vec<u8>,
}

let packet = NetworkPacket {
    packet_type: 1,
    sender_name: b"alice".to_vec(),
    receiver_name: b"bob".to_vec(),
    message_length: 5,
    message: b"hello".to_vec(),
};

let bytes = packet.to_be_bytes();
// Result: [1]["alice"][0x00]["bob"][0x00][0,5]["hello"]
```

## AfterMarker Attribute

### Basic Usage

The `#[AfterMarker(byte_or_char)]` attribute skips bytes until finding the marker, then reads all remaining bytes:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct DataPacket {
    version: u8,
    flags: u8,
    #[AfterMarker(0xDE)]  // Using byte value
    payload: Vec<u8>,
}

// Or using character literal
#[derive(BeBytes, Debug, PartialEq)]
struct TabDelimited {
    header: u8,
    #[AfterMarker('\t')]  // Skip until tab character
    content: Vec<u8>,
}
```

### Behavior

- **Reading**: Skips bytes until marker is found, then reads all bytes after the marker
- **Marker handling**: The marker byte itself is consumed but not included
- **Missing marker**: If no marker is found, the field becomes empty
- **Writing**: Writes the marker byte followed by the field data

### Example: Skip Header Pattern

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct SkipHeaderMessage {
    msg_id: u16,
    #[AfterMarker(0xBE)]  // Skip variable header until 0xBE
    body: Vec<u8>,
}

// Input bytes: [0x00,0x01][header bytes...][0xBE][actual body data]
// The header bytes are skipped, only body data is captured
```

## Vec<Vec<u8>> with Markers

### Multiple Delimited Sections

For protocols with repeated delimited sections, combine `Vec<Vec<u8>>` with `#[UntilMarker]`:

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct MultiSection {
    section_count: u8,
    #[FromField(section_count)]
    #[UntilMarker(0xFF)]
    sections: Vec<Vec<u8>>,  // Multiple sections, each ending with 0xFF
}
```

### Requirements

- **Size control required**: Must use either:
  - `#[With(size(N))]` for fixed number of sections
  - `#[FromField(field_name)]` for dynamic section count
- **Only UntilMarker**: `AfterMarker` is not supported with `Vec<Vec<u8>>`
- **Each section delimited**: Each inner `Vec<u8>` is terminated by the marker

### Example: CoAP-like Options

```rust
#[derive(BeBytes, Debug, PartialEq)]
struct CoapMessage {
    version: u8,
    msg_type: u8,
    token_length: u8,
    code: u8,
    message_id: u16,
    
    #[FromField(token_length)]
    token: Vec<u8>,
    
    option_count: u8,
    #[FromField(option_count)]
    #[UntilMarker(0xFF)]
    options: Vec<Vec<u8>>,  // Multiple options, each terminated by 0xFF
    
    payload_marker: u8,  // Should be 0xFF
    payload: Vec<u8>,    // Remaining bytes
}
```

## Edge Cases

### Missing Markers

When markers are missing:
- `UntilMarker`: Reads all available bytes for that segment
- `AfterMarker`: Results in an empty field
- `Vec<Vec<u8>>`: Missing markers create empty segments

### Marker in Data

If the marker byte appears in actual data:
- `UntilMarker`: Terminates reading at that point
- `AfterMarker`: Skips to first occurrence
- Consider escaping mechanisms if marker bytes can appear in data

### Empty Segments

Empty segments are valid:
```rust
let msg = MultiSection {
    section_count: 3,
    sections: vec![
        vec![1, 2],
        vec![],      // Empty segment (just marker in serialized form)
        vec![3, 4],
    ],
};
// Serialized: [3][1,2,0xFF][0xFF][3,4,0xFF]
```

## Performance Considerations

- **Linear scanning**: Both attributes scan bytes linearly
- **No backtracking**: Once a marker is found, parsing continues forward
- **Memory allocation**: Each segment creates a new `Vec<u8>` allocation
- **Optimal for**: Protocols where markers are guaranteed not to appear in data

## Common Use Cases

### 1. Null-Terminated Strings
```rust
#[UntilMarker('\0')]  // Using character literal
name: Vec<u8>,  // C-style null-terminated string

// Or using byte value
#[UntilMarker(0x00)]
name2: Vec<u8>,
```

### 2. Line-Based Protocols
```rust
#[UntilMarker('\n')]  // Using character literal for newline
line: Vec<u8>,

// Or using byte value
#[UntilMarker(0x0A)]
line2: Vec<u8>,
```

### 3. TLV with Delimiters
```rust
#[UntilMarker(0xFE)]
tlv_value: Vec<u8>,
```

### 4. Multi-Part Messages
```rust
#[FromField(part_count)]
#[UntilMarker(0x7E)]
parts: Vec<Vec<u8>>,  // Multiple parts with 0x7E delimiter
```

## Limitations

1. **Vec<u8> only**: Marker attributes only work with `Vec<u8>` fields
2. **No escape sequences**: No built-in support for escaping marker bytes in data
3. **AfterMarker with Vec<Vec<u8>>**: Not supported due to ambiguous semantics
4. **Single byte markers**: Only single-byte markers are supported
5. **ASCII characters only**: Character literals must be ASCII (value <= 127)

## Best Practices

1. **Choose unique markers**: Select marker bytes unlikely to appear in data
2. **Document marker choice**: Clearly document why specific markers were chosen
3. **Validate on write**: Ensure data doesn't contain marker bytes if that would cause issues
4. **Consider alternatives**: For binary data that might contain any byte value, consider length-prefixed fields instead
5. **Test edge cases**: Always test with missing markers, empty segments, and marker bytes in data