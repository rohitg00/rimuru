-- Create custom enum type for session status
CREATE TYPE session_status AS ENUM (
    'active',
    'completed',
    'failed',
    'terminated'
);

-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    status session_status NOT NULL DEFAULT 'active',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'
);

-- Create indexes for common query patterns
CREATE INDEX idx_sessions_agent_id ON sessions(agent_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_started_at ON sessions(started_at DESC);
CREATE INDEX idx_sessions_active ON sessions(agent_id) WHERE status = 'active';

-- Composite index for agent + status lookups
CREATE INDEX idx_sessions_agent_status ON sessions(agent_id, status);

-- Add comment for documentation
COMMENT ON TABLE sessions IS 'Tracks individual agent working sessions';
COMMENT ON COLUMN sessions.metadata IS 'JSONB metadata including project info, context, etc.';
