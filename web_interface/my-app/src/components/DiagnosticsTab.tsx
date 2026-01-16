import { useEffect, useMemo, useState } from 'react'
import type { AnyPacketFormat } from '../messageFormat'
import type { AnyPacketEntry } from '../logTypes'
import { getDataRootKey } from '../logUtils'
import FiltersPanel from './FiltersPanel'
import DiagnosticGraph from './DiagnosticGraph'
import LogTable from './LogTable'

interface DiagnosticsTabProps {
  registerCallback: (topic: string, callback: (message: AnyPacketFormat) => void) => () => void
}

function DiagnosticsTab({ registerCallback }: DiagnosticsTabProps) {
  const [packets, setPackets] = useState<AnyPacketEntry[]>([])
  const [filterTo, setFilterTo] = useState('')
  const [filterFrom, setFilterFrom] = useState('')
  const [filterDataKey, setFilterDataKey] = useState('')

  useEffect(() => {
    const handleMessage = (message: AnyPacketFormat) => {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet: message },
      ])
    }

    const deregister = registerCallback('all', handleMessage)
    return deregister
  }, [registerCallback])

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
    <div>
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
      <DiagnosticGraph packets={filteredPackets} />
      <LogTable packets={filteredPackets} />
    </div>
  )
}

export default DiagnosticsTab
