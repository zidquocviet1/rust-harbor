## Why

Users can view raw diffs in pull history records, but interpreting a large set of file changes is time-consuming and requires manual reading. An AI-generated summary gives users an instant, human-readable explanation of what changed in a pull — what was added, removed, or refactored — without reading every diff line by line.

## What Changes

- **New "Summarize" button** on each pull history record in the UI — triggers AI summary generation for that pull's diff files
- **AI summary result displayed** inline within the pull history detail view, rendered as formatted markdown
- **Generated summaries persisted to SQLite** so re-visiting a record shows the cached result without re-querying the AI
- **New AI Settings section** added to the application settings page where users can:
  - Select the AI provider/model (e.g., Claude, OpenAI, Ollama)
  - Configure the authentication method (API key, local endpoint)
  - Input and save credentials securely

## Capabilities

### New Capabilities

- `ai-summary-generation`: Generate and persist AI summaries of pull history diff content using a user-configured AI model, with caching to prevent redundant API calls
- `ai-settings`: User-facing settings section for selecting AI provider, model, and authentication credentials

### Modified Capabilities

- `pull-history-viewer`: The pull history detail view gains a "Summarize" button and a summary display panel per pull record

## Impact

- **Backend**: New Tauri commands for generating summary (`generate_pull_summary`) and fetching cached summary; new `summary` column in `pull_history` table; new `src-tauri/src/services/ai.rs` for provider abstraction; new `src-tauri/src/controllers/ai_summary.rs`
- **Frontend**: Updated pull history detail component to show Summarize button and summary panel; new `src/routes/settings/ai/+page.svelte` or section within existing settings; new store for AI settings
- **Dependencies**: HTTP client for AI API calls (`reqwest` already available via tauri); possible new crate for provider-specific SDKs or a generic HTTP approach
- **Database**: Additive — new `ai_summary` column on `pull_history` table; new `ai_settings` table for persisting provider/model/auth config
- **Security**: API keys stored in app settings table, not exposed to frontend in plaintext; passed server-side only for AI requests
