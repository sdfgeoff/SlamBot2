import { encode_packet as wasmEncode, decode_packet as wasmDecode } from './wasm/packet_wasm'
import type { AnyPacketFormat } from './messageFormat'

/**
 * Encode a packet from JSON to CBOR bytes using WASM
 * @param packet - The packet to encode
 * @returns CBOR encoded bytes
 * @throws Error if encoding fails
 */
export function encodePacket(packet: AnyPacketFormat): Uint8Array {
  // Convert BigInt to number for JSON serialization
  // CBOR will handle the conversion back
  const jsonPacket = JSON.stringify(packet, (_key, value) =>
    typeof value === 'bigint' ? Number(value) : value
  )

  try {
    return wasmEncode(jsonPacket)
  } catch (error) {
    throw new Error(`Failed to encode packet: ${error}`)
  }
}

/**
 * Decode a packet from CBOR bytes to JSON using WASM
 * @param bytes - CBOR encoded packet bytes
 * @returns Decoded packet
 * @throws Error if decoding fails
 */
export function decodePacket(bytes: Uint8Array): AnyPacketFormat {
  try {
    const jsonString = wasmDecode(bytes)
    const parsed = JSON.parse(jsonString, (_key, value) => {
      // Convert large numbers back to BigInt
      // This is a heuristic - assumes numbers > Number.MAX_SAFE_INTEGER should be BigInt
      if (typeof value === 'number' && !Number.isSafeInteger(value)) {
        return BigInt(value)
      }
      return value
    })
    return parsed as AnyPacketFormat
  } catch (error) {
    throw new Error(`Failed to decode packet: ${error}`)
  }
}

/**
 * Hook to use the packet codec in React components
 * The WASM module is automatically initialized on import
 */
export function usePacketCodec() {
  return {
    encodePacket,
    decodePacket,
  }
}
