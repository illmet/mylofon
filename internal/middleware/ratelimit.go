package middleware

import (
	"fmt"
	"net/http"
	"time"

	redisclient "mylofon/internal/redis"
)

type RateLimitConfig struct {
	Limit  int
	Window time.Duration
}

type RateLimiter struct {
	redis *redisclient.Client
}

func NewRateLimiter(redis *redisclient.Client) *RateLimiter {
	return &RateLimiter{redis: redis}
}

// Predefined rate limit configurations
var (
	ClaimCheckLimit  = RateLimitConfig{Limit: 10, Window: 1 * time.Minute}
	LoginAttemptLimit = RateLimitConfig{Limit: 5, Window: 15 * time.Minute}
	PostCreationLimit = RateLimitConfig{Limit: 20, Window: 1 * time.Minute}
	GlobalSignupLimit = RateLimitConfig{Limit: 100, Window: 1 * time.Hour}
)

// LimitByIP creates middleware that rate limits by IP address
func (rl *RateLimiter) LimitByIP(prefix string, config RateLimitConfig) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			ip := getClientIP(r)
			key := fmt.Sprintf("rl:%s:%s", prefix, ip)

			allowed, err := rl.redis.CheckRateLimit(r.Context(), key, config.Limit, config.Window)
			if err != nil {
				// On Redis error, allow the request but log
				next.ServeHTTP(w, r)
				return
			}

			if !allowed {
				w.Header().Set("Retry-After", fmt.Sprintf("%d", int(config.Window.Seconds())))
				http.Error(w, "Too many requests. Please try again later.", http.StatusTooManyRequests)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// LimitByUser creates middleware that rate limits by user ID
func (rl *RateLimiter) LimitByUser(prefix string, config RateLimitConfig) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			user := GetUser(r.Context())
			if user == nil {
				http.Error(w, "Unauthorized", http.StatusUnauthorized)
				return
			}

			key := fmt.Sprintf("rl:%s:%d", prefix, user.ID)

			allowed, err := rl.redis.CheckRateLimit(r.Context(), key, config.Limit, config.Window)
			if err != nil {
				next.ServeHTTP(w, r)
				return
			}

			if !allowed {
				w.Header().Set("Retry-After", fmt.Sprintf("%d", int(config.Window.Seconds())))
				http.Error(w, "Too many requests. Please try again later.", http.StatusTooManyRequests)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

// LimitGlobal creates middleware for global rate limiting (e.g., signups per hour)
func (rl *RateLimiter) LimitGlobal(key string, config RateLimitConfig) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			fullKey := fmt.Sprintf("rl:%s:global", key)

			allowed, err := rl.redis.CheckRateLimit(r.Context(), fullKey, config.Limit, config.Window)
			if err != nil {
				next.ServeHTTP(w, r)
				return
			}

			if !allowed {
				w.Header().Set("Retry-After", fmt.Sprintf("%d", int(config.Window.Seconds())))
				http.Error(w, "Service is temporarily rate limited. Please try again later.", http.StatusTooManyRequests)
				return
			}

			next.ServeHTTP(w, r)
		})
	}
}

func getClientIP(r *http.Request) string {
	// Check X-Forwarded-For first (for proxies)
	if xff := r.Header.Get("X-Forwarded-For"); xff != "" {
		return xff
	}
	// Check X-Real-IP
	if xri := r.Header.Get("X-Real-IP"); xri != "" {
		return xri
	}
	// Fall back to RemoteAddr
	return r.RemoteAddr
}
