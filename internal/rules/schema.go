package rules

// structs defining the rule schema for the rules.yaml file

import (
	"errors"
	"fmt"
)

type RulesFile struct {
	Rules []Rule `yaml:"rules"`
}

type Rule struct {
	ID      string   `yaml:"id"`
	Name    string   `yaml:"name"`
	Enabled bool     `yaml:"enabled"`
	Match   Match    `yaml:"match"`
	Actions []Action `yaml:"actions"`
	Flags   Flags    `yaml:"flags"`
}

type Match struct {
	Extensions []string          `yaml:"extensions"`
	MimeType   string            `yaml:"mime_type"`
	Pattern    string            `yaml:"pattern"`
	Metadata   MetadataMatch     `yaml:"metadata"`
	Conditions Conditions        `yaml:"conditions"`
	Any        []Match           `yaml:"any"`
	All        []Match           `yaml:"all"`
}

type MetadataMatch struct {
	ExifDate bool              `yaml:"exif_date"`
	Fields   []MetadataField   `yaml:"fields"`
}

type MetadataField struct {
	Key    string `yaml:"key"`
	Value  string `yaml:"value,omitempty"`
	Pattern string `yaml:"pattern,omitempty"`
}

type Conditions struct {
	OlderThanDays       int    `yaml:"older_than_days,omitempty"`
	SizeGreaterThanKB   int    `yaml:"size_greater_than_kb,omitempty"`
	CreatedBetween      *DateRange `yaml:"created_between,omitempty"`
	FilenameRegex       string `yaml:"filename_regex,omitempty"`
	IsSymlink           *bool  `yaml:"is_symlink,omitempty"`
	Owner               string `yaml:"owner,omitempty"`
}

type DateRange struct {
	From string `yaml:"from"`
	To   string `yaml:"to"`
}

type Action struct {
	Type           string       `yaml:"type"`
	Destination    string       `yaml:"destination,omitempty"`
	PathTemplate   *PathTemplate `yaml:"path_template,omitempty"`
	RenameTemplate string       `yaml:"rename_template,omitempty"`
	CreateDirs     bool         `yaml:"create_dirs,omitempty"`
	Format         string       `yaml:"format,omitempty"`
	Target         string       `yaml:"target,omitempty"`
}

type PathTemplate struct {
	Source string `yaml:"source"`
	Format string `yaml:"format"`
}

type Flags struct {
	DryRun bool `yaml:"dry_run"`
}

// Validator to ensure rule structure is valid
func (r *Rule) Validate() error {
	if r.ID == "" {
		return errors.New("rule id is required")
	}
	if r.Name == "" {
		return fmt.Errorf("rule %s: name is required", r.ID)
	}
	if len(r.Actions) == 0 {
		return fmt.Errorf("rule %s: at least one action is required", r.ID)
	}
	for i, act := range r.Actions {
		switch act.Type {
		case "move", "copy", "rename":
			if act.Destination == "" {
				return fmt.Errorf("rule %s: action %d missing destination", r.ID, i)
			}
		case "delete", "skip":
			// no extra validation needed
		default:
			return fmt.Errorf("rule %s: unknown action type '%s'", r.ID, act.Type)
		}
	}
	return nil
}
