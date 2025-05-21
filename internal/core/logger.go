package core

import (
	"os"
	"path/filepath"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
	"gopkg.in/natefinch/lumberjack.v2"
)

// Global logger variable, initialized after calling InitLogger
var Log zerolog.Logger

// InitLogger initializes global logger with rotation and console output.
// logsDir should be a valid directory path from config.
func InitLogger(logsDir string) {
	// Create logs directory if it doesn't exist
	if _, err := os.Stat(logsDir); os.IsNotExist(err) {
		if err := os.MkdirAll(logsDir, 0o755); err != nil {
			// We can't use log.Fatal() here because log not yet initialized.
			panic("Failed to create logs directory: " + err.Error())
		}
	}

	logFilePath := filepath.Join(logsDir, "tooka_log.json")

	// Set up lumberjack logger for rotation
	lumberjackLogger := &lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10,  // megabytes
		MaxBackups: 7,   // number of backups
		MaxAge:     28,  // days
		Compress:   true,
	}

	// MultiLevelWriter to write to both file and console
	multi := zerolog.MultiLevelWriter(os.Stderr, lumberjackLogger)

	// Initialize global logger
	Log = zerolog.New(multi).With().Timestamp().Logger()

	// Override zerolog's global logger, so log.Fatal() etc use this config
	log.Logger = Log

	// Optionally set global log level here, e.g.:
	zerolog.SetGlobalLevel(zerolog.InfoLevel)
}
