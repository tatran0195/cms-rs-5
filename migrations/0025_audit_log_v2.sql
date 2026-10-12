-- Upgrade the audit log to a tamper-evident SHA-256 hash chain.
-- Renames the old table to security_audit_logs_legacy (preserves history)
-- and creates audit_log with the full field set.

-- Preserve old data
ALTER TABLE IF EXISTS security_audit_logs
    RENAME TO security_audit_logs_legacy;

-- New table
CREATE TABLE IF NOT EXISTS audit_log (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id         TEXT,                              -- NULL for system-level events
    user_id              TEXT,                              -- NULL for system/anonymous
    action               TEXT        NOT NULL,              -- e.g. "create_page", "delete_member"
    entity_type          TEXT        NOT NULL,              -- e.g. "page", "workspace_member"
    entity_id            TEXT,                              -- the affected row id
    old_values           JSONB,                             -- snapshot before change
    new_values           JSONB,                             -- snapshot after change
    ip_address           TEXT,
    user_agent           TEXT,
    session_id           TEXT,
    correlation_id       TEXT,
    module               TEXT,                              -- cms feature area e.g. "content", "auth"
    description          TEXT,                              -- human-readable summary
    prev_hash            TEXT,                              -- SHA-256 of previous row in chain
    hash                 TEXT,                              -- SHA-256 of this row's canonical input
    hash_input_canonical TEXT,                              -- exact bytes hashed (for re-verification)
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_workspace_time
    ON audit_log (workspace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_user
    ON audit_log (user_id, created_at DESC)
    WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_entity
    ON audit_log (entity_type, entity_id)
    WHERE entity_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_hash
    ON audit_log (hash)
    WHERE hash IS NOT NULL;
