import { useState, useEffect } from 'react'
import Header from './components/Header'
import Footer from './components/Footer'
import StatusSection from './components/StatusSection'
import ResultsSection from './components/ResultsSection'
import SchemaTab from './components/tabs/SchemaTab'
import QueryTab from './components/tabs/QueryTab'
import MutationTab from './components/tabs/MutationTab'
import TransformsTab from './components/tabs/TransformsTab'
import SchemaDependenciesTab from './components/tabs/SchemaDependenciesTab'
import IngestionTab from './components/tabs/IngestionTab'
import KeyManagementTab from './components/tabs/KeyManagementTab'
import LogSidebar from './components/LogSidebar'

function App() {
  const [activeTab, setActiveTab] = useState('schemas')
  const [results, setResults] = useState(null)
  const [schemas, setSchemas] = useState([])

  useEffect(() => {
    fetchSchemas()
  }, [])

  const fetchSchemas = async () => {
    try {
      const response = await fetch('/api/schemas')
      const data = await response.json()
      console.log('Schemas API response:', data)
      
      // Convert the state map to an array of schema objects with states
      const schemasWithStates = Object.entries(data.data || {}).map(([name, state]) => ({
        name,
        state,
        fields: {} // Will be populated below for approved schemas
      }))
      
      // Filter for approved schemas only (these are the ones available for mutations)
      const approvedSchemas = schemasWithStates.filter(
        (s) => s.state && s.state.toLowerCase() === 'approved'
      )
      console.log('Approved schemas:', approvedSchemas)
      
      // Fetch detailed schema information for approved schemas
      const schemasWithDetails = await Promise.all(
        approvedSchemas.map(async (schema) => {
          try {
            console.log(`Fetching details for schema: ${schema.name}`)
            const schemaResponse = await fetch(`/api/schema/${schema.name}`)
            console.log(`Schema ${schema.name} response status:`, schemaResponse.status)
            if (schemaResponse.ok) {
              const schemaData = await schemaResponse.json()
              console.log(`Schema ${schema.name} data:`, schemaData)
              return {
                ...schema,
                ...schemaData, // Include the full schema data including schema_type
                fields: schemaData.fields || {}
              }
            } else {
              console.error(`Failed to fetch schema ${schema.name}: ${schemaResponse.status}`)
            }
          } catch (err) {
            console.error(`Failed to fetch details for schema ${schema.name}:`, err)
          }
          return schema // Return original if fetch fails
        })
      )
      
      console.log('Final schemas with details:', schemasWithDetails)
      setSchemas(schemasWithDetails)
    } catch (error) {
      console.error('Failed to fetch schemas:', error)
    }
  }

  const handleTabChange = (tab) => {
    setActiveTab(tab)
    setResults(null)
  }

  const handleOperationResult = (result) => {
    setResults(result)
  }

  const handleSchemaUpdated = () => {
    fetchSchemas()
  }

  const renderActiveTab = () => {
    switch (activeTab) {
      case 'schemas':
        return (
          <SchemaTab
            schemas={schemas}
            onResult={handleOperationResult}
            onSchemaUpdated={handleSchemaUpdated}
          />
        )
      case 'query':
        return <QueryTab schemas={schemas} onResult={handleOperationResult} />
      case 'mutation':
        return <MutationTab schemas={schemas} onResult={handleOperationResult} />
      case 'ingestion':
        return <IngestionTab onResult={handleOperationResult} />
      case 'transforms':
        return <TransformsTab schemas={schemas} onResult={handleOperationResult} />
      case 'dependencies':
        return <SchemaDependenciesTab schemas={schemas} />
      case 'keys':
        return <KeyManagementTab onResult={handleOperationResult} />
      default:
        return null
    }
  }

  return (
    <div className="min-h-screen flex bg-gray-50">
      <div className="flex flex-col flex-1">
        <Header />
        <main className="container mx-auto px-4 py-6 flex-1">
          <StatusSection />

          <div className="mt-6">
            <div className="flex border-b border-gray-200">
              <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'schemas'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('schemas')}
            >
              Schemas
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'query'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('query')}
            >
              Query
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'mutation'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('mutation')}
            >
              Mutation
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'ingestion'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('ingestion')}
            >
              Ingestion
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'transforms'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('transforms')}
            >
              Transforms
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'dependencies'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('dependencies')}
            >
              Dependencies
            </button>
            <button
              className={`px-4 py-2 text-sm font-medium ${
                activeTab === 'keys'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('keys')}
            >
              Keys
            </button>
            </div>

            <div className="mt-4">
              {renderActiveTab()}
            </div>
          </div>

          {results && <ResultsSection results={results} />}
        </main>
        <Footer />
      </div>
      <LogSidebar />
    </div>
  )
}

export default App
