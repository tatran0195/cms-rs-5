-- Email Verification Tokens for secure onboarding and email address confirmation.
-- Stores SHA-256 token hashes rather than plaintext tokens to prevent offline leak exploitation.

CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    email        TEXT        NOT NULL,
    token_hash   TEXT        NOT NULL UNIQUE,
    purpose      TEXT        NOT NULL DEFAULT 'email_verification',
    expires_at   TIMESTAMPTZ NOT NULL,
    used_at      TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Partial index for active token validation
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_lookup
    ON email_verification_tokens (token_hash)
    WHERE used_at IS NULL;

-- Email index for auditing and rate-limiting
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_email
    ON email_verification_tokens (email);
