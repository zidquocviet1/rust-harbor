## ADDED Requirements

### Requirement: Generate AI summary for a pull history record
The system SHALL provide a Tauri command `generate_pull_summary(pull_id)` that retrieves all diff files for the given pull, constructs a prompt, calls the configured AI provider, and returns the generated summary text.

#### Scenario: Successful summary generation
- **WHEN** the user triggers summary generation for a pull record and AI settings are configured
- **THEN** the system SHALL collect all diff content for that pull, send it to the configured AI provider, return the generated summary string, and persist it to the `ai_summary` column of the `pull_history` table

#### Scenario: Summary already cached
- **WHEN** `generate_pull_summary` is called for a pull record that already has a non-null `ai_summary`
- **THEN** the system SHALL return the cached summary immediately without making an AI API call

#### Scenario: AI settings not configured
- **WHEN** `generate_pull_summary` is called but no AI provider or API key has been saved
- **THEN** the command SHALL return an error with the message "AI settings not configured. Please configure an AI provider in Settings."

#### Scenario: AI API call fails
- **WHEN** the AI provider returns an error (network failure, invalid API key, rate limit)
- **THEN** the command SHALL return an error with a descriptive message; no summary SHALL be persisted

#### Scenario: Large diff truncation
- **WHEN** the total concatenated diff content exceeds 48,000 characters
- **THEN** the system SHALL truncate the diff to 48,000 characters and append a note "[diff truncated due to size]" before sending to the AI

### Requirement: Support Claude, OpenAI, and Ollama providers
The system SHALL support three AI providers for summary generation: Anthropic Claude, OpenAI, and Ollama (local).

#### Scenario: Claude provider call
- **WHEN** the configured provider is "claude"
- **THEN** the system SHALL send an HTTP POST to the Anthropic Messages API using the configured model and API key

#### Scenario: OpenAI provider call
- **WHEN** the configured provider is "openai"
- **THEN** the system SHALL send an HTTP POST to the OpenAI Chat Completions API using the configured model and API key

#### Scenario: Ollama provider call
- **WHEN** the configured provider is "ollama"
- **THEN** the system SHALL send an HTTP POST to the configured Ollama base URL (default: `http://localhost:11434`) using the configured model name, with no API key required
