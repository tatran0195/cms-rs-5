-- Workspace Invitations table
-- Manages secure invitations to company documentation workspaces with SHA-256 token hashing
CREATE TABLE IF NOT EXISTS workspace_invitations (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id         TEXT        NOT NULL,
    email                TEXT        NOT NULL,
    role                 TEXT        NOT NULL DEFAULT 'member', -- 'owner' | 'admin' | 'member' | 'viewer'
    token_hash           TEXT        NOT NULL,
    invited_by_user_id   TEXT,
    status               TEXT        NOT NULL DEFAULT 'pending', -- 'pending' | 'accepted' | 'revoked' | 'expired'
    expires_at           TIMESTAMPTZ NOT NULL,
    accepted_at          TIMESTAMPTZ,
    revoked_at           TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Lookups by workspace and status
CREATE INDEX IF NOT EXISTS idx_workspace_invitations_workspace_status
    ON workspace_invitations (workspace_id, status);

-- Token verification lookup (unique)
CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_invitations_token_hash
    ON workspace_invitations (token_hash);

-- Pending invite lookups by email
CREATE INDEX IF NOT EXISTS idx_workspace_invitations_pending_email
    ON workspace_invitations (workspace_id, email, status)
    WHERE status = 'pending';
