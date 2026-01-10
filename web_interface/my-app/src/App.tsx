import { useEffect, useRef, useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'

function App() {
  const [count, setCount] = useState(0)
  const [wsStatus, setWsStatus] = useState<'connecting' | 'open' | 'closed' | 'error'>('connecting')
  const [lastMessage, setLastMessage] = useState<string>('')
  const [outgoingMessage, setOutgoingMessage] = useState('')
  const socketRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const socket = new WebSocket('ws://localhost:9001')
    socketRef.current = socket
    setWsStatus('connecting')

    const handleOpen = () => setWsStatus('open')
    const handleClose = () => setWsStatus('closed')
    const handleError = () => setWsStatus('error')
    const handleMessage = (event: MessageEvent) => {
      if (typeof event.data === 'string') {
        setLastMessage(event.data)
      } else {
        setLastMessage('[non-text message]')
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
  }, [])

  const sendMessage = () => {
    const socket = socketRef.current
    if (!socket || socket.readyState !== WebSocket.OPEN) {
      setWsStatus('error')
      return
    }
    if (!outgoingMessage.trim()) {
      return
    }
    socket.send(outgoingMessage)
    setOutgoingMessage('')
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
