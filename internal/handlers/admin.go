package handlers

import (
	"html/template"
	"log"
	"net/http"

	"mylofon/internal/middleware"
	"mylofon/internal/models"
)

type AdminHandler struct {
	templates *template.Template
	statsRepo *models.StatsRepository
}

func NewAdminHandler(tmpl *template.Template, statsRepo *models.StatsRepository) *AdminHandler {
	return &AdminHandler{
		templates: tmpl,
		statsRepo: statsRepo,
	}
}

// Dashboard shows admin statistics
func (h *AdminHandler) Dashboard(w http.ResponseWriter, r *http.Request) {
	user := middleware.GetUser(r.Context())

	stats, err := h.statsRepo.GetAll()
	if err != nil {
		log.Printf("Error getting stats: %v", err)
		http.Error(w, "Failed to load statistics", http.StatusInternalServerError)
		return
	}

	h.render(w, "admin/dashboard.html", map[string]any{
		"User":  user,
		"Stats": stats,
	})
}

// RefreshStats returns updated stats (for htmx polling)
func (h *AdminHandler) RefreshStats(w http.ResponseWriter, r *http.Request) {
	stats, err := h.statsRepo.GetAll()
	if err != nil {
		log.Printf("Error getting stats: %v", err)
		http.Error(w, "Failed to load statistics", http.StatusInternalServerError)
		return
	}

	h.render(w, "partials/stats.html", map[string]any{
		"Stats": stats,
	})
}

func (h *AdminHandler) render(w http.ResponseWriter, name string, data any) {
	if err := h.templates.ExecuteTemplate(w, name, data); err != nil {
		log.Printf("Template error: %v", err)
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
	}
}
