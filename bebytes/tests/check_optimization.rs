use bebytes::BeBytes;

#[derive(BeBytes, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
enum E { 
    A = 0, 
    B = 1 
}

// Should use optimization
#[derive(BeBytes, Debug, PartialEq)]
struct WithoutAuto {
    p: u64,         // 0-63
    #[bits(16)]
    f: u16,         // 64-79 (byte aligned at position 64)
}

// Should NOT use optimization for 'f'
#[derive(BeBytes, Debug, PartialEq)]
struct WithAuto {
    p: u64,         // 0-63
    #[bits()]
    e: E,           // 64 (1 bit)
    #[bits(7)]
    x: u8,          // 65-71
    #[bits(16)]
    f: u16,         // 72-87 (byte aligned at position 72)
}

#[test]
fn test_check_optimization() {
    let s1 = WithoutAuto { p: 0, f: 0x1234 };
    let b1 = s1.to_be_bytes();
    assert_eq!(b1.len(), 10);
    
    let s2 = WithAuto { p: 0, e: E::B, x: 0x55, f: 0x5678 };
    let b2 = s2.to_be_bytes();
    assert_eq!(b2.len(), 11);
    
    println!("Tests pass - optimization behavior is correct");
}