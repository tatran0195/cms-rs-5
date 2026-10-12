-- Migration 0024: Report Runs
-- Tracks asynchronous report runs, exports, artifact keys, and retention lifecycles

CREATE TABLE IF NOT EXISTS report_runs (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    report_number        TEXT        NOT NULL,         -- formatted sequential code, e.g. "RPT-2026-00001"
    workspace_id         TEXT        NOT NULL,
    project_id           TEXT,                         -- optional project scope
    requested_by_user_id TEXT,                         -- user who initiated the report
    report_type          TEXT        NOT NULL,         -- e.g. "documentation_export", "page_inventory", "audit_trail"
    format               TEXT        NOT NULL DEFAULT 'markdown', -- 'markdown' | 'pdf' | 'epub' | 'json'
    status               TEXT        NOT NULL DEFAULT 'queued',   -- 'queued' | 'running' | 'completed' | 'failed'
    parameters           JSONB       NOT NULL DEFAULT '{}'::jsonb,
    artifact_key         TEXT,                         -- object storage key/path e.g. "reports/ws_1/RPT-2026-00001.pdf"
    artifact_size_bytes  BIGINT,
    error_message        TEXT,
    started_at           TIMESTAMPTZ,
    completed_at         TIMESTAMPTZ,
    expires_at           TIMESTAMPTZ,                  -- artifact retention expiration e.g. +7 days
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Workspace queries with status filtering
CREATE INDEX IF NOT EXISTS idx_report_runs_workspace_status
    ON report_runs (workspace_id, status, created_at DESC);

-- Lookup by sequential report number
CREATE INDEX IF NOT EXISTS idx_report_runs_number
    ON report_runs (workspace_id, report_number);

-- Retention cleanup query index
CREATE INDEX IF NOT EXISTS idx_report_runs_expires
    ON report_runs (expires_at)
    WHERE expires_at IS NOT NULL;
