-- Migration 0023: Scheduled jobs and CmsJob scheduled_at
-- Extends CmsJob with scheduled_at for deferred execution and creates cms_scheduled_jobs table

ALTER TABLE IF EXISTS "CmsJob"
    ADD COLUMN IF NOT EXISTS scheduled_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_cms_job_scheduled_available
    ON "CmsJob" (status, available_at, scheduled_at)
    WHERE status IN ('pending', 'retrying');

CREATE TABLE IF NOT EXISTS cms_scheduled_jobs (
    id              TEXT        PRIMARY KEY, -- unique recurring job key e.g. "session_purge"
    cron_expr       TEXT        NOT NULL,    -- standard 5/6 field cron expression e.g. "0 0 * * * *"
    job_type        TEXT        NOT NULL,    -- serialized JobType
    payload         JSONB       NOT NULL DEFAULT '{}'::jsonb,
    enabled         BOOLEAN     NOT NULL DEFAULT true,
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ NOT NULL,
    last_status     TEXT,                    -- 'success', 'failed', 'running'
    last_error      TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cms_scheduled_jobs_next_run
    ON cms_scheduled_jobs (next_run_at, enabled)
    WHERE enabled = true;
