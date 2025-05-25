import { useState, useEffect } from 'react'
import SchemaSelector from './mutation/SchemaSelector'
import MutationEditor from './mutation/MutationEditor'
import ResultViewer from './mutation/ResultViewer'

function MutationTab({ schemas, onResult }) {
  const [selectedSchema, setSelectedSchema] = useState('')
  const [mutationData, setMutationData] = useState({})
  const [mutationType, setMutationType] = useState('Create')
  const [sampleMutations, setSampleMutations] = useState([])
  const [selectedSample, setSelectedSample] = useState('')
  const [loadingSample, setLoadingSample] = useState(false)
  const [samplesError, setSamplesError] = useState(null)
  const [result, setResult] = useState(null)

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
      setResult(data)
      onResult(data)
      if (data.success) setMutationData({})
    } catch (error) {
      console.error('Failed to execute mutation:', error)
      const errData = { error: 'Failed to execute mutation' }
      setResult(errData)
      onResult(errData)
    }
  }

  const runSampleMutation = async () => {
    if (!selectedSample) return
    setLoadingSample(true)
    setSamplesError(null)

    try {
      const resp = await fetch(`/api/samples/mutation/${selectedSample}`)
      if (!resp.ok) throw new Error(`Failed to fetch sample: ${resp.status}`)
      const mutation = await resp.json()
      const execResp = await fetch('/api/mutation', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(mutation)
      })
      const data = await execResp.json()
      setResult(data)
      onResult(data)
      if (data.success) setMutationData({})
      setSelectedSample('')
    } catch (err) {
      console.error('Failed to run sample mutation:', err)
      setSamplesError('Failed to run sample mutation')
      const errData = { error: 'Failed to run sample mutation' }
      setResult(errData)
      onResult(errData)
    } finally {
      setLoadingSample(false)
    }
  }

  const selectedSchemaFields = selectedSchema
    ? schemas.find(s => s.name === selectedSchema)?.fields || {}
    : {}

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
