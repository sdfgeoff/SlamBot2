import { useEffect, useRef } from 'react'
import type { PacketEntry } from '../logTypes'
import { formatEndpoint, formatTime, getDataRootKey, summarizePacket } from '../logUtils'

type LogTableProps = {
  packets: PacketEntry[]
}

const LogTable = ({ packets }: LogTableProps) => {
  const logContainerRef = useRef<HTMLDivElement | null>(null)
  const shouldAutoScrollRef = useRef(true)

  useEffect(() => {
    if (!shouldAutoScrollRef.current) {
      return
    }
    const container = logContainerRef.current
    if (container) {
      container.scrollTop = container.scrollHeight
    }
  }, [packets.length])

  const handleLogScroll = () => {
    const container = logContainerRef.current
    if (!container) {
      return
    }
    const threshold = 12
    const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
    shouldAutoScrollRef.current = distanceFromBottom <= threshold
  }

  return (
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
            {packets.map(({ arrivalIndex, packet }) => (
              <tr key={arrivalIndex}>
                <td className="compact-cell">{arrivalIndex}</td>
                <td className="time-cell">{formatTime(packet.time)}</td>
                <td className="compact-cell">{formatEndpoint(packet.to)}</td>
                <td className="compact-cell">{formatEndpoint(packet.from)}</td>
                <td>{getDataRootKey(packet) || 'Unknown'}</td>
                <td className="summary-cell">{summarizePacket(packet)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export default LogTable
