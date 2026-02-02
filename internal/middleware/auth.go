package middleware

import (
	"context"
	"net/http"
	"time"

	"mylofon/internal/models"
	redisclient "mylofon/internal/redis"
)

type contextKey string

const UserContextKey contextKey = "user"

type AuthMiddleware struct {
	redis    *redisclient.Client
	userRepo *models.UserRepository
}

func NewAuthMiddleware(redis *redisclient.Client, userRepo *models.UserRepository) *AuthMiddleware {
	return &AuthMiddleware{redis: redis, userRepo: userRepo}
}

// RequireAuth ensures the user is logged in
func (m *AuthMiddleware) RequireAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user := m.getUserFromSession(r)
		if user == nil {
			http.Redirect(w, r, "/auth/login", http.StatusSeeOther)
			return
		}

		// Add user to context
		ctx := context.WithValue(r.Context(), UserContextKey, user)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}

// RequireComplete ensures user has completed profile setup
func (m *AuthMiddleware) RequireComplete(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user := GetUser(r.Context())
		if user == nil {
			http.Redirect(w, r, "/auth/login", http.StatusSeeOther)
			return
		}

		if !user.IsComplete {
			http.Redirect(w, r, "/auth/setup", http.StatusSeeOther)
			return
		}

		next.ServeHTTP(w, r)
	})
}

// OptionalAuth loads user if logged in, but doesn't require it
func (m *AuthMiddleware) OptionalAuth(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		user := m.getUserFromSession(r)
		if user != nil {
			ctx := context.WithValue(r.Context(), UserContextKey, user)
			r = r.WithContext(ctx)
		}
		next.ServeHTTP(w, r)
	})
}

func (m *AuthMiddleware) getUserFromSession(r *http.Request) *models.User {
	cookie, err := r.Cookie("session")
	if err != nil {
		return nil
	}

	userID, err := m.redis.GetSession(r.Context(), cookie.Value)
	if err != nil || userID == 0 {
		return nil
	}

	// Refresh session TTL
	m.redis.RefreshSession(r.Context(), cookie.Value, 7*24*time.Hour)

	user, err := m.userRepo.GetByID(userID)
	if err != nil {
		return nil
	}

	return user
}

// GetUser retrieves user from context
func GetUser(ctx context.Context) *models.User {
	user, ok := ctx.Value(UserContextKey).(*models.User)
	if !ok {
		return nil
	}
	return user
}
