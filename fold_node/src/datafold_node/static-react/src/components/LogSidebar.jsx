import { useEffect, useRef, useState } from 'react'

function LogSidebar() {
  const [logs, setLogs] = useState([])
  const endRef = useRef(null)

  useEffect(() => {
    fetch('/api/logs')
      .then(res => res.json())
      .then(data => setLogs(data))
      .catch(() => {})

    const es = new EventSource('/api/logs/stream')
    es.onmessage = (e) => {
      setLogs(prev => [...prev, e.data])
    }
    return () => es.close()
  }, [])

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  return (
    <aside className="w-80 h-screen bg-gray-900 text-white p-4 overflow-y-auto">
      <h2 className="text-lg font-semibold mb-2">Logs</h2>
      <div className="space-y-1 text-xs font-mono">
        {logs.map((line, idx) => (
          <div key={idx}>{line}</div>
        ))}
        <div ref={endRef}></div>
      </div>
    </aside>
  )
}

export default LogSidebar
