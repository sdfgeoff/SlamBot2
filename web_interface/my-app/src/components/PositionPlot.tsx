import { useEffect, useRef } from 'react'
import type { AnyPacketEntry, PacketEntry } from '../logTypes'
import type { PositionEstimate } from '../messageFormat'

interface PositionPlotProps {
  packets: AnyPacketEntry[]
}

function isPositionEstimate(data: AnyPacketEntry): data is PacketEntry<PositionEstimate> {
  return 'PositionEstimate' in data.packet.data
}

function PositionPlot({ packets }: PositionPlotProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    // Set up coordinate system
    const width = canvas.width
    const height = canvas.height
    const centerX = width / 2
    const centerY = height / 2
    const scale = 100 // pixels per meter

    // Draw grid
    ctx.strokeStyle = '#e0e0e0'
    ctx.lineWidth = 1
    for (let i = -10; i <= 10; i++) {
      // Vertical lines
      ctx.beginPath()
      ctx.moveTo(centerX + i * scale, 0)
      ctx.lineTo(centerX + i * scale, height)
      ctx.stroke()
      
      // Horizontal lines
      ctx.beginPath()
      ctx.moveTo(0, centerY + i * scale)
      ctx.lineTo(width, centerY + i * scale)
      ctx.stroke()
    }

    // Draw axes
    ctx.strokeStyle = '#000000'
    ctx.lineWidth = 2
    ctx.beginPath()
    ctx.moveTo(centerX, 0)
    ctx.lineTo(centerX, height)
    ctx.stroke()
    ctx.beginPath()
    ctx.moveTo(0, centerY)
    ctx.lineTo(width, centerY)
    ctx.stroke()

    // Extract position estimates
    const positions = packets.filter(isPositionEstimate).map(({ packet }) => packet.data.PositionEstimate )

    if (positions.length === 0) return

    // Draw path
    ctx.strokeStyle = '#2196F3'
    ctx.lineWidth = 2
    ctx.beginPath()
    positions.forEach((pos, index) => {
      const canvasX = centerX + pos.position[0] * scale
      const canvasY = centerY - pos.position[1] * scale // Invert Y for screen coordinates
      
      if (index === 0) {
        ctx.moveTo(canvasX, canvasY)
      } else {
        ctx.lineTo(canvasX, canvasY)
      }
    })
    ctx.stroke()

    // Draw position points and orientation
    positions.forEach((pos, index) => {
      const canvasX = centerX + pos.position[0] * scale
      const canvasY = centerY - pos.position[1] * scale
      
      // Draw point
      ctx.fillStyle = index === positions.length - 1 ? '#FF5722' : '#2196F3'
      ctx.beginPath()
      ctx.arc(canvasX, canvasY, 4, 0, 2 * Math.PI)
      ctx.fill()
      
      // Draw orientation arrow for the most recent position
      if (index === positions.length - 1) {
        const arrowLength = 30
        const arrowX = canvasX - Math.sin(pos.orientation) * arrowLength
        const arrowY = canvasY - Math.cos(pos.orientation) * arrowLength
        
        ctx.strokeStyle = '#FF5722'
        ctx.lineWidth = 3
        ctx.beginPath()
        ctx.moveTo(canvasX, canvasY)
        ctx.lineTo(arrowX, arrowY)
        ctx.stroke()
        
        // Draw arrowhead
        const headLength = 10
        const angle = Math.atan2(canvasY - arrowY, canvasX - arrowX)
        ctx.beginPath()
        ctx.moveTo(arrowX, arrowY)
        ctx.lineTo(
          arrowX + headLength * Math.cos(angle - Math.PI / 6),
          arrowY + headLength * Math.sin(angle - Math.PI / 6)
        )
        ctx.moveTo(arrowX, arrowY)
        ctx.lineTo(
          arrowX + headLength * Math.cos(angle + Math.PI / 6),
          arrowY + headLength * Math.sin(angle + Math.PI / 6)
        )
        ctx.stroke()
      }
    })

    // Draw position info
    if (positions.length > 0) {
      const lastPos = positions[positions.length - 1]
      ctx.fillStyle = '#000000'
      ctx.font = '14px monospace'
      ctx.fillText(`Position: (${lastPos.position[0].toFixed(2)}m, ${lastPos.position[1].toFixed(2)}m)`, 10, 20)
      ctx.fillText(`Orientation: ${(lastPos.orientation * 180 / Math.PI).toFixed(1)}°`, 10, 40)
      ctx.fillText(`Points: ${positions.length}`, 10, 60)
    }
  }, [packets])

  return (
    <div style={{ margin: '20px', padding: '10px', border: '1px solid #ccc', borderRadius: '4px' }}>
      <h2>Robot Position Plot</h2>
      <canvas
        ref={canvasRef}
        width={800}
        height={600}
        style={{ border: '1px solid #333', backgroundColor: '#f5f5f5' }}
      />
    </div>
  )
}

export default PositionPlot
