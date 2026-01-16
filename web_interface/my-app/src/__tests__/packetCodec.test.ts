import { describe, it, expect, beforeAll } from 'vitest'
import { encodePacket, decodePacket } from '../usePacketCodec'
import type { AnyPacketFormat } from '../messageFormat'
import { MotionRequestMode } from '../messageFormat'

// Give WASM time to initialize
beforeAll(async () => {
  await new Promise(resolve => setTimeout(resolve, 100))
})

describe('WASM Packet Codec', () => {
  it('should encode and decode a MotionTargetRequest packet', () => {
    const packet: AnyPacketFormat = {
      to: null,
      from: null,
      time: BigInt(1234567890),
      id: 42,
      data: {
        MotionTargetRequest: {
          linear: [1.5, 2.0],
          angular: 0.5,
          motion_mode: MotionRequestMode.Position,
        },
      },
    }

    // Encode
    const encoded = encodePacket(packet)
    expect(encoded).toBeInstanceOf(Uint8Array)
    expect(encoded.length).toBeGreaterThan(0)

    // Decode
    const decoded = decodePacket(encoded)
    expect(decoded).toBeDefined()
    expect(decoded.id).toBe(42)
    expect(decoded.data).toHaveProperty('MotionTargetRequest')
    
    if ('MotionTargetRequest' in decoded.data) {
      const motion = decoded.data.MotionTargetRequest
      expect(motion.linear[0]).toBeCloseTo(1.5)
      expect(motion.linear[1]).toBeCloseTo(2.0)
      expect(motion.angular).toBeCloseTo(0.5)
      expect(motion.motion_mode).toBe(MotionRequestMode.Position)
    }
  })

  it('should encode and decode a PositionEstimate packet', () => {
    const packet: AnyPacketFormat = {
      to: null,
      from: null,
      time: BigInt(9999999999),
      id: 123,
      data: {
        PositionEstimate: {
          timestamp: BigInt(1111111111),
          position: [3.14, 2.71],
          orientation: 1.57,
        },
      },
    }

    const encoded = encodePacket(packet)
    const decoded = decodePacket(encoded)

    expect(decoded.id).toBe(123)
    if ('PositionEstimate' in decoded.data) {
      const pos = decoded.data.PositionEstimate
      expect(pos.position[0]).toBeCloseTo(3.14)
      expect(pos.position[1]).toBeCloseTo(2.71)
      expect(pos.orientation).toBeCloseTo(1.57)
    }
  })

  it('should encode and decode a SubscriptionRequest packet', () => {
    const packet: AnyPacketFormat = {
      to: null,
      from: null,
      time: BigInt(5555555555),
      id: 999,
      data: {
        SubscriptionRequest: {
          topics: ['PositionEstimate', 'OdometryDelta'],
        },
      },
    }

    const encoded = encodePacket(packet)
    const decoded = decodePacket(encoded)

    expect(decoded.id).toBe(999)
    if ('SubscriptionRequest' in decoded.data) {
      const sub = decoded.data.SubscriptionRequest
      expect(sub.topics).toEqual(['PositionEstimate', 'OdometryDelta'])
    }
  })

  it('should handle all three motion modes', () => {
    const modes = [
      MotionRequestMode.Velocity,
      MotionRequestMode.Position,
      MotionRequestMode.Stop,
    ]

    modes.forEach((mode) => {
      const packet: AnyPacketFormat = {
        to: null,
        from: null,
        time: BigInt(Date.now()),
        id: mode,
        data: {
          MotionTargetRequest: {
            linear: [0.0, 0.0],
            angular: 0.0,
            motion_mode: mode,
          },
        },
      }

      const encoded = encodePacket(packet)
      const decoded = decodePacket(encoded)

      if ('MotionTargetRequest' in decoded.data) {
        expect(decoded.data.MotionTargetRequest.motion_mode).toBe(mode)
      }
    })
  })
})
