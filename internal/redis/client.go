package redis

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/redis/go-redis/v9"
)

type Client struct {
	rdb *redis.Client
}

func New(addr string) (*Client, error) {
	rdb := redis.NewClient(&redis.Options{
		Addr:         addr,
		Password:     "", // No password by default
		DB:           0,
		DialTimeout:  5 * time.Second,
		ReadTimeout:  3 * time.Second,
		WriteTimeout: 3 * time.Second,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := rdb.Ping(ctx).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	log.Println("Redis connected successfully")
	return &Client{rdb: rdb}, nil
}

func (c *Client) Close() error {
	return c.rdb.Close()
}

// Rate limiting methods

// CheckRateLimit returns true if the action is allowed, false if rate limited
func (c *Client) CheckRateLimit(ctx context.Context, key string, limit int, window time.Duration) (bool, error) {
	pipe := c.rdb.Pipeline()

	incr := pipe.Incr(ctx, key)
	pipe.Expire(ctx, key, window)

	_, err := pipe.Exec(ctx)
	if err != nil {
		return false, fmt.Errorf("rate limit check failed: %w", err)
	}

	count := incr.Val()
	return count <= int64(limit), nil
}

// GetRateLimitRemaining returns how many requests are left in the current window
func (c *Client) GetRateLimitRemaining(ctx context.Context, key string, limit int) (int, error) {
	count, err := c.rdb.Get(ctx, key).Int()
	if err == redis.Nil {
		return limit, nil
	}
	if err != nil {
		return 0, err
	}

	remaining := limit - count
	if remaining < 0 {
		remaining = 0
	}
	return remaining, nil
}

// Session management

func (c *Client) SetSession(ctx context.Context, sessionID string, userID int64, ttl time.Duration) error {
	return c.rdb.Set(ctx, "session:"+sessionID, userID, ttl).Err()
}

func (c *Client) GetSession(ctx context.Context, sessionID string) (int64, error) {
	return c.rdb.Get(ctx, "session:"+sessionID).Int64()
}

func (c *Client) DeleteSession(ctx context.Context, sessionID string) error {
	return c.rdb.Del(ctx, "session:"+sessionID).Err()
}

func (c *Client) RefreshSession(ctx context.Context, sessionID string, ttl time.Duration) error {
	return c.rdb.Expire(ctx, "session:"+sessionID, ttl).Err()
}

// Temporary auth code storage (for signup flow)

func (c *Client) SetTempCode(ctx context.Context, claimID string, code string, ttl time.Duration) error {
	return c.rdb.Set(ctx, "tempcode:"+claimID, code, ttl).Err()
}

func (c *Client) GetTempCode(ctx context.Context, claimID string) (string, error) {
	code, err := c.rdb.Get(ctx, "tempcode:"+claimID).Result()
	if err == redis.Nil {
		return "", nil
	}
	return code, err
}

func (c *Client) DeleteTempCode(ctx context.Context, claimID string) error {
	return c.rdb.Del(ctx, "tempcode:"+claimID).Err()
}
