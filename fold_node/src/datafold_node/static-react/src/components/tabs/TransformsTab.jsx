import { useState, useEffect } from 'react'

const TransformsTab = ({ schemas, onResult }) => {
  const [transforms, setTransforms] = useState([])
  const [loading, setLoading] = useState({})
  const [error, setError] = useState({})
  const [queueInfo, setQueueInfo] = useState({
    queue: [],
    length: 0,
    isEmpty: true
  })

  useEffect(() => {
    // Filter schemas to find those with transform fields
    const transformSchemas = schemas.filter(schema =>
      Object.values(schema.fields).some(field => field.transform)
    )
    setTransforms(transformSchemas)

    // Fetch queue information
    const fetchQueueInfo = async () => {
      try {
        const response = await fetch('/api/transforms/queue')
        const data = await response.json()
        setQueueInfo(data)
      } catch (error) {
        console.error('Failed to fetch transform queue info:', error)
      }
    }

    fetchQueueInfo()
    // Poll for queue updates every 5 seconds
    const interval = setInterval(fetchQueueInfo, 5000)
    return () => clearInterval(interval)
  }, [schemas])

  const handleAddToQueue = async (schemaName, fieldName, transform) => {
    const transformId = `${schemaName}.${fieldName}`
    console.log('Adding transform to queue:', transformId)
    setLoading(prev => ({ ...prev, [transformId]: true }))
    setError(prev => ({ ...prev, [transformId]: null }))
    
    try {
      const response = await fetch(`/api/transforms/queue/${transformId}`, {
        method: 'POST'
      })
      const responseData = await response.json()
      
      if (!response.ok) {
        throw new Error(responseData.error || 'Failed to add transform to queue')
      }
      
      console.log('Transform added successfully:', responseData)
      
      // Refresh queue info immediately
      const queueResponse = await fetch('/api/transforms/queue')
      const data = await queueResponse.json()
      console.log('Updated queue info:', data)
      setQueueInfo(data)
    } catch (error) {
      console.error('Failed to add transform to queue:', error)
      setError(prev => ({ ...prev, [transformId]: error.message }))
    } finally {
      setLoading(prev => ({ ...prev, [transformId]: false }))
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-xl font-semibold text-gray-800">Transforms</h2>
        <div className="text-sm text-gray-600">
          Queue Status: {queueInfo.isEmpty ? 'Empty' : `${queueInfo.length} transform(s) queued`}
        </div>
      </div>

      {!queueInfo.isEmpty && (
        <div className="bg-blue-50 p-4 rounded-lg mb-4">
          <h3 className="text-md font-medium text-blue-800 mb-2">Transform Queue</h3>
          <ul className="list-disc list-inside space-y-1">
            {queueInfo.queue.map((transformId, index) => (
              <li key={index} className="text-blue-700">
                {transformId}
              </li>
            ))}
          </ul>
        </div>
      )}

      {transforms.length === 0 ? (
        <p className="text-gray-500">No transforms found in schemas</p>
      ) : (
        <div className="space-y-6">
          {transforms.map((schema) => (
            <div key={schema.name} className="bg-white shadow rounded-lg p-4">
              <h3 className="text-lg font-medium text-gray-800 mb-2">{schema.name}</h3>
              <div className="space-y-4">
                {Object.entries(schema.fields).map(([fieldName, field]) => {
                  if (!field.transform) return null
                  return (
                    <div key={fieldName} className="border-l-4 border-primary pl-4">
                      <h4 className="font-medium text-gray-700">{fieldName}</h4>
                      <div className="mt-2 space-y-2">
                        <div className="text-sm">
                          <span className="font-medium">Transform Name:</span>{' '}
                          <span className="text-gray-600">{field.transform.name}</span>
                          <div className="flex items-center">
                            <button
                              onClick={() => handleAddToQueue(schema.name, fieldName, field.transform)}
                              disabled={loading[field.transform.name]}
                              className="ml-4 px-3 py-1 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-blue-300"
                            >
                              {loading[field.transform.name] ? 'Adding...' : 'Add to Queue'}
                            </button>
                            {error[field.transform.name] && (
                              <span className="ml-2 text-sm text-red-600">
                                Error: {error[field.transform.name]}
                              </span>
                            )}
                          </div>
                        </div>
                        <div className="text-sm">
                          <span className="font-medium">Logic:</span>{' '}
                          <code className="bg-gray-100 px-2 py-1 rounded text-gray-800">
                            {field.transform.logic}
                          </code>
                        </div>
                        <div className="text-sm">
                          <span className="font-medium">Reversible:</span>{' '}
                          <span className="text-gray-600">
                            {field.transform.reversible ? 'Yes' : 'No'}
                          </span>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default TransformsTab