use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum Permissions {
    Read = 1,
    Write = 2,
    Execute = 4,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum FileAttributes {
    Hidden = 1,
    System = 2,
    ReadOnly = 4,
    Archive = 8,
    Directory = 16,
    Compressed = 32,
    Encrypted = 64,
}

#[test]
fn test_bitwise_or() {
    let read_write = Permissions::Read | Permissions::Write;
    assert_eq!(read_write, 3);

    let all_perms = Permissions::Read | Permissions::Write | Permissions::Execute;
    assert_eq!(all_perms, 7);
}

#[test]
fn test_bitwise_and() {
    let read_write = Permissions::Read | Permissions::Write;

    // Check if has Read permission
    assert_eq!(read_write & Permissions::Read as u8, 1);

    // Check if has Execute permission
    assert_eq!(read_write & Permissions::Execute as u8, 0);
}

#[test]
fn test_bitwise_xor() {
    let perms = Permissions::Read | Permissions::Write;

    // Toggle Write permission
    let toggled = perms ^ Permissions::Write as u8;
    assert_eq!(toggled, 1); // Only Read remains

    // Toggle Write again
    let toggled_back = toggled ^ Permissions::Write as u8;
    assert_eq!(toggled_back, 3); // Read and Write
}

#[test]
fn test_not_operator() {
    let perm = Permissions::Read;
    let inverted = !perm;
    assert_eq!(inverted & 0b111, 0b110); // Within 3-bit range, Read bit is off
}

#[test]
fn test_contains_method() {
    let perms = Permissions::Read | Permissions::Execute;

    assert!(Permissions::Read.contains(Permissions::Read));
    assert!(!Permissions::Read.contains(Permissions::Write));

    // Test with combined flags
    let read_u8 = perms;
    let read_perm = Permissions::Read;
    assert_eq!((read_u8 & read_perm as u8), read_perm as u8);
}

#[test]
fn test_from_bits() {
    // Valid combination
    assert_eq!(Permissions::from_bits(7), Some(7)); // Read | Write | Execute
    assert_eq!(Permissions::from_bits(3), Some(3)); // Read | Write

    // Invalid bits (8 is not a valid permission)
    assert_eq!(Permissions::from_bits(8), None);
    assert_eq!(Permissions::from_bits(15), None); // 15 = 1111, includes invalid bit 8
}

#[test]
fn test_serialization_with_flags() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct FileInfo {
        #[bits(8)]
        flags: u8, // Store combined permissions
        size: u32,
    }

    let info = FileInfo {
        flags: Permissions::Read | Permissions::Execute,
        size: 1024,
    };

    let bytes = info.to_be_bytes();
    assert_eq!(bytes.len(), 5); // 1 byte for flags + 4 bytes for u32
    assert_eq!(bytes[0], 5); // Read (1) | Execute (4) = 5

    let (decoded, _) = FileInfo::try_from_be_bytes(&bytes).unwrap();
    assert_eq!(decoded.flags, 5);
    assert_eq!(decoded.size, 1024);
}

#[test]
fn test_large_flag_enum() {
    let attrs = FileAttributes::Hidden | FileAttributes::ReadOnly | FileAttributes::Compressed;
    assert_eq!(attrs, 1 | 4 | 32);
    assert_eq!(attrs, 37);

    // Test from_bits with large values
    assert_eq!(FileAttributes::from_bits(127), Some(127)); // All 7 flags
    assert_eq!(FileAttributes::from_bits(128), None); // Invalid bit
}

#[test]
fn test_flag_enum_in_struct() {
    #[derive(BeBytes, Debug, PartialEq)]
    struct FileMetadata {
        #[bits(7)]
        attributes: u8, // Store FileAttributes flags
        #[bits(1)]
        is_modified: u8,
        name_length: u8,
        #[FromField(name_length)]
        name: Vec<u8>,
    }

    let meta = FileMetadata {
        attributes: FileAttributes::Hidden | FileAttributes::Archive,
        is_modified: 1,
        name_length: 8,
        name: b"test.txt".to_vec(),
    };

    let bytes = meta.to_be_bytes();
    let (decoded, _) = FileMetadata::try_from_be_bytes(&bytes).unwrap();

    assert_eq!(decoded.attributes, 9); // Hidden (1) | Archive (8)
    assert_eq!(decoded.is_modified, 1);
    assert_eq!(decoded.name, b"test.txt");
}
