import { useState, useEffect } from 'react'

function QueryTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [queryFields, setQueryFields] = useState([])
  const [sampleQueries, setSampleQueries] = useState([])
  const [selectedSample, setSelectedSample] = useState('')
  const [loadingSample, setLoadingSample] = useState(false)
  const [samplesError, setSamplesError] = useState(null)

  useEffect(() => {
    fetchSampleQueries()
  }, [])

  const fetchSampleQueries = async () => {
    try {
      const resp = await fetch('/api/samples/queries')
      const data = await resp.json()
      setSampleQueries(data.data || [])
    } catch (err) {
      console.error('Failed to fetch sample queries:', err)
      setSamplesError('Failed to load sample queries')
    }
  }

  const handleSchemaChange = (e) => {
    const schemaName = e.target.value
    setSelectedSchema(schemaName)
    setQueryFields([])
  }

  const handleFieldToggle = (fieldName) => {
    setQueryFields(prev => {
      if (prev.includes(fieldName)) {
        return prev.filter(f => f !== fieldName)
      }
      return [...prev, fieldName]
    })
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    
    if (!selectedSchema || queryFields.length === 0) {
      return
    }

    const query = {
      type: 'query',
      schema: selectedSchema,
      fields: queryFields
    }

    try {
      const response = await fetch('/api/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(query)
      })

      const data = await response.json()
      onResult(data)
    } catch (error) {
      console.error('Failed to execute query:', error)
      onResult({ error: 'Failed to execute query' })
    }
  }

  const runSampleQuery = async () => {
    if (!selectedSample) return
    setLoadingSample(true)
    setSamplesError(null)

    try {
      const resp = await fetch(`/api/samples/query/${selectedSample}`)
      if (!resp.ok) {
        throw new Error(`Failed to fetch sample: ${resp.status}`)
      }
      const query = await resp.json()
      const execResp = await fetch('/api/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(query)
      })
      const data = await execResp.json()
      onResult(data)
      setSelectedSample('')
    } catch (err) {
      console.error('Failed to run sample query:', err)
      setSamplesError('Failed to run sample query')
      onResult({ error: 'Failed to run sample query' })
    } finally {
      setLoadingSample(false)
    }
  }

  const selectedSchemaFields = selectedSchema ? 
    schemas.find(s => s.name === selectedSchema)?.fields || {} : 
    {}

  return (
    <div className="p-6">
      <div className="mb-6">
        <h3 className="text-lg font-medium text-gray-900 mb-2">Run Sample Query</h3>
        <div className="flex items-center space-x-2">
          <select
            className="border-gray-300 rounded-md px-3 py-2"
            value={selectedSample}
            onChange={(e) => setSelectedSample(e.target.value)}
          >
            <option value="">Select a sample...</option>
            {sampleQueries.map(name => (
              <option key={name} value={name}>{name}</option>
            ))}
          </select>
          <button
            type="button"
            onClick={runSampleQuery}
            disabled={!selectedSample || loadingSample}
            className={`px-4 py-2 text-sm font-medium rounded-md text-white ${
              !selectedSample || loadingSample
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90'
            }`}
          >
            {loadingSample ? 'Running...' : 'Run'}
          </button>
        </div>
        {samplesError && (
          <p className="mt-2 text-sm text-red-600">{samplesError}</p>
        )}
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Select Fold
          </label>
          <select
            className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-primary focus:border-primary rounded-md"
            value={selectedSchema}
            onChange={handleSchemaChange}
          >
            <option value="">Select a fold...</option>
            {schemas.map(schema => (
              <option key={schema.name} value={schema.name}>
                {schema.name}
              </option>
            ))}
          </select>
        </div>

        {selectedSchema && (
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-3">
              Select Fields
            </label>
            <div className="bg-gray-50 rounded-md p-4">
              <div className="space-y-3">
                {Object.entries(selectedSchemaFields).map(([fieldName, field]) => (
                  <label key={fieldName} className="relative flex items-start">
                    <div className="flex items-center h-5">
                      <input
                        type="checkbox"
                        className="h-4 w-4 text-primary border-gray-300 rounded focus:ring-primary"
                        checked={queryFields.includes(fieldName)}
                        onChange={() => handleFieldToggle(fieldName)}
                      />
                    </div>
                    <div className="ml-3 flex items-center">
                      <span className="text-sm font-medium text-gray-700">{fieldName}</span>
                      <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
                        {field.field_type}
                      </span>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </div>
        )}

        <div className="flex justify-end">
          <button
            type="submit"
            className={`
              inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white
              ${!selectedSchema || queryFields.length === 0
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
              }
            `}
            disabled={!selectedSchema || queryFields.length === 0}
          >
            Execute Query
          </button>
        </div>
      </form>
    </div>
  )
}

export default QueryTab