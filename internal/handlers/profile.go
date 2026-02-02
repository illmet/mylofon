package handlers

import (
	"html/template"
	"log"
	"net/http"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/go-chi/chi/v5"

	"mylofon/internal/middleware"
	"mylofon/internal/models"
	"mylofon/internal/services"
)

type ProfileHandler struct {
	templates    *template.Template
	userRepo     *models.UserRepository
	postRepo     *models.PostRepository
	imageService *services.ImageService
}

func NewProfileHandler(tmpl *template.Template, userRepo *models.UserRepository, postRepo *models.PostRepository, uploadDir string) *ProfileHandler {
	return &ProfileHandler{
		templates:    tmpl,
		userRepo:     userRepo,
		postRepo:     postRepo,
		imageService: services.NewImageService(uploadDir),
	}
}

// ViewProfile shows a user's profile page
func (h *ProfileHandler) ViewProfile(w http.ResponseWriter, r *http.Request) {
	currentUser := middleware.GetUser(r.Context())
	viewerID := int64(0)
	if currentUser != nil {
		viewerID = currentUser.ID
	}

	claimID := chi.URLParam(r, "claimID")
	claimID = strings.ToLower(claimID)

	profileUser, err := h.userRepo.GetByClaimID(claimID)
	if err != nil || profileUser == nil || !profileUser.IsComplete {
		http.NotFound(w, r)
		return
	}

	// Get user's posts
	posts, err := h.postRepo.GetUserPosts(profileUser.ID, viewerID, 50, 0)
	if err != nil {
		log.Printf("Error getting user posts: %v", err)
	}

	isOwner := currentUser != nil && currentUser.ID == profileUser.ID

	h.render(w, "profile.html", map[string]any{
		"ProfileUser": profileUser,
		"Posts":       posts,
		"User":        currentUser,
		"IsOwner":     isOwner,
	})
}

// UpdateUsername handles username change
func (h *ProfileHandler) UpdateUsername(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	username := strings.TrimSpace(r.FormValue("username"))
	if username == "" {
		username = user.ClaimID
	}

	if err := h.userRepo.UpdateUsername(user.ID, username); err != nil {
		log.Printf("Error updating username: %v", err)
		http.Error(w, "Failed to update username", http.StatusInternalServerError)
		return
	}

	if r.Header.Get("HX-Request") == "true" {
		w.Write([]byte(username))
		return
	}

	http.Redirect(w, r, "/@"+user.ClaimID, http.StatusSeeOther)
}

// UpdateAvatar handles avatar upload
func (h *ProfileHandler) UpdateAvatar(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	// Max 5MB
	r.ParseMultipartForm(5 << 20)

	file, _, err := r.FormFile("avatar")
	if err != nil {
		http.Error(w, "No file provided", http.StatusBadRequest)
		return
	}
	defer file.Close()

	avatarPath, err := h.imageService.SaveAvatar(user.ID, file)
	if err != nil {
		log.Printf("Error saving avatar: %v", err)
		http.Error(w, "Failed to save avatar", http.StatusInternalServerError)
		return
	}

	if err := h.userRepo.UpdateAvatar(user.ID, avatarPath); err != nil {
		log.Printf("Error updating avatar path: %v", err)
		http.Error(w, "Failed to update profile", http.StatusInternalServerError)
		return
	}

	if r.Header.Get("HX-Request") == "true" {
		// Return the new avatar URL
		w.Write([]byte("/static/" + avatarPath))
		return
	}

	http.Redirect(w, r, "/@"+user.ClaimID, http.StatusSeeOther)
}

// UpdateHeader handles header selection
func (h *ProfileHandler) UpdateHeader(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	headerIndex, err := strconv.Atoi(r.FormValue("header_index"))
	if err != nil || headerIndex < 1 || headerIndex > 10 {
		http.Error(w, "Invalid header selection", http.StatusBadRequest)
		return
	}

	if err := h.userRepo.UpdateHeader(user.ID, headerIndex); err != nil {
		log.Printf("Error updating header: %v", err)
		http.Error(w, "Failed to update header", http.StatusInternalServerError)
		return
	}

	if r.Header.Get("HX-Request") == "true" {
		// Return the new header URL
		w.Write([]byte("/static/headers/header_" + strconv.Itoa(headerIndex) + ".jpg"))
		return
	}

	http.Redirect(w, r, "/@"+user.ClaimID, http.StatusSeeOther)
}

// GetHeaderOptions returns available header images (for modal)
func (h *ProfileHandler) GetHeaderOptions(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())

	h.render(w, "partials/header_options.html", map[string]any{
		"User":         user,
		"HeaderCount":  10,
	})
}

func (h *ProfileHandler) render(w http.ResponseWriter, name string, data any) {
	if err := h.templates.ExecuteTemplate(w, name, data); err != nil {
		log.Printf("Template error: %v", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
	}
}

// Static helper for getting avatar URL
func GetAvatarURL(avatarPath string) string {
	if avatarPath == "" {
		return "/static/default_avatar.svg"
	}
	return filepath.Join("/static", avatarPath)
}

// Static helper for getting header URL
func GetHeaderURL(index int) string {
	if index < 1 || index > 10 {
		index = 1
	}
	return "/static/headers/header_" + strconv.Itoa(index) + ".jpg"
}
