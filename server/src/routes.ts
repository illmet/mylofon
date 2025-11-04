import { Router, Request, Response } from 'express';
import db from './db';
import type { Post, CreatePostRequest, AddReactionRequest, GetPostsQuery, ReactionType } from '../../shared/types';

const router = Router();

// Get posts with pagination
router.get('/posts', (req: Request<{}, {}, {}, GetPostsQuery>, res: Response) => {
  const limit = parseInt(req.query.limit as string) || 20;
  const offset = parseInt(req.query.offset as string) || 0;
  const username = req.query.username as string || '';

  try {
    // Get posts
    const posts = db.prepare(`
      SELECT id, username, content, created_at as createdAt
      FROM posts
      ORDER BY created_at DESC
      LIMIT ? OFFSET ?
    `).all(limit, offset) as Array<Omit<Post, 'reactions' | 'userReaction'>>;

    // Get reaction counts for each post
    const postsWithReactions: Post[] = posts.map(post => {
      const reactionCounts = db.prepare(`
        SELECT
          reaction_type,
          COUNT(*) as count
        FROM reactions
        WHERE post_id = ?
        GROUP BY reaction_type
      `).all(post.id) as Array<{ reaction_type: ReactionType; count: number }>;

      const reactions = {
        kino: 0,
        info: 0,
        slop: 0
      };

      reactionCounts.forEach(r => {
        reactions[r.reaction_type] = r.count;
      });

      // Get user's reaction if username provided
      let userReaction: ReactionType | null = null;
      if (username) {
        const userReactionRow = db.prepare(`
          SELECT reaction_type
          FROM reactions
          WHERE post_id = ? AND username = ?
        `).get(post.id, username) as { reaction_type: ReactionType } | undefined;

        if (userReactionRow) {
          userReaction = userReactionRow.reaction_type;
        }
      }

      return {
        ...post,
        reactions,
        userReaction
      };
    });

    res.json(postsWithReactions);
  } catch (error) {
    console.error('Error fetching posts:', error);
    res.status(500).json({ error: 'Failed to fetch posts' });
  }
});

// Create a new post
router.post('/posts', (req: Request<{}, {}, CreatePostRequest>, res: Response) => {
  const { username, content } = req.body;

  if (!username || !content) {
    return res.status(400).json({ error: 'Username and content are required' });
  }

  if (content.length > 280) {
    return res.status(400).json({ error: 'Content must be 280 characters or less' });
  }

  try {
    const result = db.prepare(`
      INSERT INTO posts (username, content)
      VALUES (?, ?)
    `).run(username, content);

    const post = db.prepare(`
      SELECT id, username, content, created_at as createdAt
      FROM posts
      WHERE id = ?
    `).get(result.lastInsertRowid) as Post;

    post.reactions = { kino: 0, info: 0, slop: 0 };
    post.userReaction = null;

    res.status(201).json(post);
  } catch (error) {
    console.error('Error creating post:', error);
    res.status(500).json({ error: 'Failed to create post' });
  }
});

// Add or update reaction to a post
router.post('/reactions', (req: Request<{}, {}, AddReactionRequest>, res: Response) => {
  const { postId, username, reactionType } = req.body;

  if (!postId || !username || !reactionType) {
    return res.status(400).json({ error: 'postId, username, and reactionType are required' });
  }

  if (!['kino', 'info', 'slop'].includes(reactionType)) {
    return res.status(400).json({ error: 'Invalid reaction type' });
  }

  try {
    // Check if post exists
    const post = db.prepare('SELECT id FROM posts WHERE id = ?').get(postId);
    if (!post) {
      return res.status(404).json({ error: 'Post not found' });
    }

    // Check if user already reacted
    const existingReaction = db.prepare(`
      SELECT reaction_type FROM reactions WHERE post_id = ? AND username = ?
    `).get(postId, username) as { reaction_type: string } | undefined;

    if (existingReaction) {
      if (existingReaction.reaction_type === reactionType) {
        // Remove reaction if clicking the same one
        db.prepare('DELETE FROM reactions WHERE post_id = ? AND username = ?')
          .run(postId, username);
        return res.json({ message: 'Reaction removed' });
      } else {
        // Update to new reaction
        db.prepare(`
          UPDATE reactions SET reaction_type = ? WHERE post_id = ? AND username = ?
        `).run(reactionType, postId, username);
        return res.json({ message: 'Reaction updated' });
      }
    } else {
      // Add new reaction
      db.prepare(`
        INSERT INTO reactions (post_id, username, reaction_type)
        VALUES (?, ?, ?)
      `).run(postId, username, reactionType);
      return res.json({ message: 'Reaction added' });
    }
  } catch (error) {
    console.error('Error adding reaction:', error);
    res.status(500).json({ error: 'Failed to add reaction' });
  }
});

export default router;
