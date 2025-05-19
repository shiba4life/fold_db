import { useState, useEffect } from 'react'
import Header from './components/Header'
import Footer from './components/Footer'
import StatusSection from './components/StatusSection'
import ResultsSection from './components/ResultsSection'
import SchemaTab from './components/tabs/SchemaTab'
import QueryTab from './components/tabs/QueryTab'
import MutationTab from './components/tabs/MutationTab'
import TransformsTab from './components/tabs/TransformsTab'

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
      setSchemas(data.data || [])
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

  const renderActiveTab = () => {
    switch (activeTab) {
      case 'schemas':
        return <SchemaTab schemas={schemas} onResult={handleOperationResult} />
      case 'query':
        return <QueryTab schemas={schemas} onResult={handleOperationResult} />
      case 'mutation':
        return <MutationTab schemas={schemas} onResult={handleOperationResult} />
      case 'transforms':
        return <TransformsTab schemas={schemas} onResult={handleOperationResult} />
      default:
        return null
    }
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <Header />
      <main className="container mx-auto px-4 py-6">
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
                activeTab === 'transforms'
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
              onClick={() => handleTabChange('transforms')}
            >
              Transforms
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
  )
}

export default App
