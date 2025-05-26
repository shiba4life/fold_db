import { CheckCircleIcon, ArrowPathIcon } from '@heroicons/react/24/solid'
import { useState } from 'react'

function StatusSection() {
  const [isRestarting, setIsRestarting] = useState(false)
  const [isSoftRestarting, setIsSoftRestarting] = useState(false)

  const handleRestart = async (soft = false) => {
    if (isRestarting || isSoftRestarting) return
    
    const restartType = soft ? 'soft restart' : 'full restart'
    const confirmed = window.confirm(
      `Are you sure you want to ${restartType} the fold_node? ${
        soft
          ? 'This will reinitialize the database while preserving network connections.'
          : 'This will stop all services and reinitialize everything.'
      }`
    )
    if (!confirmed) return

    const setLoading = soft ? setIsSoftRestarting : setIsRestarting
    setLoading(true)
    
    try {
      const endpoint = soft ? '/api/system/soft-restart' : '/api/system/restart'
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
      })
      
      const result = await response.json()
      
      if (response.ok && result.success) {
        alert(`${restartType} completed successfully!`)
      } else {
        throw new Error(result.error || `Failed to ${restartType} node`)
      }
    } catch (error) {
      console.error(`${restartType} failed:`, error)
      alert(`Failed to ${restartType} node: ${error.message}`)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="bg-white rounded-lg shadow-sm p-4 mb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <CheckCircleIcon className="icon icon-md text-green-500" />
          <span className="text-gray-700 font-medium">
            Node is running successfully
          </span>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => handleRestart(true)}
            disabled={isRestarting || isSoftRestarting}
            className={`flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              isSoftRestarting
                ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                : 'bg-blue-50 text-blue-700 hover:bg-blue-100 border border-blue-200'
            }`}
            title="Soft restart - reinitialize database while preserving network connections"
          >
            <ArrowPathIcon className={`w-4 h-4 ${isSoftRestarting ? 'animate-spin' : ''}`} />
            {isSoftRestarting ? 'Soft Restarting...' : 'Soft Restart'}
          </button>
          <button
            onClick={() => handleRestart(false)}
            disabled={isRestarting || isSoftRestarting}
            className={`flex items-center gap-2 px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              isRestarting
                ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                : 'bg-red-50 text-red-700 hover:bg-red-100 border border-red-200'
            }`}
            title="Full restart - stop all services and reinitialize everything"
          >
            <ArrowPathIcon className={`w-4 h-4 ${isRestarting ? 'animate-spin' : ''}`} />
            {isRestarting ? 'Restarting...' : 'Full Restart'}
          </button>
        </div>
      </div>
      <div className="mt-2 flex items-center gap-2">
        <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
        <span className="text-sm text-gray-500">Active and healthy</span>
      </div>
    </div>
  )
}

export default StatusSection