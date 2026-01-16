import { useState } from 'react'
import './App.css'
import { useHostConnection } from './useHostConnection'
import ConnectionPanel from './components/ConnectionPanel'
import PositionTab from './components/PositionTab'
import DiagnosticsTab from './components/DiagnosticsTab'

type Tab = 'position' | 'diagnostics'

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('position')
  const { status: wsStatus, registerCallback, send } = useHostConnection()

  return (
    <>
      <ConnectionPanel wsStatus={wsStatus} />
      <div style={{ margin: '20px', padding: '10px', borderBottom: '1px solid #ccc' }}>
        <button
          onClick={() => setActiveTab('position')}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            backgroundColor: activeTab === 'position' ? '#2196F3' : '#f0f0f0',
            color: activeTab === 'position' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '16px',
            fontWeight: activeTab === 'position' ? 'bold' : 'normal',
          }}
        >
          Position
        </button>
        <button
          onClick={() => setActiveTab('diagnostics')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeTab === 'diagnostics' ? '#2196F3' : '#f0f0f0',
            color: activeTab === 'diagnostics' ? 'white' : 'black',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            fontSize: '16px',
            fontWeight: activeTab === 'diagnostics' ? 'bold' : 'normal',
          }}
        >
          Diagnostics
        </button>
      </div>
      {activeTab === 'position' && <PositionTab registerCallback={registerCallback} send={send} />}
      {activeTab === 'diagnostics' && <DiagnosticsTab registerCallback={registerCallback} />}
    </>
  )
}

export default App
