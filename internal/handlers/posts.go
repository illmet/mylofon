package handlers

import (
	"html/template"
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/go-chi/chi/v5"

	"mylofon/internal/middleware"
	"mylofon/internal/models"
)

type PostHandler struct {
	templates *template.Template
	postRepo  *models.PostRepository
	userRepo  *models.UserRepository
}

func NewPostHandler(tmpl *template.Template, postRepo *models.PostRepository, userRepo *models.UserRepository) *PostHandler {
	return &PostHandler{
		templates: tmpl,
		postRepo:  postRepo,
		userRepo:  userRepo,
	}
}

// Timeline shows the main feed
func (h *PostHandler) Timeline(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	viewerID := int64(0)
	if user != nil {
		viewerID = user.ID
	}

	page, _ := strconv.Atoi(r.URL.Query().Get("page"))
	if page < 1 {
		page = 1
	}
	limit := 20
	offset := (page - 1) * limit

	posts, err := h.postRepo.GetTimeline(viewerID, limit, offset)
	if err != nil {
		log.Printf("Error getting timeline: %v", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	// Check if this is an htmx request for more posts
	if r.Header.Get("HX-Request") == "true" {
		h.render(w, "partials/posts.html", map[string]any{
			"Posts":    posts,
			"User":     user,
			"NextPage": page + 1,
			"HasMore":  len(posts) == limit,
		})
		return
	}

	h.render(w, "timeline.html", map[string]any{
		"Posts":    posts,
		"User":     user,
		"NextPage": page + 1,
		"HasMore":  len(posts) == limit,
	})
}

// CreatePost handles new post/tweet creation
func (h *PostHandler) CreatePost(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	content := strings.TrimSpace(r.FormValue("content"))
	if content == "" {
		http.Error(w, "Content is required", http.StatusBadRequest)
		return
	}
	if len(content) > 280 {
		http.Error(w, "Content must be 280 characters or less", http.StatusBadRequest)
		return
	}

	// Check for parent_id (reply)
	var parentID *int64
	if pid := r.FormValue("parent_id"); pid != "" {
		id, err := strconv.ParseInt(pid, 10, 64)
		if err == nil {
			parentID = &id
		}
	}

	post, err := h.postRepo.Create(user.ID, content, parentID)
	if err != nil {
		log.Printf("Error creating post: %v", err)
		http.Error(w, "Failed to create post", http.StatusInternalServerError)
		return
	}

	// Return the new post partial for htmx
	if r.Header.Get("HX-Request") == "true" {
		h.render(w, "partials/post.html", map[string]any{
			"Post": post,
			"User": user,
		})
		return
	}

	http.Redirect(w, r, "/", http.StatusSeeOther)
}

// ViewPost shows a single post with its replies
func (h *PostHandler) ViewPost(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	viewerID := int64(0)
	if user != nil {
		viewerID = user.ID
	}

	postIDStr := chi.URLParam(r, "id")
	postID, err := strconv.ParseInt(postIDStr, 10, 64)
	if err != nil {
		http.NotFound(w, r)
		return
	}

	post, err := h.postRepo.GetByID(postID, viewerID)
	if err != nil || post == nil {
		http.NotFound(w, r)
		return
	}

	replies, err := h.postRepo.GetReplies(postID, viewerID, 50, 0)
	if err != nil {
		log.Printf("Error getting replies: %v", err)
	}

	h.render(w, "post.html", map[string]any{
		"Post":    post,
		"Replies": replies,
		"User":    user,
	})
}

// ToggleLike handles like/unlike
func (h *PostHandler) ToggleLike(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	postIDStr := chi.URLParam(r, "id")
	postID, err := strconv.ParseInt(postIDStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid post ID", http.StatusBadRequest)
		return
	}

	liked, err := h.postRepo.ToggleLike(postID, user.ID)
	if err != nil {
		log.Printf("Error toggling like: %v", err)
		http.Error(w, "Failed to toggle like", http.StatusInternalServerError)
		return
	}

	// Get updated post for htmx response
	post, err := h.postRepo.GetByID(postID, user.ID)
	if err != nil || post == nil {
		http.Error(w, "Post not found", http.StatusNotFound)
		return
	}

	if r.Header.Get("HX-Request") == "true" {
		h.render(w, "partials/like_button.html", map[string]any{
			"Post":   post,
			"Liked":  liked,
			"User":   user,
		})
		return
	}

	http.Redirect(w, r, r.Referer(), http.StatusSeeOther)
}

// DeletePost handles post deletion
func (h *PostHandler) DeletePost(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())
	if user == nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	postIDStr := chi.URLParam(r, "id")
	postID, err := strconv.ParseInt(postIDStr, 10, 64)
	if err != nil {
		http.Error(w, "Invalid post ID", http.StatusBadRequest)
		return
	}

	if err := h.postRepo.Delete(postID, user.ID); err != nil {
		log.Printf("Error deleting post: %v", err)
		http.Error(w, "Failed to delete post", http.StatusInternalServerError)
		return
	}

	if r.Header.Get("HX-Request") == "true" {
		w.WriteHeader(http.StatusOK)
		return
	}

	http.Redirect(w, r, "/", http.StatusSeeOther)
}

func (h *PostHandler) render(w http.ResponseWriter, name string, data any) {
	if err := h.templates.ExecuteTemplate(w, name, data); err != nil {
		log.Printf("Template error: %v", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
	}
}
