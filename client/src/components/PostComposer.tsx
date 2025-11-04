import { useState } from 'react'
import type { CreatePostRequest } from '../../../shared/types'

interface PostComposerProps {
  username: string
  onPostCreated: () => void
}

export default function PostComposer({ username, onPostCreated }: PostComposerProps) {
  const [content, setContent] = useState('')
  const [isPosting, setIsPosting] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!username.trim()) {
      setError('Please enter a username first')
      return
    }

    if (!content.trim()) {
      setError('Please enter some content')
      return
    }

    if (content.length > 280) {
      setError('Post must be 280 characters or less')
      return
    }

    setIsPosting(true)
    setError('')

    try {
      const postData: CreatePostRequest = {
        username: username.trim(),
        content: content.trim()
      }

      const response = await fetch('/api/posts', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(postData),
      })

      if (!response.ok) {
        throw new Error('Failed to create post')
      }

      setContent('')
      onPostCreated()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create post')
    } finally {
      setIsPosting(false)
    }
  }

  return (
    <div className="border-b border-slate-700 p-4">
      <form onSubmit={handleSubmit}>
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="What's happening?"
          className="w-full px-4 py-3 rounded-lg bg-slate-800 border border-slate-600 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
          rows={3}
          maxLength={280}
          disabled={isPosting}
        />
        <div className="flex items-center justify-between mt-2">
          <span className={`text-sm ${content.length > 280 ? 'text-red-400' : 'text-slate-400'}`}>
            {content.length}/280
          </span>
          <button
            type="submit"
            disabled={isPosting || !content.trim() || !username.trim()}
            className="px-6 py-2 rounded-full bg-blue-500 hover:bg-blue-600 disabled:bg-slate-600 disabled:cursor-not-allowed font-semibold transition-colors"
          >
            {isPosting ? 'Posting...' : 'Post'}
          </button>
        </div>
        {error && (
          <div className="mt-2 text-red-400 text-sm">{error}</div>
        )}
      </form>
    </div>
  )
}
