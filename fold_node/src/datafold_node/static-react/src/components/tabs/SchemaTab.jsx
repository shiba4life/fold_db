import { useState, useEffect } from 'react'
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/24/solid'

function SchemaTab({ schemas, onResult, onSchemaUpdated }) {
  const [expandedSchemas, setExpandedSchemas] = useState({})
  const [sampleSchemas, setSampleSchemas] = useState([])
  const [selectedSample, setSelectedSample] = useState('')
  const [loadingSample, setLoadingSample] = useState(false)
  const [samplesError, setSamplesError] = useState(null)

  useEffect(() => {
    fetchSampleSchemas()
  }, [])

  const fetchSampleSchemas = async () => {
    try {
      const resp = await fetch('/api/samples/schemas')
      const data = await resp.json()
      setSampleSchemas(data.data || [])
    } catch (err) {
      console.error('Failed to fetch sample schemas:', err)
      setSamplesError('Failed to load sample schemas')
    }
  }

  const toggleSchema = (schemaName) => {
    setExpandedSchemas(prev => ({
      ...prev,
      [schemaName]: !prev[schemaName]
    }))
  }

  const removeSchema = async (schemaName) => {
    try {
      const resp = await fetch(`/api/schema/${schemaName}`, { method: 'DELETE' })
      if (!resp.ok) {
        throw new Error(`Failed to remove schema: ${resp.status}`)
      }
      if (onSchemaUpdated) {
        onSchemaUpdated()
      }
    } catch (err) {
      console.error('Failed to remove schema:', err)
    }
  }

  const loadSampleSchema = async () => {
    if (!selectedSample) return
    setLoadingSample(true)
    setSamplesError(null)

    try {
      const resp = await fetch(`/api/samples/schema/${selectedSample}`)
      if (!resp.ok) {
        throw new Error(`Failed to fetch sample: ${resp.status}`)
      }
      const schema = await resp.json()
      const createResp = await fetch('/api/schema', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(schema)
      })

      const data = await createResp.json()

      if (!createResp.ok) {
        throw new Error(data.error || 'Failed to load schema')
      }

      if (onResult) {
        onResult(data)
      }
      if (onSchemaUpdated) {
        onSchemaUpdated()
      }
      setSelectedSample('')
    } catch (err) {
      console.error('Failed to load sample schema:', err)
      setSamplesError('Failed to load sample schema')
      if (onResult) {
        onResult({ error: 'Failed to load sample schema' })
      }
    } finally {
      setLoadingSample(false)
    }
  }

  const renderField = (field, fieldName) => {
    return (
      <div key={fieldName} className="bg-gray-50 rounded-md p-4 hover:bg-gray-100 transition-colors duration-200">
        <div className="flex justify-between items-start">
          <div className="space-y-1">
            <div className="flex items-center">
              <span className="font-medium text-gray-900">{fieldName}</span>
              <span className="ml-2 px-2 py-0.5 text-xs font-medium rounded-full bg-gray-200 text-gray-700">
                {field.field_type}
              </span>
            </div>
            {field.transform && (
              <div className="flex items-center text-sm text-gray-600">
                <svg className="icon icon-xs mr-1" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clipRule="evenodd" />
                </svg>
                {field.transform.name}
              </div>
            )}
            {field.ref_atom_uuid && (
              <div className="text-xs text-gray-500 break-all">
                {field.ref_atom_uuid}
              </div>
            )}
          </div>
          <span className={`
            inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium
            ${field.writable
              ? 'bg-green-100 text-green-800'
              : 'bg-gray-100 text-gray-800'
            }
          `}>
            {field.writable ? 'Writable' : 'Read-only'}
          </span>
        </div>
      </div>
    )
  }

  const renderSchema = (schema) => {
    const isExpanded = expandedSchemas[schema.name]

    return (
      <div key={schema.name} className="bg-white rounded-lg border border-gray-200 shadow-sm overflow-hidden transition-all duration-200 hover:shadow-md">
        <div
          className="px-4 py-3 bg-gray-50 cursor-pointer select-none transition-colors duration-200 hover:bg-gray-100"
          onClick={() => toggleSchema(schema.name)}
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              {isExpanded ? (
                <ChevronDownIcon className="icon icon-sm text-gray-400 transition-transform duration-200" />
              ) : (
                <ChevronRightIcon className="icon icon-sm text-gray-400 transition-transform duration-200" />
              )}
              <h3 className="font-medium text-gray-900">{schema.name}</h3>
              <span className="text-xs text-gray-500">
                ({Object.keys(schema.fields).length} fields)
              </span>
            </div>
            <button
              className="group inline-flex items-center px-2 py-1 text-xs font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
              onClick={(e) => {
                e.stopPropagation()
                removeSchema(schema.name)
              }}
            >
              Remove
            </button>
          </div>
        </div>
        
        {isExpanded && (
          <div className="p-4 border-t border-gray-200">
            <div className="space-y-3">
              {Object.entries(schema.fields).map(([fieldName, field]) =>
                renderField(field, fieldName)
              )}
            </div>
          </div>
        )}
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h3 className="text-lg font-medium text-gray-900 mb-2">Load Sample Schema</h3>
        <div className="flex items-center space-x-2">
          <select
            className="border-gray-300 rounded-md px-3 py-2"
            value={selectedSample}
            onChange={(e) => setSelectedSample(e.target.value)}
          >
            <option value="">Select a sample...</option>
            {sampleSchemas.map(name => (
              <option key={name} value={name}>{name}</option>
            ))}
          </select>
          <button
            onClick={loadSampleSchema}
            disabled={!selectedSample || loadingSample}
            className={`px-4 py-2 text-sm font-medium rounded-md text-white ${
              !selectedSample || loadingSample
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90'
            }`}
          >
            {loadingSample ? 'Loading...' : 'Load'}
          </button>
        </div>
        {samplesError && (
          <p className="mt-2 text-sm text-red-600">{samplesError}</p>
        )}
      </div>

      <div className="space-y-4">
        {schemas.map(renderSchema)}
      </div>
    </div>
  )
}

export default SchemaTab