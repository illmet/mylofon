import { useState, useEffect, useRef, useCallback } from 'react'
import PostCard from './PostCard'
import type { Post } from '../../../shared/types'

interface PostFeedProps {
  username: string
  refreshTrigger: number
}

export default function PostFeed({ username, refreshTrigger }: PostFeedProps) {
  const [posts, setPosts] = useState<Post[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const [error, setError] = useState('')
  const observerTarget = useRef<HTMLDivElement>(null)

  const loadPosts = useCallback(async (offset: number = 0, append: boolean = false) => {
    setIsLoading(true)
    setError('')

    try {
      const params = new URLSearchParams({
        limit: '20',
        offset: offset.toString(),
        username: username || '',
      })

      const response = await fetch(`/api/posts?${params}`)

      if (!response.ok) {
        throw new Error('Failed to load posts')
      }

      const newPosts: Post[] = await response.json()

      if (newPosts.length < 20) {
        setHasMore(false)
      }

      setPosts(prev => append ? [...prev, ...newPosts] : newPosts)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load posts')
    } finally {
      setIsLoading(false)
    }
  }, [username])

  // Initial load and refresh
  useEffect(() => {
    setPosts([])
    setHasMore(true)
    loadPosts(0, false)
  }, [refreshTrigger, loadPosts])

  // Infinite scroll with Intersection Observer
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !isLoading) {
          loadPosts(posts.length, true)
        }
      },
      { threshold: 0.1 }
    )

    const currentTarget = observerTarget.current
    if (currentTarget) {
      observer.observe(currentTarget)
    }

    return () => {
      if (currentTarget) {
        observer.unobserve(currentTarget)
      }
    }
  }, [posts.length, hasMore, isLoading, loadPosts])

  const handleReactionUpdate = useCallback(() => {
    // Reload current posts to update reaction counts
    loadPosts(0, false)
  }, [loadPosts])

  return (
    <div>
      {posts.map((post) => (
        <PostCard
          key={post.id}
          post={post}
          currentUsername={username}
          onReactionUpdate={handleReactionUpdate}
        />
      ))}

      {/* Loading indicator */}
      {isLoading && (
        <div className="p-8 text-center">
          <div className="inline-block w-8 h-8 border-4 border-slate-600 border-t-blue-500 rounded-full animate-spin"></div>
        </div>
      )}

      {/* Error message */}
      {error && (
        <div className="p-4 text-center text-red-400">
          {error}
        </div>
      )}

      {/* No posts message */}
      {!isLoading && posts.length === 0 && !error && (
        <div className="p-8 text-center text-slate-400">
          No posts yet. Be the first to post!
        </div>
      )}

      {/* End of feed */}
      {!hasMore && posts.length > 0 && (
        <div className="p-8 text-center text-slate-500">
          You've reached the end
        </div>
      )}

      {/* Intersection observer target */}
      <div ref={observerTarget} className="h-4" />
    </div>
  )
}
