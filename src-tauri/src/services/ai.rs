use crate::config::AiConfig;
use crate::error::{Error, Result};
use crate::services::database::{get_pull_history_entry, get_pull_history_detail, get_pull_summary, save_pull_summary, DbPool};
use serde_json::{json, Value};
use std::sync::OnceLock;
use reqwest::Client;

const MAX_DIFF_CHARS: usize = 48_000;

#[cfg(not(test))]
const CLAUDE_URL: &str = "https://api.anthropic.com/v1/messages";
#[cfg(not(test))]
const OPENAI_URL: &str = "https://api.openai.com/v1/chat/completions";
#[cfg(not(test))]
const GROK_URL: &str = "https://api.x.ai/v1/chat/completions";

#[cfg(test)]
thread_local! {
    pub static TEST_URLS: std::cell::RefCell<std::collections::HashMap<&'static str, String>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(test)]
fn get_url(key: &'static str, default: &str) -> String {
    TEST_URLS.with(|urls| urls.borrow().get(key).cloned().unwrap_or_else(|| default.to_string()))
}

// --- Shared Client ---
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(Client::new)
}

// --- AI Providers Enum ---
#[derive(Debug, Clone, Copy, PartialEq)]
enum Provider {
    Claude,
    OpenAi,
    Gemini,
    Grok,
    Ollama,
}

impl Provider {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(Self::Claude),
            "openai" => Some(Self::OpenAi),
            "gemini" => Some(Self::Gemini),
            "grok" => Some(Self::Grok),
            "ollama" => Some(Self::Ollama),
            _ => None,
        }
    }

    fn get_url(&self, model: &str, config: &AiConfig) -> String {
        match self {
            Self::Claude => {
                #[cfg(not(test))] { CLAUDE_URL.to_string() }
                #[cfg(test)] { get_url("claude", "https://api.anthropic.com/v1/messages") }
            }
            Self::OpenAi => {
                #[cfg(not(test))] { OPENAI_URL.to_string() }
                #[cfg(test)] { get_url("openai", "https://api.openai.com/v1/chat/completions") }
            }
            Self::Grok => {
                #[cfg(not(test))] { GROK_URL.to_string() }
                #[cfg(test)] { get_url("grok", "https://api.x.ai/v1/chat/completions") }
            }
            Self::Gemini => {
                let base = {
                    #[cfg(not(test))] { "https://generativelanguage.googleapis.com/v1beta/models" }
                    #[cfg(test)] { get_url("gemini_base", "https://generativelanguage.googleapis.com/v1beta/models") }
                };
                if config.auth_method == "oauth_token" {
                    format!("{}/{}:generateContent", base, model)
                } else {
                    format!("{}/{}:generateContent?key={}", base, model, config.api_key.as_deref().unwrap_or(""))
                }
            }
            Self::Ollama => {
                let base = config.ollama_base_url.as_deref().unwrap_or("http://localhost:11434");
                format!("{}/api/chat", base.trim_end_matches('/'))
            }
        }
    }

    fn build_body(&self, model: &str, prompt: &str) -> Value {
        match self {
            Self::Claude => json!({
                "model": model,
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}]
            }),
            Self::OpenAi | Self::Grok => json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "max_completion_tokens": 4096
            }),
            Self::Gemini => json!({
                "contents": [{"parts": [{"text": prompt}]}]
            }),
            Self::Ollama => json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "stream": false
            }),
        }
    }

    fn parse_response(&self, json: &Value) -> Option<String> {
        let content = match self {
            Self::Claude => json["content"][0]["text"].as_str(),
            Self::OpenAi | Self::Grok => json["choices"][0]["message"]["content"].as_str(),
            Self::Gemini => {
                // Gemini có thể trả về candidates trống nếu bị chặn bởi Safety Filter
                json["candidates"][0]["content"]["parts"][0]["text"].as_str()
                    .or_else(|| json["candidates"][0]["finishReason"].as_str()) // Thử lấy lý do kết thúc nếu không có text
            },
            Self::Ollama => json["message"]["content"].as_str(),
        };
        content.map(|s| s.trim().to_string())
    }
}

// --- Internal API Client ---
async fn call_provider(prompt: &str, model: &str, config: &AiConfig) -> Result<String> {
    let p = Provider::from_str(&config.provider)
        .ok_or_else(|| Error::SystemError(format!("Unsupported provider: {}", config.provider)))?;

    let url = p.get_url(model, config);
    let body = p.build_body(model, prompt);
    let client = get_client();
    let api_key = config.api_key.as_deref().unwrap_or("");

    let mut rb = client.post(&url).header("content-type", "application/json");

    // Auth setup
    match p {
        Provider::Claude => rb = rb.header("x-api-key", api_key).header("anthropic-version", "2023-06-01"),
        Provider::OpenAi | Provider::Grok => rb = rb.bearer_auth(api_key),
        Provider::Gemini if config.auth_method == "oauth_token" => rb = rb.bearer_auth(api_key),
        _ => {}
    }

    let resp = rb.json(&body).send().await
        .map_err(|e| Error::SystemError(format!("AI Request failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(Error::SystemError(format!("AI error {}: {}", status, text)));
    }

    let json: Value = resp.json().await
        .map_err(|e| Error::SystemError(format!("Failed to parse response JSON: {}", e)))?;

    p.parse_response(&json).ok_or_else(|| {
        Error::SystemError("Unexpected AI response structure or blocked content.".to_string())
    })
}

// --- Prompt Builder ---
fn build_prompt(repo_name: &str, branch: &str, before: &str, after: &str, diffs: &str) -> String {
    format!(
        "You are a code review assistant. Summarize the changes introduced by this git pull.\n\
         Structure your response in this exact order, using markdown headers, and only include a section if relevant changes exist:\n\n\
         ## Code Changes\n\
         What was added, modified, or removed in production/source code\n\n\
         ## Tests\n\
         Any new or updated test files\n\n\
         ## Configuration\n\
         Changes to config files, dependencies, build scripts, etc.\n\n\
         Keep the total summary under 350 words.\n\n\
         Repository: {repo_name}\n\
         Branch: {branch}\n\
         Commits: {before} → {after}\n\n\
         Diff:\n{diffs}"
    )
}

// --- Main Service Logic ---
pub async fn generate_summary(
    pull_id: i64,
    force_regenerate: bool,
    db: &DbPool,
    ai_config: &AiConfig,
    selected_model: Option<&str>,
) -> Result<String> {
    validate_config(ai_config, selected_model)?;
    let effective_model = selected_model.unwrap_or(&ai_config.model);

    if !force_regenerate {
        if let Some(cached) = check_cache(db, pull_id)? { return Ok(cached); }
    }

    let (entry, files) = fetch_pull_data(db, pull_id)?;
    let all_diffs = build_diff_text(files);
    let prompt = build_prompt(&entry.repo_name, &entry.branch, &entry.commit_before, &entry.commit_after, &all_diffs);

    let summary = call_provider(&prompt, effective_model, ai_config).await?;

    if summary.is_empty() {
        return Err(Error::SystemError("AI returned an empty string. This could be due to safety filters or context limits.".into()));
    }

    save_summary(db, pull_id, &summary, &ai_config.provider, effective_model)?;
    Ok(summary)
}

// --- Helpers ---
fn validate_config(cfg: &AiConfig, model: Option<&str>) -> Result<()> {
    if cfg.provider.is_empty() { return Err(Error::SystemError("AI provider not configured.".into())); }
    if cfg.model.is_empty() && model.is_none() { return Err(Error::SystemError("AI model name not configured.".into())); }
    Ok(())
}

fn check_cache(db: &DbPool, pull_id: i64) -> Result<Option<String>> {
    let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
    get_pull_summary(&conn, pull_id).map(|s| s.filter(|v| !v.is_empty()))
}

fn fetch_pull_data(db: &DbPool, pull_id: i64) -> Result<(crate::models::pull_history::PullHistoryEntry, Vec<crate::models::pull_history::PullHistoryFile>)> {
    let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
    let entry = get_pull_history_entry(&conn, pull_id)?;
    let files = get_pull_history_detail(&conn, pull_id)?;
    Ok((entry, files))
}

fn build_diff_text(files: Vec<crate::models::pull_history::PullHistoryFile>) -> String {
    let mut text = String::new();
    for file in files {
        text.push_str(&format!("--- {}\n{}\n", file.file_path, file.diff_content));
    }
    if text.len() > MAX_DIFF_CHARS {
        let mut truncated: String = text.chars().take(MAX_DIFF_CHARS).collect();
        truncated.push_str("\n[diff truncated due to size]");
        truncated
    } else {
        text
    }
}

fn save_summary(db: &DbPool, pull_id: i64, summary: &str, provider: &str, model: &str) -> Result<()> {
    let conn = db.0.lock().map_err(|e| Error::DbError(e.to_string()))?;
    save_pull_summary(&conn, pull_id, summary, provider, model)
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, header};

    async fn setup_mock_url(key: &'static str, base_url: String) {
        TEST_URLS.with(|urls| { urls.borrow_mut().insert(key, base_url); });
    }

    #[tokio::test]
    async fn test_provider_claude() {
        let server = MockServer::start().await;
        setup_mock_url("claude", server.uri()).await;
        Mock::given(method("POST")).and(header("x-api-key", "key")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"content": [{"text": "Claude Summary"}]}))).mount(&server).await;
        let res = call_provider("hi", "m", &AiConfig { provider: "claude".into(), api_key: Some("key".into()), ..Default::default() }).await;
        assert_eq!(res.unwrap(), "Claude Summary");
    }

    #[tokio::test]
    async fn test_provider_openai() {
        let server = MockServer::start().await;
        setup_mock_url("openai", server.uri()).await;
        Mock::given(method("POST")).and(header("Authorization", "Bearer key")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"choices": [{"message": {"content": "GPT Summary"}}]}))).mount(&server).await;
        let res = call_provider("hi", "m", &AiConfig { provider: "openai".into(), api_key: Some("key".into()), ..Default::default() }).await;
        assert_eq!(res.unwrap(), "GPT Summary");
    }

    #[tokio::test]
    async fn test_provider_gemini() {
        let server = MockServer::start().await;
        setup_mock_url("gemini_base", server.uri()).await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"candidates": [{"content": {"parts": [{"text": "Gemini Summary"}]}}]}))).mount(&server).await;
        let res = call_provider("hi", "m", &AiConfig { provider: "gemini".into(), api_key: Some("key".into()), ..Default::default() }).await;
        assert_eq!(res.unwrap(), "Gemini Summary");
    }
}
