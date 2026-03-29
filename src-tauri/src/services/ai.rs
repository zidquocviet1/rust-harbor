use crate::config::AiConfig;
use crate::error::{Error, Result};
use crate::services::database::{get_pull_history, get_pull_history_detail, get_pull_summary, save_pull_summary, DbPool};
use serde_json::json;

const MAX_DIFF_CHARS: usize = 48_000;

/// Builds the prompt text from pull metadata and concatenated file diffs.
fn build_prompt(
    repo_name: &str,
    branch: &str,
    commit_before: &str,
    commit_after: &str,
    diffs: &str,
) -> String {
    format!(
        "You are a code review assistant. Summarize the changes introduced by this git pull.\n\
         Structure your response in this exact order, using markdown headers, and only include a section if relevant changes exist:\n\n\
         ## Code Changes\n\
         What was added, modified, or removed in production/source code — logic, functions, APIs, data models, UI components. This is the most important section.\n\n\
         ## Tests\n\
         Any new or updated test files, what they cover, and whether coverage improved.\n\n\
         ## Configuration\n\
         Changes to config files, dependencies, build scripts, environment variables, or tooling.\n\n\
         Be specific and concise. Reference file names and function names where relevant. Keep the total summary under 350 words.\n\n\
         Repository: {repo_name}\n\
         Branch: {branch}\n\
         Commits: {commit_before} → {commit_after}\n\n\
         Diff:\n{diffs}"
    )
}

/// Calls the Anthropic Claude Messages API and returns the assistant reply.
async fn call_claude(prompt: &str, model: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = json!({
        "model": model,
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": prompt}]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::SystemError(format!("Claude API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("Claude API error {}: {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::SystemError(format!("Failed to parse Claude response: {}", e)))?;

    json["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::SystemError("Unexpected Claude response structure".to_string()))
}

/// Calls the OpenAI Chat Completions API and returns the assistant reply.
async fn call_openai(prompt: &str, model: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_completion_tokens": 1024
    });

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::SystemError(format!("OpenAI API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("OpenAI API error {}: {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::SystemError(format!("Failed to parse OpenAI response: {}", e)))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::SystemError("Unexpected OpenAI response structure".to_string()))
}

/// Calls the Google Gemini API and returns the model reply.
///
/// Supports two auth modes:
/// - `use_oauth = false`: API key appended as `?key=` query parameter.
/// - `use_oauth = true`:  Bearer token sent in `Authorization` header (Google OAuth 2.0).
async fn call_gemini(prompt: &str, model: &str, credential: &str, use_oauth: bool) -> Result<String> {
    let client = reqwest::Client::new();
    let url = if use_oauth {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model
        )
    } else {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, credential
        )
    };

    let body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}]
    });

    let mut req = client.post(&url).header("content-type", "application/json");
    if use_oauth {
        req = req.bearer_auth(credential);
    }

    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::SystemError(format!("Gemini API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("Gemini API error {}: {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::SystemError(format!("Failed to parse Gemini response: {}", e)))?;

    json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::SystemError("Unexpected Gemini response structure".to_string()))
}

/// Calls the xAI Grok API (OpenAI-compatible) and returns the model reply.
async fn call_grok(prompt: &str, model: &str, api_key: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_completion_tokens": 1024
    });

    let resp = client
        .post("https://api.x.ai/v1/chat/completions")
        .bearer_auth(api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::SystemError(format!("Grok API request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("Grok API error {}: {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::SystemError(format!("Failed to parse Grok response: {}", e)))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::SystemError("Unexpected Grok response structure".to_string()))
}

/// Calls the Ollama local API and returns the model reply.
async fn call_ollama(prompt: &str, model: &str, base_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": false
    });

    let resp = client
        .post(&url)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::SystemError(format!("Ollama request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("Ollama error {}: {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::SystemError(format!("Failed to parse Ollama response: {}", e)))?;

    json["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::SystemError("Unexpected Ollama response structure".to_string()))
}

/// Generates (or returns cached) AI summary for a pull history entry.
///
/// - If `force_regenerate` is false and a cached summary exists, returns it immediately.
/// - Otherwise calls the configured AI provider and persists the result.
pub async fn generate_summary(
    pull_id: i64,
    force_regenerate: bool,
    db: &DbPool,
    ai_config: &AiConfig,
    selected_model: Option<&str>,
) -> Result<String> {
    // Check AI config
    if ai_config.provider.is_empty() {
        return Err(Error::SystemError(
            "AI settings not configured. Please configure an AI provider in Settings.".to_string(),
        ));
    }
    if ai_config.model.is_empty() && selected_model.is_none() {
        return Err(Error::SystemError(
            "AI model not configured. Please specify a model name in Settings.".to_string(),
        ));
    }

    let effective_model = selected_model.unwrap_or(&ai_config.model);

    // Return cached summary if available and not forcing regeneration
    if !force_regenerate {
        let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
        if let Ok(Some(cached)) = get_pull_summary(&conn, pull_id) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }
    }

    // Load pull entry and file diffs
    let (repo_name, branch, commit_before, commit_after, diffs) = {
        let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
        let entries = get_pull_history(&conn, None)?;
        let entry = entries
            .into_iter()
            .find(|e| e.id == pull_id)
            .ok_or_else(|| Error::DbError(format!("Pull history entry {} not found", pull_id)))?;

        let files = get_pull_history_detail(&conn, pull_id)?;
        let mut all_diffs = String::new();
        for file in &files {
            all_diffs.push_str(&format!("--- {}\n", file.file_path));
            all_diffs.push_str(&file.diff_content);
            all_diffs.push('\n');
        }

        (
            entry.repo_name,
            entry.branch,
            entry.commit_before,
            entry.commit_after,
            all_diffs,
        )
    };

    // Truncate diff if too large
    let diffs = if diffs.len() > MAX_DIFF_CHARS {
        format!("{}\n[diff truncated due to size]", &diffs[..MAX_DIFF_CHARS])
    } else {
        diffs
    };

    let prompt = build_prompt(&repo_name, &branch, &commit_before, &commit_after, &diffs);

    // Call the configured provider
    let summary = match ai_config.provider.as_str() {
        "claude" => {
            let key = ai_config.api_key.as_deref().unwrap_or("");
            if key.is_empty() {
                return Err(Error::SystemError("API key is required for Claude.".to_string()));
            }
            call_claude(&prompt, effective_model, key).await?
        }
        "openai" => {
            let key = ai_config.api_key.as_deref().unwrap_or("");
            if key.is_empty() {
                return Err(Error::SystemError("API key is required for OpenAI.".to_string()));
            }
            call_openai(&prompt, effective_model, key).await?
        }
        "gemini" => {
            let credential = ai_config.api_key.as_deref().unwrap_or("");
            if credential.is_empty() {
                return Err(Error::SystemError(
                    "An API key or OAuth token is required for Gemini.".to_string(),
                ));
            }
            let use_oauth = ai_config.auth_method == "oauth_token";
            call_gemini(&prompt, effective_model, credential, use_oauth).await?
        }
        "grok" => {
            let key = ai_config.api_key.as_deref().unwrap_or("");
            if key.is_empty() {
                return Err(Error::SystemError("API key is required for Grok.".to_string()));
            }
            call_grok(&prompt, effective_model, key).await?
        }
        "ollama" => {
            let base_url = ai_config
                .ollama_base_url
                .as_deref()
                .unwrap_or("http://localhost:11434");
            call_ollama(&prompt, effective_model, base_url).await?
        }
        other => {
            return Err(Error::SystemError(format!("Unknown AI provider: {}", other)));
        }
    };

    if summary.trim().is_empty() {
        return Err(Error::SystemError(
            "AI returned an empty response. Please check your model settings and try again.".to_string(),
        ));
    }

    // Persist to database
    {
        let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
        save_pull_summary(&conn, pull_id, &summary, &ai_config.provider, effective_model)?;
    }

    Ok(summary)
}
