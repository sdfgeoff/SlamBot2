import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './App.css'
import type { PacketFormat } from './messageFormat'
import type { PacketEntry } from './logTypes'
import { getDataRootKey } from './logUtils'
import { useWebSocket } from './useWebSocket'
import ConnectionPanel from './components/ConnectionPanel'
import FiltersPanel from './components/FiltersPanel'
import DiagnosticGraph from './components/DiagnosticGraph'
import LogTable from './components/LogTable'

function App() {
  const [packets, setPackets] = useState<PacketEntry[]>([])
  const [filterTo, setFilterTo] = useState('')
  const [filterFrom, setFilterFrom] = useState('')
  const [filterDataKey, setFilterDataKey] = useState('')

  const handleMessage = useCallback((message: PacketFormat) => {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet: message },
      ])
  }, [])

  const { send, status: wsStatus } = useWebSocket<PacketFormat>(handleMessage)

  const sendSubscriptionRequest = useCallback(() => {
      const message: PacketFormat = {
        to: null,
        from: null,
        time: BigInt(Date.now()),
        id: Date.now() % 0xffffffff,
        data: {
          SubscriptionRequest: {
            topics: ['all'],
          },
        },
      }
      console.log(message)
      send(message)
    }, [send])


  useEffect(() => {
    if (wsStatus === 'open') {
      const timerId = window.setInterval(() => {
        console.log("Sending subscription request")
        sendSubscriptionRequest()
      }, 2000)
      return () => {
        window.clearInterval(timerId)
      }
    }
    return () => {
      /*pass*/
    }
  }, [wsStatus])


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
      <DiagnosticGraph packets={filteredPackets} />
      <LogTable packets={filteredPackets} />
    </>
  )
}

export default App
