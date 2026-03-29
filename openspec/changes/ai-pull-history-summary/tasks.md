## 1. Database Migration

- [x] 1.1 Add `ai_summary TEXT` nullable column to `pull_history` table via `ALTER TABLE` migration in `src-tauri/src/services/database.rs`
- [x] 1.2 Add `get_pull_summary(pull_id)` query — returns the `ai_summary` value for a given pull record
- [x] 1.3 Add `save_pull_summary(pull_id, summary)` query — updates `ai_summary` on a pull record

## 2. AI Config Backend

- [x] 2.1 Add `AiConfig` struct to `src-tauri/src/config/mod.rs` with fields: `provider: String`, `model: String`, `api_key: Option<String>`, `ollama_base_url: Option<String>`
- [x] 2.2 Add `AiConfigPublic` struct (no `api_key` field, has `has_api_key: bool`) for safe frontend exposure
- [x] 2.3 Implement `load_ai_config` and `save_ai_config` functions in `src-tauri/src/config/mod.rs` (read/write `ai-config.json`)
- [x] 2.4 Add `reqwest` dependency to `src-tauri/Cargo.toml` with `json` and `rustls-tls` features

## 3. AI Summary Service

- [x] 3.1 Create `src-tauri/src/services/ai.rs` with `generate_summary(pull_id, force_regenerate, db, ai_config) -> Result<String>` function
- [x] 3.2 Implement prompt construction: concatenate repo name, branch, commit range, and all diff content; truncate at 48,000 chars with `[diff truncated due to size]` note
- [x] 3.3 Implement Claude provider HTTP call (POST to Anthropic Messages API with `x-api-key` header)
- [x] 3.4 Implement OpenAI provider HTTP call (POST to OpenAI Chat Completions API with Bearer token)
- [x] 3.5 Implement Ollama provider HTTP call (POST to `{base_url}/api/chat` with model name, no auth)
- [x] 3.6 Return cached summary immediately when `ai_summary` is non-null and `force_regenerate` is false

## 4. Tauri Commands

- [x] 4.1 Create `src-tauri/src/controllers/ai_summary.rs` with `get_ai_config`, `save_ai_config`, and `generate_pull_summary` Tauri commands
- [x] 4.2 `get_ai_config` returns `AiConfigPublic` (provider, model, ollama_base_url, has_api_key)
- [x] 4.3 `save_ai_config` accepts full config including raw api_key and writes to disk
- [x] 4.4 `generate_pull_summary(pull_id: i64, force_regenerate: bool)` — calls AI service, persists result, returns summary string
- [x] 4.5 Register all new commands in `src-tauri/src/lib.rs`
- [x] 4.6 Add `ai.rs` to `src-tauri/src/services/mod.rs` and `ai_summary.rs` to `src-tauri/src/controllers/mod.rs`

## 5. AI Settings UI

- [x] 5.1 Add AI settings section/tab to `src/routes/settings/+page.svelte` (or create a sub-route `src/routes/settings/ai/+page.svelte`)
- [x] 5.2 Build provider selector dropdown (Claude / OpenAI / Ollama)
- [x] 5.3 Show API key input field (password type) for Claude/OpenAI; show Ollama base URL input for Ollama
- [x] 5.4 Show model name text input field for all providers
- [x] 5.5 Load and display current config on mount via `get_ai_config` (show "API key configured" indicator, not the raw key)
- [x] 5.6 Implement Save button that calls `save_ai_config` and shows success/error feedback
- [x] 5.7 Implement Clear button that resets the AI config

## 6. Pull History Summary UI

- [x] 6.1 Add `ai_summary: string | null` field to the pull history detail model/store in the frontend
- [x] 6.2 Add "Summarize with AI" button to the pull history detail view component
- [x] 6.3 Disable the button with tooltip "Configure an AI provider in Settings first" when `has_api_key` is false and provider is not Ollama
- [x] 6.4 Show loading spinner and "Generating summary…" label while the Tauri command is in progress
- [x] 6.5 Render summary result as formatted markdown in the summary panel when available
- [x] 6.6 Show "Regenerate" button when a summary already exists; clicking it calls `generate_pull_summary` with `force_regenerate: true`
- [x] 6.7 Display backend error message in the summary panel on failure with retry affordance
- [x] 6.8 Fetch and display cached summary on pull detail load (if `ai_summary` is non-null)
