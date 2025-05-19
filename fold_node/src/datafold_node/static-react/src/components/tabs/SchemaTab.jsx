import { useState } from 'react'
import { ChevronDownIcon, ChevronRightIcon } from '@heroicons/react/solid'

function SchemaTab({ schemas, onResult }) {
  const [expandedSchemas, setExpandedSchemas] = useState({})

  const toggleSchema = (schemaName) => {
    setExpandedSchemas(prev => ({
      ...prev,
      [schemaName]: !prev[schemaName]
    }))
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
    <div className="p-6 space-y-4">
      {schemas.map(renderSchema)}
    </div>
  )
}

export default SchemaTab