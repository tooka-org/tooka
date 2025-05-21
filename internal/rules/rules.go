package rules

import (
	"errors"
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// LoadRules loads rules from a YAML file.
func LoadRules(path string) (*RulesFile, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read rules file: %w", err)
	}

	var rf RulesFile
	if err := yaml.Unmarshal(data, &rf); err != nil {
		return nil, fmt.Errorf("failed to unmarshal rules: %w", err)
	}

	for _, rule := range rf.Rules {
		if err := rule.Validate(); err != nil {
			return nil, fmt.Errorf("invalid rule %q: %w", rule.ID, err)
		}
	}

	return &rf, nil
}

// SaveRules writes all rules back to the YAML file.
func SaveRules(path string, rf *RulesFile) error {
	data, err := yaml.Marshal(rf)
	if err != nil {
		return fmt.Errorf("failed to marshal rules: %w", err)
	}
	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write rules file: %w", err)
	}
	return nil
}

// AddRule appends a new rule to the file, ensuring unique IDs.
func AddRule(path string, newRule Rule) error {
	rf, err := LoadRules(path)
	if err != nil {
		return err
	}

	for _, rule := range rf.Rules {
		if rule.ID == newRule.ID {
			return fmt.Errorf("rule with ID %q already exists", newRule.ID)
		}
	}

	if err := newRule.Validate(); err != nil {
		return fmt.Errorf("invalid rule: %w", err)
	}

	rf.Rules = append(rf.Rules, newRule)
	return SaveRules(path, rf)
}

// RemoveRule deletes a rule by ID.
func RemoveRule(path, ruleID string) error {
	rf, err := LoadRules(path)
	if err != nil {
		return err
	}

	updated := make([]Rule, 0, len(rf.Rules))
	found := false
	for _, rule := range rf.Rules {
		if rule.ID == ruleID {
			found = true
			continue
		}
		updated = append(updated, rule)
	}

	if !found {
		return fmt.Errorf("no rule found with ID %q", ruleID)
	}

	rf.Rules = updated
	return SaveRules(path, rf)
}

// ExportRule writes a single rule to its own YAML file.
func ExportRule(path, ruleID, outPath string) error {
	rf, err := LoadRules(path)
	if err != nil {
		return err
	}

	rule, err := FindRule(rf, ruleID)
	if err != nil {
		return err
	}

	export := RulesFile{Rules: []Rule{*rule}}
	data, err := yaml.Marshal(export)
	if err != nil {
		return fmt.Errorf("failed to marshal exported rule: %w", err)
	}

	if err := os.WriteFile(outPath, data, 0644); err != nil {
		return fmt.Errorf("failed to write exported file: %w", err)
	}

	return nil
}

// FindRule looks up a rule by ID.
func FindRule(rf *RulesFile, ruleID string) (*Rule, error) {
	for _, rule := range rf.Rules {
		if rule.ID == ruleID {
			return &rule, nil
		}
	}
	return nil, errors.New("rule not found")
}
