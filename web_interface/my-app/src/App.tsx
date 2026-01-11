import { useCallback, useMemo, useState } from 'react'
import './App.css'
import { DiagnosticStatus, type PacketFormat } from './messageFormat'
import { useWebSocket } from './useWebSocket'

type PacketEntry = {
  arrivalIndex: number
  packet: PacketFormat
}

function App() {
  const [lastMessage, setLastMessage] = useState<string>('')
  const [outgoingMessage, setOutgoingMessage] = useState('')
  const [packets, setPackets] = useState<PacketEntry[]>([])
  const [filterTo, setFilterTo] = useState('')
  const [filterFrom, setFilterFrom] = useState('')
  const [filterDataKey, setFilterDataKey] = useState('')

  const handleMessage = useCallback((message: unknown) => {
    if (typeof message === 'string') {
      setLastMessage(message)
      return
    }
    const packet = message as PacketFormat
    if (packet && typeof packet === 'object' && 'data' in packet && 'time' in packet) {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet },
      ])
      const summary = summarizePacket(packet)
      setLastMessage(summary)
      return
    }
    try {
      setLastMessage(
        JSON.stringify(message, (_, value) => (typeof value === 'bigint' ? Number(value) : value)),
      )
    } catch {
      setLastMessage('[unserializable message]')
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
      <div className="card">
        <p>
          WebSocket: <strong>{wsStatus}</strong>
        </p>
        <p>
          Last message: <code>{lastMessage || '—'}</code>
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
          <input
            type="text"
            value={filterTo}
            onChange={(event) => setFilterTo(event.target.value)}
            placeholder="Filter to (number or null)"
          />
          <input
            type="text"
            value={filterFrom}
            onChange={(event) => setFilterFrom(event.target.value)}
            placeholder="Filter from (number or null)"
          />
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
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>Time (UTC)</th>
              <th>Time (µs)</th>
              <th>To</th>
              <th>From</th>
              <th>Type</th>
              <th>Summary</th>
            </tr>
          </thead>
          <tbody>
            {filteredPackets.map(({ arrivalIndex, packet }) => (
              <tr key={arrivalIndex}>
                <td>{arrivalIndex}</td>
                <td>{formatTime(packet.time)}</td>
                <td>{packet.time.toString()}</td>
                <td>{formatEndpoint(packet.to)}</td>
                <td>{formatEndpoint(packet.from)}</td>
                <td>{getDataRootKey(packet) || 'Unknown'}</td>
                <td>{summarizePacket(packet)}</td>
              </tr>
            ))}
          </tbody>
        </table>
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
  if (dataKey === 'OdometryDelta') {
    const odometry = (packet.data as { OdometryDelta: {
      delta_position: [number, number]
      delta_orientation: number
    } }).OdometryDelta
    return `dx=${odometry.delta_position[0]} dy=${odometry.delta_position[1]} dθ=${odometry.delta_orientation}`
  }
  try {
    return JSON.stringify(packet.data, (_, value) => (typeof value === 'bigint' ? Number(value) : value))
  } catch {
    return '[unserializable]'
  }
}

export default App
