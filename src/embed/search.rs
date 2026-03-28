use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct EmbeddingEntry {
    pub entity_type: String,
    pub entity_key: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchResult {
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    pub score: f32,
}

pub struct EmbeddingIndex {
    entries: Vec<EmbeddingEntry>,
}

impl EmbeddingIndex {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn load_from_rows(rows: Vec<(String, String, Vec<u8>)>) -> Self {
        let entries = rows
            .into_iter()
            .map(|(entity_type, entity_key, bytes)| {
                let vector = bytes_to_f32(&bytes);
                EmbeddingEntry {
                    entity_type,
                    entity_key,
                    vector,
                }
            })
            .collect();
        Self { entries }
    }

    pub fn upsert(&mut self, entity_type: &str, entity_key: &str, vector: Vec<f32>) {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.entity_type == entity_type && e.entity_key == entity_key)
        {
            entry.vector = vector;
        } else {
            self.entries.push(EmbeddingEntry {
                entity_type: entity_type.to_string(),
                entity_key: entity_key.to_string(),
                vector,
            });
        }
    }

    pub fn remove(&mut self, entity_type: &str, entity_key: &str) {
        self.entries
            .retain(|e| !(e.entity_type == entity_type && e.entity_key == entity_key));
    }

    pub fn search(
        &self,
        query_vec: &[f32],
        type_filter: Option<&str>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let mut scored: Vec<SearchResult> = self
            .entries
            .iter()
            .filter(|e| type_filter.map_or(true, |t| e.entity_type == t))
            .map(|e| SearchResult {
                entity_type: e.entity_type.clone(),
                name: e.entity_key.clone(),
                score: cosine_similarity(query_vec, &e.vector),
            })
            .collect();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

pub fn f32_to_bytes(v: &[f32]) -> Vec<u8> {
    bytemuck::cast_slice(v).to_vec()
}

pub fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytemuck::cast_slice(bytes).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let score = cosine_similarity(&a, &b);
        assert!((score + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_search_returns_top_k() {
        let mut index = EmbeddingIndex::new();
        for i in 0..10 {
            let mut vec = vec![0.0f32; 3];
            vec[0] = i as f32;
            index.upsert("service", &format!("svc-{i}"), vec);
        }
        let query = vec![9.0, 0.0, 0.0];
        let results = index.search(&query, None, 3);
        assert_eq!(results.len(), 3);
        // highest score first
        assert!(results[0].score >= results[1].score);
        assert!(results[1].score >= results[2].score);
    }

    #[test]
    fn test_search_with_type_filter() {
        let mut index = EmbeddingIndex::new();
        index.upsert("service", "svc-a", vec![1.0, 0.0, 0.0]);
        index.upsert("table", "tbl-a", vec![1.0, 0.0, 0.0]);
        index.upsert("queue", "q-a", vec![1.0, 0.0, 0.0]);

        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, Some("table"), 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity_type, "table");
    }

    #[test]
    fn test_empty_index_returns_empty() {
        let index = EmbeddingIndex::new();
        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, None, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_f32_bytes_roundtrip() {
        let original = vec![1.0f32, 2.5, -3.14, 0.0];
        let bytes = f32_to_bytes(&original);
        let restored = bytes_to_f32(&bytes);
        assert_eq!(original, restored);
    }
}
