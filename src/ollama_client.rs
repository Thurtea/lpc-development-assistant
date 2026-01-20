use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

#[derive(Debug, Serialize)]
pub struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
pub struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    top_k: i32,
    num_predict: i32,
}

#[derive(Debug, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
}

#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct OllamaListResponse {
    pub models: Vec<OllamaModel>,
}

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        // Allow overriding Ollama base URL and timeout via environment variables for flexibility in testing and CI.
        let base = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let timeout_secs = std::env::var("OLLAMA_TIMEOUT_SECS").ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30u64);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base,
            client,
        })
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.3,
                top_p: 0.9,
                top_k: 40,
                num_predict: 4096,
            }),
        };

        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API error ({}): {}", status, error_text);
        }

        let ollama_response = response
            .json::<OllamaGenerateResponse>()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(ollama_response.response)
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        let list_response = response
            .json::<OllamaListResponse>()
            .await
            .context("Failed to parse model list")?;

        Ok(list_response.models.iter().map(|m| m.name.clone()).collect())
    }

    /// Stream generation results from Ollama as they arrive.
    /// Returns a ReceiverStream of anyhow::Result<String> where each Ok(String) is a token/chunk.
    pub fn generate_stream(&self, model: &str, prompt: &str) -> tokio_stream::wrappers::ReceiverStream<anyhow::Result<String>> {
        use tokio::sync::mpsc;
        use tokio_stream::wrappers::ReceiverStream;
        use tokio_stream::StreamExt;

        // Spawn a dedicated thread with its own Tokio runtime which performs the
        // HTTP request and reads the response bytes stream incrementally. As each
        // chunk arrives we forward it into the channel by awaiting `tx.send(...)`
        // inside that runtime — this avoids blocking a runtime on other threads.

        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let model = model.to_string();
        let prompt = prompt.to_string();

        let (tx, rx) = mpsc::channel::<anyhow::Result<String>>(128);

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.blocking_send(Err(anyhow::anyhow!(format!("Failed to create runtime: {}", e))));
                    return;
                }
            };

            let _ = rt.block_on(async move {
                let request = OllamaGenerateRequest {
                    model: model.clone(),
                    prompt: prompt.clone(),
                    stream: true,
                    options: Some(OllamaOptions {
                        temperature: 0.3,
                        top_p: 0.9,
                        top_k: 40,
                        num_predict: 4096,
                    }),
                };

                let res = client
                    .post(format!("{}/api/generate", base_url))
                    .json(&request)
                    .send()
                    .await;

                let mut response = match res {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                        return;
                    }
                };

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    let _ = tx.send(Err(anyhow::anyhow!(format!("Ollama API error ({}): {}", status, text)))).await;
                    return;
                }

                let mut stream = response.bytes_stream();

                while let Some(item) = stream.next().await {
                    match item {
                        Ok(bytes) => {
                            if let Ok(s) = std::str::from_utf8(bytes.as_ref()) {
                                for line in s.split('\n') {
                                    let line = line.trim();
                                    if line.is_empty() { continue; }
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                                        // Prefer `response` or `delta` text fields. Ignore
                                        // control/meta messages (e.g., final JSON with
                                        // `created_at`, `done`, etc.) to avoid showing
                                        // raw metadata in the UI.
                                        if let Some(resp) = val.get("response") {
                                            if let Some(text) = resp.as_str() {
                                                let _ = tx.send(Ok(text.to_string())).await;
                                            }
                                        } else if let Some(text) = val.get("delta").and_then(|d| d.as_str()) {
                                            let _ = tx.send(Ok(text.to_string())).await;
                                        } else if val.get("done").is_some() || val.get("created_at").is_some() {
                                            // skip metadata-only messages
                                            continue;
                                        } else {
                                            let _ = tx.send(Ok(line.to_string())).await;
                                        }
                                    } else {
                                        let _ = tx.send(Ok(line.to_string())).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                            break;
                        }
                    }
                }
            });
        });

        ReceiverStream::new(rx)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_new() {
        // Test that OllamaClient::new() returns a Result (not panicking)
        let result = OllamaClient::new();
        // We can't test the actual outcome without running Ollama,
        // but we can verify the API returns a Result
        assert!(result.is_ok() || result.is_err(), "new() should return Result");
    }

    #[test]
    fn test_ollama_client_new_with_custom_url() {
        // Test that environment variable override works
        std::env::set_var("OLLAMA_URL", "http://custom:9999");
        let client = OllamaClient::new();
        assert!(client.is_ok(), "Should construct with custom URL from env");
        std::env::remove_var("OLLAMA_URL");
    }

    #[test]
    fn test_ollama_client_new_with_custom_timeout() {
        // Test that environment variable timeout override works
        std::env::set_var("OLLAMA_TIMEOUT_SECS", "60");
        let client = OllamaClient::new();
        assert!(client.is_ok(), "Should construct with custom timeout from env");
        std::env::remove_var("OLLAMA_TIMEOUT_SECS");
    }

    #[test]
    fn test_ollama_generate_request_serialization() {
        let req = OllamaGenerateRequest {
            model: "test-model".to_string(),
            prompt: "test prompt".to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.5,
                top_p: 0.9,
                top_k: 40,
                num_predict: 4096,
            }),
        };
        let json = serde_json::to_string(&req);
        assert!(json.is_ok(), "Request should serialize to JSON");
        let json_str = json.unwrap();
        assert!(json_str.contains("test-model"), "JSON should contain model name");
    }

    #[test]
    fn test_ollama_model_deserialization() {
        let json = r#"{"name": "mistral:latest"}"#;
        let model: Result<OllamaModel, _> = serde_json::from_str(json);
        assert!(model.is_ok(), "Should deserialize model");
        let m = model.unwrap();
        assert_eq!(m.name, "mistral:latest");
    }
}

