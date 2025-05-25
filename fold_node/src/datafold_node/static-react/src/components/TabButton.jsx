function TabButton({ label, tabKey, activeTab, onChange }) {
  const isActive = activeTab === tabKey
  const classes = `px-4 py-2 text-sm font-medium ${
    isActive
      ? 'text-primary border-b-2 border-primary'
      : 'text-gray-500 hover:text-gray-700 hover:border-gray-300'
  }`

  return (
    <button className={classes} onClick={() => onChange(tabKey)}>
      {label}
    </button>
  )
}

export default TabButton
