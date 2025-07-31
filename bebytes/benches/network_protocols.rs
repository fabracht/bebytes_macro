use bebytes::BeBytes;
use bytes::BytesMut;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};

// ============ Network Protocol Structures ============

// MQTT v5 Connect Packet
#[derive(BeBytes, Clone, Debug)]
struct MqttConnectPacket {
    #[bits(4)]
    packet_type: u8,
    #[bits(4)]
    flags: u8,
    remaining_length: u8,
    protocol_name_length: u16,
    #[With(size(4))]
    protocol_name: Vec<u8>, // "MQTT"
    protocol_version: u8,
    connect_flags: u8,
    keep_alive: u16,
    client_id_length: u16,
    #[FromField(client_id_length)]
    client_id: Vec<u8>,
}

// HTTP/2 Frame Header
#[derive(BeBytes, Clone, Debug)]
struct Http2FrameHeader {
    #[bits(24)]
    length: u32,
    frame_type: u8,
    flags: u8,
    #[bits(1)]
    reserved: u8,
    #[bits(31)]
    stream_id: u32,
}

// DNS Query Header
#[derive(BeBytes, Clone, Debug)]
struct DnsHeader {
    transaction_id: u16,
    #[bits(1)]
    query_response: u8,
    #[bits(4)]
    opcode: u8,
    #[bits(1)]
    authoritative: u8,
    #[bits(1)]
    truncated: u8,
    #[bits(1)]
    recursion_desired: u8,
    #[bits(1)]
    recursion_available: u8,
    #[bits(3)]
    reserved: u8,
    #[bits(4)]
    response_code: u8,
    question_count: u16,
    answer_count: u16,
    authority_count: u16,
    additional_count: u16,
}

// TCP Header
#[derive(BeBytes, Clone, Debug)]
struct TcpHeader {
    source_port: u16,
    destination_port: u16,
    sequence_number: u32,
    acknowledgment_number: u32,
    #[bits(4)]
    data_offset: u8,
    #[bits(6)]
    reserved: u8,
    #[bits(1)]
    urg: u8,
    #[bits(1)]
    ack: u8,
    #[bits(1)]
    psh: u8,
    #[bits(1)]
    rst: u8,
    #[bits(1)]
    syn: u8,
    #[bits(1)]
    fin: u8,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}

// UDP Header
#[derive(BeBytes, Clone, Debug)]
struct UdpHeader {
    source_port: u16,
    destination_port: u16,
    length: u16,
    checksum: u16,
}

// Ethernet Frame Header
#[derive(BeBytes, Clone, Debug)]
struct EthernetHeader {
    destination_mac: [u8; 6],
    source_mac: [u8; 6],
    ethertype: u16,
}

// IPv4 Header
#[derive(BeBytes, Clone, Debug)]
struct Ipv4Header {
    #[bits(4)]
    version: u8,
    #[bits(4)]
    ihl: u8,
    #[bits(6)]
    dscp: u8,
    #[bits(2)]
    ecn: u8,
    total_length: u16,
    identification: u16,
    #[bits(3)]
    flags: u8,
    #[bits(13)]
    fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    header_checksum: u16,
    source_ip: u32,
    destination_ip: u32,
}

// TLS Record Header
#[derive(BeBytes, Clone, Debug)]
struct TlsRecordHeader {
    content_type: u8,
    protocol_version: u16,
    length: u16,
}

// WebSocket Frame Header
#[derive(BeBytes, Clone, Debug)]
struct WebSocketFrameHeader {
    #[bits(1)]
    fin: u8,
    #[bits(3)]
    reserved: u8,
    #[bits(4)]
    opcode: u8,
    #[bits(1)]
    mask: u8,
    #[bits(7)]
    payload_length: u8,
}

// CoAP Message Header
#[derive(BeBytes, Clone, Debug)]
struct CoapHeader {
    #[bits(2)]
    version: u8,
    #[bits(2)]
    message_type: u8,
    #[bits(4)]
    token_length: u8,
    code: u8,
    message_id: u16,
}

// ============ Protocol Data Generators ============

fn create_mqtt_connect() -> MqttConnectPacket {
    MqttConnectPacket {
        packet_type: 1,
        flags: 0,
        remaining_length: 20,
        protocol_name_length: 4,
        protocol_name: b"MQTT".to_vec(),
        protocol_version: 5,
        connect_flags: 0x02,
        keep_alive: 60,
        client_id_length: 8,
        client_id: b"client01".to_vec(),
    }
}

fn create_http2_frame() -> Http2FrameHeader {
    Http2FrameHeader {
        length: 1024,
        frame_type: 0, // DATA
        flags: 0x01,   // END_STREAM
        reserved: 0,
        stream_id: 1,
    }
}

fn create_dns_header() -> DnsHeader {
    DnsHeader {
        transaction_id: 0x1234,
        query_response: 0,
        opcode: 0,
        authoritative: 0,
        truncated: 0,
        recursion_desired: 1,
        recursion_available: 0,
        reserved: 0,
        response_code: 0,
        question_count: 1,
        answer_count: 0,
        authority_count: 0,
        additional_count: 0,
    }
}

fn create_tcp_header() -> TcpHeader {
    TcpHeader {
        source_port: 80,
        destination_port: 8080,
        sequence_number: 0x12345678,
        acknowledgment_number: 0x87654321,
        data_offset: 5,
        reserved: 0,
        urg: 0,
        ack: 1,
        psh: 0,
        rst: 0,
        syn: 0,
        fin: 0,
        window_size: 65535,
        checksum: 0xABCD,
        urgent_pointer: 0,
    }
}

fn create_udp_header() -> UdpHeader {
    UdpHeader {
        source_port: 53,
        destination_port: 12345,
        length: 32,
        checksum: 0x1234,
    }
}

fn create_ethernet_header() -> EthernetHeader {
    EthernetHeader {
        destination_mac: [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        source_mac: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66],
        ethertype: 0x0800, // IPv4
    }
}

fn create_ipv4_header() -> Ipv4Header {
    Ipv4Header {
        version: 4,
        ihl: 5,
        dscp: 0,
        ecn: 0,
        total_length: 60,
        identification: 0x1234,
        flags: 2, // Don't fragment
        fragment_offset: 0,
        ttl: 64,
        protocol: 6, // TCP
        header_checksum: 0xABCD,
        source_ip: 0xC0A80001,      // 192.168.0.1
        destination_ip: 0xC0A80002, // 192.168.0.2
    }
}

fn create_tls_record() -> TlsRecordHeader {
    TlsRecordHeader {
        content_type: 22,         // Handshake
        protocol_version: 0x0303, // TLS 1.2
        length: 256,
    }
}

fn create_websocket_frame() -> WebSocketFrameHeader {
    WebSocketFrameHeader {
        fin: 1,
        reserved: 0,
        opcode: 1, // Text frame
        mask: 1,
        payload_length: 125,
    }
}

fn create_coap_header() -> CoapHeader {
    CoapHeader {
        version: 1,
        message_type: 0, // Confirmable
        token_length: 4,
        code: 1, // GET
        message_id: 0x1234,
    }
}

// ============ Protocol-Specific Benchmarks ============

fn bench_mqtt_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("mqtt_protocol");

    let connect_packet = create_mqtt_connect();
    group.throughput(Throughput::Bytes(MqttConnectPacket::field_size() as u64));

    group.bench_function("mqtt_connect_serialization", |b| {
        b.iter(|| black_box(connect_packet.to_be_bytes()))
    });

    group.bench_function("mqtt_connect_bytes_buf", |b| {
        b.iter(|| black_box(connect_packet.to_be_bytes_buf()))
    });

    // Test deserialization
    let serialized = connect_packet.to_be_bytes();
    group.bench_function("mqtt_connect_deserialization", |b| {
        b.iter(|| {
            let (decoded, _) =
                MqttConnectPacket::try_from_be_bytes(black_box(&serialized)).unwrap();
            black_box(decoded);
        })
    });

    group.finish();
}

fn bench_http2_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("http2_protocol");

    let frame_header = create_http2_frame();
    group.throughput(Throughput::Bytes(Http2FrameHeader::field_size() as u64));

    group.bench_function("http2_frame_serialization", |b| {
        b.iter(|| black_box(frame_header.to_be_bytes()))
    });

    // HTTP/2 frame header has bit fields, so raw pointer optimization is not available
    group.bench_function("http2_frame_direct_bufmut", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(Http2FrameHeader::field_size()),
            |mut buf| {
                black_box(frame_header.encode_be_to(&mut buf).unwrap());
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_dns_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("dns_protocol");

    let dns_header = create_dns_header();
    group.throughput(Throughput::Bytes(DnsHeader::field_size() as u64));

    group.bench_function("dns_header_serialization", |b| {
        b.iter(|| black_box(dns_header.to_be_bytes()))
    });

    group.bench_function("dns_header_bytes_buf", |b| {
        b.iter(|| black_box(dns_header.to_be_bytes_buf()))
    });

    group.finish();
}

fn bench_tcp_udp_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport_protocols");

    let tcp_header = create_tcp_header();
    let udp_header = create_udp_header();

    group.throughput(Throughput::Bytes(
        (TcpHeader::field_size() + UdpHeader::field_size()) as u64,
    ));

    group.bench_function("tcp_header_serialization", |b| {
        b.iter(|| black_box(tcp_header.to_be_bytes()))
    });

    group.bench_function("udp_header_serialization", |b| {
        b.iter(|| black_box(udp_header.to_be_bytes()))
    });

    // TCP header has bit fields, so raw pointer optimization is not available
    group.bench_function("tcp_header_direct_bufmut", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(TcpHeader::field_size()),
            |mut buf| {
                black_box(tcp_header.encode_be_to(&mut buf).unwrap());
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    if UdpHeader::supports_raw_pointer_encoding() {
        group.bench_function("udp_header_raw_pointer", |b| {
            b.iter(|| black_box(udp_header.encode_be_to_raw_stack()))
        });
    }

    group.finish();
}

fn bench_layered_protocols(c: &mut Criterion) {
    let mut group = c.benchmark_group("layered_protocols");

    let ethernet = create_ethernet_header();
    let ipv4 = create_ipv4_header();
    let tcp = create_tcp_header();

    // Simulate a complete packet stack
    group.throughput(Throughput::Bytes(
        (EthernetHeader::field_size() + Ipv4Header::field_size() + TcpHeader::field_size()) as u64,
    ));

    group.bench_function("layered_individual_serialization", |b| {
        b.iter(|| {
            let eth_bytes = ethernet.to_be_bytes();
            let ip_bytes = ipv4.to_be_bytes();
            let tcp_bytes = tcp.to_be_bytes();
            black_box((eth_bytes, ip_bytes, tcp_bytes));
        })
    });

    group.bench_function("layered_single_buffer", |b| {
        b.iter_batched(
            || {
                BytesMut::with_capacity(
                    EthernetHeader::field_size()
                        + Ipv4Header::field_size()
                        + TcpHeader::field_size(),
                )
            },
            |mut buf| {
                ethernet.encode_be_to(&mut buf).unwrap();
                ipv4.encode_be_to(&mut buf).unwrap();
                tcp.encode_be_to(&mut buf).unwrap();
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_secure_protocols(c: &mut Criterion) {
    let mut group = c.benchmark_group("secure_protocols");

    let tls_record = create_tls_record();
    let websocket = create_websocket_frame();

    group.throughput(Throughput::Bytes(
        (TlsRecordHeader::field_size() + WebSocketFrameHeader::field_size()) as u64,
    ));

    group.bench_function("tls_record_serialization", |b| {
        b.iter(|| black_box(tls_record.to_be_bytes()))
    });

    group.bench_function("websocket_frame_serialization", |b| {
        b.iter(|| black_box(websocket.to_be_bytes()))
    });

    if TlsRecordHeader::supports_raw_pointer_encoding() {
        group.bench_function("tls_record_raw_pointer", |b| {
            b.iter(|| black_box(tls_record.encode_be_to_raw_stack()))
        });
    }

    group.finish();
}

fn bench_iot_protocols(c: &mut Criterion) {
    let mut group = c.benchmark_group("iot_protocols");

    let coap_header = create_coap_header();

    group.throughput(Throughput::Bytes(CoapHeader::field_size() as u64));

    group.bench_function("coap_header_serialization", |b| {
        b.iter(|| black_box(coap_header.to_be_bytes()))
    });

    // CoAP header has bit fields, so raw pointer optimization is not available
    group.bench_function("coap_header_direct_bufmut", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(CoapHeader::field_size()),
            |mut buf| {
                black_box(coap_header.encode_be_to(&mut buf).unwrap());
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_protocol_parsing_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol_pipeline");

    // Simulate parsing multiple protocol headers in sequence
    let protocols = vec![
        (create_ethernet_header().to_be_bytes(), "ethernet"),
        (create_ipv4_header().to_be_bytes(), "ipv4"),
        (create_tcp_header().to_be_bytes(), "tcp"),
        (create_tls_record().to_be_bytes(), "tls"),
        (create_http2_frame().to_be_bytes(), "http2"),
    ];

    group.throughput(Throughput::Elements(protocols.len() as u64));

    group.bench_function("sequential_parsing", |b| {
        b.iter(|| {
            for (bytes, _name) in &protocols {
                // Simulate protocol detection and parsing
                if bytes.len() == EthernetHeader::field_size() {
                    let (decoded, _) = EthernetHeader::try_from_be_bytes(bytes).unwrap();
                    black_box(decoded);
                } else if bytes.len() == Ipv4Header::field_size() {
                    let (decoded, _) = Ipv4Header::try_from_be_bytes(bytes).unwrap();
                    black_box(decoded);
                } else if bytes.len() == TcpHeader::field_size() {
                    let (decoded, _) = TcpHeader::try_from_be_bytes(bytes).unwrap();
                    black_box(decoded);
                } else if bytes.len() == TlsRecordHeader::field_size() {
                    let (decoded, _) = TlsRecordHeader::try_from_be_bytes(bytes).unwrap();
                    black_box(decoded);
                } else if bytes.len() == Http2FrameHeader::field_size() {
                    let (decoded, _) = Http2FrameHeader::try_from_be_bytes(bytes).unwrap();
                    black_box(decoded);
                }
            }
        })
    });

    group.finish();
}

fn bench_packet_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    // Simulate processing a batch of UDP packets
    let udp_packets: Vec<UdpHeader> = (0..1000)
        .map(|i| UdpHeader {
            source_port: (i % 65536) as u16,
            destination_port: ((i + 1000) % 65536) as u16,
            length: 64,
            checksum: (i % 65536) as u16,
        })
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("batch_individual_allocation", |b| {
        b.iter(|| {
            let results: Vec<Vec<u8>> = udp_packets
                .iter()
                .map(|packet| packet.to_be_bytes())
                .collect();
            black_box(results);
        })
    });

    group.bench_function("batch_shared_buffer", |b| {
        b.iter_batched(
            || BytesMut::with_capacity(UdpHeader::field_size() * 1000),
            |mut buf| {
                for packet in &udp_packets {
                    packet.encode_be_to(&mut buf).unwrap();
                }
                black_box(buf);
            },
            BatchSize::SmallInput,
        )
    });

    if UdpHeader::supports_raw_pointer_encoding() {
        group.bench_function("batch_raw_pointer", |b| {
            b.iter(|| {
                let results: Vec<[u8; 8]> = udp_packets
                    .iter()
                    .map(|packet| packet.encode_be_to_raw_stack())
                    .collect();
                black_box(results);
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_mqtt_performance,
    bench_http2_performance,
    bench_dns_performance,
    bench_tcp_udp_performance,
    bench_layered_protocols,
    bench_secure_protocols,
    bench_iot_protocols,
    bench_protocol_parsing_pipeline,
    bench_packet_batch_processing
);
criterion_main!(benches);
