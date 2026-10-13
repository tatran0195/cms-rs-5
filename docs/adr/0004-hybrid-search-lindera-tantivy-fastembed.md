# ADR 0004: Hybrid Search Architecture with Lindera, Tantivy, and FastEmbed

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

Technical documentation requires ultra-fast, accurate search across multi-language content, including English, Japanese, and source code identifiers. 

External search clusters (e.g. Elasticsearch, OpenSearch) require separate distributed infrastructure, high operational overhead, and significant RAM footprints, violating our goal of single-machine deployment for internal enterprise usage.

## Decision

We use an **in-process hybrid search engine** implemented in `cms-search`:

1. **Full-Text BM25 via Tantivy**:
   - Tantivy provides fast, Lucene-class inverted indexing directly in Rust.
   - Indices are stored locally on disk (`./data/indexes`).
2. **Japanese Morphological Analysis via Lindera**:
   - Uses `lindera` with embedded `sudachidict` for accurate Japanese tokenization and decompounding.
   - No external dictionary files are required to be distributed or configured at runtime.
3. **Semantic Vector Search via FastEmbed**:
   - In-process vector embeddings using `fastembed` with lightweight ONNX models (`multilingual-e5-small`).
   - Runs in-process without needing an external Python model server or GPU infrastructure.
   - Allows hybrid reciprocal rank fusion (RRF) combining BM25 keyword matching with semantic vector similarity.

## Consequences

- Zero external search service dependencies (no Elasticsearch/Meilisearch server to provision or monitor).
- Exceptional search quality for technical documentation in both English and Japanese.
- Minimal operational footprint that runs smoothly on standard virtual machines.
