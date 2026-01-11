import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './App.css'
import { DiagnosticStatus, type PacketFormat } from './messageFormat'
import { useWebSocket } from './useWebSocket'

type PacketEntry = {
  arrivalIndex: number
  packet: PacketFormat
}

function App() {
  const [outgoingMessage, setOutgoingMessage] = useState('')
  const [packets, setPackets] = useState<PacketEntry[]>([])
  const [filterTo, setFilterTo] = useState('')
  const [filterFrom, setFilterFrom] = useState('')
  const [filterDataKey, setFilterDataKey] = useState('')
  const logContainerRef = useRef<HTMLDivElement | null>(null)
  const shouldAutoScrollRef = useRef(true)

  const handleMessage = useCallback((message: unknown) => {
    const packet = message as PacketFormat
    if (packet && typeof packet === 'object' && 'data' in packet && 'time' in packet) {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet },
      ])
      return
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

  useEffect(() => {
    if (!shouldAutoScrollRef.current) {
      return
    }
    const container = logContainerRef.current
    if (container) {
      container.scrollTop = container.scrollHeight
    }
  }, [filteredPackets.length])

  const handleLogScroll = () => {
    const container = logContainerRef.current
    if (!container) {
      return
    }
    const threshold = 100
    const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
    shouldAutoScrollRef.current = distanceFromBottom <= threshold
  }

  return (
    <>
      <div className="card">
        <p>
          WebSocket: <strong>{wsStatus}</strong>
        </p>
        <div>
          <input
            type="text"
            value={outgoingMessage}
            onChange={(event) => setOutgoingMessage(event.target.value)}
            placeholder="Type a message"
          />
          <button onClick={sendMessage} disabled={wsStatus !== 'open'}>
            Send
          </button>
        </div>
      </div>
      <div className="card">
        <p>Filters</p>
        <div>
          <select value={filterTo} onChange={(event) => setFilterTo(event.target.value)}>
            <option value="">All to</option>
            {nodeOptions.map((node) => (
              <option key={`to-${node}`} value={node}>
                {node}
              </option>
            ))}
          </select>
          <select value={filterFrom} onChange={(event) => setFilterFrom(event.target.value)}>
            <option value="">All from</option>
            {nodeOptions.map((node) => (
              <option key={`from-${node}`} value={node}>
                {node}
              </option>
            ))}
          </select>
          <select
            value={filterDataKey}
            onChange={(event) => setFilterDataKey(event.target.value)}
          >
            <option value="">All data types</option>
            {dataKeys.map((key) => (
              <option key={key} value={key}>
                {key}
              </option>
            ))}
          </select>
          <button
            onClick={() => {
              setFilterTo('')
              setFilterFrom('')
              setFilterDataKey('')
            }}
          >
            Clear
          </button>
        </div>
      </div>
      <div className="card">
        <p>Log</p>
        <div className="log-container" ref={logContainerRef} onScroll={handleLogScroll}>
          <table className="log-table">
            <thead>
              <tr>
                <th>#</th>
                <th>Time (UTC)</th>
                <th>To</th>
                <th>From</th>
                <th>Type</th>
                <th className="summary-cell">Summary</th>
              </tr>
            </thead>
            <tbody>
              {filteredPackets.map(({ arrivalIndex, packet }) => (
                <tr key={arrivalIndex}>
                  <td>{arrivalIndex}</td>
                  <td className="time-cell">{formatTime(packet.time)}</td>
                  <td>{formatEndpoint(packet.to)}</td>
                  <td>{formatEndpoint(packet.from)}</td>
                  <td>{getDataRootKey(packet) || 'Unknown'}</td>
                  <td className="summary-cell">{summarizePacket(packet)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </>
  )
}

const formatEndpoint = (value: number | null) => (value === null ? 'null' : value)

const formatTime = (time: bigint) => {
  const ms = Number(time / 1000n)
  const date = new Date(ms)
  return Number.isNaN(date.getTime()) ? 'Invalid' : date.toISOString()
}

const getDataRootKey = (packet: PacketFormat) => {
  const keys = Object.keys(packet.data ?? {})
  return keys.length > 0 ? keys[0] : ''
}

const summarizePacket = (packet: PacketFormat) => {
  const dataKey = getDataRootKey(packet)
  if (!dataKey) {
    return 'No data'
  }
  if (dataKey === 'DiagnosticMsg') {
    const diagnostic = (packet.data as { DiagnosticMsg: {
      level: DiagnosticStatus
      name: string
      message: string
      values: { key: string; value: string }[]
    } }).DiagnosticMsg
    return `${diagnostic.name}: ${diagnostic.message} (level ${diagnostic.level})`
  }
  try {
    return JSON.stringify(packet.data[dataKey], (_, value) => (typeof value === 'bigint' ? Number(value) : value))
  } catch {
    return '[unserializable]'
  }
}

export default App
