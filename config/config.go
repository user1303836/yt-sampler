package config

import (
	"os"
	"strconv"
	"time"
)

type Config struct {
	RustServiceURL string
	ServerHost	 string
	ServerPort	 string
	MaxFileSize int64
	TempDir string
	HTTPTimeout time.Duration
}

func getEnvOr(key string, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func getEnvOrInt64(key string, defaultValue int64) int64 {
	if value := os.Getenv(key); value != "" {
		if intValue, err := strconv.ParseInt(value, 10, 64); err == nil {
			return intValue
		}
	}
	return defaultValue
}

func NewConfig() *Config {
	return &Config{
		RustServiceURL: getEnvOr("RUST_SERVICE_URL", "http://localhost:8081"),
		ServerHost: getEnvOr("SERVER_HOST", "localhost"),
		ServerPort: getEnvOr("SERVER_PORT", "8080"),
		MaxFileSize: getEnvOrInt64("MAX_FILE_SIZE", 50 * 1024 * 1024),
		TempDir: getEnvOr("TEMP_DIR", os.TempDir()),
		HTTPTimeout: time.Duration(getEnvOrInt64("HTTP_TIMEOUT_SECONDS", 30)) * time.Second,
	}
}