import { useEffect, useRef, useState } from 'react'

function LogSidebar() {
  const [logs, setLogs] = useState([])
  const endRef = useRef(null)

  const handleCopy = () => {
    Promise.resolve(
      navigator.clipboard.writeText(logs.join('\n'))
    ).catch(() => {})
  }

  useEffect(() => {
    fetch('/api/logs')
      .then(res => res.json())
      .then(data => setLogs(Array.isArray(data) ? data : []))
      .catch(() => setLogs([]))

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
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-lg font-semibold">Logs</h2>
        <button
          onClick={handleCopy}
          className="text-xs text-blue-300 hover:underline"
        >
          Copy
        </button>
      </div>
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
