import type { WebSocketStatus } from '../useWebSocket'

type ConnectionPanelProps = {
  wsStatus: WebSocketStatus
}

const ConnectionPanel = ({
  wsStatus,
}: ConnectionPanelProps) => (
  <div className="card">
    <p>
      WebSocket: <strong>{wsStatus}</strong>
    </p>
  </div>
)

export default ConnectionPanel
