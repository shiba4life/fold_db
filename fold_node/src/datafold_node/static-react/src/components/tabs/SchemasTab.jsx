import { useState, useEffect } from 'react'

const ChevronDownIcon = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path d="M19 9l-7 7-7-7" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
)

const ViewListIcon = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path d="M4 6h16M4 10h16M4 14h16M4 18h16" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
)

const UploadIcon = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
)

const TrashIcon = ({ className }) => (
  <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor">
    <path d="M3 6h18M8 6V4h8v2m1 0v12a2 2 0 01-2 2H9a2 2 0 01-2-2V6h10z" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
  </svg>
)

function SchemasTab() {
  const [schemas, setSchemas] = useState([])
  const [expandedSchema, setExpandedSchema] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    loadSchemas()
  }, [])

  const loadSchemas = async () => {
    try {
      setLoading(true)
      setError(null)
      const response = await fetch('/api/schemas')
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }
      const result = await response.json()
      // The backend wraps the schemas in a data field
      setSchemas(result.data || [])
    } catch (err) {
      console.error('Failed to load schemas:', err)
      setError('Failed to load schemas. Please try again.')
    } finally {
      setLoading(false)
    }
  }

  const toggleSchema = (schemaId) => {
    setExpandedSchema(expandedSchema === schemaId ? null : schemaId)
  }

  const removeSchema = async (schemaName) => {
    try {
      const resp = await fetch(`/api/schema/${schemaName}`, {
        method: 'DELETE'
      })
      if (!resp.ok) {
        throw new Error(`Failed to remove schema: ${resp.status}`)
      }
      await loadSchemas()
    } catch (err) {
      console.error('Failed to remove schema:', err)
      setError('Failed to remove schema. Please try again.')
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-pulse flex items-center text-gray-500">
          <div className="h-5 w-5 border-2 border-current border-r-transparent rounded-full animate-spin mr-3"></div>
          Loading schemas...
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="rounded-md bg-red-50 p-4 text-red-700">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <p className="text-sm font-medium">{error}</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {schemas.map(schema => (
        <div
          key={schema.name}
          className="bg-white rounded-lg border border-gray-200 shadow-sm overflow-hidden transition-shadow hover:shadow-md"
        >
          <div
            className="px-4 py-3 bg-gray-50 cursor-pointer select-none"
            onClick={() => toggleSchema(schema.name)}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <ChevronDownIcon
                  className={`icon icon-sm text-gray-400 transform transition-transform duration-200 ${
                    expandedSchema === schema.name ? '' : '-rotate-90'
                  }`}
                />
                <ViewListIcon className="icon icon-sm text-primary" />
                <h3 className="font-medium text-gray-900">{schema.name}</h3>
              </div>
              <div className="flex items-center space-x-2">
                <button className="group inline-flex items-center px-3 py-1.5 text-sm font-medium rounded-md text-gray-700 bg-white border border-gray-300 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary">
                  <ViewListIcon className="icon icon-xs mr-1.5 text-gray-600 group-hover:text-gray-700" />
                  View
                </button>
                <button className="group inline-flex items-center px-3 py-1.5 text-sm font-medium rounded-md text-white bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary">
                  <UploadIcon className="icon icon-xs mr-1.5 text-white" />
                  Load
                </button>
                <button
                  className="group inline-flex items-center px-3 py-1.5 text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
                  onClick={(e) => {
                    e.stopPropagation()
                    removeSchema(schema.name)
                  }}
                >
                  <TrashIcon className="icon icon-xs mr-1.5 text-white" />
                  Remove
                </button>
              </div>
            </div>
          </div>
          {expandedSchema === schema.name && (
            <div className="px-4 py-3 border-t border-gray-200">
              <h4 className="text-lg font-medium text-gray-900 mb-3">Fields</h4>
              <div className="space-y-3">
                {Object.entries(schema.fields).map(([fieldName, fieldData]) => (
                  <div key={fieldName} className="bg-gray-50 rounded-md p-3">
                    <div className="flex justify-between items-center mb-2">
                      <span className="font-medium text-gray-900">{fieldName}</span>
                      <span className="px-2 py-1 text-xs font-medium rounded-full bg-gray-100 text-gray-600">
                        {fieldData.field_type}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 text-sm text-gray-600">
                      <div className="flex items-center">
                        <span className="font-medium mr-2">Read Policy:</span>
                        {fieldData.permission_policy.read_policy}
                      </div>
                      <div className="flex items-center">
                        <span className="font-medium mr-2">Write Policy:</span>
                        {fieldData.permission_policy.write_policy}
                      </div>
                      {fieldData.payment_config.min_payment && (
                        <div className="col-span-2 flex items-center text-primary">
                          <span className="font-medium mr-2">Min Payment:</span>
                          {fieldData.payment_config.min_payment}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      ))}
      {schemas.length === 0 && !loading && !error && (
        <div className="rounded-md bg-blue-50 p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <p className="text-sm font-medium text-blue-700">
                No schemas loaded. Use the Schema tab to load a schema.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default SchemasTab