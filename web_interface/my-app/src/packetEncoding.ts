import { decode as cborDecode, encode as cborEncode } from 'cbor-x'

const CRC16_ARC_POLY = 0xA001

const crc16Arc = (data: Uint8Array): number => {
  let crc = 0x0000
  for (const byte of data) {
    crc ^= byte
    for (let bit = 0; bit < 8; bit += 1) {
      const lsb = crc & 0x0001
      crc >>= 1
      if (lsb) {
        crc ^= CRC16_ARC_POLY
      }
    }
  }
  return crc & 0xffff
}

const cobsEncode = (input: Uint8Array): Uint8Array => {
  const output = new Uint8Array(input.length + Math.ceil(input.length / 254) + 1)
  let outputIndex = 1
  let codeIndex = 0
  let code = 1

  for (const byte of input) {
    if (byte === 0) {
      output[codeIndex] = code
      code = 1
      codeIndex = outputIndex
      output[outputIndex] = 0
      outputIndex += 1
    } else {
      output[outputIndex] = byte
      outputIndex += 1
      code += 1
      if (code === 0xff) {
        output[codeIndex] = code
        code = 1
        codeIndex = outputIndex
        output[outputIndex] = 0
        outputIndex += 1
      }
    }
  }

  output[codeIndex] = code
  return output.slice(0, outputIndex)
}

const cobsDecode = (input: Uint8Array): Uint8Array => {
  const output: number[] = []
  let index = 0

  while (index < input.length) {
    const code = input[index]
    if (code === 0) {
      throw new Error('COBS decode error: zero code byte')
    }
    index += 1
    const end = index + code - 1
    if (end > input.length) {
      throw new Error('COBS decode error: overrun')
    }
    while (index < end) {
      output.push(input[index])
      index += 1
    }
    if (code < 0xff && index < input.length) {
      output.push(0)
    }
  }

  return new Uint8Array(output)
}

export const encodePacket = (message: unknown): Uint8Array => {
  const cborPayload = cborEncode(message)
  const crc = crc16Arc(cborPayload)
  const framed = new Uint8Array(cborPayload.length + 2)
  framed.set(cborPayload, 0)
  framed[framed.length - 2] = crc & 0xff
  framed[framed.length - 1] = (crc >> 8) & 0xff
  return cobsEncode(framed)
}

export const decodePacket = <T = unknown>(data: Uint8Array): T => {
  const decoded = cobsDecode(data)
  if (decoded.length < 2) {
    throw new Error('Packet too small')
  }
  const payload = decoded.slice(0, decoded.length - 2)
  const receivedCrc = decoded[decoded.length - 2] | (decoded[decoded.length - 1] << 8)
  const calculatedCrc = crc16Arc(payload)
  if (receivedCrc !== calculatedCrc) {
    throw new Error('CRC mismatch')
  }
  return cborDecode(payload) as T
}

/**
 * Puts a 0x00 on the start and end of some data
 */
export const framePacket = (encoded: Uint8Array): Uint8Array => {
  const framed = new Uint8Array(encoded.length + 2)
  framed[0] = 0x00
  framed.set(encoded, 1)
  framed[framed.length - 1] = 0x00
  return framed
}

export class PacketFinder {
  private buffer: number[]
  private maxBufferSize: number

  constructor(maxBufferSize = 512) {
    this.buffer = []
    this.maxBufferSize = maxBufferSize
  }

  pushByte(byte: number): Uint8Array | null {
    if (this.buffer.length >= this.maxBufferSize) {
      this.buffer = []
      return null
    }

    if (byte === 0x00) {
      if (this.buffer.length > 0) {
        const packet = new Uint8Array(this.buffer.slice(1))
        this.buffer = [0x00]
        return packet
      }
      this.buffer = [0x00]
      return null
    }

    if (this.buffer.length > 0) {
      this.buffer.push(byte)
    }

    return null
  }

  pushBytes(data: Uint8Array): Uint8Array[] {
    const packets: Uint8Array[] = []
    for (const byte of data) {
      const packet = this.pushByte(byte)
      if (packet) {
        packets.push(packet)
      }
    }
    return packets
  }
}
