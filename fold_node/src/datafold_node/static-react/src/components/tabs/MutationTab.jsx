import { useState } from 'react'

function MutationTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [mutationData, setMutationData] = useState({})

  const handleSchemaChange = (e) => {
    const schemaName = e.target.value
    setSelectedSchema(schemaName)
    setMutationData({})
  }

  const handleFieldChange = (fieldName, value) => {
    setMutationData(prev => ({
      ...prev,
      [fieldName]: value
    }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    
    if (!selectedSchema || Object.keys(mutationData).length === 0) {
      return
    }

    const mutation = {
      type: 'mutation',
      schema: selectedSchema,
      data: mutationData
    }

    try {
      const response = await fetch('/api/mutation', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(mutation)
      })

      const data = await response.json()
      onResult(data)
      
      if (data.success) {
        setMutationData({})
      }
    } catch (error) {
      console.error('Failed to execute mutation:', error)
      onResult({ error: 'Failed to execute mutation' })
    }
  }

  const selectedSchemaFields = selectedSchema ? 
    schemas.find(s => s.name === selectedSchema)?.fields || {} : 
    {}

  const renderField = (fieldName, field) => {
    if (!field.writable) {
      return null
    }

    const value = mutationData[fieldName] || ''

    switch (field.field_type) {
      case 'Collection':
        return (
          <div key={fieldName} className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {fieldName}
              <span className="ml-2 text-xs text-gray-500">Collection</span>
            </label>
            <textarea
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary focus:border-primary sm:text-sm font-mono"
              value={value}
              onChange={(e) => handleFieldChange(fieldName, e.target.value)}
              placeholder="Enter JSON array"
              rows={4}
            />
            <p className="mt-1 text-xs text-gray-500">Enter data as a JSON array</p>
          </div>
        )
      default:
        return (
          <div key={fieldName} className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {fieldName}
              <span className="ml-2 text-xs text-gray-500">Single</span>
            </label>
            <input
              type="text"
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary focus:border-primary sm:text-sm"
              value={value}
              onChange={(e) => handleFieldChange(fieldName, e.target.value)}
              placeholder={`Enter ${fieldName}`}
            />
          </div>
        )
    }
  }

  return (
    <div className="p-6">
      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Select Schema
          </label>
          <select
            className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-primary focus:border-primary rounded-md"
            value={selectedSchema}
            onChange={handleSchemaChange}
          >
            <option value="">Select a schema...</option>
            {schemas.map(schema => (
              <option key={schema.name} value={schema.name}>
                {schema.name}
              </option>
            ))}
          </select>
        </div>

        {selectedSchema && (
          <div className="bg-gray-50 rounded-lg p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Schema Fields</h3>
            <div className="space-y-6">
              {Object.entries(selectedSchemaFields).map(([fieldName, field]) =>
                renderField(fieldName, field)
              )}
            </div>
          </div>
        )}

        <div className="flex justify-end pt-4">
          <button
            type="submit"
            className={`
              inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white
              ${!selectedSchema || Object.keys(mutationData).length === 0
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
              }
            `}
            disabled={!selectedSchema || Object.keys(mutationData).length === 0}
          >
            Execute Mutation
          </button>
        </div>
      </form>
    </div>
  )
}

export default MutationTab