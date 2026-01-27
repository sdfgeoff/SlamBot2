import { type CSSProperties, useEffect, useMemo, useRef, useState } from 'react'
import { FixedSizeList, type ListOnScrollProps } from 'react-window'
import type { AnyPacketEntry, PacketEntry } from '../logTypes'
import { formatEndpoint, formatTime, getDataRootKey, summarizePacket } from '../logUtils'

type LogTableProps = {
  packets: AnyPacketEntry[]
}

const ROW_HEIGHT = 44
const AUTO_SCROLL_THRESHOLD = 16

const LogTable = ({ packets }: LogTableProps) => {
  const listContainerRef = useRef<HTMLDivElement | null>(null)
  const listRef = useRef<FixedSizeList | null>(null)
  const shouldAutoScrollRef = useRef(true)
  const [selectedPacket, setSelectedPacket] = useState<AnyPacketEntry | null>(null)
  const [listHeight, setListHeight] = useState(360)
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
    if (packets.length > 0) {
      listRef.current?.scrollToItem(packets.length - 1, 'end')
    }
  }, [packets.length])

  useEffect(() => {
    const container = listContainerRef.current
    if (!container) {
      return
    }
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) {
        setListHeight(entry.contentRect.height)
      }
    })
    observer.observe(container)
    return () => observer.disconnect()
  }, [])

  const handleLogScroll = ({ scrollOffset }: ListOnScrollProps) => {
    const totalHeight = packets.length * ROW_HEIGHT
    const distanceFromBottom = totalHeight - (scrollOffset + listHeight)
    shouldAutoScrollRef.current = distanceFromBottom <= AUTO_SCROLL_THRESHOLD
  }

  const renderRow = useMemo(
    () =>
      ({ index, style }: { index: number; style: CSSProperties }) => {
        const entry = packets[index]
        if (!entry) {
          return null
        }
        const { arrivalIndex, packet } = entry
        const isSelected = selectedPacket?.arrivalIndex === arrivalIndex
        return (
          <div
            className={`log-row log-grid${isSelected ? ' is-selected' : ''}`}
            style={style}
            onClick={() => setSelectedPacket({ arrivalIndex, packet })}
          >
            <div className="log-cell compact-cell">{arrivalIndex}</div>
            <div className="log-cell time-cell">{formatTime(packet.time)}</div>
            <div className="log-cell compact-cell">{formatEndpoint(packet.to)}</div>
            <div className="log-cell compact-cell">{formatEndpoint(packet.from)}</div>
            <div className="log-cell">{getDataRootKey(packet) || 'Unknown'}</div>
            <div className="log-cell summary-cell">{summarizePacket(packet)}</div>
          </div>
        )
      },
    [packets, selectedPacket]
  )

  return (
    <div className="card">
      <p>Log</p>
      <div className="log-container">
        <div className="log-header log-grid">
          <div className="log-cell compact-cell">#</div>
          <div className="log-cell time-cell">Time (UTC)</div>
          <div className="log-cell compact-cell">To</div>
          <div className="log-cell compact-cell">From</div>
          <div className="log-cell">Type</div>
          <div className="log-cell summary-cell">Summary</div>
        </div>
        <div className="log-body" ref={listContainerRef}>
          <FixedSizeList
            height={listHeight}
            width="100%"
            itemCount={packets.length}
            itemSize={ROW_HEIGHT}
            onScroll={handleLogScroll}
            ref={listRef}
          >
            {renderRow}
          </FixedSizeList>
        </div>
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
