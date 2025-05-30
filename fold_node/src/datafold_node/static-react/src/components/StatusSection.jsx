import { CheckCircleIcon } from '@heroicons/react/24/solid'

function StatusSection() {
  return (
    <div className="bg-white rounded-lg shadow-sm p-4 mb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <CheckCircleIcon className="icon icon-md text-green-500" />
          <span className="text-gray-700 font-medium">
            Node is running successfully
          </span>
        </div>
      </div>
      <div className="mt-2 flex items-center gap-2">
        <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
        <span className="text-sm text-gray-500">Active and healthy</span>
      </div>
    </div>
  )
}

export default StatusSection