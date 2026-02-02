package handlers

import (
	"html/template"
	"log"
	"net/http"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	"mylofon/internal/middleware"
	"mylofon/internal/models"
	redisclient "mylofon/internal/redis"
	"mylofon/internal/services"
)

var claimIDRegex = regexp.MustCompile(`^[a-zA-Z0-9_]{3,20}$`)

type AuthHandler struct {
	templates   *template.Template
	userRepo    *models.UserRepository
	redis       *redisclient.Client
	authService *services.AuthService
}

func NewAuthHandler(tmpl *template.Template, userRepo *models.UserRepository, redis *redisclient.Client, authService *services.AuthService) *AuthHandler {
	return &AuthHandler{
		templates:   tmpl,
		userRepo:    userRepo,
		redis:       redis,
		authService: authService,
	}
}

// LoginPage shows the claim ID entry form
func (h *AuthHandler) LoginPage(w http.ResponseWriter, r *http.Request) {
	h.render(w, "auth/claim.html", nil)
}

// CheckClaim handles claim ID submission - either login or start signup
func (h *AuthHandler) CheckClaim(w http.ResponseWriter, r *http.Request) {
	claimID := strings.TrimSpace(r.FormValue("claim_id"))
	claimID = strings.ToLower(claimID)

	// Validate claim ID format
	if !claimIDRegex.MatchString(claimID) {
		h.renderError(w, "auth/claim.html", "Claim ID must be 3-20 characters, alphanumeric and underscores only")
		return
	}

	// Check if claim exists
	exists, err := h.userRepo.ClaimExists(claimID)
	if err != nil {
		log.Printf("Error checking claim: %v", err)
		h.renderError(w, "auth/claim.html", "An error occurred. Please try again.")
		return
	}

	if exists {
		// Existing user - show login form
		h.render(w, "auth/login.html", map[string]any{
			"ClaimID": claimID,
		})
		return
	}

	// New user - generate code and show danger zone
	code, err := h.authService.GenerateCode(4)
	if err != nil {
		log.Printf("Error generating code: %v", err)
		h.renderError(w, "auth/claim.html", "An error occurred. Please try again.")
		return
	}

	// Store temp code in Redis (5 minute TTL)
	if err := h.redis.SetTempCode(r.Context(), claimID, code, 5*time.Minute); err != nil {
		log.Printf("Error storing temp code: %v", err)
		h.renderError(w, "auth/claim.html", "An error occurred. Please try again.")
		return
	}

	h.render(w, "auth/danger.html", map[string]any{
		"ClaimID": claimID,
		"Code":    code,
	})
}

// ConfirmSignup handles the danger zone confirmation
func (h *AuthHandler) ConfirmSignup(w http.ResponseWriter, r *http.Request) {
	claimID := strings.TrimSpace(r.FormValue("claim_id"))
	claimID = strings.ToLower(claimID)

	// Get temp code from Redis
	code, err := h.redis.GetTempCode(r.Context(), claimID)
	if err != nil || code == "" {
		h.renderError(w, "auth/claim.html", "Session expired. Please try again.")
		return
	}

	// Hash the code
	hash, err := h.authService.HashCode(code)
	if err != nil {
		log.Printf("Error hashing code: %v", err)
		h.renderError(w, "auth/claim.html", "An error occurred. Please try again.")
		return
	}

	// Create user (is_complete = 0)
	user, err := h.userRepo.Create(claimID, hash)
	if err != nil {
		log.Printf("Error creating user: %v", err)
		h.renderError(w, "auth/claim.html", "Claim ID is already taken or an error occurred.")
		return
	}

	// Delete temp code
	h.redis.DeleteTempCode(r.Context(), claimID)

	// Create session
	if err := h.createSession(w, r, user.ID); err != nil {
		log.Printf("Error creating session: %v", err)
		h.renderError(w, "auth/claim.html", "An error occurred. Please try again.")
		return
	}

	// Redirect to setup
	http.Redirect(w, r, "/auth/setup", http.StatusSeeOther)
}

// Login handles existing user login
func (h *AuthHandler) Login(w http.ResponseWriter, r *http.Request) {
	claimID := strings.TrimSpace(r.FormValue("claim_id"))
	claimID = strings.ToLower(claimID)
	code := strings.TrimSpace(r.FormValue("code"))

	user, err := h.userRepo.GetByClaimID(claimID)
	if err != nil || user == nil {
		h.render(w, "auth/login.html", map[string]any{
			"ClaimID": claimID,
			"Error":   "Invalid claim ID or code",
		})
		return
	}

	// Verify code
	valid, err := h.authService.VerifyCode(code, user.AuthHash)
	if err != nil || !valid {
		h.render(w, "auth/login.html", map[string]any{
			"ClaimID": claimID,
			"Error":   "Invalid claim ID or code",
		})
		return
	}

	// Create session
	if err := h.createSession(w, r, user.ID); err != nil {
		log.Printf("Error creating session: %v", err)
		h.render(w, "auth/login.html", map[string]any{
			"ClaimID": claimID,
			"Error":   "An error occurred. Please try again.",
		})
		return
	}

	// Redirect based on completion status
	if user.IsComplete {
		http.Redirect(w, r, "/", http.StatusSeeOther)
	} else {
		http.Redirect(w, r, "/auth/setup", http.StatusSeeOther)
	}
}

// SetupPage shows the profile setup form
func (h *AuthHandler) SetupPage(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Redirect(w, r, "/auth/login", http.StatusSeeOther)
		return
	}

	if user.IsComplete {
		http.Redirect(w, r, "/", http.StatusSeeOther)
		return
	}

	h.render(w, "auth/setup.html", map[string]any{
		"User": user,
	})
}

// CompleteSetup handles profile setup submission
func (h *AuthHandler) CompleteSetup(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Redirect(w, r, "/auth/login", http.StatusSeeOther)
		return
	}

	username := strings.TrimSpace(r.FormValue("username"))
	if username == "" {
		username = user.ClaimID // Default to claim ID
	}

	// Handle file upload if present
	avatarPath := user.AvatarPath
	file, header, err := r.FormFile("avatar")
	if err == nil && header.Size > 0 {
		defer file.Close()

		imgService := services.NewImageService(filepath.Join("static", "uploads"))
		avatarPath, err = imgService.SaveAvatar(user.ID, file)
		if err != nil {
			log.Printf("Error saving avatar: %v", err)
			// Continue without avatar
		}
	}

	// Complete the setup
	if err := h.userRepo.CompleteSetup(user.ID, username, avatarPath); err != nil {
		log.Printf("Error completing setup: %v", err)
		h.render(w, "auth/setup.html", map[string]any{
			"User":  user,
			"Error": "An error occurred. Please try again.",
		})
		return
	}

	http.Redirect(w, r, "/", http.StatusSeeOther)
}

// Logout destroys the session
func (h *AuthHandler) Logout(w http.ResponseWriter, r *http.Request) {
	cookie, err := r.Cookie("session")
	if err == nil {
		h.redis.DeleteSession(r.Context(), cookie.Value)
	}

	// Clear cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    "",
		Path:     "/",
		MaxAge:   -1,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	})

	http.Redirect(w, r, "/auth/login", http.StatusSeeOther)
}

func (h *AuthHandler) createSession(w http.ResponseWriter, r *http.Request, userID int64) error {
	sessionID, err := h.authService.GenerateSessionID()
	if err != nil {
		return err
	}

	// Store in Redis with 7 day TTL
	if err := h.redis.SetSession(r.Context(), sessionID, userID, 7*24*time.Hour); err != nil {
		return err
	}

	// Set cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "session",
		Value:    sessionID,
		Path:     "/",
		MaxAge:   7 * 24 * 60 * 60, // 7 days
		HttpOnly: true,
		Secure:   r.TLS != nil,
		SameSite: http.SameSiteLaxMode,
	})

	return nil
}

func (h *AuthHandler) render(w http.ResponseWriter, name string, data any) {
	if err := h.templates.ExecuteTemplate(w, name, data); err != nil {
		log.Printf("Template error: %v", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
	}
}

func (h *AuthHandler) renderError(w http.ResponseWriter, name string, errMsg string) {
	h.render(w, name, map[string]any{"Error": errMsg})
}
