import { useCallback, useState } from 'react'
import './App.css'
import { useWebSocket } from './useWebSocket'

function App() {
  const [lastMessage, setLastMessage] = useState<string>('')
  const [outgoingMessage, setOutgoingMessage] = useState('')

  const handleMessage = useCallback((message: unknown) => {
    if (typeof message === 'string') {
      setLastMessage(message)
      return
    }
    try {
      setLastMessage(
        JSON.stringify(message, (_, value) => (typeof value === 'bigint' ? Number(value) : value)),
      )
    } catch {
      setLastMessage('[unserializable message]')
    }
  }, [])

  const { send, status: wsStatus } = useWebSocket(handleMessage)

  const sendMessage = () => {
    if (!outgoingMessage.trim()) {
      return
    }
    const sent = send(outgoingMessage)
    if (sent) {
      setOutgoingMessage('')
    }
  }

  return (
    <>
      <div className="card">
        <p>
          WebSocket: <strong>{wsStatus}</strong>
        </p>
        <p>
          Last message: <code>{lastMessage || '—'}</code>
        </p>
        <div>
          <input
            type="text"
            value={outgoingMessage}
            onChange={(event) => setOutgoingMessage(event.target.value)}
            placeholder="Type a message"
          />
          <button onClick={sendMessage} disabled={wsStatus !== 'open'}>
            Send
          </button>
        </div>
      </div>
    </>
  )
}

export default App
