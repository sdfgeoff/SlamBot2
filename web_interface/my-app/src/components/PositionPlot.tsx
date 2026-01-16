import { useEffect, useRef, useState } from 'react'
import type { AnyPacketEntry, PacketEntry } from '../logTypes'
import type { PositionEstimate, AnyPacketFormat } from '../messageFormat'
import { MotionRequestMode } from '../messageFormat'

interface PositionPlotProps {
  packets: AnyPacketEntry[]
  send?: (message: AnyPacketFormat) => boolean
}

function isPositionEstimate(data: AnyPacketEntry): data is PacketEntry<PositionEstimate> {
  return 'PositionEstimate' in data.packet.data
}

function PositionPlot({ packets, send }: PositionPlotProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [targetPosition, setTargetPosition] = useState<[number, number] | null>(null)

  const handleCanvasClick = (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!send) return

    const canvas = canvasRef.current
    if (!canvas) return

    const rect = canvas.getBoundingClientRect()
    const clickX = event.clientX - rect.left
    const clickY = event.clientY - rect.top

    // Set up coordinate system (same as in useEffect)
    const centerX = canvas.width / 2
    const centerY = canvas.height / 2
    const scale = 100 // pixels per meter

    // Convert canvas coordinates to world coordinates
    const worldX = (clickX - centerX) / scale
    const worldY = -(clickY - centerY) / scale // Invert Y for world coordinates

    setTargetPosition([worldX, worldY])

    // Send MotionTargetRequest
    const message: AnyPacketFormat = {
      to: null,
      from: null,
      time: BigInt(Date.now()),
      id: 0,
      data: {
        MotionTargetRequest: {
          linear: [worldX, worldY],
          angular: 0.0, // TODO: this should be a f32
          // motion_mode: MotionRequestMode.Position,
        },
      },
    }

    send(message)
    console.log(`Navigating to: (${worldX.toFixed(2)}m, ${worldY.toFixed(2)}m)`)
  }

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

    // Draw target position marker
    if (targetPosition) {
      const targetCanvasX = centerX + targetPosition[0] * scale
      const targetCanvasY = centerY - targetPosition[1] * scale
      
      // Draw target circle
      ctx.strokeStyle = '#4CAF50'
      ctx.lineWidth = 3
      ctx.beginPath()
      ctx.arc(targetCanvasX, targetCanvasY, 10, 0, 2 * Math.PI)
      ctx.stroke()
      
      // Draw crosshair
      ctx.beginPath()
      ctx.moveTo(targetCanvasX - 15, targetCanvasY)
      ctx.lineTo(targetCanvasX + 15, targetCanvasY)
      ctx.moveTo(targetCanvasX, targetCanvasY - 15)
      ctx.lineTo(targetCanvasX, targetCanvasY + 15)
      ctx.stroke()
      
      // Draw label
      ctx.fillStyle = '#4CAF50'
      ctx.font = '12px monospace'
      ctx.fillText(`Target: (${targetPosition[0].toFixed(2)}m, ${targetPosition[1].toFixed(2)}m)`, targetCanvasX + 20, targetCanvasY - 10)
    }
  }, [packets, targetPosition])

  return (
    <div style={{ margin: '20px', padding: '10px', border: '1px solid #ccc', borderRadius: '4px' }}>
      <h2>Robot Position Plot</h2>
      <p style={{ color: '#666', fontSize: '14px', margin: '10px 0' }}>
        Click on the plot to send the robot to that position
      </p>
      <canvas
        ref={canvasRef}
        width={800}
        height={600}
        style={{ border: '1px solid #333', backgroundColor: '#f5f5f5', cursor: send ? 'crosshair' : 'default' }}
        onClick={handleCanvasClick}
      />
    </div>
  )
}

export default PositionPlot
