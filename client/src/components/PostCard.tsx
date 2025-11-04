import { useState } from 'react'
import type { Post, ReactionType, AddReactionRequest } from '../../../shared/types'

interface PostCardProps {
  post: Post
  currentUsername: string
  onReactionUpdate: () => void
}

const reactionEmojis: Record<ReactionType, string> = {
  kino: '🎬', // cinema/quality
  info: '📚', // informative
  slop: '🗑️', // low quality
}

const reactionLabels: Record<ReactionType, string> = {
  kino: 'Kino',
  info: 'Info',
  slop: 'Slop',
}

export default function PostCard({ post, currentUsername, onReactionUpdate }: PostCardProps) {
  const [isReacting, setIsReacting] = useState(false)

  const handleReaction = async (reactionType: ReactionType) => {
    if (!currentUsername.trim()) {
      alert('Please enter a username first')
      return
    }

    setIsReacting(true)

    try {
      const reactionData: AddReactionRequest = {
        postId: post.id,
        username: currentUsername,
        reactionType,
      }

      const response = await fetch('/api/reactions', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(reactionData),
      })

      if (!response.ok) {
        throw new Error('Failed to add reaction')
      }

      onReactionUpdate()
    } catch (err) {
      console.error('Error adding reaction:', err)
    } finally {
      setIsReacting(false)
    }
  }

  const formatDate = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m`
    if (diffHours < 24) return `${diffHours}h`
    if (diffDays < 7) return `${diffDays}d`
    return date.toLocaleDateString()
  }

  return (
    <div className="border-b border-slate-700 p-4 hover:bg-slate-800/50 transition-colors">
      {/* Header */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2">
          <div className="w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-500 flex items-center justify-center font-bold text-white">
            {post.username.charAt(0).toUpperCase()}
          </div>
          <div>
            <div className="font-semibold text-slate-200">@{post.username}</div>
            <div className="text-sm text-slate-400">{formatDate(post.createdAt)}</div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="mb-3 text-slate-100 whitespace-pre-wrap break-words">
        {post.content}
      </div>

      {/* Reactions */}
      <div className="flex items-center gap-2">
        {(['kino', 'info', 'slop'] as ReactionType[]).map((type) => {
          const isActive = post.userReaction === type
          const count = post.reactions[type]

          return (
            <button
              key={type}
              onClick={() => handleReaction(type)}
              disabled={isReacting}
              className={`
                flex items-center gap-1 px-3 py-1.5 rounded-full text-sm font-medium
                transition-all duration-200
                ${isActive
                  ? 'bg-blue-500 text-white shadow-lg shadow-blue-500/50'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                }
                disabled:opacity-50 disabled:cursor-not-allowed
              `}
              title={reactionLabels[type]}
            >
              <span className="text-base">{reactionEmojis[type]}</span>
              {count > 0 && <span>{count}</span>}
            </button>
          )
        })}
      </div>
    </div>
  )
}
