package models

import (
	"database/sql"
	"time"
)

type User struct {
	ID          int64
	ClaimID     string
	Username    string
	AuthHash    string
	AvatarPath  string
	HeaderIndex int
	IsAdmin     bool
	IsComplete  bool
	CreatedAt   time.Time
	CompletedAt sql.NullTime
}

type UserRepository struct {
	db *sql.DB
}

func NewUserRepository(db *sql.DB) *UserRepository {
	return &UserRepository{db: db}
}

func (r *UserRepository) Create(claimID, authHash string) (*User, error) {
	result, err := r.db.Exec(`
		INSERT INTO users (claim_id, auth_hash, is_complete) 
		VALUES (?, ?, 0)`,
		claimID, authHash,
	)
	if err != nil {
		return nil, err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return nil, err
	}

	// First user becomes admin
	if id == 1 {
		_, err = r.db.Exec(`UPDATE users SET is_admin = 1 WHERE id = 1`)
		if err != nil {
			return nil, err
		}
	}

	return r.GetByID(id)
}

func (r *UserRepository) GetByID(id int64) (*User, error) {
	user := &User{}
	err := r.db.QueryRow(`
		SELECT id, claim_id, username, auth_hash, avatar_path, header_index, 
		       is_admin, is_complete, created_at, completed_at
		FROM users WHERE id = ?`, id,
	).Scan(
		&user.ID, &user.ClaimID, &user.Username, &user.AuthHash,
		&user.AvatarPath, &user.HeaderIndex, &user.IsAdmin,
		&user.IsComplete, &user.CreatedAt, &user.CompletedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	return user, err
}

func (r *UserRepository) GetByClaimID(claimID string) (*User, error) {
	user := &User{}
	err := r.db.QueryRow(`
		SELECT id, claim_id, username, auth_hash, avatar_path, header_index,
		       is_admin, is_complete, created_at, completed_at
		FROM users WHERE claim_id = ?`, claimID,
	).Scan(
		&user.ID, &user.ClaimID, &user.Username, &user.AuthHash,
		&user.AvatarPath, &user.HeaderIndex, &user.IsAdmin,
		&user.IsComplete, &user.CreatedAt, &user.CompletedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	return user, err
}

func (r *UserRepository) CompleteSetup(id int64, username, avatarPath string) error {
	_, err := r.db.Exec(`
		UPDATE users 
		SET username = ?, avatar_path = ?, is_complete = 1, completed_at = CURRENT_TIMESTAMP
		WHERE id = ?`,
		username, avatarPath, id,
	)
	return err
}

func (r *UserRepository) UpdateProfile(id int64, username, avatarPath string, headerIndex int) error {
	_, err := r.db.Exec(`
		UPDATE users 
		SET username = ?, avatar_path = ?, header_index = ?
		WHERE id = ?`,
		username, avatarPath, headerIndex, id,
	)
	return err
}

func (r *UserRepository) UpdateAvatar(id int64, avatarPath string) error {
	_, err := r.db.Exec(`UPDATE users SET avatar_path = ? WHERE id = ?`, avatarPath, id)
	return err
}

func (r *UserRepository) UpdateHeader(id int64, headerIndex int) error {
	_, err := r.db.Exec(`UPDATE users SET header_index = ? WHERE id = ?`, headerIndex, id)
	return err
}

func (r *UserRepository) UpdateUsername(id int64, username string) error {
	_, err := r.db.Exec(`UPDATE users SET username = ? WHERE id = ?`, username, id)
	return err
}

func (r *UserRepository) ClaimExists(claimID string) (bool, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM users WHERE claim_id = ?`, claimID).Scan(&count)
	return count > 0, err
}

func (r *UserRepository) DeleteIncomplete(olderThan time.Duration) (int64, error) {
	cutoff := time.Now().Add(-olderThan)
	result, err := r.db.Exec(`
		DELETE FROM users 
		WHERE is_complete = 0 AND created_at < ?`,
		cutoff,
	)
	if err != nil {
		return 0, err
	}
	return result.RowsAffected()
}

func (r *UserRepository) CountTotal() (int, error) {
	var count int
	err := r.db.QueryRow(`SELECT COUNT(*) FROM users WHERE is_complete = 1`).Scan(&count)
	return count, err
}

func (r *UserRepository) CountToday() (int, error) {
	var count int
	err := r.db.QueryRow(`
		SELECT COUNT(*) FROM users 
		WHERE is_complete = 1 AND date(completed_at) = date('now')
	`).Scan(&count)
	return count, err
}
