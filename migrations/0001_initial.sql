-- Core PostgreSQL Baseline Schema for cms-rs-5

-- 1. Identity & Authentication
CREATE TABLE IF NOT EXISTS "User" (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    avatar_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_superadmin BOOLEAN NOT NULL DEFAULT FALSE,
    mfa_secret TEXT,
    mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Session" (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES "User"(id) ON DELETE CASCADE,
    token_hash TEXT UNIQUE NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "ApiKey" (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES "User"(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_hash TEXT UNIQUE NOT NULL,
    prefix TEXT NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. Workspaces & Projects
CREATE TABLE IF NOT EXISTS "Workspace" (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Project" (
    id TEXT PRIMARY KEY,
    workspace_id TEXT REFERENCES "Workspace"(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "ProjectMember" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES "User"(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'viewer',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, user_id)
);

-- 3. Content: Branches, Languages, Pages
CREATE TABLE IF NOT EXISTS "Branch" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, slug)
);

CREATE TABLE IF NOT EXISTS "Language" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, code)
);

CREATE TABLE IF NOT EXISTS "Page" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    branch_id TEXT NOT NULL REFERENCES "Branch"(id) ON DELETE CASCADE,
    language_id TEXT REFERENCES "Language"(id) ON DELETE SET NULL,
    parent_id TEXT REFERENCES "Page"(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    path TEXT NOT NULL,
    content TEXT,
    description TEXT,
    sort_order INT NOT NULL DEFAULT 0,
    is_published BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(branch_id, language_id, path)
);

CREATE TABLE IF NOT EXISTS "Comment" (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL REFERENCES "Page"(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES "User"(id) ON DELETE CASCADE,
    parent_id TEXT REFERENCES "Comment"(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. Assets & Storage
CREATE TABLE IF NOT EXISTS "Asset" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    storage_key TEXT UNIQUE NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 5. Deployments & Custom Domains
CREATE TABLE IF NOT EXISTS "Deployment" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    branch_id TEXT NOT NULL REFERENCES "Branch"(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    commit_sha TEXT,
    artifact_url TEXT,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "Domain" (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
    hostname TEXT UNIQUE NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    tls_status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 6. Indexes for Performance
CREATE INDEX IF NOT EXISTS idx_page_project_branch ON "Page"(project_id, branch_id);
CREATE INDEX IF NOT EXISTS idx_page_path ON "Page"(path);
CREATE INDEX IF NOT EXISTS idx_asset_project ON "Asset"(project_id);
CREATE INDEX IF NOT EXISTS idx_deployment_project ON "Deployment"(project_id, created_at DESC);