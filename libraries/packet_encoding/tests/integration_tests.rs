use packet_encoding::{encode_packet, decode_packet, PacketEncodeErr, PacketDecodeErr};
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
    let result: Result<SimpleMessage, PacketDecodeErr> = decode_packet(&mut decode_buffer[..encoded_size]);
    
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


    fn yes() -> bool { true }

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
