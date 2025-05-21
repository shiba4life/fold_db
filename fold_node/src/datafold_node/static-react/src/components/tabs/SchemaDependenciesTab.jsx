import { useMemo } from 'react'
import { getSchemaDependencies } from '../../utils/dependencyUtils'

function SchemaDependenciesTab({ schemas }) {
  const dependencies = useMemo(() => getSchemaDependencies(schemas), [schemas])

  return (
    <div className="p-6 space-y-4">
      {Object.entries(dependencies).map(([schema, deps]) => (
        <div key={schema} className="bg-white rounded-lg border border-gray-200 shadow-sm p-4">
          <h3 className="font-medium text-gray-900 mb-2">{schema}</h3>
          {deps.length === 0 ? (
            <p className="text-sm text-gray-500">No dependencies</p>
          ) : (
            <ul className="list-disc list-inside space-y-1">
              {deps.map(dep => (
                <li key={dep.schema} className="text-sm text-gray-700">
                  {dep.schema}{' '}
                  <span className="text-gray-500">({dep.types.join(', ')})</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </div>
  )
}

export default SchemaDependenciesTab
