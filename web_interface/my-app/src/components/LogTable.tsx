import { useEffect, useRef, useState } from 'react'
import type { PacketEntry } from '../logTypes'
import { formatEndpoint, formatTime, getDataRootKey, summarizePacket } from '../logUtils'

type LogTableProps = {
  packets: PacketEntry[]
}

const LogTable = ({ packets }: LogTableProps) => {
  const logContainerRef = useRef<HTMLDivElement | null>(null)
  const shouldAutoScrollRef = useRef(true)
  const [selectedPacket, setSelectedPacket] = useState<PacketEntry | null>(null)
  const jsonReplacer = (_key: string, value: unknown) =>
    typeof value === 'bigint' ? value.toString() : value
  const formattedPacket = selectedPacket
    ? JSON.stringify(
        {
          arrivalIndex: selectedPacket.arrivalIndex,
          packet: selectedPacket.packet,
        },
        jsonReplacer,
        2
      )
    : ''

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
              <th className="compact-cell">#</th>
              <th className="time-cell">Time (UTC)</th>
              <th className="compact-cell">To</th>
              <th className="compact-cell">From</th>
              <th>Type</th>
              <th className="summary-cell">Summary</th>
            </tr>
          </thead>
          <tbody>
            {packets.map(({ arrivalIndex, packet }) => (
              <tr
                key={arrivalIndex}
                onClick={() => setSelectedPacket({ arrivalIndex, packet })}
              >
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
      {selectedPacket ? (
        <div className="log-detail">
          <p>Selected packet</p>
          <pre>{formattedPacket}</pre>
        </div>
      ) : null}
    </div>
  )
}

export default LogTable
