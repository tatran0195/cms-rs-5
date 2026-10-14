//! In-process vector embeddings and per-project vector index
//!
//! Provides text embedding via fastembed-rs (ONNX Runtime) and per-project
//! vector storage with cosine similarity search.
//!
//! Supported models:
//! - `multilingual-e5-small` — 384 dimensions, ~117MB, best balance of speed and quality
//! - `multilingual-e5-base`  — 768 dimensions, ~470MB, higher quality for complex queries

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use cms_error::AppError;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

// =========================================================================
// Embedder (fastembed ONNX wrapper)
// =========================================================================

/// Thread-safe embedder wrapping fastembed's synchronous TextEmbedding model.
///
/// The fastembed `TextEmbedding` is `!Send + !Sync` because it holds ONNX Runtime
/// session state. We wrap it in a `Mutex` and perform embedding on a blocking
/// thread via `tokio::task::spawn_blocking`.
pub struct Embedder {
    model: Arc<Mutex<TextEmbedding>>,
    dim: usize,
    model_name: String,
}

impl std::fmt::Debug for Embedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Embedder")
            .field("dim", &self.dim)
            .field("model_name", &self.model_name)
            .finish()
    }
}

impl Embedder {
    /// Create a new embedder with the specified model and cache directory.
    ///
    /// The model weights are downloaded on first use and cached to `cache_dir`.
    /// Subsequent calls reuse the cached model files.
    pub fn new(model_name: &str, cache_dir: &Path) -> Result<Self, AppError> {
        std::fs::create_dir_all(cache_dir).map_err(|e| {
            AppError::IndexingError(format!(
                "Failed to create model cache directory at '{}': {}",
                cache_dir.display(),
                e
            ))
        })?;

        let (embedding_model, dim) = match model_name {
            "multilingual-e5-small" => (EmbeddingModel::MultilingualE5Small, 384),
            "multilingual-e5-base" => (EmbeddingModel::MultilingualE5Base, 768),
            other => {
                return Err(AppError::IndexingError(format!(
                    "Unsupported embedding model: '{}'. Use 'multilingual-e5-small' or \
                     'multilingual-e5-base'.",
                    other
                )));
            }
        };

        tracing::info!(
            model = model_name,
            dim = dim,
            cache = %cache_dir.display(),
            "Initializing embedding model"
        );

        let options = TextInitOptions::new(embedding_model)
            .with_cache_dir(cache_dir.to_path_buf())
            .with_show_download_progress(true);

        let model = TextEmbedding::try_new(options).map_err(|e| {
            AppError::IndexingError(format!(
                "Failed to initialize embedding model '{}': {}",
                model_name, e
            ))
        })?;

        tracing::info!(model = model_name, "Embedding model loaded successfully");

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            dim,
            model_name: model_name.to_string(),
        })
    }

    /// Embedding dimension for this model (384 or 768)
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// Model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Embed a single text string. Returns a vector of `self.dimension()` floats.
    ///
    /// Uses `spawn_blocking` because fastembed is synchronous under the hood.
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let texts = vec![text.to_string()];
        let mut embeddings = self.embed_batch(&texts).await?;
        embeddings
            .pop()
            .ok_or_else(|| AppError::IndexingError("Embedding returned empty result".to_string()))
    }

    /// Embed multiple texts in a single batch (more efficient than one-by-one).
    ///
    /// For E5 models, query texts should be prefixed with "query: " and
    /// document/passage texts with "passage: " for best results.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let model = Arc::clone(&self.model);
        let texts = texts.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut model_guard = model.lock();
            model_guard
                .embed(texts, None)
                .map_err(|e| AppError::IndexingError(format!("Embedding generation failed: {}", e)))
        })
        .await
        .map_err(|e| AppError::IndexingError(format!("Embedding task panicked: {}", e)))?
    }

    /// Embed a query string with the "query: " prefix for E5 models.
    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, AppError> {
        self.embed_text(&format!("query: {}", query)).await
    }

    /// Embed a document/passage string with the "passage: " prefix for E5 models.
    pub async fn embed_passage(&self, text: &str) -> Result<Vec<f32>, AppError> {
        self.embed_text(&format!("passage: {}", text)).await
    }

    /// Embed multiple passages in batch with the "passage: " prefix.
    pub async fn embed_passages(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        let prefixed: Vec<String> = texts.iter().map(|t| format!("passage: {}", t)).collect();
        self.embed_batch(&prefixed).await
    }
}

/// Compute cosine similarity between two vectors.
///
/// Returns a value in [-1.0, 1.0] where 1.0 means identical direction.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vector dimensions must match");

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

// =========================================================================
// ProjectVectorIndex
// =========================================================================

/// Metadata for a single stored vector (maps vector index → document info)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMeta {
    pub page_id: String,
    pub project_id: String,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub title: String,
    pub path: String,
}

/// Persistent metadata for the vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexMetadata {
    /// Embedding dimension
    dim: usize,
    /// Model name used to generate embeddings
    model: String,
    /// Vector entries in insertion order (index corresponds to vector position)
    entries: Vec<VectorMeta>,
    /// All stored vectors (flattened: entries.len() × dim floats)
    vectors: Vec<f32>,
}

/// Result from a vector similarity search
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub meta: VectorMeta,
    /// Cosine similarity score (0.0 – 1.0 for normalized vectors)
    pub score: f32,
}

/// A per-project vector index
pub struct ProjectVectorIndex {
    dir_path: PathBuf,
    dim: usize,
    model: String,
    /// Document metadata in insertion order
    entries: Vec<VectorMeta>,
    /// Flattened vector storage: entries.len() × dim
    vectors: Vec<f32>,
    /// page_id → list of vector indices (for delete/update)
    page_index: HashMap<String, Vec<usize>>,
    /// True when in-memory state differs from disk
    dirty: bool,
    /// mtime of metadata.json when last loaded/saved (detects writes by other processes)
    loaded_mtime: Option<SystemTime>,
}

impl std::fmt::Debug for ProjectVectorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectVectorIndex")
            .field("dir_path", &self.dir_path)
            .field("dim", &self.dim)
            .field("model", &self.model)
            .field("entries_count", &self.entries.len())
            .finish()
    }
}

impl ProjectVectorIndex {
    /// Open an existing vector index or create a new empty one for a project.
    pub fn open_or_create(
        base_dir: &Path,
        project_id: &str,
        dim: usize,
        model: &str,
    ) -> Result<Self, AppError> {
        let dir_path = base_dir.join("projects").join(project_id).join("vectors");
        std::fs::create_dir_all(&dir_path).map_err(|e| {
            AppError::IndexingError(format!(
                "Failed to create vector index directory at '{}': {}",
                dir_path.display(),
                e
            ))
        })?;

        let metadata_path = dir_path.join("metadata.json");
        if metadata_path.exists() {
            let mut idx = Self::create_empty(dir_path, dim, model)?;
            idx.load_from_disk(project_id)?;
            Ok(idx)
        } else {
            Self::create_empty(dir_path, dim, model)
        }
    }

    fn metadata_mtime(&self) -> Option<SystemTime> {
        std::fs::metadata(self.dir_path.join("metadata.json"))
            .and_then(|m| m.modified())
            .ok()
    }

    /// Replace in-memory state with what is on disk. Resets to empty on dimension mismatch
    /// (e.g. the embedding model was changed) so the next reindex rebuilds it.
    fn load_from_disk(&mut self, project_id: &str) -> Result<(), AppError> {
        let metadata_path = self.dir_path.join("metadata.json");
        let mtime = self.metadata_mtime();

        let mut file = std::fs::File::open(&metadata_path).map_err(|e| {
            AppError::IndexingError(format!("Failed to open vector metadata: {}", e))
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            AppError::IndexingError(format!("Failed to read vector metadata: {}", e))
        })?;
        let meta: IndexMetadata = serde_json::from_str(&contents).map_err(|e| {
            AppError::IndexingError(format!("Failed to parse vector metadata: {}", e))
        })?;

        if meta.dim != self.dim || meta.model != self.model {
            tracing::warn!(
                project_id = project_id,
                stored_dim = meta.dim,
                stored_model = %meta.model,
                requested_dim = self.dim,
                requested_model = %self.model,
                "Vector index model mismatch — starting empty; run a reindex to rebuild"
            );
            self.entries.clear();
            self.vectors.clear();
            self.page_index.clear();
            self.dirty = true;
            self.loaded_mtime = mtime;
            return Ok(());
        }

        let mut page_index: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, entry) in meta.entries.iter().enumerate() {
            page_index
                .entry(entry.page_id.clone())
                .or_default()
                .push(idx);
        }

        tracing::debug!(
            project_id = project_id,
            entries = meta.entries.len(),
            "Loaded vector index from disk"
        );

        self.entries = meta.entries;
        self.vectors = meta.vectors;
        self.page_index = page_index;
        self.dirty = false;
        self.loaded_mtime = mtime;
        Ok(())
    }

    /// Reload from disk if another process (API server / background worker) has saved
    /// a newer version since we last loaded. Unsaved local changes take precedence.
    pub fn reload_if_stale(&mut self) -> Result<(), AppError> {
        if self.dirty {
            return Ok(());
        }
        let on_disk = self.metadata_mtime();
        if on_disk.is_some() && on_disk != self.loaded_mtime {
            let project_id = self
                .dir_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            self.load_from_disk(&project_id)?;
        }
        Ok(())
    }

    fn create_empty(dir_path: PathBuf, dim: usize, model: &str) -> Result<Self, AppError> {
        Ok(Self {
            dir_path,
            dim,
            model: model.to_string(),
            entries: Vec::new(),
            vectors: Vec::new(),
            page_index: HashMap::new(),
            dirty: false,
            loaded_mtime: None,
        })
    }

    /// Embedding dimension
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// Model used to produce the stored embeddings
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Number of stored vectors
    pub fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| !e.page_id.is_empty())
            .count()
    }

    /// Whether the index is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all vectors for a given page_id, returning how many were removed.
    pub fn remove_page(&mut self, page_id: &str) -> usize {
        if let Some(indices) = self.page_index.remove(page_id) {
            let count = indices.len();
            if count > 0 {
                // Mark for rebuild — we'll compact on save
                // For now, mark entries as tombstoned by clearing page_id
                for &idx in &indices {
                    if idx < self.entries.len() {
                        self.entries[idx].page_id = String::new();
                    }
                }
                self.dirty = true;
            }
            count
        } else {
            0
        }
    }

    /// Add a vector with its metadata. Assumes the vector has exactly `self.dim` elements.
    pub fn add(&mut self, embedding: Vec<f32>, meta: VectorMeta) {
        debug_assert_eq!(
            embedding.len(),
            self.dim,
            "Embedding dimension mismatch: expected {}, got {}",
            self.dim,
            embedding.len()
        );

        let idx = self.entries.len();
        self.page_index
            .entry(meta.page_id.clone())
            .or_default()
            .push(idx);
        self.entries.push(meta);
        self.vectors.extend(embedding);
        self.dirty = true;
    }

    /// Search for the top-k most similar vectors to the query embedding.
    ///
    /// Uses brute-force cosine similarity (sub-millisecond for < 10K vectors,
    /// which covers virtually all documentation projects).
    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<VectorSearchResult> {
        if self.entries.is_empty() || query_embedding.len() != self.dim {
            return Vec::new();
        }

        let mut scored: Vec<(usize, f32)> = Vec::with_capacity(self.entries.len());

        for (i, entry) in self.entries.iter().enumerate() {
            // Skip tombstoned entries
            if entry.page_id.is_empty() {
                continue;
            }

            let start = i * self.dim;
            let end = start + self.dim;
            if end > self.vectors.len() {
                break;
            }
            let vec_slice = &self.vectors[start..end];
            let score = cosine_similarity(query_embedding, vec_slice);
            scored.push((i, score));
        }

        // Partial sort — only need top_k
        scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored
            .into_iter()
            .map(|(idx, score)| VectorSearchResult {
                meta: self.entries[idx].clone(),
                score,
            })
            .collect()
    }

    /// Persist the index to disk. Compacts tombstoned entries.
    pub fn save(&mut self) -> Result<(), AppError> {
        if !self.dirty && self.dir_path.join("metadata.json").exists() {
            return Ok(());
        }

        // Compact: remove tombstoned entries (empty page_id)
        let mut new_entries = Vec::with_capacity(self.entries.len());
        let mut new_vectors = Vec::with_capacity(self.vectors.len());
        let mut new_page_index: HashMap<String, Vec<usize>> = HashMap::new();

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.page_id.is_empty() {
                continue;
            }
            let new_idx = new_entries.len();
            new_page_index
                .entry(entry.page_id.clone())
                .or_default()
                .push(new_idx);
            new_entries.push(entry.clone());

            let start = i * self.dim;
            let end = start + self.dim;
            if end <= self.vectors.len() {
                new_vectors.extend_from_slice(&self.vectors[start..end]);
            }
        }

        self.entries = new_entries;
        self.vectors = new_vectors;
        self.page_index = new_page_index;

        let meta = IndexMetadata {
            dim: self.dim,
            model: self.model.clone(),
            entries: self.entries.clone(),
            vectors: self.vectors.clone(),
        };

        let json = serde_json::to_string(&meta).map_err(|e| {
            AppError::IndexingError(format!("Failed to serialize vector metadata: {}", e))
        })?;

        let metadata_path = self.dir_path.join("metadata.json");
        let tmp_path = self.dir_path.join("metadata.json.tmp");

        // Atomic write via rename
        let mut file = std::fs::File::create(&tmp_path).map_err(|e| {
            AppError::IndexingError(format!("Failed to create vector metadata file: {}", e))
        })?;
        file.write_all(json.as_bytes()).map_err(|e| {
            AppError::IndexingError(format!("Failed to write vector metadata: {}", e))
        })?;
        file.flush().map_err(|e| {
            AppError::IndexingError(format!("Failed to flush vector metadata: {}", e))
        })?;
        drop(file);

        std::fs::rename(&tmp_path, &metadata_path).map_err(|e| {
            AppError::IndexingError(format!("Failed to finalize vector metadata: {}", e))
        })?;

        self.dirty = false;
        self.loaded_mtime = self.metadata_mtime();

        tracing::debug!(
            entries = self.entries.len(),
            dim = self.dim,
            bytes = self.vectors.len() * 4,
            "Vector index saved to disk"
        );

        Ok(())
    }

    /// Delete the vector index directory for this project.
    pub fn delete(dir_path: &Path) -> Result<(), AppError> {
        if dir_path.exists() {
            std::fs::remove_dir_all(dir_path).map_err(|e| {
                AppError::IndexingError(format!(
                    "Failed to delete vector index at '{}': {}",
                    dir_path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    fn make_meta(page_id: &str, chunk: i32) -> VectorMeta {
        VectorMeta {
            page_id: page_id.to_string(),
            project_id: "test-project".to_string(),
            chunk_index: chunk,
            chunk_text: format!("chunk {} of {}", chunk, page_id),
            title: format!("Page {}", page_id),
            path: format!("/docs/{}", page_id),
        }
    }

    #[test]
    fn test_add_and_search() {
        let tmp = TempDir::new().unwrap();
        let mut idx =
            ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();

        idx.add(vec![1.0, 0.0, 0.0], make_meta("p1", 0));
        idx.add(vec![0.0, 1.0, 0.0], make_meta("p2", 0));
        idx.add(vec![0.9, 0.1, 0.0], make_meta("p3", 0));

        let results = idx.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].meta.page_id, "p1"); // exact match
        assert_eq!(results[1].meta.page_id, "p3"); // closest
    }

    #[test]
    fn test_remove_page() {
        let tmp = TempDir::new().unwrap();
        let mut idx =
            ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();

        idx.add(vec![1.0, 0.0, 0.0], make_meta("p1", 0));
        idx.add(vec![1.0, 0.1, 0.0], make_meta("p1", 1));
        idx.add(vec![0.0, 1.0, 0.0], make_meta("p2", 0));

        assert_eq!(idx.len(), 3);
        let removed = idx.remove_page("p1");
        assert_eq!(removed, 2);

        // Search should only find p2 now
        let results = idx.search(&[1.0, 0.0, 0.0], 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].meta.page_id, "p2");
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = TempDir::new().unwrap();

        {
            let mut idx =
                ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();
            idx.add(vec![1.0, 0.0, 0.0], make_meta("p1", 0));
            idx.add(vec![0.0, 1.0, 0.0], make_meta("p2", 0));
            idx.save().unwrap();
        }

        // Reload from disk
        let idx = ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();
        assert_eq!(idx.len(), 2);

        let results = idx.search(&[1.0, 0.0, 0.0], 1);
        assert_eq!(results[0].meta.page_id, "p1");
    }

    #[test]
    fn test_save_compacts_tombstones() {
        let tmp = TempDir::new().unwrap();

        {
            let mut idx =
                ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();
            idx.add(vec![1.0, 0.0, 0.0], make_meta("p1", 0));
            idx.add(vec![0.0, 1.0, 0.0], make_meta("p2", 0));
            idx.add(vec![0.0, 0.0, 1.0], make_meta("p3", 0));
            idx.remove_page("p2");
            idx.save().unwrap();
        }

        let idx = ProjectVectorIndex::open_or_create(tmp.path(), "proj1", 3, "test-model").unwrap();
        assert_eq!(idx.len(), 2); // p2 was compacted away
    }
}
