//! CMS Search
//!
//! This crate provides production search functionality with:
//! - Tantivy embedded full-text search with isolated per-project index directories
//! - Japanese morphological analysis using Lindera with embedded SudachiDict (2026 vocabulary)
//! - Spelling variation and dictionary-form normalization (e.g. サーバ → サーバー, 引っ越す → 引越す)
//! - Markdown text extraction and heading-based document chunking
//! - Contextual snippets with search hit highlighting
//! - Multi-process lock handling for safe API and background worker concurrency
//! - Semantic (vector) search with in-process ONNX embeddings (feature `vector`), stored
//!   per project next to the Tantivy index, merged with BM25 via Reciprocal Rank Fusion
//! - Optional RAG answers via Gemini / OpenAI-compatible / Ollama LLMs

use std::sync::Arc;

use cms_error::AppError;

use cms_infra::config::SearchConfig;

pub mod engine;
pub mod handlers;
#[cfg(feature = "vector")]
pub mod rag;
pub mod router;
pub mod services;
pub mod traits;

pub use engine::{markdown, tantivy, tokenizer};
#[cfg(feature = "vector")]
pub use engine::vector::{Embedder, ProjectVectorIndex, VectorMeta, VectorSearchResult};
pub use engine::tantivy::{ProjectIndex, TantivyFields, TantivySearchEngine};
pub use engine::tokenizer::{JapaneseToken, JapaneseTokenizer, LinderaTantivyTokenizer};
pub use router::router;
pub use services::{process_search_job, SearchService};
pub use traits::SearchEngine;

/// Create the SearchEngine implementation described by configuration.
///
/// Only the embedded `tantivy` backend is supported. Vector search is enabled when
/// `search.vector_search_enabled = true` and the crate is built with the `vector` feature.
pub fn create_search_engine(config: &SearchConfig) -> Result<Arc<dyn SearchEngine>, AppError> {
    match config.backend.as_str() {
        "tantivy" => {
            let embedding_model = config
                .vector_search_enabled
                .then_some(config.embedding_model.as_str());
            let engine = TantivySearchEngine::with_vector_config(
                &config.index_dir,
                config.writer_memory_mb,
                config.lindera_dict_path.as_deref(),
                embedding_model,
                Some(config.model_cache_dir.as_str()),
                config.rag.clone(),
            )?;
            Ok(Arc::new(engine))
        }
        other => Err(AppError::SearchUnavailable(format!(
            "Unknown search backend: '{}'. Only 'tantivy' is supported.",
            other
        ))),
    }
}
