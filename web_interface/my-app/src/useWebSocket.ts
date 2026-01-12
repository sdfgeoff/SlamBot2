import { useCallback, useEffect, useRef, useState } from 'react'
import { decodePacket, encodePacket, framePacket, PacketFinder } from './packetEncoding'

export type WebSocketStatus = 'connecting' | 'open' | 'closed' | 'error'

const defaultUrl = () => `ws://${window.location.hostname}:9001`
const RETRY_INTERVAL_MS = 10000
export const useWebSocket = <T>(
  onMessage: (message: T) => void,
  url = defaultUrl(),
) => {
  const [status, setStatus] = useState<WebSocketStatus>('connecting')
  const socketRef = useRef<WebSocket | null>(null)
  const packetFinderRef = useRef(new PacketFinder())
  const reconnectTimerRef = useRef<number | null>(null)

  useEffect(() => {
    let isClosed = false

    const clearReconnectTimer = () => {
      if (reconnectTimerRef.current !== null) {
        window.clearTimeout(reconnectTimerRef.current)
        reconnectTimerRef.current = null
      }
    }

    const scheduleReconnect = () => {
      if (isClosed || reconnectTimerRef.current !== null) {
        return
      }
      reconnectTimerRef.current = window.setTimeout(() => {
        reconnectTimerRef.current = null
        connect()
      }, RETRY_INTERVAL_MS)
    }

    const handleMessage = (event: MessageEvent) => {
      if (event.data instanceof ArrayBuffer) {
        const bytes = new Uint8Array(event.data)
        const packets = packetFinderRef.current.pushBytes(bytes)
        const decodedMessages: T[] = []

        const tryDecode = (packet: Uint8Array) => {
          if (packet.length === 0) {
            return
          }
          try {
            decodedMessages.push(decodePacket<T>(packet))
          } catch (error) {
            // eslint-disable-next-line no-console
            console.error(error)
          }
        }

        if (packets.length > 0) {
          packets.forEach(tryDecode)
        }


        if (decodedMessages.length > 0) {
          onMessage(decodedMessages[decodedMessages.length - 1])
        }
        return
      }
    }

    const connect = () => {
      clearReconnectTimer()
      setStatus('connecting')
      const socket = new WebSocket(url)
      socket.binaryType = 'arraybuffer'
      socketRef.current = socket

      const handleOpen = () => {
        clearReconnectTimer()
        setStatus('open')
      }
      const handleClose = () => {
        setStatus('closed')
        scheduleReconnect()
      }
      const handleError = () => {
        setStatus('error')
        scheduleReconnect()
      }

      socket.addEventListener('open', handleOpen)
      socket.addEventListener('close', handleClose)
      socket.addEventListener('error', handleError)
      socket.addEventListener('message', handleMessage)
    }

    connect()

    return () => {
      isClosed = true
      clearReconnectTimer()
      const socket = socketRef.current
      if (socket) {
        socket.close()
      }
      socketRef.current = null
    }
  }, [onMessage, url])

  const send = useCallback((message: T): boolean => {
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
  }, [setStatus, socketRef])

  return { send, status }
}
