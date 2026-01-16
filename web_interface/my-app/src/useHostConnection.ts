import { useCallback, useEffect, useRef } from 'react'
import { useWebSocket } from './useWebSocket'
import type { WebSocketStatus } from './useWebSocket'
import type { AnyPacketFormat } from './messageFormat'

type MessageCallback = (message: AnyPacketFormat) => void
type SubscriptionMap = Map<string, Set<MessageCallback>>

export const useHostConnection = (url?: string) => {
  const subscriptionsRef = useRef<SubscriptionMap>(new Map())
  const activeTopicsRef = useRef<Set<string>>(new Set())
  const subscriptionRequestIntervalRef = useRef<number | null>(null)
  const sendRef = useRef<((message: AnyPacketFormat) => boolean) | null>(null)
  const statusRef = useRef<WebSocketStatus>('connecting')

  const handleMessage = useCallback((message: AnyPacketFormat) => {
    // Notify all subscribers for 'all' topic
    const allCallbacks = subscriptionsRef.current.get('all')
    if (allCallbacks) {
      allCallbacks.forEach((callback) => callback(message))
    }

    // Notify subscribers for specific data keys
    const dataKeys = Object.keys(message.data)
    dataKeys.forEach((key) => {
      const callbacks = subscriptionsRef.current.get(key)
      if (callbacks) {
        callbacks.forEach((callback) => callback(message))
      }
    })
  }, [])

  const { send, status } = useWebSocket<AnyPacketFormat>(handleMessage, url)

  // Update refs when values change
  useEffect(() => {
    sendRef.current = send
    statusRef.current = status
  }, [send, status])

  const sendSubscriptionRequest = useCallback(() => {
    if (!sendRef.current || statusRef.current !== 'open') {
      return
    }

    const topics = Array.from(activeTopicsRef.current)
    if (topics.length === 0) {
      return
    }

    const message: AnyPacketFormat = {
      to: null,
      from: null,
      time: BigInt(Date.now()),
      id: Date.now() % 0xffffffff,
      data: {
        SubscriptionRequest: {
          topics,
        },
      },
    }

    sendRef.current(message)
  }, [])

  const registerCallback = useCallback(
    (topic: string, callback: MessageCallback): (() => void) => {
      // Add callback to the subscription map
      if (!subscriptionsRef.current.has(topic)) {
        subscriptionsRef.current.set(topic, new Set())
      }
      subscriptionsRef.current.get(topic)!.add(callback)
      activeTopicsRef.current.add(topic)

      // Send subscription request immediately if connected
      if (statusRef.current === 'open') {
        sendSubscriptionRequest()
      }

      // Return deregister function
      return () => {
        const callbacks = subscriptionsRef.current.get(topic)
        if (callbacks) {
          callbacks.delete(callback)
          if (callbacks.size === 0) {
            subscriptionsRef.current.delete(topic)
            activeTopicsRef.current.delete(topic)
            // Send updated subscription request
            if (statusRef.current === 'open') {
              sendSubscriptionRequest()
            }
          }
        }
      }
    },
    [sendSubscriptionRequest]
  )

  // Set up periodic subscription requests
  useEffect(() => {
    if (status === 'open') {
      // Send initial subscription request
      sendSubscriptionRequest()

      // Set up periodic subscription requests
      subscriptionRequestIntervalRef.current = window.setInterval(() => {
        sendSubscriptionRequest()
      }, 2000)

      return () => {
        if (subscriptionRequestIntervalRef.current !== null) {
          window.clearInterval(subscriptionRequestIntervalRef.current)
          subscriptionRequestIntervalRef.current = null
        }
      }
    }
  }, [status, sendSubscriptionRequest])

  return { send, status, registerCallback }
}
