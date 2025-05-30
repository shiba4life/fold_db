import { useState } from 'react'
import SchemaSelector from './mutation/SchemaSelector'
import MutationEditor from './mutation/MutationEditor'
import ResultViewer from './mutation/ResultViewer'

function MutationTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [mutationData, setMutationData] = useState({})
  const [mutationType, setMutationType] = useState('Create')
  const [result, setResult] = useState(null)

  const handleSchemaChange = (schemaName) => {
    setSelectedSchema(schemaName)
    setMutationData({})
  }

  const handleFieldChange = (fieldName, value) => {
    setMutationData(prev => ({ ...prev, [fieldName]: value }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!selectedSchema) return
    if (mutationType !== 'Delete' && Object.keys(mutationData).length === 0) return

    const mutation = {
      type: 'mutation',
      schema: selectedSchema,
      mutation_type: mutationType.toLowerCase(),
      data: mutationType === 'Delete' ? {} : mutationData
    }

    try {
      const response = await fetch('/api/mutation', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(mutation)
      })
      const data = await response.json()
      
      // Check if the HTTP response was successful
      if (!response.ok) {
        console.error('Mutation failed with status:', response.status, data)
        const errData = {
          error: data.error || `Mutation failed with status ${response.status}`,
          status: response.status,
          details: data
        }
        setResult(errData)
        onResult(errData)
        return
      }
      
      setResult(data)
      onResult(data)
      if (data.success) setMutationData({})
    } catch (error) {
      console.error('Failed to execute mutation:', error)
      const errData = {
        error: `Network error: ${error.message}`,
        details: error
      }
      setResult(errData)
      onResult(errData)
    }
  }

  const selectedSchemaFields = selectedSchema
    ? schemas.find(s => s.name === selectedSchema)?.fields || {}
    : {}

  return (
    <div className="p-6">
      <form onSubmit={handleSubmit} className="space-y-6">
        <SchemaSelector
          schemas={schemas}
          selectedSchema={selectedSchema}
          mutationType={mutationType}
          onSchemaChange={handleSchemaChange}
          onTypeChange={setMutationType}
        />

        {selectedSchema && (
          <MutationEditor
            fields={selectedSchemaFields}
            mutationType={mutationType}
            mutationData={mutationData}
            onFieldChange={handleFieldChange}
          />
        )}

        <div className="flex justify-end pt-4">
          <button
            type="submit"
            className={`inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white ${
              !selectedSchema || (mutationType !== 'Delete' && Object.keys(mutationData).length === 0)
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
            }`}
            disabled={!selectedSchema || (mutationType !== 'Delete' && Object.keys(mutationData).length === 0)}
          >
            Execute Mutation
          </button>
        </div>
      </form>

      <ResultViewer result={result} />
    </div>
  )
}

export default MutationTab
