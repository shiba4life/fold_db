import { useState, useEffect } from 'react'

function MutationTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [mutationData, setMutationData] = useState({})
  const [mutationType, setMutationType] = useState('create')
  const [sampleMutations, setSampleMutations] = useState([])
  const [selectedSample, setSelectedSample] = useState('')
  const [loadingSample, setLoadingSample] = useState(false)
  const [samplesError, setSamplesError] = useState(null)

  useEffect(() => {
    fetchSampleMutations()
  }, [])

  const fetchSampleMutations = async () => {
    try {
      const resp = await fetch('/api/samples/mutations')
      const data = await resp.json()
      setSampleMutations(data.data || [])
    } catch (err) {
      console.error('Failed to fetch sample mutations:', err)
      setSamplesError('Failed to load sample mutations')
    }
  }

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
    
    if (!selectedSchema) {
      return
    }

    // For delete operations, we don't need mutation data
    if (mutationType !== 'delete' && Object.keys(mutationData).length === 0) {
      return
    }

    const mutation = {
      type: 'mutation',
      schema: selectedSchema,
      mutation_type: mutationType.toLowerCase(), // Ensure lowercase mutation type
      data: mutationType === 'Delete' ? {} : mutationData
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

  const runSampleMutation = async () => {
    if (!selectedSample) return
    setLoadingSample(true)
    setSamplesError(null)

    try {
      const resp = await fetch(`/api/samples/mutation/${selectedSample}`)
      if (!resp.ok) {
        throw new Error(`Failed to fetch sample: ${resp.status}`)
      }
      const mutation = await resp.json()
      const execResp = await fetch('/api/mutation', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(mutation)
      })
      const data = await execResp.json()
      onResult(data)
      if (data.success) {
        setMutationData({})
      }
      setSelectedSample('')
    } catch (err) {
      console.error('Failed to run sample mutation:', err)
      setSamplesError('Failed to run sample mutation')
      onResult({ error: 'Failed to run sample mutation' })
    } finally {
      setLoadingSample(false)
    }
  }

  const selectedSchemaFields = selectedSchema ? 
    schemas.find(s => s.name === selectedSchema)?.fields || {} : 
    {}

  const renderField = (fieldName, field) => {
    // Don't show fields for delete operations
    if (mutationType === 'Delete' || !field.writable) {
      return null
    }

    const value = mutationData[fieldName] || ''

    switch (field.field_type) {
      case 'Collection':
        // Ensure value is always an array
        let arrayValue = [];
        if (value) {
          try {
            const parsed = typeof value === 'string' ? JSON.parse(value) : value;
            arrayValue = Array.isArray(parsed) ? parsed : [parsed];
          } catch (err) {
            // If parsing fails and we have a non-empty string, treat it as a single item
            arrayValue = value.trim() ? [value] : [];
          }
        }

        return (
          <div key={fieldName} className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {fieldName}
              <span className="ml-2 text-xs text-gray-500">Collection</span>
            </label>
            <textarea
              className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-primary focus:border-primary sm:text-sm font-mono"
              value={arrayValue.length > 0 ? JSON.stringify(arrayValue, null, 2) : ''}
              onChange={(e) => {
                const inputValue = e.target.value.trim();
                if (!inputValue) {
                  handleFieldChange(fieldName, []);
                  return;
                }
                try {
                  const parsed = JSON.parse(inputValue);
                  handleFieldChange(fieldName, Array.isArray(parsed) ? parsed : [parsed]);
                } catch (err) {
                  // If not valid JSON, treat as a single item
                  handleFieldChange(fieldName, [inputValue]);
                }
              }}
              placeholder={'Enter JSON array (e.g., ["item1", "item2"])'}
              rows={4}
            />
            <p className="mt-1 text-xs text-gray-500">
              Enter data as a JSON array. Empty input will create an empty array.
            </p>
          </div>
        )
      case 'Range':
        // Handle Range fields as key:value pairs
        let rangeValue = {};
        if (value) {
          try {
            rangeValue = typeof value === 'string' ? JSON.parse(value) : value;
            if (typeof rangeValue !== 'object' || Array.isArray(rangeValue)) {
              rangeValue = {};
            }
          } catch (err) {
            rangeValue = {};
          }
        }

        const rangeEntries = Object.entries(rangeValue);

        const addKeyValuePair = () => {
          const newEntries = [...rangeEntries, ['', '']];
          const newRangeValue = Object.fromEntries(newEntries.filter(([k, v]) => k.trim() !== ''));
          handleFieldChange(fieldName, newRangeValue);
        };

        const updateKeyValuePair = (index, key, val) => {
          const newEntries = [...rangeEntries];
          newEntries[index] = [key, val];
          const newRangeValue = Object.fromEntries(newEntries.filter(([k, v]) => k.trim() !== ''));
          handleFieldChange(fieldName, newRangeValue);
        };

        const removeKeyValuePair = (index) => {
          const newEntries = rangeEntries.filter((_, i) => i !== index);
          const newRangeValue = Object.fromEntries(newEntries);
          handleFieldChange(fieldName, newRangeValue);
        };

        return (
          <div key={fieldName} className="mb-6">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {fieldName}
              <span className="ml-2 text-xs text-gray-500">Range</span>
            </label>
            <div className="border border-gray-300 rounded-md p-4 bg-gray-50">
              <div className="space-y-3">
                {rangeEntries.length === 0 ? (
                  <p className="text-sm text-gray-500 italic">No key-value pairs added yet</p>
                ) : (
                  rangeEntries.map(([key, val], index) => (
                    <div key={index} className="flex items-center space-x-2">
                      <input
                        type="text"
                        placeholder="Key"
                        className="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-primary focus:border-primary sm:text-sm"
                        value={key}
                        onChange={(e) => updateKeyValuePair(index, e.target.value, val)}
                      />
                      <span className="text-gray-500">:</span>
                      <input
                        type="text"
                        placeholder="Value"
                        className="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-primary focus:border-primary sm:text-sm"
                        value={val}
                        onChange={(e) => updateKeyValuePair(index, key, e.target.value)}
                      />
                      <button
                        type="button"
                        onClick={() => removeKeyValuePair(index)}
                        className="text-red-600 hover:text-red-800 p-1"
                        title="Remove this key-value pair"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  ))
                )}
                <button
                  type="button"
                  onClick={addKeyValuePair}
                  className="inline-flex items-center px-3 py-2 border border-gray-300 shadow-sm text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary"
                >
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Add Key-Value Pair
                </button>
              </div>
            </div>
            <p className="mt-1 text-xs text-gray-500">
              Add key-value pairs for this range field. Empty keys will be filtered out.
            </p>
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
      <div className="mb-6">
        <h3 className="text-lg font-medium text-gray-900 mb-2">Run Sample Mutation</h3>
        <div className="flex items-center space-x-2">
          <select
            className="border-gray-300 rounded-md px-3 py-2"
            value={selectedSample}
            onChange={(e) => setSelectedSample(e.target.value)}
          >
            <option value="">Select a sample...</option>
            {sampleMutations.map(name => (
              <option key={name} value={name}>{name}</option>
            ))}
          </select>
          <button
            type="button"
            onClick={runSampleMutation}
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
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Schema
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
            <p className="mt-1 text-xs text-gray-500">Select the schema to operate on</p>
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Operation Type
            </label>
            <select
              className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-primary focus:border-primary rounded-md"
              value={mutationType}
              onChange={(e) => setMutationType(e.target.value)}
            >
              <option value="Create">Create - Add new data</option>
              <option value="Update">Update - Modify existing data</option>
              <option value="Delete">Delete - Remove existing data</option>
            </select>
            <p className="mt-1 text-xs text-gray-500">Choose the type of mutation to perform</p>
          </div>
        </div>

        {selectedSchema && (
          <>
            {mutationType === 'Delete' ? (
              <div className="bg-gray-50 rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Delete Operation</h3>
                <p className="text-sm text-gray-600">
                  This will delete the selected schema. No additional fields are required.
                </p>
              </div>
            ) : (
              <div className="bg-gray-50 rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Schema Fields</h3>
                <div className="space-y-6">
                  {Object.entries(selectedSchemaFields).map(([fieldName, field]) =>
                    renderField(fieldName, field)
                  )}
                </div>
              </div>
            )}
          </>
        )}

        <div className="flex justify-end pt-4">
          <button
            type="submit"
            className={`
              inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white
              ${!selectedSchema || (mutationType !== 'Delete' && Object.keys(mutationData).length === 0)
                ? 'bg-gray-300 cursor-not-allowed'
                : 'bg-primary hover:bg-primary/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary'
              }
            `}
            disabled={!selectedSchema || (mutationType !== 'Delete' && Object.keys(mutationData).length === 0)}
          >
            Execute Mutation
          </button>
        </div>
      </form>
    </div>
  )
}

export default MutationTab