import { useEffect, useRef, useState } from 'react'
import { decodePacket, encodePacket, framePacket, PacketFinder } from './packetEncoding'

type WebSocketStatus = 'connecting' | 'open' | 'closed' | 'error'

export const useWebSocket = (onMessage: (message: unknown) => void) => {
  const [status, setStatus] = useState<WebSocketStatus>('connecting')
  const socketRef = useRef<WebSocket | null>(null)
  const packetFinderRef = useRef(new PacketFinder())

  useEffect(() => {
    const socket = new WebSocket('ws://localhost:9001')
    socket.binaryType = 'arraybuffer'
    socketRef.current = socket
    setStatus('connecting')

    const handleOpen = () => setStatus('open')
    const handleClose = () => setStatus('closed')
    const handleError = () => setStatus('error')
    const handleMessage = (event: MessageEvent) => {
      if (typeof event.data === 'string') {
        onMessage(event.data)
        return
      }

      if (event.data instanceof ArrayBuffer) {
        const bytes = new Uint8Array(event.data)
        const packets = packetFinderRef.current.pushBytes(bytes)
        const decodedMessages: unknown[] = []

        const tryDecode = (packet: Uint8Array) => {
          try {
            decodedMessages.push(decodePacket<unknown>(packet))
          } catch (error) {
            // eslint-disable-next-line no-console
            console.error(error)
          }
        }

        if (packets.length > 0) {
          packets.forEach(tryDecode)
        } else if (bytes.length > 0 && bytes[0] !== 0x00) {
          tryDecode(bytes)
        }

        if (decodedMessages.length > 0) {
          onMessage(decodedMessages[decodedMessages.length - 1])
        }
        return
      }
    }

    socket.addEventListener('open', handleOpen)
    socket.addEventListener('close', handleClose)
    socket.addEventListener('error', handleError)
    socket.addEventListener('message', handleMessage)

    return () => {
      socket.removeEventListener('open', handleOpen)
      socket.removeEventListener('close', handleClose)
      socket.removeEventListener('error', handleError)
      socket.removeEventListener('message', handleMessage)
      socket.close()
      socketRef.current = null
    }
  }, [onMessage])

  const send = (message: unknown): boolean => {
    const socket = socketRef.current
    if (!socket || socket.readyState !== WebSocket.OPEN) {
      setStatus('error')
      return false
    }
    try {
      const encoded = encodePacket(message)
      const framed = framePacket(encoded)
      socket.send(framed)
      return true
    } catch (error) {
      setStatus('error')
      // eslint-disable-next-line no-console
      console.error(error)
      return false
    }
  }

  return { send, status }
}
