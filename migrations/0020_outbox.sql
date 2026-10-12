-- Transactional Outbox for resilient, asynchronous message delivery.
-- Guarantees at-least-once delivery with exponential backoff and Dead Letter Queue (DLQ).

CREATE TABLE IF NOT EXISTS outbox_events (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    TEXT,                               -- Optional workspace isolation scope
    event_type      TEXT        NOT NULL,               -- e.g. "email.verification", "webhook.page_published"
    payload         JSONB       NOT NULL,               -- Serialized event payload
    status          TEXT        NOT NULL DEFAULT 'pending'
                                CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    attempts        INT         NOT NULL DEFAULT 0,
    max_attempts    INT         NOT NULL DEFAULT 10,
    scheduled_at    TIMESTAMPTZ NOT NULL DEFAULT now(), -- When the event is eligible to run
    last_attempt_at TIMESTAMPTZ,
    last_error      TEXT,
    locked_at       TIMESTAMPTZ,                        -- Concurrency lock timestamp
    locked_by       TEXT,                               -- Worker identifier holding lock
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Partial index for high-throughput drain polling:
-- Only index pending or failed events that have not exceeded max_attempts.
CREATE INDEX IF NOT EXISTS idx_outbox_events_drain
    ON outbox_events (scheduled_at, status)
    WHERE status IN ('pending', 'failed') AND attempts < max_attempts;

-- Partial index for dead worker lock recovery (reaper):
CREATE INDEX IF NOT EXISTS idx_outbox_events_locked
    ON outbox_events (locked_at)
    WHERE status = 'processing';

-- Workspace filtering index
CREATE INDEX IF NOT EXISTS idx_outbox_events_workspace
    ON outbox_events (workspace_id)
    WHERE workspace_id IS NOT NULL;

-- Event type index
CREATE INDEX IF NOT EXISTS idx_outbox_events_type
    ON outbox_events (event_type);

-- Dead Letter Queue (DLQ) for permanently failed or retry-exhausted events
CREATE TABLE IF NOT EXISTS outbox_dlq (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    original_event_id UUID        NOT NULL,
    workspace_id      TEXT,
    event_type        TEXT        NOT NULL,
    payload           JSONB       NOT NULL,
    attempts          INT         NOT NULL,
    error_reason      TEXT        NOT NULL,
    failed_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata          JSONB
);

CREATE INDEX IF NOT EXISTS idx_outbox_dlq_event_type
    ON outbox_dlq (event_type);

CREATE INDEX IF NOT EXISTS idx_outbox_dlq_failed_at
    ON outbox_dlq (failed_at DESC);

CREATE INDEX IF NOT EXISTS idx_outbox_dlq_workspace
    ON outbox_dlq (workspace_id)
    WHERE workspace_id IS NOT NULL;
