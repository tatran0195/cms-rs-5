-- Atomic workspace-scoped sequence generator.
-- Produces formatted IDs like "RPT-2026-00042" for reports, exports, etc.
CREATE TABLE IF NOT EXISTS sequences (
    workspace_id  TEXT        NOT NULL,
    seq_type      TEXT        NOT NULL,   -- e.g. "report", "export", "document"
    prefix        TEXT        NOT NULL,   -- e.g. "RPT-2026-", "EXP-"
    pad_width     INT         NOT NULL DEFAULT 5,
    current_val   BIGINT      NOT NULL DEFAULT 0,
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, seq_type)
);
