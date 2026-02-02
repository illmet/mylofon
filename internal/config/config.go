package config

import (
	"os"
	"strconv"
)

type Config struct {
	Port        string
	DatabaseURL string
	RedisURL    string
	SessionKey  string
	UploadDir   string
	Debug       bool
}

func Load() *Config {
	return &Config{
		Port:        getEnv("PORT", "8080"),
		DatabaseURL: getEnv("DATABASE_URL", "./data/mylofon.db"),
		RedisURL:    getEnv("REDIS_URL", "localhost:6379"),
		SessionKey:  getEnv("SESSION_KEY", "change-me-in-production-32chars!"),
		UploadDir:   getEnv("UPLOAD_DIR", "./static/uploads"),
		Debug:       getEnvBool("DEBUG", true),
	}
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}

func getEnvBool(key string, fallback bool) bool {
	if val := os.Getenv(key); val != "" {
		b, err := strconv.ParseBool(val)
		if err == nil {
			return b
		}
	}
	return fallback
}
