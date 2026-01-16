import { useEffect, useState } from 'react'
import type { AnyPacketFormat } from '../messageFormat'
import type { AnyPacketEntry } from '../logTypes'
import PositionPlot from './PositionPlot'

interface PositionTabProps {
  registerCallback: (topic: string, callback: (message: AnyPacketFormat) => void) => () => void
}

function PositionTab({ registerCallback }: PositionTabProps) {
  const [packets, setPackets] = useState<AnyPacketEntry[]>([])

  useEffect(() => {
    const handleMessage = (message: AnyPacketFormat) => {
      setPackets((prev) => [
        ...prev,
        { arrivalIndex: prev.length + 1, packet: message },
      ])
    }

    return registerCallback('PositionEstimate', handleMessage)
  }, [registerCallback])

  return (
    <div>
      <PositionPlot packets={packets} />
    </div>
  )
}

export default PositionTab
