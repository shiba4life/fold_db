function Header() {
  return (
    <header className="bg-white border-b border-gray-200 py-4 px-6 shadow-sm">
      <div className="max-w-7xl mx-auto flex items-center justify-between">
        <a href="/" className="flex items-center gap-3 text-primary hover:text-primary/90 transition-colors">
          <svg className="w-8 h-8 flex-shrink-0 text-primary" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 4C7.58172 4 4 5.79086 4 8C4 10.2091 7.58172 12 12 12C16.4183 12 20 10.2091 20 8C20 5.79086 16.4183 4 12 4Z" />
            <path d="M4 12V16C4 18.2091 7.58172 20 12 20C16.4183 20 20 18.2091 20 16V12" strokeWidth="2" strokeLinecap="round" />
            <path d="M4 8V12C4 14.2091 7.58172 16 12 16C16.4183 16 20 14.2091 20 12V8" strokeWidth="2" strokeLinecap="round" />
          </svg>
          <span className="text-xl font-semibold">DataFold Node</span>
        </a>
        <div className="flex items-center">
          <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-800">
            <span className="w-2 h-2 rounded-full bg-green-500 mr-2"></span>
            Node Active
          </span>
        </div>
      </div>
    </header>
  )
}

export default Header