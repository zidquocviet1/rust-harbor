## Context

Rust Harbor already has a pull history system that captures before/after commit SHAs and per-file diffs. The diff content is stored as raw unified diff text in `pull_history_files.diff_content`. Users must read these raw diffs manually to understand what changed in a pull.

The app uses a JSON-based config system (`config.json`, `window-state.json`) stored in Tauri's app config directory. Settings are loaded/saved via `AppConfig` in `src-tauri/src/config/mod.rs`. The app has no existing HTTP client dependency — all external calls are git CLI invocations.

## Goals / Non-Goals

**Goals:**
- Add a "Summarize" button per pull history record that calls an AI API and displays a human-readable summary
- Cache generated summaries in the `pull_history` SQLite table to avoid redundant API calls
- Provide an AI settings UI section for configuring provider, model, and API key
- Support Claude (Anthropic) and OpenAI as initial providers, with Ollama (local) as a third option

**Non-Goals:**
- Streaming AI responses (summaries are persisted atomically — no partial display)
- Fine-tuning or custom model uploads
- Per-repo AI settings (one global AI config)
- Automatic/background AI summarization on pull

## Decisions

### Decision 1: AI config stored in a separate JSON file (not SQLite)

`ai-config.json` in Tauri's app config directory, following the same pattern as `config.json` and `window-state.json`. A new `AiConfig` struct in `src-tauri/src/config/mod.rs` with `load_ai_config` / `save_ai_config` functions.

**Why not SQLite?** The existing settings pattern is JSON files. SQLite is used for transactional data (pull history). Keeping the same pattern avoids introducing a second persistence mechanism for configuration.

**Why not embed in `AppConfig`?** The API key is sensitive and separation keeps security concerns isolated. A dedicated file also makes it easier to apply OS-level file permissions in the future.

**Alternative considered:** Tauri's secure store plugin — rejected for now as it adds a dependency and the API key is only ever used server-side (never sent to the frontend), reducing the sensitivity risk.

### Decision 2: API key never leaves the Rust backend

The frontend sends only the `pull_id` to trigger summarization. The Tauri command reads the AI config (including API key) from disk server-side and makes the HTTP call. The frontend only receives the generated summary text.

**Why:** Prevents accidental key exposure via IPC inspection. Aligns with the existing pattern where git credentials are handled by the git CLI, not passed through the app.

### Decision 3: Summary stored as a new nullable column in `pull_history`

Add `ai_summary TEXT` to the `pull_history` table. `NULL` means no summary has been generated; a non-null value is the cached result. No separate table needed.

**Why:** Simple additive migration (one ALTER TABLE), avoids a join on every pull history query, and fits the "one record per pull" mental model. The summary is intrinsic to the pull record.

### Decision 4: Add `reqwest` as an HTTP dependency

Use `reqwest` with `json` and `rustls-tls` features for making AI API calls. This is the standard async HTTP client for Tauri apps.

**Alternative considered:** Shell out to `curl` — rejected because it's fragile, harder to handle errors, and harder to pass structured JSON bodies securely.

### Decision 5: Provider abstraction via enum, not trait objects

An `AiProvider` enum (`Claude`, `OpenAI`, `Ollama`) with a `generate_summary(diff_content: &str, config: &AiConfig) -> Result<String>` free function dispatching on the enum variant. No trait objects or dynamic dispatch.

**Why:** Only three providers, all known at compile time. An enum match is simpler to follow and avoids the complexity of trait objects for a feature unlikely to need runtime pluggability.

### Decision 6: Prompt construction — concatenate all file diffs

The summary prompt sends: repository name, branch, commit range, and the concatenated diff content of all files in the pull (up to a token limit). If total diff size exceeds ~12,000 tokens (~48KB of text), truncate with a note.

**Why:** Sending the full diff gives the AI the most context. Truncation prevents API errors on very large pulls.

## Risks / Trade-offs

- **API cost**: Users control which provider and model they choose; Claude Haiku and GPT-4o-mini are cheap options. No automatic triggers — user always initiates.
- **Large diffs truncated**: Very large pulls may produce less accurate summaries. Mitigation: truncation note in the prompt tells the AI the diff is partial.
- **Ollama availability**: Ollama requires a local server running. If unavailable, the command returns a clear error. Mitigation: surface a friendly error message in the UI.
- **API key stored in plaintext JSON**: Acceptable for v1 because the key never leaves the Rust process. Future improvement: use Tauri's secure keychain store.
- **Schema migration**: `ALTER TABLE pull_history ADD COLUMN ai_summary TEXT` is safe and backward-compatible. Old records will have `NULL`.

## Migration Plan

1. Run `ALTER TABLE pull_history ADD COLUMN ai_summary TEXT` on first app launch after update (in the database service migration logic)
2. No data migration needed — existing records start with `NULL` summary
3. Rollback: the column is nullable; removing it is a non-destructive migration
