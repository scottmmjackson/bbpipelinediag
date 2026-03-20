use crate::config::LlmConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn diagnose(log: &str, config: &LlmConfig) -> Result<()> {
    let base_url = config
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");
    let base_url = base_url.trim_end_matches('/');

    let model = config.model.as_deref().unwrap_or("gpt-4o");

    let api_key = env::var("LLM_API_KEY")
        .ok()
        .or_else(|| config.api_key.clone())
        .context("LLM API Key not found (LLM_API_KEY env var or diagnosis.api_key in config)")?;

    let client = Client::new();

    let system_prompt = config
        .system_prompt
        .as_deref()
        .unwrap_or("You are a technical triage engine. Output high-density, low-token \
        analysis for a DevOps engineer or a downstream LLM.");
    let user_prompt_template = config.user_prompt.as_deref().unwrap_or("\
Analyze the log below. Identify the specific root cause and the direct fix.

Strictly adhere to this format:
CAUSE: <1-sentence technical root cause if certain>
ERROR: <ONLY single most relevant log line OR ONLY a file reference, whichever is more helpful>
REMEDY: <1-sentence description of the fix>

Rules:
1. No conversational filler or markdown headers.
2. File line references should be in the following format: `path/to/file:FROM_LINE-TO_LINE`
   or `path/to/file:LINE`
   Example: src/components/MyComponent.tsx:123
   Example: src/main/java/package/Main.java:23-26
3. When writing file references, DO NOT explain why the line is relevant. ONLY include the file
   reference.
4. CAUSE, ERROR, and REMEDY should be a single sentence or line.
5. REMEDY should be abstract but concise. Be aware that the only context you have is the log. You
   should therefore NEVER suggest a specific command. If you are unsure, say that further triage is
   needed.
6. Trim extraneous context from ERRORs. For example, don't include timestamps, log level labels, or
   anything else of questionable relevance.
");

    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": format!("{}\n\nLog:\n```\n{}\n```", user_prompt_template, log)}
        ]
    });

    let resp = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .context("Failed to send request to LLM")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("LLM request failed: {} - {}", status, text);
    }

    let json: serde_json::Value = resp.json().await?;
    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        println!("--- LLM Diagnosis ---\n{}", content);
    } else {
        println!("LLM returned unexpected response structure.");
    }

    Ok(())
}
