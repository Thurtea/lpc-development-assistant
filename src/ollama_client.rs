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
    pub fn new() -> Self {
        // Allow overriding Ollama base URL and timeout via environment variables for flexibility in testing and CI.
        let base = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let timeout_secs = std::env::var("OLLAMA_TIMEOUT_SECS").ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30u64);

        Self {
            base_url: base,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .unwrap(),
        }
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

        let base_url = self.base_url.clone();
        let client = self.client.clone();
        let model = model.to_string();
        let prompt = prompt.to_string();

        let (tx, rx) = mpsc::channel::<anyhow::Result<String>>(100);

        tokio::spawn(async move {
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
                    let _ = tx.send(Err(anyhow::Error::new(e))).await;
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
                        // Bytes may contain NDJSON or partial chunks; split on newlines
                        if let Ok(s) = std::str::from_utf8(bytes.as_ref()) {
                            for line in s.split('\n') {
                                let line = line.trim();
                                if line.is_empty() { continue; }
                                // Try parsing JSON object per line
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                                    if let Some(resp) = val.get("response") {
                                        if let Some(text) = resp.as_str() {
                                            let _ = tx.send(Ok(text.to_string())).await;
                                        }
                                    } else if let Some(text) = val.get("delta").and_then(|d| d.as_str()) {
                                        // Some streaming formats use delta
                                        let _ = tx.send(Ok(text.to_string())).await;
                                    } else {
                                        // fallback: send the raw line
                                        let _ = tx.send(Ok(line.to_string())).await;
                                    }
                                } else {
                                    // not JSON, forward raw chunk
                                    let _ = tx.send(Ok(line.to_string())).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::Error::new(e))).await;
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
}
