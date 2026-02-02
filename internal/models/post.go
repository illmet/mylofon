package models

import (
	"database/sql"
	"time"
)

type Post struct {
	ID        int64
	UserID    int64
	Content   string
	ParentID  sql.NullInt64
	CreatedAt time.Time

	// Joined fields
	Author     *User
	LikeCount  int
	ReplyCount int
	HasLiked   bool // Whether current user has liked
}

type PostRepository struct {
	db *sql.DB
}

func NewPostRepository(db *sql.DB) *PostRepository {
	return &PostRepository{db: db}
}

func (r *PostRepository) Create(userID int64, content string, parentID *int64) (*Post, error) {
	var parent sql.NullInt64
	if parentID != nil {
		parent = sql.NullInt64{Int64: *parentID, Valid: true}
	}

	result, err := r.db.Exec(`
		INSERT INTO posts (user_id, content, parent_id)
		VALUES (?, ?, ?)`,
		userID, content, parent,
	)
	if err != nil {
		return nil, err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, err
	}

	return r.GetByID(id, userID)
}

func (r *PostRepository) GetByID(id int64, viewerID int64) (*Post, error) {
	post := &Post{Author: &User{}}

	err := r.db.QueryRow(`
		SELECT 
			p.id, p.user_id, p.content, p.parent_id, p.created_at,
			u.id, u.claim_id, u.username, u.avatar_path,
			(SELECT COUNT(*) FROM likes WHERE post_id = p.id) as like_count,
			(SELECT COUNT(*) FROM posts WHERE parent_id = p.id) as reply_count,
			EXISTS(SELECT 1 FROM likes WHERE post_id = p.id AND user_id = ?) as has_liked
		FROM posts p
		JOIN users u ON p.user_id = u.id
		WHERE p.id = ?`,
		viewerID, id,
	).Scan(
		&post.ID, &post.UserID, &post.Content, &post.ParentID, &post.CreatedAt,
		&post.Author.ID, &post.Author.ClaimID, &post.Author.Username, &post.Author.AvatarPath,
		&post.LikeCount, &post.ReplyCount, &post.HasLiked,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	return post, err
}

func (r *PostRepository) GetTimeline(viewerID int64, limit, offset int) ([]*Post, error) {
	rows, err := r.db.Query(`
		SELECT 
			p.id, p.user_id, p.content, p.parent_id, p.created_at,
			u.id, u.claim_id, u.username, u.avatar_path,
			(SELECT COUNT(*) FROM likes WHERE post_id = p.id) as like_count,
			(SELECT COUNT(*) FROM posts WHERE parent_id = p.id) as reply_count,
			EXISTS(SELECT 1 FROM likes WHERE post_id = p.id AND user_id = ?) as has_liked
		FROM posts p
		JOIN users u ON p.user_id = u.id
		WHERE p.parent_id IS NULL
		ORDER BY p.created_at DESC
		LIMIT ? OFFSET ?`,
		viewerID, limit, offset,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanPosts(rows)
}

func (r *PostRepository) GetUserPosts(userID, viewerID int64, limit, offset int) ([]*Post, error) {
	rows, err := r.db.Query(`
		SELECT 
			p.id, p.user_id, p.content, p.parent_id, p.created_at,
			u.id, u.claim_id, u.username, u.avatar_path,
			(SELECT COUNT(*) FROM likes WHERE post_id = p.id) as like_count,
			(SELECT COUNT(*) FROM posts WHERE parent_id = p.id) as reply_count,
			EXISTS(SELECT 1 FROM likes WHERE post_id = p.id AND user_id = ?) as has_liked
		FROM posts p
		JOIN users u ON p.user_id = u.id
		WHERE p.user_id = ? AND p.parent_id IS NULL
		ORDER BY p.created_at DESC
		LIMIT ? OFFSET ?`,
		viewerID, userID, limit, offset,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanPosts(rows)
}

func (r *PostRepository) GetReplies(parentID, viewerID int64, limit, offset int) ([]*Post, error) {
	rows, err := r.db.Query(`
		SELECT 
			p.id, p.user_id, p.content, p.parent_id, p.created_at,
			u.id, u.claim_id, u.username, u.avatar_path,
			(SELECT COUNT(*) FROM likes WHERE post_id = p.id) as like_count,
			(SELECT COUNT(*) FROM posts WHERE parent_id = p.id) as reply_count,
			EXISTS(SELECT 1 FROM likes WHERE post_id = p.id AND user_id = ?) as has_liked
		FROM posts p
		JOIN users u ON p.user_id = u.id
		WHERE p.parent_id = ?
		ORDER BY p.created_at ASC
		LIMIT ? OFFSET ?`,
		viewerID, parentID, limit, offset,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return r.scanPosts(rows)
}

func (r *PostRepository) scanPosts(rows *sql.Rows) ([]*Post, error) {
	var posts []*Post
	for rows.Next() {
		post := &Post{Author: &User{}}
		err := rows.Scan(
			&post.ID, &post.UserID, &post.Content, &post.ParentID, &post.CreatedAt,
			&post.Author.ID, &post.Author.ClaimID, &post.Author.Username, &post.Author.AvatarPath,
			&post.LikeCount, &post.ReplyCount, &post.HasLiked,
		)
		if err != nil {
			return nil, err
		}
		posts = append(posts, post)
	}
	return posts, rows.Err()
}

func (r *PostRepository) Delete(id, userID int64) error {
	_, err := r.db.Exec(`DELETE FROM posts WHERE id = ? AND user_id = ?`, id, userID)
	return err
}

// Like operations

func (r *PostRepository) Like(postID, userID int64) error {
	_, err := r.db.Exec(`
		INSERT OR IGNORE INTO likes (post_id, user_id) VALUES (?, ?)`,
		postID, userID,
	)
	return err
}

func (r *PostRepository) Unlike(postID, userID int64) error {
	_, err := r.db.Exec(`DELETE FROM likes WHERE post_id = ? AND user_id = ?`, postID, userID)
	return err
}

func (r *PostRepository) ToggleLike(postID, userID int64) (bool, error) {
	// Check if already liked
	var exists bool
	err := r.db.QueryRow(`
		SELECT EXISTS(SELECT 1 FROM likes WHERE post_id = ? AND user_id = ?)`,
		postID, userID,
	).Scan(&exists)
	if err != nil {
		return false, err
	}

	if exists {
		err = r.Unlike(postID, userID)
		return false, err
	}
	err = r.Like(postID, userID)
	return true, err
}

// Stats

func (r *PostRepository) CountTotal() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM posts WHERE parent_id IS NULL`).Scan(&count)
	return count, err
}

func (r *PostRepository) CountComments() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM posts WHERE parent_id IS NOT NULL`).Scan(&count)
	return count, err
}

func (r *PostRepository) CountLikes() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM likes`).Scan(&count)
	return count, err
}

func (r *PostRepository) CountToday() (int, error) {
	var count int
	err := r.db.QueryRow(`
		SELECT COUNT(*) FROM posts 
		WHERE parent_id IS NULL AND date(created_at) = date('now')
	`).Scan(&count)
	return count, err
}
