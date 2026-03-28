use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct EmbedClient {
    http: reqwest::Client,
    base_url: String,
}

#[derive(Serialize)]
struct EmbedRequest {
    text: String,
}

#[derive(Serialize)]
struct EmbedBatchRequest {
    texts: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct EmbedBatchResponse {
    embeddings: Vec<Vec<f32>>,
}

impl EmbedClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let resp = self
            .http
            .post(format!("{}/embed", self.base_url))
            .json(&EmbedRequest {
                text: text.to_string(),
            })
            .send()
            .await
            .map_err(|e| format!("embed request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("embed service returned {}", resp.status()));
        }

        let body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse embed response: {e}"))?;
        Ok(body.embedding)
    }

    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let resp = self
            .http
            .post(format!("{}/embed-batch", self.base_url))
            .json(&EmbedBatchRequest {
                texts: texts.to_vec(),
            })
            .send()
            .await
            .map_err(|e| format!("embed batch request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("embed service returned {}", resp.status()));
        }

        let body: EmbedBatchResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse embed batch response: {e}"))?;
        Ok(body.embeddings)
    }
}
