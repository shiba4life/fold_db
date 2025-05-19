function ResultsSection({ results }) {
  if (!results) return null

  return (
    <div className="bg-white rounded-lg shadow-sm p-6 mt-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center">
        <span className="mr-2">Results</span>
        <span className="text-xs font-normal text-gray-500">
          ({typeof results === 'string' ? 'Text' : 'JSON'})
        </span>
      </h3>
      <div className="bg-gray-50 rounded-md p-4 overflow-auto max-h-[500px]">
        <pre className="font-mono text-sm text-gray-700 whitespace-pre-wrap">
          {typeof results === 'string'
            ? results
            : JSON.stringify(results, null, 2)
          }
        </pre>
      </div>
    </div>
  )
}

export default ResultsSection