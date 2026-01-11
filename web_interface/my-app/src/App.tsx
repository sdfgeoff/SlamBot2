import { useCallback, useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'
import { useWebSocket } from './useWebSocket'

function App() {
  const [count, setCount] = useState(0)
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
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
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
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
