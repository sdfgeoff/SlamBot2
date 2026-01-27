use wasm_bindgen::prelude::*;
use topics::{PacketFormat, PacketData};

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Encode a packet from JSON to CBOR bytes
/// 
/// # Arguments
/// * `json` - JSON string representation of a PacketFormat
/// 
/// # Returns
/// * `Result<Vec<u8>, JsValue>` - CBOR encoded bytes or error
#[wasm_bindgen]
pub fn encode_packet(json: &str) -> Result<Vec<u8>, JsValue> {
    // Parse JSON to PacketFormat
    let packet: PacketFormat<PacketData> = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}, {}", e, json)))?;
    
    // Encode to CBOR
    let mut buffer = [0u8; 1024];
    let size = packet_encoding::encode_packet(&packet, &mut buffer)
        .map_err(|e| JsValue::from_str(&format!("Failed to encode to CBOR: {:?}, {:?}", e, packet)))?;
    
    Ok(buffer[..size].to_vec())
}

/// Decode a packet from CBOR bytes to JSON
/// 
/// # Arguments
/// * `bytes` - CBOR encoded packet bytes
/// 
/// # Returns
/// * `Result<String, JsValue>` - JSON string representation or error
#[wasm_bindgen]
pub fn decode_packet(bytes: &[u8]) -> Result<String, JsValue> {
    // Need to make a mutable copy for decode_in_place
    let mut buffer = bytes.to_vec();
    
    // Decode from CBOR
    let packet: PacketFormat<PacketData> = packet_encoding::decode_packet(&mut buffer)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode CBOR: {:?}", e)))?;
    
    // Convert to JSON
    let json = serde_json::to_string(&packet)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize to JSON: {}", e)))?;
    
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let json = r#"{
            "to": null,
            "from": null,
            "time": 1234567890,
            "id": 42,
            "data": {
                "ClockRequest": {
                    "request_time": 1000
                }
            }
        }"#;

        // Encode
        let encoded = encode_packet(json).expect("Encoding should succeed");
        assert!(!encoded.is_empty());

        // Decode
        let decoded = decode_packet(&encoded).expect("Decoding should succeed");
        
        // Parse both to compare structure (not exact string match due to formatting)
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        let result: serde_json::Value = serde_json::from_str(&decoded).unwrap();
        
        assert_eq!(original, result);
    }
}
