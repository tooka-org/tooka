package core

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"gopkg.in/yaml.v3"
	"github.com/Benji377/tooka/internal/rules"
)

const (
	appName          = "tooka"
	configFileName   = "config.yaml"
	rulesFileName    = "rules.yaml"
	defaultVersion   = "0.1.0"
	defaultLogsFolder = "logs"
)

// Config represents the global configuration
type Config struct {
	Version          string `yaml:"version"`
	SourceFolder     string `yaml:"source_folder"`
	RulesFile        string `yaml:"rules_file"`
	LogsFolder       string `yaml:"logs_folder"`
	FirstRunComplete bool   `yaml:"first_run_complete"`
}

// ConfigDir returns the OS-specific config directory for Tooka
func ConfigDir() (string, error) {
	var configDir string
	if runtime.GOOS == "windows" {
		appdata := os.Getenv("APPDATA")
		if appdata == "" {
			return "", errors.New("APPDATA environment variable is not set")
		}
		configDir = filepath.Join(appdata, appName)
	} else {
		// Linux and others: ~/.config/tooka
		homeDir, err := os.UserHomeDir()
		if err != nil {
			return "", fmt.Errorf("failed to get user home directory: %w", err)
		}
		configDir = filepath.Join(homeDir, ".config", appName)
	}
	return configDir, nil
}

// ConfigFilePath returns the full path to the config.yaml file
func ConfigFilePath() (string, error) {
	configDir, err := ConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(configDir, configFileName), nil
}

// DefaultRulesFilePath returns the default path to the rules.yaml file (next to config.yaml)
func DefaultRulesFilePath() (string, error) {
	configDir, err := ConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(configDir, rulesFileName), nil
}

// DefaultLogsFolderPath returns the default logs folder (inside config dir)
func DefaultLogsFolderPath() (string, error) {
	configDir, err := ConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(configDir, defaultLogsFolder), nil
}

// LoadConfig reads the config from disk or triggers first run setup
func LoadConfig() (*Config, error) {
	cfgPath, err := ConfigFilePath()
	if err != nil {
		return nil, err
	}

	if _, err := os.Stat(cfgPath); errors.Is(err, os.ErrNotExist) {
		// Config missing: first run setup
		return FirstRunSetup()
	} else if err != nil {
		return nil, fmt.Errorf("error checking config file: %w", err)
	}

	data, err := os.ReadFile(cfgPath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var cfg Config
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config YAML: %w", err)
	}

	return &cfg, nil
}

// SaveConfig writes the config back to disk
func SaveConfig(cfg *Config) error {
	cfgPath, err := ConfigFilePath()
	if err != nil {
		return err
	}

	// Ensure config dir exists
	configDir, _ := filepath.Split(cfgPath)
	if err := os.MkdirAll(configDir, 0o755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	data, err := yaml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal config YAML: %w", err)
	}

	if err := os.WriteFile(cfgPath, data, 0o644); err != nil {
		return fmt.Errorf("failed to write config file: %w", err)
	}

	return nil
}

// FirstRunSetup prompts the user and creates default config + empty rules file
func FirstRunSetup() (*Config, error) {
	fmt.Println("Welcome to Tooka! It looks like this is your first run.")
	fmt.Println("Please enter the default folder to sort (e.g. ~/Downloads):")

	reader := bufio.NewReader(os.Stdin)
	for {
		fmt.Print("Source folder: ")
		input, err := reader.ReadString('\n')
		if err != nil {
			return nil, fmt.Errorf("failed to read input: %w", err)
		}
		input = strings.TrimSpace(input)
		if input == "" {
			fmt.Println("Source folder cannot be empty. Please enter a valid path.")
			continue
		}
		// Expand ~ for home dir if present
		if strings.HasPrefix(input, "~") {
			homeDir, err := os.UserHomeDir()
			if err == nil {
				input = filepath.Join(homeDir, input[1:])
			}
		}

		// Check if folder exists
		if stat, err := os.Stat(input); err != nil || !stat.IsDir() {
			fmt.Println("Folder does not exist or is not a directory. Please enter a valid existing folder path.")
			continue
		}

		// Prepare config directory and files
		configDir, err := ConfigDir()
		if err != nil {
			return nil, err
		}

		if err := os.MkdirAll(configDir, 0o755); err != nil {
			return nil, fmt.Errorf("failed to create config directory: %w", err)
		}

		rulesPath := filepath.Join(configDir, rulesFileName)
		logsPath := filepath.Join(configDir, defaultLogsFolder)

		// Create empty rules file if it doesn't exist
		if _, err := os.Stat(rulesPath); errors.Is(err, os.ErrNotExist) {
			emptyRules := &rules.RulesFile{Rules: []rules.Rule{}}
			data, err := yaml.Marshal(emptyRules)
			if err != nil {
				return nil, fmt.Errorf("failed to marshal empty rules: %w", err)
			}
			if err := os.WriteFile(rulesPath, data, 0o644); err != nil {
				return nil, fmt.Errorf("failed to create empty rules file: %w", err)
			}
		}

		// Create logs folder if missing
		if err := os.MkdirAll(logsPath, 0o755); err != nil {
			return nil, fmt.Errorf("failed to create logs folder: %w", err)
		}

		cfg := &Config{
			Version:          defaultVersion,
			SourceFolder:     input,
			RulesFile:        rulesPath,
			LogsFolder:       logsPath,
			FirstRunComplete: true,
		}

		if err := SaveConfig(cfg); err != nil {
			return nil, err
		}

		fmt.Println("Setup complete! Configuration saved.")
		return cfg, nil
	}
}
