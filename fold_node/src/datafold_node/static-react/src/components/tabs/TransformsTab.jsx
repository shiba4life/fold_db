import { useState, useEffect } from 'react'

const TransformsTab = ({ schemas, onResult }) => {
  const [transforms, setTransforms] = useState([])

  useEffect(() => {
    // Filter schemas to find those with transform fields
    const transformSchemas = schemas.filter(schema => 
      Object.values(schema.fields).some(field => field.transform)
    )
    setTransforms(transformSchemas)
  }, [schemas])

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold text-gray-800">Transforms</h2>
      
      {transforms.length === 0 ? (
        <p className="text-gray-500">No transforms found in schemas</p>
      ) : (
        <div className="space-y-6">
          {transforms.map((schema) => (
            <div key={schema.name} className="bg-white shadow rounded-lg p-4">
              <h3 className="text-lg font-medium text-gray-800 mb-2">{schema.name}</h3>
              <div className="space-y-4">
                {Object.entries(schema.fields).map(([fieldName, field]) => {
                  if (!field.transform) return null
                  return (
                    <div key={fieldName} className="border-l-4 border-primary pl-4">
                      <h4 className="font-medium text-gray-700">{fieldName}</h4>
                      <div className="mt-2 space-y-2">
                        <div className="text-sm">
                          <span className="font-medium">Transform Name:</span>{' '}
                          <span className="text-gray-600">{field.transform.name}</span>
                        </div>
                        <div className="text-sm">
                          <span className="font-medium">Logic:</span>{' '}
                          <code className="bg-gray-100 px-2 py-1 rounded text-gray-800">
                            {field.transform.logic}
                          </code>
                        </div>
                        <div className="text-sm">
                          <span className="font-medium">Reversible:</span>{' '}
                          <span className="text-gray-600">
                            {field.transform.reversible ? 'Yes' : 'No'}
                          </span>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default TransformsTab