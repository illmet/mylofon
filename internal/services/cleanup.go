package services

import (
	"log"
	"time"

	"mylofon/internal/models"
)

type CleanupService struct {
	userRepo *models.UserRepository
	interval time.Duration
	maxAge   time.Duration
	stopCh   chan struct{}
}

func NewCleanupService(userRepo *models.UserRepository) *CleanupService {
	return &CleanupService{
		userRepo: userRepo,
		interval: 1 * time.Hour,  // Run every hour
		maxAge:   24 * time.Hour, // Delete after 24 hours
		stopCh:   make(chan struct{}),
	}
}

// Start begins the background cleanup goroutine
func (s *CleanupService) Start() {
	go s.run()
	log.Println("Cleanup service started (interval: 1h, max age: 24h)")
}

// Stop signals the cleanup goroutine to stop
func (s *CleanupService) Stop() {
	close(s.stopCh)
}

func (s *CleanupService) run() {
	// Run immediately on start
	s.cleanup()

	ticker := time.NewTicker(s.interval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			s.cleanup()
		case <-s.stopCh:
			log.Println("Cleanup service stopped")
			return
		}
	}
}

func (s *CleanupService) cleanup() {
	deleted, err := s.userRepo.DeleteIncomplete(s.maxAge)
	if err != nil {
		log.Printf("Cleanup error: %v", err)
		return
	}
	if deleted > 0 {
		log.Printf("Cleaned up %d incomplete registrations", deleted)
	}
}

// RunOnce performs a single cleanup (for testing or manual trigger)
func (s *CleanupService) RunOnce() (int64, error) {
	return s.userRepo.DeleteIncomplete(s.maxAge)
}
