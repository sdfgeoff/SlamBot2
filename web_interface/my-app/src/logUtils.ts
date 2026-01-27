import type { AnyPacketFormat, PacketFormat } from './messageFormat'

export const formatEndpoint = (value: number | null) => (value === null ? 'null' : value)

export const formatTime = (time: number) => {
  const ms = Number(time / 1000)
  const date = new Date(ms)
  return Number.isNaN(date.getTime()) ? 'Invalid' : date.toISOString()
}

export const getDataRootKey = (packet: AnyPacketFormat) => {
  const keys = Object.keys(packet.data ?? {})
  return keys.length > 0 ? keys[0] : ''
}

export const summarizePacket = (packet: AnyPacketFormat) => {
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
