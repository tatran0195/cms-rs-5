//! Shared server-side pagination — offset pagination + keyset cursor pagination.
//!
//! Replaces hardcoded LIMIT patterns and provides structured metadata.
//! Reuse this across all CMS domain endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default page size when the client doesn't specify one, and the upper bound
/// preventing unbounded memory consumption.
pub const DEFAULT_LIMIT: i64 = 50;
pub const MAX_LIMIT: i64 = 200;

/// `?page=..&limit=..` query parameters.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Pagination {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl Pagination {
    /// Rows per page, clamped to `[1, MAX_LIMIT]`.
    #[must_use]
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }

    /// 1-based page number, floored at 1.
    #[must_use]
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    /// SQL `OFFSET` for the current page.
    #[must_use]
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.limit()
    }
}

/// Metadata describing page offsets and total row count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

/// A page of results plus totals required by client tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

impl<T> Paginated<T> {
    #[must_use]
    pub fn new(data: Vec<T>, total: i64, p: &Pagination) -> Self {
        Self {
            data,
            meta: PageMeta {
                total,
                page: p.page(),
                limit: p.limit(),
            },
        }
    }
}

// ── Keyset / Cursor Pagination ──────────────────────────────────────────────
//
// Optimized for high-volume feeds (audit logs, activity feeds, notifications)
// where offset pagination degrades to O(N). Keyset pagination maintains O(log N)
// performance with stable index seek.

/// `?after=<id>&limit=..` — keyset cursor query.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct Cursor {
    pub after: Option<Uuid>,
    pub limit: Option<i64>,
}

impl Cursor {
    /// Rows per page, clamped to `[1, MAX_LIMIT]`.
    #[must_use]
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
    }
}

/// A cursor-based page of results plus the next page cursor token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<Uuid>,
}

impl<T> CursorPage<T> {
    /// Build from an over-fetched vector (`limit + 1`).
    /// If rows exceed `limit`, truncates and exposes the last item's id as `next_cursor`.
    #[must_use]
    pub fn build(mut rows: Vec<T>, limit: i64, id_of: impl Fn(&T) -> Uuid) -> Self {
        let keep = usize::try_from(limit).unwrap_or(0);
        let next_cursor = if rows.len() > keep {
            rows.truncate(keep);
            rows.last().map(&id_of)
        } else {
            None
        };
        Self {
            data: rows,
            next_cursor,
        }
    }
}
