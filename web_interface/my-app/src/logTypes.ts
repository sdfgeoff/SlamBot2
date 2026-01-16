import type { AnyPacketData, PacketFormat } from './messageFormat'

export type PacketEntry<T> = {
  arrivalIndex: number
  packet: PacketFormat<T>
}

export type AnyPacketEntry = PacketEntry<AnyPacketData>