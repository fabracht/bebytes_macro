#![allow(clippy::assign_op_pattern)]

use bebytes::BeBytes;

fn main() {
    println!("\n=== BeBytes Serialization Test Suite ===");
    println!("\nShowing side-by-side comparison of original vs deserialized values\n");

    demo_endianness();
    demo_bit_fields();
    demo_multi_byte_types();
    demo_enum_serialization();
    demo_enum_bit_packing();
    demo_flag_enums();
    demo_options();
    demo_vectors();
    demo_nested_structs();
    demo_complete_functionality();

    println!("\n=== All tests completed ===");
}

fn print_section(title: &str) {
    println!("\n--- {title} ---");
}

fn print_bytes(bytes: &[u8]) -> String {
    let hex: Vec<String> = bytes.iter().map(|b| format!("0x{b:02X}")).collect();
    format!("[{}] ({} bytes)", hex.join(", "), bytes.len())
}

fn compare_values<T: std::fmt::Debug + PartialEq>(
    name: &str,
    original: &T,
    decoded: &T,
    bytes: &[u8],
) {
    let original_str = format!("{original:#?}");
    let decoded_str = format!("{decoded:#?}");

    println!("\n{name} Comparison:");
    println!("Serialized: {}", print_bytes(bytes));
    println!("\nORIGINAL                          | DECODED");
    println!("---------------------------------+---------------------------------");

    let orig_lines: Vec<&str> = original_str.lines().collect();
    let dec_lines: Vec<&str> = decoded_str.lines().collect();
    let max_lines = orig_lines.len().max(dec_lines.len());

    for i in 0..max_lines {
        let orig = orig_lines.get(i).unwrap_or(&"");
        let dec = dec_lines.get(i).unwrap_or(&"");
        println!("{orig:<33} | {dec:<33}");
    }

    println!(
        "\nMatch: {}",
        if original == decoded {
            "YES ✓"
        } else {
            "NO ✗"
        }
    );
}

fn compare_enum_values<T>(name: &str, original: T, decoded: T)
where
    T: std::fmt::Debug + PartialEq + Copy + BeBytes,
{
    let bytes = original.to_be_bytes();

    println!("\n{name} Comparison:");
    println!("Serialized: {}", print_bytes(&bytes));
    println!("\nORIGINAL: {original:?}");
    println!("DECODED:  {decoded:?}");
    println!(
        "Match: {}",
        if original == decoded {
            "YES ✓"
        } else {
            "NO ✗"
        }
    );
}

fn demo_endianness() {
    print_section("1. ENDIANNESS DEMONSTRATION");

    let original = U16 {
        first: 1,
        second: 16383,
        fourth: 0,
    };

    println!("\nBig-Endian vs Little-Endian:");
    let be_bytes = original.to_be_bytes();
    let le_bytes = original.to_le_bytes();

    println!("BE bytes: {}", print_bytes(&be_bytes));
    println!("LE bytes: {}", print_bytes(&le_bytes));

    let (decoded_be, _) = U16::try_from_be_bytes(&be_bytes).unwrap();
    let (decoded_le, _) = U16::try_from_le_bytes(&le_bytes).unwrap();

    compare_values("U16 (BE)", &original, &decoded_be, &be_bytes);
    compare_values("U16 (LE)", &original, &decoded_le, &le_bytes);
}

fn demo_bit_fields() {
    print_section("2. BIT FIELD DEMONSTRATION");

    let original = U8 {
        first: 1,  // 1 bit
        second: 2, // 3 bits
        third: 3,  // 4 bits
        fourth: 4, // 8 bits (full byte)
    };

    let bytes = original.to_be_bytes();
    let (decoded, _) = U8::try_from_be_bytes(&bytes).unwrap();

    compare_values("U8", &original, &decoded, &bytes);

    println!("\nBit layout breakdown:");
    println!(
        "first  (1 bit):  {:01b} = {}",
        original.first, original.first
    );
    println!(
        "second (3 bits): {:03b} = {}",
        original.second, original.second
    );
    println!(
        "third  (4 bits): {:04b} = {}",
        original.third, original.third
    );
    println!(
        "fourth (8 bits): {:08b} = {}",
        original.fourth, original.fourth
    );

    // Complex bit fields
    let error = ErrorEstimateMini {
        s_bit: 1,
        z_bit: 0,
        scale: 63, // 6 bits: 111111
        multiplier: 3,
    };

    let bytes = error.to_be_bytes();
    let (decoded, _) = ErrorEstimateMini::try_from_be_bytes(&bytes).unwrap();

    compare_values("ErrorEstimateMini", &error, &decoded, &bytes);
}

fn demo_multi_byte_types() {
    print_section("3. MULTI-BYTE TYPE DEMONSTRATION");

    let u16_val = U16 {
        first: 1,
        second: 16383,
        fourth: 0,
    };
    let bytes = u16_val.to_be_bytes();
    let (decoded, _) = U16::try_from_be_bytes(&bytes).unwrap();
    compare_values("U16", &u16_val, &decoded, &bytes);

    let u32_val = U32 {
        first: 1,
        second: 32383,
        fourth: 1,
    };
    let bytes = u32_val.to_be_bytes();
    let (decoded, _) = U32::try_from_be_bytes(&bytes).unwrap();
    compare_values("U32", &u32_val, &decoded, &bytes);
}

fn demo_enum_serialization() {
    print_section("4. ENUM SERIALIZATION");

    println!("\nEnum discriminant mapping:");
    println!("SetupResponse = 0");
    println!("ServerStart   = 1");
    println!("SetupRequest  = 2");

    let original = DummyEnum::ServerStart;
    let bytes = original.to_be_bytes();
    let (decoded, _) = DummyEnum::try_from_be_bytes(&bytes).unwrap();

    compare_enum_values("DummyEnum", original, decoded);

    // Show all variants
    println!("\nAll variants test:");
    for variant in &[
        DummyEnum::SetupResponse,
        DummyEnum::ServerStart,
        DummyEnum::SetupRequest,
    ] {
        let bytes = variant.to_be_bytes();
        let (decoded, _) = DummyEnum::try_from_be_bytes(&bytes).unwrap();
        println!(
            "{:?} -> {} -> {:?} ({})",
            variant,
            print_bytes(&bytes),
            decoded,
            if variant == &decoded { "✓" } else { "✗" }
        );
    }
}

fn demo_enum_bit_packing() {
    print_section("5. ENUM BIT PACKING (Auto-sized)");

    println!("\nMinimum bits required:");
    println!(
        "Status (4 variants)    = {} bits",
        Status::__BEBYTES_MIN_BITS
    );
    println!(
        "Priority (3 variants)  = {} bits",
        Priority::__BEBYTES_MIN_BITS
    );
    println!(
        "LargeEnum (17 variants) = {} bits",
        LargeEnum::__BEBYTES_MIN_BITS
    );

    let original = PacketHeader {
        version: 15,              // 4 bits: 1111
        status: Status::Running,  // 2 bits: 01
        priority: Priority::High, // 2 bits: 10
    };

    let bytes = original.to_be_bytes();
    let (decoded, _) = PacketHeader::try_from_be_bytes(&bytes).unwrap();

    println!("\nPacketHeader Comparison:");
    println!("Serialized: {}", print_bytes(&bytes));
    println!("\nORIGINAL:");
    println!(
        "  version:  {} (0b{:04b})",
        original.version, original.version
    );
    println!("  status:   {:?}", original.status);
    println!("  priority: {:?}", original.priority);
    println!("\nDECODED:");
    println!(
        "  version:  {} (0b{:04b})",
        decoded.version, decoded.version
    );
    println!("  status:   {:?}", decoded.status);
    println!("  priority: {:?}", decoded.priority);
    println!(
        "\nMatch: {}",
        if original.version == decoded.version
            && original.status == decoded.status
            && original.priority == decoded.priority
        {
            "YES ✓"
        } else {
            "NO ✗"
        }
    );
}

fn demo_flag_enums() {
    print_section("6. FLAG ENUMS (Bitwise Operations)");

    println!("\nFlag values:");
    println!(
        "FilePermissions::None    = {} (0b{:08b})",
        FilePermissions::None as u8,
        FilePermissions::None as u8
    );
    println!(
        "FilePermissions::Read    = {} (0b{:08b})",
        FilePermissions::Read as u8,
        FilePermissions::Read as u8
    );
    println!(
        "FilePermissions::Write   = {} (0b{:08b})",
        FilePermissions::Write as u8,
        FilePermissions::Write as u8
    );
    println!(
        "FilePermissions::Execute = {} (0b{:08b})",
        FilePermissions::Execute as u8,
        FilePermissions::Execute as u8
    );
    println!(
        "FilePermissions::Delete  = {} (0b{:08b})",
        FilePermissions::Delete as u8,
        FilePermissions::Delete as u8
    );

    println!("\nFlag combinations:");
    let read_write = FilePermissions::Read | FilePermissions::Write;
    println!("Read | Write = {read_write} (0b{read_write:08b})");

    let all_perms = FilePermissions::Read
        | FilePermissions::Write
        | FilePermissions::Execute
        | FilePermissions::Delete;
    println!("All perms    = {all_perms} (0b{all_perms:08b})");

    let original = SecurityContext {
        user_id: 15,
        group_id: 3,
        permissions: FilePermissions::Read | FilePermissions::Execute,
        network_flags: NetworkFlags::Connected | NetworkFlags::Authenticated,
    };

    let bytes = original.to_be_bytes();
    let (decoded, _) = SecurityContext::try_from_be_bytes(&bytes).unwrap();

    println!("\nSecurityContext Comparison:");
    println!("Serialized: {}", print_bytes(&bytes));
    println!("\nORIGINAL:");
    println!("  user_id:       {}", original.user_id);
    println!("  group_id:      {}", original.group_id);
    println!(
        "  permissions:   {} (0b{:08b})",
        original.permissions, original.permissions
    );
    println!(
        "  network_flags: {} (0b{:08b})",
        original.network_flags, original.network_flags
    );
    println!("\nDECODED:");
    println!("  user_id:       {}", decoded.user_id);
    println!("  group_id:      {}", decoded.group_id);
    println!(
        "  permissions:   {} (0b{:08b})",
        decoded.permissions, decoded.permissions
    );
    println!(
        "  network_flags: {} (0b{:08b})",
        decoded.network_flags, decoded.network_flags
    );
    println!(
        "\nMatch: {}",
        if original.user_id == decoded.user_id
            && original.group_id == decoded.group_id
            && original.permissions == decoded.permissions
            && original.network_flags == decoded.network_flags
        {
            "YES ✓"
        } else {
            "NO ✗"
        }
    );
}

fn demo_options() {
    print_section("7. OPTION TYPE DEMONSTRATION");

    let some_val = Optional {
        optional_number: Some(42),
    };
    let bytes = some_val.to_be_bytes();
    let (decoded, _) = Optional::try_from_be_bytes(&bytes).unwrap();
    compare_values("Optional (Some)", &some_val, &decoded, &bytes);

    let none_val = Optional {
        optional_number: None,
    };
    let bytes = none_val.to_be_bytes();
    let (decoded, _) = Optional::try_from_be_bytes(&bytes).unwrap();
    compare_values("Optional (None)", &none_val, &decoded, &bytes);
}

fn demo_vectors() {
    print_section("8. VECTOR DEMONSTRATION");

    let with_vec = WithTailingVec {
        pre_tail: 3,
        tail: vec![10, 20, 30],
        post_tail: 99,
    };

    let bytes = with_vec.to_be_bytes();
    let (decoded, _) = WithTailingVec::try_from_be_bytes(&bytes).unwrap();
    compare_values("WithTailingVec", &with_vec, &decoded, &bytes);

    let with_size = WithSizeStruct {
        innocent: 1,
        real_tail: vec![2, 3, 4],
    };

    let bytes = with_size.to_be_bytes();
    let (decoded, _) = WithSizeStruct::try_from_be_bytes(&bytes).unwrap();
    compare_values("WithSizeStruct", &with_size, &decoded, &bytes);
}

fn demo_nested_structs() {
    print_section("9. NESTED STRUCT DEMONSTRATION");

    let dummy = DummyStruct {
        dummy0: [0, 2],
        dummy1: 1,
        dummy2: 2,
    };

    let error_estimate = ErrorEstimate {
        s_bit: 1,
        z_bit: 0,
        scale: 63,
        dummy_struct: dummy.clone(),
    };

    let nested = NestedStruct {
        dummy_struct: dummy,
        optional_number: Some(42),
        error_estimate,
    };

    let bytes = nested.to_be_bytes();
    let (decoded, _) = NestedStruct::try_from_be_bytes(&bytes).unwrap();
    compare_values("NestedStruct", &nested, &decoded, &bytes);
}

fn demo_complete_functionality() {
    print_section("10. COMPLETE FUNCTIONALITY EXAMPLE");

    let complete = CompleteFunctionality {
        first: 1,
        second: 2,
        third: 3,
        with_size: vec![6, 7, 8],
        fourth: 4,
        body: vec![5, 4, 3, 2],
        u_16: U16 {
            first: 1,
            second: 2,
            fourth: 1,
        },
        arrayed: ArrayedStruct {
            mode: Modes { bits: 1 },
            key_id: [2],
            token: [3, 3],
            client_iv: [4, 4, 4],
        },
        dummy_enum: DummyEnum::ServerStart,
        optional: Some(5),
        modes: Modes { bits: 3 },
        vecty: WithTailingVec {
            pre_tail: 4,
            tail: vec![1, 2, 3, 4],
            post_tail: 5,
        },
        u_32: U32 {
            first: 1,
            second: 57,
            fourth: 1,
        },
        rattle: vec![1, 2, 3, 4, 5],
    };

    let bytes = complete.to_be_bytes();
    println!("\nComplete struct size: {} bytes", bytes.len());
    println!("First 20 bytes: {:?}", &bytes[..20.min(bytes.len())]);

    let (decoded, _) = CompleteFunctionality::try_from_be_bytes(&bytes).unwrap();

    println!("\nField comparison:");
    println!("first:      {} | {}", complete.first, decoded.first);
    println!("second:     {} | {}", complete.second, decoded.second);
    println!("third:      {} | {}", complete.third, decoded.third);
    println!(
        "with_size:  {:?} | {:?}",
        complete.with_size, decoded.with_size
    );
    println!("fourth:     {} | {}", complete.fourth, decoded.fourth);
    println!("body:       {:?} | {:?}", complete.body, decoded.body);
    println!(
        "dummy_enum: {:?} | {:?}",
        complete.dummy_enum, decoded.dummy_enum
    );
    println!(
        "optional:   {:?} | {:?}",
        complete.optional, decoded.optional
    );
    println!("rattle:     {:?} | {:?}", complete.rattle, decoded.rattle);

    println!(
        "\nMatch: {}",
        if complete == decoded {
            "YES ✓"
        } else {
            "NO ✗"
        }
    );
}

// ============ Test Structures ============

#[derive(BeBytes, Debug, PartialEq)]
struct U8 {
    #[bits(1)]
    first: u8,
    #[bits(3)]
    second: u8,
    #[bits(4)]
    third: u8,
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
struct U16 {
    #[bits(1)]
    first: u8,
    #[bits(14)]
    second: u16,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct U32 {
    #[bits(1)]
    first: u8,
    #[bits(30)]
    second: u32,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct U64 {
    #[bits(1)]
    first: u8,
    #[bits(62)]
    second: u64,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct U128 {
    #[bits(1)]
    first: u8,
    #[bits(126)]
    second: u128,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
struct I8 {
    #[bits(1)]
    first: u8,
    #[bits(6)]
    second: i8,
    #[bits(1)]
    fourth: u8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
pub enum DummyEnum {
    SetupResponse,
    ServerStart,
    SetupRequest,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct DummyStruct {
    pub dummy0: [u8; 2],
    #[bits(1)]
    pub dummy1: u8,
    #[bits(7)]
    pub dummy2: u8,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ErrorEstimate {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub dummy_struct: DummyStruct,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct ErrorEstimateMini {
    #[bits(1)]
    pub s_bit: u8,
    #[bits(1)]
    pub z_bit: u8,
    #[bits(6)]
    pub scale: u8,
    pub multiplier: u32,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct NestedStruct {
    pub dummy_struct: DummyStruct,
    pub optional_number: Option<i32>,
    pub error_estimate: ErrorEstimate,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct Optional {
    pub optional_number: Option<i32>,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct SmallStruct {
    pub small: u8,
}

#[derive(BeBytes, Debug, PartialEq)]
pub struct SmallStructFather {
    small_struct: SmallStruct,
    num1: u32,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
pub struct ArrayedStruct {
    pub mode: Modes,
    pub key_id: [u8; 1],
    pub token: [u8; 2],
    pub client_iv: [u8; 3],
}

#[derive(BeBytes, Debug, PartialEq, Clone, Default)]
pub struct Modes {
    pub bits: u8,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithTailingVec {
    pre_tail: u8,
    #[FromField(pre_tail)]
    tail: Vec<u8>,
    post_tail: u8,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct InnocentStruct {
    innocent: u8,
    real_tail: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, BeBytes)]
struct WithSizeStruct {
    innocent: u8,
    #[With(size(3))]
    real_tail: Vec<u8>,
}

#[derive(BeBytes, Debug, Clone, PartialEq)]
struct DnsNameSegment {
    length: u8,
    #[FromField(length)]
    segment: Vec<u8>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct DnsName {
    segments: Vec<DnsNameSegment>,
}

#[derive(BeBytes, Debug, PartialEq)]
struct CompleteFunctionality {
    #[bits(1)]
    first: u8,
    #[bits(3)]
    second: u8,
    #[bits(4)]
    third: u8,
    #[With(size(3))]
    with_size: Vec<u8>,
    fourth: u8,
    #[FromField(fourth)]
    body: Vec<u8>,
    u_16: U16,
    arrayed: ArrayedStruct,
    dummy_enum: DummyEnum,
    optional: Option<i32>,
    modes: Modes,
    vecty: WithTailingVec,
    u_32: U32,
    rattle: Vec<u8>,
}

// ============ Enum Bit Packing Examples ============

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
enum Status {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
enum Priority {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(BeBytes, Debug, PartialEq)]
struct PacketHeader {
    #[bits(4)]
    version: u8,
    #[bits()] // Auto-sized to Status::__BEBYTES_MIN_BITS (2 bits)
    status: Status,
    #[bits()] // Auto-sized to Priority::__BEBYTES_MIN_BITS (2 bits)
    priority: Priority,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
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

#[derive(BeBytes, Debug, PartialEq)]
struct ComplexPacket {
    #[bits(3)]
    flags: u8,
    #[bits()] // Auto-sized to LargeEnum::__BEBYTES_MIN_BITS (5 bits)
    large_enum: LargeEnum,
    payload_size: u16,
    #[FromField(payload_size)]
    payload: Vec<u8>,
}

// ============ Flag Enum Examples ============

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum FilePermissions {
    None = 0,
    Read = 1,
    Write = 2,
    Execute = 4,
    Delete = 8,
}

#[derive(BeBytes, Debug, PartialEq, Copy, Clone)]
#[bebytes(flags)]
#[repr(u8)]
enum NetworkFlags {
    Connected = 1,
    Authenticated = 2,
    Encrypted = 4,
    Compressed = 8,
    KeepAlive = 16,
}

#[derive(BeBytes, Debug, PartialEq)]
struct SecurityContext {
    #[bits(5)]
    user_id: u8,
    #[bits(3)]
    group_id: u8,
    permissions: u8,   // Store FilePermissions flags
    network_flags: u8, // Store NetworkFlags
}
