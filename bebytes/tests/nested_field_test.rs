use bebytes::BeBytes;

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct DnsHeader {
    #[bits(16)]
    transaction_id: u16,
    #[bits(1)]
    qr: u8,
    #[bits(4)]
    opcode: u8,
    #[bits(1)]
    aa: u8,
    #[bits(1)]
    tc: u8,
    #[bits(1)]
    rd: u8,
    #[bits(1)]
    ra: u8,
    #[bits(3)]
    z: u8,
    #[bits(4)]
    rcode: u8,
    #[bits(16)]
    qdcount: u16,
    #[bits(16)]
    ancount: u16,
    #[bits(16)]
    nscount: u16,
    #[bits(16)]
    arcount: u16,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct DnsQuestion {
    name: u8, // Simplified for test
    qtype: u16,
    qclass: u16,
}

#[derive(BeBytes, Debug, PartialEq, Clone)]
struct DnsPacket {
    header: DnsHeader,
    #[FromField(header.qdcount)]
    questions: Vec<DnsQuestion>,
    #[FromField(header.ancount)]
    answers: Vec<DnsQuestion>, // Reusing DnsQuestion for simplicity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_field_access() {
        let packet = DnsPacket {
            header: DnsHeader {
                transaction_id: 0x1234,
                qr: 0,
                opcode: 0,
                aa: 0,
                tc: 0,
                rd: 1,
                ra: 0,
                z: 0,
                rcode: 0,
                qdcount: 2,
                ancount: 1,
                nscount: 0,
                arcount: 0,
            },
            questions: vec![
                DnsQuestion {
                    name: 3,
                    qtype: 1,
                    qclass: 1,
                },
                DnsQuestion {
                    name: 4,
                    qtype: 1,
                    qclass: 1,
                },
            ],
            answers: vec![DnsQuestion {
                name: 5,
                qtype: 1,
                qclass: 1,
            }],
        };

        // Serialize
        let bytes = packet.to_be_bytes();

        // Deserialize
        let (deserialized, _) = DnsPacket::try_from_be_bytes(&bytes).unwrap();

        // Verify
        assert_eq!(packet, deserialized);
        assert_eq!(deserialized.questions.len(), 2);
        assert_eq!(deserialized.answers.len(), 1);
    }

    #[test]
    fn test_deeply_nested_field_access() {
        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct Counts {
            #[bits(16)]
            ancount: u16,
            #[bits(16)]
            qdcount: u16,
        }

        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct NestedHeader {
            #[bits(16)]
            flags: u16,
            counts: Counts,
        }

        #[derive(BeBytes, Debug, PartialEq, Clone)]
        struct ComplexPacket {
            header: NestedHeader,
            #[FromField(header.counts.ancount)]
            answers: Vec<u8>,
            #[FromField(header.counts.qdcount)]
            questions: Vec<u8>,
        }

        let packet = ComplexPacket {
            header: NestedHeader {
                flags: 0x8180,
                counts: Counts {
                    ancount: 3,
                    qdcount: 2,
                },
            },
            answers: vec![1, 2, 3],
            questions: vec![4, 5],
        };

        let bytes = packet.to_be_bytes();
        let (deserialized, _) = ComplexPacket::try_from_be_bytes(&bytes).unwrap();

        assert_eq!(packet, deserialized);
        assert_eq!(deserialized.answers.len(), 3);
        assert_eq!(deserialized.questions.len(), 2);
    }
}

