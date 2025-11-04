import { useState, useEffect } from 'react'
import PostComposer from './components/PostComposer'
import PostFeed from './components/PostFeed'

function App() {
  const [username, setUsername] = useState<string>('')
  const [refreshTrigger, setRefreshTrigger] = useState(0)

  // Load username from localStorage
  useEffect(() => {
    const savedUsername = localStorage.getItem('mylofon_username')
    if (savedUsername) {
      setUsername(savedUsername)
    }
  }, [])

  // Save username to localStorage
  const handleUsernameChange = (newUsername: string) => {
    setUsername(newUsername)
    localStorage.setItem('mylofon_username', newUsername)
  }

  const handlePostCreated = () => {
    setRefreshTrigger(prev => prev + 1)
  }

  return (
    <div className="min-h-screen bg-slate-900">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <header className="sticky top-0 z-10 bg-slate-900 border-b border-slate-700 px-4 py-3">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-blue-400">mylofon</h1>
            <div className="flex items-center gap-2">
              <input
                type="text"
                placeholder="Enter username..."
                value={username}
                onChange={(e) => handleUsernameChange(e.target.value)}
                className="px-3 py-1 rounded-lg bg-slate-800 border border-slate-600 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
        </header>

        {/* Post Composer */}
        <PostComposer username={username} onPostCreated={handlePostCreated} />

        {/* Feed */}
        <PostFeed username={username} refreshTrigger={refreshTrigger} />
      </div>
    </div>
  )
}

export default App
