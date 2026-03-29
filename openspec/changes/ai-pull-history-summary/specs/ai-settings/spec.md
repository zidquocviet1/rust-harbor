## ADDED Requirements

### Requirement: AI settings section in application settings
The system SHALL provide an AI settings section accessible from the Settings page where users can configure their preferred AI provider, model, and authentication credentials.

#### Scenario: Navigating to AI settings
- **WHEN** the user opens the Settings page
- **THEN** an "AI" section or tab SHALL be present and selectable

#### Scenario: AI settings form displays current configuration
- **WHEN** the user opens the AI settings section
- **THEN** the form SHALL display the currently saved provider, model, and a masked API key field (showing only last 4 characters if set), and an Ollama base URL field

#### Scenario: Saving AI settings
- **WHEN** the user selects a provider, enters a model name, fills in the API key (for Claude/OpenAI), and clicks Save
- **THEN** the system SHALL persist the configuration via `save_ai_config` Tauri command and display a success confirmation

#### Scenario: Clearing AI settings
- **WHEN** the user clicks "Clear" or removes the API key and saves
- **THEN** the system SHALL persist an empty/reset AI config and future summary generation SHALL return a "not configured" error

### Requirement: Provider selection controls which credential fields are shown
The system SHALL show contextually relevant credential fields based on the selected provider.

#### Scenario: Claude or OpenAI selected
- **WHEN** the user selects "Claude" or "OpenAI" as the provider
- **THEN** the form SHALL show an API key input field and hide the Ollama base URL field

#### Scenario: Ollama selected
- **WHEN** the user selects "Ollama" as the provider
- **THEN** the form SHALL show a base URL input field (defaulting to `http://localhost:11434`) and hide the API key field

### Requirement: AI config persisted server-side only
The system SHALL expose Tauri commands `get_ai_config` and `save_ai_config` that read/write `ai-config.json` in the app config directory. The API key SHALL never be returned in plaintext to the frontend.

#### Scenario: Getting AI config for display
- **WHEN** the frontend calls `get_ai_config`
- **THEN** the system SHALL return provider, model, ollama_base_url, and a boolean `has_api_key` — the actual key value SHALL NOT be included

#### Scenario: Saving AI config
- **WHEN** the frontend calls `save_ai_config` with provider, model, api_key, and ollama_base_url
- **THEN** the system SHALL write all fields including the raw API key to `ai-config.json` on disk
