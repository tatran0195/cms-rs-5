# ADR 0006: Zero-Docker Single Machine Windows Deployment Strategy

- **Status**: Accepted
- **Date**: 2026-10-12
- **Deciders**: Architecture Team

## Context

The CMS is targeted for deployment within internal company infrastructure, specifically AWS Windows / Windows Server environments (`config/deploy.toml`).

Running Docker on Windows instances introduces container virtualization overhead, WSL2 complexities, high disk usage, and slower file system I/O for large document indexes.

## Decision

1. **Bare-Metal / Native Windows Process Execution**:
   - The entire stack is optimized to run natively as Windows processes / services without Docker.
   - The composition root binary (`cms-server.exe`) runs as a native Windows service or process.
   - Background workers (`cms-worker`) run in-process or as dedicated child worker threads.
2. **In-Process and Local Native Services**:
   - PostgreSQL runs as a native Windows service.
   - Search indexing (Tantivy + FastEmbed) operates directly against local disk storage (`local_root = "C:/cms/uploads"`, `index_dir = "./data/indexes"`).
   - In-memory queue backend (`queue.backend = "memory"`) for single-instance deployments, with optional Redis if scaled.
3. **Optimized Build Profiles**:
   - Dev builds use `line-tables-only` and `unpacked` debug info for fast incremental linking.
   - Release builds use ThinLTO and stripped binaries for minimal memory and disk footprints.

## Consequences

- Ultra-low operational overhead and zero container virtualization penalty on Windows.
- Simple, predictable operations with standard Windows service managers (NSSM / Windows Service Controller).
- Maximizes I/O throughput for search index writing and asset reads.
