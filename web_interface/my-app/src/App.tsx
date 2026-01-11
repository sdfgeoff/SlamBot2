import { useCallback, useMemo, useState } from 'react'
import './App.css'
import type { PacketFormat } from './messageFormat'
import type { PacketEntry } from './logTypes'
import { getDataRootKey } from './logUtils'
import { useWebSocket } from './useWebSocket'
import ConnectionPanel from './components/ConnectionPanel'
import FiltersPanel from './components/FiltersPanel'
import LogTable from './components/LogTable'

function App() {
  const [outgoingMessage, setOutgoingMessage] = useState('')
  const [packets, setPackets] = useState<PacketEntry[]>([])
  const [filterTo, setFilterTo] = useState('')
  const [filterFrom, setFilterFrom] = useState('')
  const [filterDataKey, setFilterDataKey] = useState('')

  const handleMessage = useCallback((message: unknown) => {
    const packet = message as PacketFormat
    if (packet && typeof packet === 'object' && 'data' in packet && 'time' in packet) {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet },
      ])
    }
  }, [])

  const { send, status: wsStatus } = useWebSocket(handleMessage)

  const sendMessage = () => {
    if (!outgoingMessage.trim()) {
      return
    }
    const sent = send(outgoingMessage)
    if (sent) {
      setOutgoingMessage('')
    }
  }

  const dataKeys = useMemo(() => {
    const keys = new Set<string>()
    packets.forEach(({ packet }) => {
      const key = getDataRootKey(packet)
      if (key) {
        keys.add(key)
      }
    })
    return Array.from(keys).sort()
  }, [packets])

  const nodeOptions = useMemo(() => {
    const nodes = new Set<number | null>()
    packets.forEach(({ packet }) => {
      nodes.add(packet.to)
      nodes.add(packet.from)
    })
    const sorted = Array.from(nodes).sort((a, b) => {
      if (a === null && b === null) return 0
      if (a === null) return -1
      if (b === null) return 1
      return a - b
    })
    return sorted.map((node) => (node === null ? 'null' : String(node)))
  }, [packets])

  const filteredPackets = useMemo(() => {
    return packets.filter(({ packet }) => {
      if (filterTo) {
        if (filterTo === 'null') {
          if (packet.to !== null) {
            return false
          }
        } else if (String(packet.to ?? '') !== filterTo) {
          return false
        }
      }
      if (filterFrom) {
        if (filterFrom === 'null') {
          if (packet.from !== null) {
            return false
          }
        } else if (String(packet.from ?? '') !== filterFrom) {
          return false
        }
      }
      if (filterDataKey) {
        const key = getDataRootKey(packet)
        if (key !== filterDataKey) {
          return false
        }
      }
      return true
    })
  }, [packets, filterTo, filterFrom, filterDataKey])

  return (
    <>
      <ConnectionPanel
        wsStatus={wsStatus}
        outgoingMessage={outgoingMessage}
        onOutgoingMessageChange={setOutgoingMessage}
        onSend={sendMessage}
      />
      <FiltersPanel
        filterTo={filterTo}
        filterFrom={filterFrom}
        filterDataKey={filterDataKey}
        nodeOptions={nodeOptions}
        dataKeys={dataKeys}
        onFilterToChange={setFilterTo}
        onFilterFromChange={setFilterFrom}
        onFilterDataKeyChange={setFilterDataKey}
        onClear={() => {
          setFilterTo('')
          setFilterFrom('')
          setFilterDataKey('')
        }}
      />
      <LogTable packets={filteredPackets} />
    </>
  )
}

export default App
