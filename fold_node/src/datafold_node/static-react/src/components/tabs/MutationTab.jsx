import { useState } from 'react'
import SchemaSelector from './mutation/SchemaSelector'
import MutationEditor from './mutation/MutationEditor'
import ResultViewer from './mutation/ResultViewer'
import {
  isRangeSchema,
  formatRangeSchemaMutation,
  formatEnhancedRangeSchemaMutation,
  validateRangeKey,
  getRangeKey,
  getNonRangeKeyFields
} from '../../utils/rangeSchemaUtils'

function MutationTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [mutationData, setMutationData] = useState({})
  const [mutationType, setMutationType] = useState('Create')
  const [result, setResult] = useState(null)
  const [rangeKeyValue, setRangeKeyValue] = useState('')

  const handleSchemaChange = (schemaName) => {
    setSelectedSchema(schemaName)
    setMutationData({})
    setRangeKeyValue('')
  }

  const handleFieldChange = (fieldName, value) => {
    setMutationData(prev => ({ ...prev, [fieldName]: value }))
  }

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (!selectedSchema) return
    
    const selectedSchemaObj = schemas.find(s => s.name === selectedSchema)
    let mutation

    // Check if this is a range schema
    if (isRangeSchema(selectedSchemaObj)) {
      // Validate range key for range schemas
      const rangeKeyError = validateRangeKey(rangeKeyValue, mutationType !== 'Delete')
      if (rangeKeyError) {
        const errData = {
          error: rangeKeyError,
          details: 'Range key validation failed'
        }
        setResult(errData)
        onResult(errData)
        return
      }
      
      // For range schemas, use the enhanced format
      if (mutationType !== 'Delete' && Object.keys(mutationData).length === 0 && !rangeKeyValue.trim()) return
      mutation = formatEnhancedRangeSchemaMutation(selectedSchemaObj, mutationType, rangeKeyValue, mutationData)
    } else {
      // For regular schemas, use the existing logic
      if (mutationType !== 'Delete' && Object.keys(mutationData).length === 0) return
      mutation = {
        type: 'mutation',
        schema: selectedSchema,
        mutation_type: mutationType.toLowerCase(),
        data: mutationType === 'Delete' ? {} : mutationData
      }
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
      if (data.success) {
        setMutationData({})
        setRangeKeyValue('')
      }
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

  const selectedSchemaObj = selectedSchema
    ? schemas.find(s => s.name === selectedSchema)
    : null

  const isCurrentSchemaRangeSchema = selectedSchemaObj ? isRangeSchema(selectedSchemaObj) : false
  const rangeKey = selectedSchemaObj ? getRangeKey(selectedSchemaObj) : null

  // For range schemas, show only non-range-key fields in the editor
  // For regular schemas, show all fields
  const selectedSchemaFields = selectedSchemaObj
    ? (isCurrentSchemaRangeSchema ? getNonRangeKeyFields(selectedSchemaObj) : selectedSchemaObj.fields || {})
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

        {/* Range Key Input - show for range schemas */}
        {selectedSchema && isCurrentSchemaRangeSchema && (
          <div className="bg-yellow-50 rounded-lg p-4">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Range Schema Configuration</h3>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                {rangeKey} (Range Key)
                {mutationType !== 'Delete' && (
                  <span className="ml-2 text-xs text-red-500">Required</span>
                )}
                {mutationType === 'Delete' && (
                  <span className="ml-2 text-xs text-blue-500">Optional for targeting</span>
                )}
              </label>
              <input
                type="text"
                className={`mt-1 block w-full rounded-md shadow-sm sm:text-sm ${
                  mutationType !== 'Delete' && !rangeKeyValue.trim()
                    ? 'border-red-300 focus:ring-red-500 focus:border-red-500'
                    : 'border-gray-300 focus:ring-primary focus:border-primary'
                }`}
                value={rangeKeyValue}
                onChange={(e) => setRangeKeyValue(e.target.value)}
                placeholder={`Enter ${rangeKey} value`}
                required={mutationType !== 'Delete'}
              />
              <p className="mt-1 text-xs text-gray-500">
                {mutationType === 'Delete'
                  ? `Optional: Provide ${rangeKey} value to target specific records for deletion.`
                  : `This value will be used as the range key for this ${mutationType.toLowerCase()} mutation.`
                }
              </p>
              {mutationType !== 'Delete' && !rangeKeyValue.trim() && (
                <p className="mt-1 text-xs text-red-500">
                  Range key is required for {mutationType.toLowerCase()} operations on range schemas.
                </p>
              )}
            </div>
          </div>
        )}

        {selectedSchema && (
          <MutationEditor
            fields={selectedSchemaFields}
            mutationType={mutationType}
            mutationData={mutationData}
            onFieldChange={handleFieldChange}
            isRangeSchema={isCurrentSchemaRangeSchema}
          />
        )}

        <div className="flex justify-end pt-4">
          <button
            type="submit"
            className={`inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white ${
              !selectedSchema ||
              (mutationType !== 'Delete' && Object.keys(mutationData).length === 0) ||
              (isCurrentSchemaRangeSchema && mutationType !== 'Delete' && !rangeKeyValue.trim())
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
            }`}
            disabled={
              !selectedSchema ||
              (mutationType !== 'Delete' && Object.keys(mutationData).length === 0) ||
              (isCurrentSchemaRangeSchema && mutationType !== 'Delete' && !rangeKeyValue.trim())
            }
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
