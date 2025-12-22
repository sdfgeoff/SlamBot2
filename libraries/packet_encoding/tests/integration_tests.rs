use packet_encoding::{
    PacketDecodeErr, PacketEncodeErr, PacketFinder, decode_packet, encode_packet,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestMessage {
    id: u32,
    value: i16,
    flag: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleMessage {
    data: u8,
}

#[test]
fn test_encode_decode_success() {
    let message = TestMessage {
        id: 12345,
        value: -678,
        flag: true,
    };

    let mut encode_buffer = [0u8; 100];
    let encoded_size = encode_packet(&message, &mut encode_buffer).unwrap();

    // Verify that we got a reasonable size
    assert!(encoded_size > 0);
    assert!(encoded_size <= 100);

    let mut decode_buffer = encode_buffer.clone();
    let decoded_message: TestMessage = decode_packet(&mut decode_buffer[..encoded_size]).unwrap();

    assert_eq!(message, decoded_message);
}

#[test]
fn test_encode_decode_simple_message() {
    let message = SimpleMessage { data: 42 };

    let mut encode_buffer = [0u8; 50];
    let encoded_size = encode_packet(&message, &mut encode_buffer).unwrap();

    let decoded_message: SimpleMessage = decode_packet(&mut encode_buffer[..encoded_size]).unwrap();

    assert_eq!(message, decoded_message);
}

#[test]
fn test_encode_buffer_too_small() {
    let message = TestMessage {
        id: 12345,
        value: -678,
        flag: true,
    };

    let mut encode_buffer = [0u8; 5]; // Too small
    let result = encode_packet(&message, &mut encode_buffer);

    assert!(matches!(result, Err(PacketEncodeErr::DestBufTooSmallError)));
}

#[test]
fn test_decode_corrupted_cobs_data() {
    let mut corrupted_data = [0u8, 1u8, 2u8, 0u8, 4u8]; // Invalid COBS data
    let result: Result<SimpleMessage, PacketDecodeErr> = decode_packet(&mut corrupted_data);

    assert!(matches!(result, Err(PacketDecodeErr::CobsError)));
}

#[test]
fn test_decode_too_short_data() {
    let mut short_data = [0u8]; // Too short for CRC
    let result: Result<SimpleMessage, PacketDecodeErr> = decode_packet(&mut short_data);

    assert!(matches!(result, Err(PacketDecodeErr::CobsError)));
}

#[test]
fn test_decode_crc_mismatch() {
    let message = SimpleMessage { data: 42 };

    let mut encode_buffer = [0u8; 50];
    let encoded_size = encode_packet(&message, &mut encode_buffer).unwrap();

    // Corrupt the CRC bytes
    encode_buffer[encoded_size - 3] ^= 0xFF;

    let mut decode_buffer = encode_buffer.clone();
    let result: Result<SimpleMessage, PacketDecodeErr> =
        decode_packet(&mut decode_buffer[..encoded_size]);

    assert!(matches!(result, Err(PacketDecodeErr::CrcMismatchError)));
}

#[test]
fn test_empty_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct EmptyStruct {}

    let message = EmptyStruct {};
    let mut encode_buffer = [0u8; 50];
    let encoded_size = encode_packet(&message, &mut encode_buffer).unwrap();

    let mut decode_buffer = encode_buffer.clone();
    let decoded_message: EmptyStruct = decode_packet(&mut decode_buffer[..encoded_size]).unwrap();

    assert_eq!(message, decoded_message);
}

#[test]
fn test_schema_missing_optional_field() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct OldMessage {
        id: u32,
        value: i16,
    }

    fn yes() -> bool {
        true
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NewMessage {
        id: u32,
        value: i16,

        #[serde(default = "yes")]
        flag: bool, // New field
    }

    let old_message = OldMessage {
        id: 123,
        value: -456,
    };

    let mut encode_buffer = [0u8; 100];
    let encoded_size = encode_packet(&old_message, &mut encode_buffer).unwrap();

    let mut decode_buffer = encode_buffer.clone();
    let decoded_message: NewMessage = decode_packet(&mut decode_buffer[..encoded_size]).unwrap();

    assert_eq!(decoded_message.id, old_message.id);
    assert_eq!(decoded_message.value, old_message.value);
    assert_eq!(decoded_message.flag, true); // Default value for missing field
}

#[test]
fn test_enum_structs() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct A {
        x: u8,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct B {
        y: u16,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum MessageEnum {
        VariantA(A),
        VariantB(B),
    }
    let message = MessageEnum::VariantB(B { y: 7890 });
    let mut encode_buffer = [0u8; 100];
    let encoded_size = encode_packet(&message, &mut encode_buffer).unwrap();

    let decoded_message: MessageEnum = decode_packet(&mut encode_buffer[..encoded_size]).unwrap();
    assert_eq!(message, decoded_message);
}

// PacketFinder tests
#[test]
fn test_packet_finder_single_packet() {
    let mut finder = PacketFinder::new();

    // Push a packet: [0x00, 0x01, 0x02, 0x03, 0x00]
    assert!(finder.push_byte(0x00).is_none()); // Start packet delimiter
    assert!(finder.push_byte(0x01).is_none()); // Packet data
    assert!(finder.push_byte(0x02).is_none()); // Packet data
    assert!(finder.push_byte(0x03).is_none()); // Packet data

    // End packet delimiter should return the complete packet
    let packet = finder.push_byte(0x00).expect("Should return a packet");
    assert_eq!(packet.as_slice(), &[0x01, 0x02, 0x03]);
}

#[test]
fn test_packet_finder_multiple_packets() {
    let mut finder = PacketFinder::new();

    // First packet: [0x00, 0xAA, 0x00]
    assert!(finder.push_byte(0x00).is_none());
    assert!(finder.push_byte(0xAA).is_none());
    let packet1 = finder.push_byte(0x00).expect("Should return first packet");
    assert_eq!(packet1.as_slice(), &[0xAA]);

    // Second packet: [0x00, 0xBB, 0xCC, 0x00]
    let gap = finder.push_byte(0x00).expect("Gap");
    assert_eq!(gap.as_slice(), &[]);
    assert!(finder.push_byte(0xBB).is_none());
    assert!(finder.push_byte(0xCC).is_none());
    let packet2 = finder.push_byte(0x00).expect("Should return second packet");
    assert_eq!(packet2.as_slice(), &[0xBB, 0xCC]);
}

#[test]
fn test_packet_finder_empty_packet() {
    let mut finder = PacketFinder::new();

    // Two consecutive delimiters should produce a packet with just the delimiter
    assert!(finder.push_byte(0x00).is_none()); // Start first packet
    let packet = finder.push_byte(0x00).expect("Should return packet");
    assert_eq!(packet.as_slice(), &[]);
}

#[test]
fn test_packet_finder_no_start_delimiter() {
    let mut finder = PacketFinder::new();

    // Push data without starting with 0x00 - should be ignored
    assert!(finder.push_byte(0x01).is_none());
    assert!(finder.push_byte(0x02).is_none());
    assert!(finder.push_byte(0x03).is_none());

    // Now start a proper packet
    assert!(finder.push_byte(0x00).is_none());
    assert!(finder.push_byte(0xAA).is_none());
    let packet = finder.push_byte(0x00).expect("Should return packet");
    assert_eq!(packet.as_slice(), &[0xAA]);
}

#[test]
fn test_packet_finder_buffer_overflow() {
    let mut finder = PacketFinder::new();

    // Start a packet
    assert!(finder.push_byte(0x00).is_none());

    // Fill buffer to near capacity (511 more bytes since we already have one 0x00)
    for i in 1..512 {
        let result = finder.push_byte(0x02);
        assert!(
            result.is_none(),
            "Should not return packet until buffer full or reset"
        );
    }

    // This should trigger buffer overflow and reset
    assert!(finder.push_byte(0xFF).is_none());

    // After buffer overflow, it should reset and we can start a new packet
    assert!(finder.push_byte(0x00).is_none());
    assert!(finder.push_byte(0xAA).is_none());
    let packet = finder
        .push_byte(0x00)
        .expect("Should return packet after reset");
    assert_eq!(packet.as_slice(), &[0xAA]);
}

#[test]
fn test_packet_finder_partial_packet_then_reset() {
    let mut finder = PacketFinder::new();

    // Start a packet but don't complete it
    assert!(finder.push_byte(0x00).is_none());
    assert!(finder.push_byte(0x01).is_none());
    assert!(finder.push_byte(0x02).is_none());

    // Start a new packet (this should return the current buffer and start fresh)
    let packet = finder
        .push_byte(0x00)
        .expect("Should return partial packet");
    assert_eq!(packet.as_slice(), &[0x01, 0x02]);

    // Now we should be able to continue with a new packet
    assert!(finder.push_byte(0xBB).is_none());
    let packet2 = finder.push_byte(0x00).expect("Should return new packet");
    assert_eq!(packet2.as_slice(), &[0xBB]);
}

#[test]
fn test_packet_finder_consecutive_delimiters() {
    let mut finder = PacketFinder::new();

    // Multiple consecutive delimiters
    assert!(finder.push_byte(0x00).is_none()); // Start first packet
    let packet1 = finder
        .push_byte(0x00)
        .expect("Should return packet with single delimiter");
    assert_eq!(packet1.as_slice(), &[]);

    let packet2 = finder
        .push_byte(0x00)
        .expect("Should return another packet with single delimiter");
    assert_eq!(packet2.as_slice(), &[]);

    let packet3 = finder
        .push_byte(0x00)
        .expect("Should return a third packet with single delimiter");
    assert_eq!(packet3.as_slice(), &[]);
}
