import { useEffect, useMemo, useState } from 'react'
import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis, Legend } from 'recharts'
import type { AnyPacketData, DiagnosticMsg } from '../messageFormat'
import type { AnyPacketEntry, PacketEntry } from '../logTypes'



type DiagnosticGraphProps = {
  packets: AnyPacketEntry[]
}

const isDiagnostic = (data: AnyPacketData): data is DiagnosticMsg => {
  return 'DiagnosticMsg' in data 
}

const isDiagnosticEntry = (entry: AnyPacketEntry): entry is PacketEntry<DiagnosticMsg> => {
  return isDiagnostic(entry.packet.data)
}

const parseNumeric = (value: string | null | undefined) => {
  if (value === null || value === undefined) {
    return null
  }
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

const palette = ['#0f766e', '#2563eb', '#f97316', '#9333ea', '#dc2626', '#16a34a']

const DiagnosticGraph = ({ packets }: DiagnosticGraphProps) => {

  const [selectedEvent, setSelectedEvent] = useState('')
  const [selectedFields, setSelectedFields] = useState<string[]>([])


  const diagnostics = useMemo<PacketEntry<DiagnosticMsg>[]>(() => {
    return packets.filter(isDiagnosticEntry)
  }, [packets])

  const selectedDiagnostics = useMemo(() => {
    return diagnostics.filter(
      ({ packet }) => packet.data.DiagnosticMsg.name === selectedEvent
    )
  }, [diagnostics, selectedEvent])


  const eventNames = useMemo(() => {
    const names = new Set<string>()
    diagnostics.forEach((packet) => names.add(packet.packet.data.DiagnosticMsg.name))
    return Array.from(names).sort()
  }, [selectedDiagnostics])




  useEffect(() => {
    if (!eventNames.includes(selectedEvent)) {
      setSelectedEvent(eventNames[0] ?? '')
    }
  }, [eventNames, selectedEvent])


  const availableFields = useMemo(() => {
    const counts = new Map<string, number>()
    selectedDiagnostics.forEach(({ packet }) => {
      packet.data.DiagnosticMsg.values.forEach(({ key, value }) => {
        if (parseNumeric(value) !== null) {
          counts.set(key, (counts.get(key) ?? 0) + 1)
        }
      })
    })
    return Array.from(counts.keys()).sort()
  }, [selectedDiagnostics])

  useEffect(() => {
    setSelectedFields((prev) => {
      const next = prev.filter((field) => availableFields.includes(field))
      if (next.length > 0) {
        return next
      }
      return availableFields
    })
  }, [availableFields])

  const chartData = useMemo(() => {
    if (!selectedEvent || selectedFields.length === 0) {
      return []
    }
    return selectedDiagnostics.map(({ packet, arrivalIndex }) => {
      const row: Record<string, number | null> = {
        index: arrivalIndex,
      }
      selectedFields.forEach((field) => {
        const match = packet.data.DiagnosticMsg.values.find((item) => item.key === field)
        row[field] = parseNumeric(match?.value)
      })
      return row
    })
  }, [selectedDiagnostics, selectedEvent, selectedFields])

  const hasData = chartData.some((row) =>
    selectedFields.some((field) => typeof row[field] === 'number')
  )

  return (
    <div className="card">
      <p>Diagnostic Graph</p>
      <div className="graph-controls">
        <select
          value={selectedEvent}
          onChange={(event) => setSelectedEvent(event.target.value)}
        >
          <option value="">Select event</option>
          {eventNames.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
        <div className="field-list">
          {availableFields.length === 0 ? (
            <span className="field-empty">No numeric fields</span>
          ) : (
            availableFields.map((field) => (
              <label key={field}>
                <input
                  type="checkbox"
                  checked={selectedFields.includes(field)}
                  onChange={() =>
                    setSelectedFields((prev) =>
                      prev.includes(field)
                        ? prev.filter((item) => item !== field)
                        : [...prev, field]
                    )
                  }
                />
                {field}
              </label>
            ))
          )}
        </div>
      </div>
      <div className="graph-body">
        {!selectedEvent ? (
          <div className="graph-empty">Choose a diagnostic event to graph.</div>
        ) : selectedFields.length === 0 ? (
          <div className="graph-empty">Select at least one field to graph.</div>
        ) : !hasData ? (
          <div className="graph-empty">No numeric samples available for this event.</div>
        ) : (
          <div className="graph-chart">
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={chartData}>
                <XAxis dataKey="index" />
                <YAxis />
                <Tooltip />
                <Legend />
                {selectedFields.map((field, index) => (
                  <Line
                    key={field}
                    type="monotone"
                    dataKey={field}
                    stroke={palette[index % palette.length]}
                    strokeWidth={2}
                  />
                ))}
              </LineChart>
            </ResponsiveContainer>
            <div className="graph-subtitle">{`Samples: ${selectedDiagnostics.length}`}</div>
          </div>
        )}
      </div>
      {selectedFields.length > 0 ? (
        <div className="graph-legend">
          {selectedFields.map((field, index) => (
            <span key={field}>
              <i style={{ background: palette[index % palette.length] }} />
              {field}
            </span>
          ))}
        </div>
      ) : null}
    </div>
  )
}

export default DiagnosticGraph
