package main

import (
	"context"
	"fmt"
	"html/template"
	"log"
	"net/http"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"
	"time"

	"github.com/go-chi/chi/v5"
	chimw "github.com/go-chi/chi/v5/middleware"

	"mylofon/internal/config"
	"mylofon/internal/database"
	"mylofon/internal/handlers"
	"mylofon/internal/middleware"
	"mylofon/internal/models"
	redisclient "mylofon/internal/redis"
	"mylofon/internal/services"
)

func main() {
	cfg := config.Load()

	// Initialize database
	db, err := database.New(cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	defer db.Close()

	// Initialize Redis
	redis, err := redisclient.New(cfg.RedisURL)
	if err != nil {
		log.Fatalf("Failed to connect to Redis: %v", err)
	}
	defer redis.Close()

	// Initialize repositories
	userRepo := models.NewUserRepository(db.DB)
	postRepo := models.NewPostRepository(db.DB)
	statsRepo := models.NewStatsRepository(userRepo, postRepo)

	// Initialize services
	authService := services.NewAuthService()
	cleanupService := services.NewCleanupService(userRepo)

	// Start cleanup background job
	cleanupService.Start()
	defer cleanupService.Stop()

	// Load templates
	templates := loadTemplates()

	// Initialize handlers
	authHandler := handlers.NewAuthHandler(templates, userRepo, redis, authService)
	postHandler := handlers.NewPostHandler(templates, postRepo, userRepo)
	profileHandler := handlers.NewProfileHandler(templates, userRepo, postRepo, filepath.Join("static", "uploads"))
	adminHandler := handlers.NewAdminHandler(templates, statsRepo)

	// Initialize middleware
	authMw := middleware.NewAuthMiddleware(redis, userRepo)
	rateLimiter := middleware.NewRateLimiter(redis)

	// Setup router
	r := chi.NewRouter()

	// Global middleware
	r.Use(chimw.Logger)
	r.Use(chimw.Recoverer)
	r.Use(chimw.RealIP)
	r.Use(chimw.Compress(5))

	// Static files
	fileServer := http.FileServer(http.Dir("static"))
	r.Handle("/static/*", http.StripPrefix("/static/", fileServer))

	// Auth routes (public)
	r.Group(func(r chi.Router) {
		r.Use(rateLimiter.LimitByIP("auth", middleware.ClaimCheckLimit))

		r.Get("/auth/login", authHandler.LoginPage)
		r.Post("/auth/check", authHandler.CheckClaim)
		r.Post("/auth/login", authHandler.Login)
	})

	// Signup routes (with global rate limit)
	r.Group(func(r chi.Router) {
		r.Use(rateLimiter.LimitByIP("signup", middleware.ClaimCheckLimit))
		r.Use(rateLimiter.LimitGlobal("signup", middleware.GlobalSignupLimit))

		r.Post("/auth/confirm", authHandler.ConfirmSignup)
	})

	// Setup route (requires auth but not complete)
	r.Group(func(r chi.Router) {
		r.Use(authMw.RequireAuth)

		r.Get("/auth/setup", authHandler.SetupPage)
		r.Post("/auth/setup", authHandler.CompleteSetup)
	})

	// Logout
	r.Post("/auth/logout", authHandler.Logout)

	// Timeline (public view, optional auth)
	r.Group(func(r chi.Router) {
		r.Use(authMw.OptionalAuth)

		r.Get("/", postHandler.Timeline)
		r.Get("/post/{id}", postHandler.ViewPost)
		r.Get("/@{claimID}", profileHandler.ViewProfile)
	})

	// Authenticated routes
	r.Group(func(r chi.Router) {
		r.Use(authMw.RequireAuth)
		r.Use(authMw.RequireComplete)

		// Posts
		r.Group(func(r chi.Router) {
			r.Use(rateLimiter.LimitByUser("post", middleware.PostCreationLimit))
			r.Post("/api/posts", postHandler.CreatePost)
		})

		r.Post("/api/posts/{id}/like", postHandler.ToggleLike)
		r.Delete("/api/posts/{id}", postHandler.DeletePost)

		// Profile updates
		r.Post("/api/profile/username", profileHandler.UpdateUsername)
		r.Post("/api/profile/avatar", profileHandler.UpdateAvatar)
		r.Post("/api/profile/header", profileHandler.UpdateHeader)
		r.Get("/api/profile/headers", profileHandler.GetHeaderOptions)
	})

	// Admin routes
	r.Group(func(r chi.Router) {
		r.Use(authMw.RequireAuth)
		r.Use(middleware.RequireAdmin)

		r.Get("/admin", adminHandler.Dashboard)
		r.Get("/admin/stats", adminHandler.RefreshStats)
	})

	// Start server
	server := &http.Server{
		Addr:         ":" + cfg.Port,
		Handler:      r,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Graceful shutdown
	go func() {
		log.Printf("Server starting on http://localhost:%s", cfg.Port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server error: %v", err)
		}
	}()

	// Wait for interrupt signal
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}

	log.Println("Server stopped")
}

func loadTemplates() *template.Template {
	tmpl := template.New("").Funcs(template.FuncMap{
		"avatarURL": handlers.GetAvatarURL,
		"headerURL": handlers.GetHeaderURL,
		"timeAgo":   timeAgo,
		"dict": func(values ...interface{}) (map[string]interface{}, error) {
			if len(values)%2 != 0 {
				return nil, fmt.Errorf("invalid dict call")
			}
			dict := make(map[string]interface{}, len(values)/2)
			for i := 0; i < len(values); i += 2 {
				key, ok := values[i].(string)
				if !ok {
					return nil, fmt.Errorf("dict keys must be strings")
				}
				dict[key] = values[i+1]
			}
			return dict, nil
		},
	})

	// Load all templates
	patterns := []string{
		"internal/templates/*.html",
		"internal/templates/**/*.html",
	}

	for _, pattern := range patterns {
		files, err := filepath.Glob(pattern)
		if err != nil {
			log.Printf("Warning: template glob error: %v", err)
			continue
		}
		for _, file := range files {
			_, err := tmpl.ParseFiles(file)
			if err != nil {
				log.Printf("Warning: failed to parse template %s: %v", file, err)
			}
		}
	}

	return tmpl
}

func timeAgo(t time.Time) string {
	diff := time.Since(t)

	switch {
	case diff < time.Minute:
		return "now"
	case diff < time.Hour:
		return fmt.Sprintf("%dm", int(diff.Minutes()))
	case diff < 24*time.Hour:
		return fmt.Sprintf("%dh", int(diff.Hours()))
	case diff < 7*24*time.Hour:
		return fmt.Sprintf("%dd", int(diff.Hours()/24))
	default:
		return t.Format("Jan 2")
	}
}
