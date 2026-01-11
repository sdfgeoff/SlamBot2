import { describe, expect, it } from 'vitest'
import { decodePacket, PacketFinder } from './packetEncoding'

const VALID_PACKET_BASE64 =
  'AC2lYnRv9mRmcm9tAWRkYXRhoW1PZG9tZXRyeURlbHRhpGpzdGFydF90aW1lGxIGSBQBxONSaGVuZF90aW1lGxkGSBQBxmnxbmRlbHRhX3Bvc2l0aW9ugvkBAvkBFHFkZWx0YV9vcmllbnRhdGlvbvkBB2R0aW1lGxAGSBQBxmnyYmlkGWGDBL4A'

describe('packetEncoding', () => {
  it('decodes a framed packet payload', () => {
    const bytes = Uint8Array.from(Buffer.from(VALID_PACKET_BASE64, 'base64'))
    const finder = new PacketFinder()
    const packets = finder.pushBytes(bytes)

    expect(packets).toHaveLength(1)

    const decoded = decodePacket(packets[0]) as {
      to: null
      from: number
      data: {
        OdometryDelta: {
          start_time: bigint
          end_time: bigint
          delta_position: number[]
          delta_orientation: number
        }
      }
      time: bigint
      id: number
    }

    expect(decoded).toEqual({
      to: null,
      from: 1,
      data: {
        OdometryDelta: {
          start_time: 1768100626490194n,
          end_time: 1768100626590193n,
          delta_position: [0, 0],
          delta_orientation: 0,
        },
      },
      time: 1768100626590194n,
      id: 24963,
    })
  })
})
