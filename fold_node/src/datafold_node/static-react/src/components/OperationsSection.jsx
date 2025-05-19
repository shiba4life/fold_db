import { useState } from 'react'
import { 
  FolderIcon, 
  DocumentIcon, 
  SearchIcon, 
  CodeIcon, 
  LibraryIcon, 
  CogIcon, 
  ServerIcon
} from '@heroicons/react/24/solid'

import SchemasTab from './tabs/SchemasTab'
import SchemaTab from './tabs/SchemaTab'
import QueryTab from './tabs/QueryTab'
import MutationTab from './tabs/MutationTab'

const TABS = [
  { id: 'schemas', label: 'Loaded Schemas', icon: FolderIcon, component: SchemasTab },
  { id: 'schema', label: 'Schema', icon: DocumentIcon, component: SchemaTab },
  { id: 'query', label: 'Query', icon: SearchIcon, component: QueryTab },
  { id: 'mutation', label: 'Mutation', icon: CodeIcon, component: MutationTab },
  { id: 'samples', label: 'Samples', icon: LibraryIcon },
  { id: 'transforms', label: 'Transforms', icon: CogIcon },
  { id: 'network', label: 'Network', icon: ServerIcon }
]

function OperationsSection({ setResults }) {
  const [activeTab, setActiveTab] = useState('schemas')

  const renderTabContent = () => {
    const tab = TABS.find(t => t.id === activeTab)
    if (tab?.component) {
      const TabComponent = tab.component
      return <TabComponent setResults={setResults} />
    }
    return (
      <div className="rounded-md bg-blue-50 p-4 text-blue-700">
        <p className="text-sm">This tab is under development.</p>
      </div>
    )
  }

  return (
    <div className="bg-white rounded-lg shadow-sm mt-6">
      <div className="border-b border-gray-200 px-6 py-4">
        <h2 className="text-xl font-semibold text-gray-900">Operations</h2>
      </div>
      
      <div className="p-6">
        <div className="flex flex-col">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-2">
              {TABS.map(tab => (
                <button
                  key={tab.id}
                  className={`
                    group flex items-center px-4 py-2 border-b-2 text-sm font-medium
                    ${activeTab === tab.id
                      ? 'border-primary text-primary'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                    }
                    transition-colors duration-200
                  `}
                  onClick={() => setActiveTab(tab.id)}
                >
                  <tab.icon className="icon icon-sm mr-2 text-gray-500 group-hover:text-gray-700" />
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
          
          <div className="mt-6">
            {renderTabContent()}
          </div>
        </div>
      </div>
    </div>
  )
}

export default OperationsSection