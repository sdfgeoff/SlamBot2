import type { PacketFormat } from './messageFormat'

export const formatEndpoint = (value: number | null) => (value === null ? 'null' : value)

export const formatTime = (time: bigint) => {
  const ms = Number(time / 1000n)
  const date = new Date(ms)
  return Number.isNaN(date.getTime()) ? 'Invalid' : date.toISOString()
}

export const getDataRootKey = (packet: PacketFormat) => {
  const keys = Object.keys(packet.data ?? {})
  return keys.length > 0 ? keys[0] : ''
}

export const summarizePacket = (packet: PacketFormat) => {
  const dataKey = getDataRootKey(packet)
  if (!dataKey) {
    return 'No data'
  }
  try {
    return JSON.stringify(
      (packet.data as Record<string, unknown>)[dataKey],
      (_, value) => (typeof value === 'bigint' ? Number(value) : value),
    )
  } catch {
    return '[unserializable]'
  }
}
