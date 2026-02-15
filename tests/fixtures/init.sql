-- Rimuru Test Database Initialization Script
-- This script sets up the test database with sample data for integration tests

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create agents table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_type VARCHAR(50) NOT NULL,
    name VARCHAR(255),
    version VARCHAR(50),
    config_path TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    status VARCHAR(50) DEFAULT 'active',
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    project_path TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create cost_records table
CREATE TABLE IF NOT EXISTS cost_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id UUID REFERENCES agents(id) ON DELETE CASCADE,
    model_name VARCHAR(255) NOT NULL,
    input_tokens BIGINT DEFAULT 0,
    output_tokens BIGINT DEFAULT 0,
    cost_usd DECIMAL(12, 6) DEFAULT 0,
    recorded_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create models table
CREATE TABLE IF NOT EXISTS models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    input_cost_per_1k DECIMAL(12, 8) NOT NULL,
    output_cost_per_1k DECIMAL(12, 8) NOT NULL,
    context_window INTEGER NOT NULL,
    max_output_tokens INTEGER,
    capabilities TEXT[] DEFAULT '{}',
    is_deprecated BOOLEAN DEFAULT false,
    aliases TEXT[] DEFAULT '{}',
    last_synced_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(provider, name)
);

-- Create metrics table
CREATE TABLE IF NOT EXISTS metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    cpu_percent DECIMAL(5, 2),
    memory_used_mb BIGINT,
    memory_total_mb BIGINT,
    active_sessions INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}'
);

-- Create sync_history table
CREATE TABLE IF NOT EXISTS sync_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider VARCHAR(100) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    models_synced INTEGER DEFAULT 0,
    error_message TEXT
);

-- Create plugins table
CREATE TABLE IF NOT EXISTS plugins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255),
    description TEXT,
    plugin_type VARCHAR(50) DEFAULT 'native',
    status VARCHAR(50) DEFAULT 'unloaded',
    config JSONB DEFAULT '{}',
    capabilities TEXT[] DEFAULT '{}',
    loaded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create hooks table
CREATE TABLE IF NOT EXISTS hooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    hook_type VARCHAR(100) NOT NULL,
    handler_type VARCHAR(100) NOT NULL,
    config JSONB DEFAULT '{}',
    enabled BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert sample agents for testing
INSERT INTO agents (id, agent_type, name, version, is_active) VALUES
    ('a0000000-0000-0000-0000-000000000001', 'claude_code', 'Claude Code', '1.0.0', true),
    ('a0000000-0000-0000-0000-000000000002', 'codex', 'OpenAI Codex', '1.0.0', true),
    ('a0000000-0000-0000-0000-000000000003', 'copilot', 'GitHub Copilot', '1.0.0', true),
    ('a0000000-0000-0000-0000-000000000004', 'cursor', 'Cursor', '0.45.0', true),
    ('a0000000-0000-0000-0000-000000000005', 'goose', 'Goose', '1.0.0', false),
    ('a0000000-0000-0000-0000-000000000006', 'opencode', 'OpenCode', '0.1.0', true)
ON CONFLICT DO NOTHING;

-- Insert sample sessions for testing
INSERT INTO sessions (id, agent_id, status, started_at, ended_at, project_path) VALUES
    ('s0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'active', NOW() - INTERVAL '2 hours', NULL, '/projects/rimuru'),
    ('s0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001', 'completed', NOW() - INTERVAL '1 day', NOW() - INTERVAL '23 hours', '/projects/rimuru'),
    ('s0000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000002', 'completed', NOW() - INTERVAL '2 days', NOW() - INTERVAL '2 days' + INTERVAL '1 hour', '/projects/other'),
    ('s0000000-0000-0000-0000-000000000004', 'a0000000-0000-0000-0000-000000000003', 'active', NOW() - INTERVAL '30 minutes', NULL, '/projects/frontend'),
    ('s0000000-0000-0000-0000-000000000005', 'a0000000-0000-0000-0000-000000000004', 'error', NOW() - INTERVAL '5 hours', NOW() - INTERVAL '4 hours', '/projects/backend')
ON CONFLICT DO NOTHING;

-- Insert sample cost records for testing
INSERT INTO cost_records (id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at) VALUES
    ('c0000000-0000-0000-0000-000000000001', 's0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'claude-3-opus', 10000, 5000, 0.525, NOW() - INTERVAL '1 hour'),
    ('c0000000-0000-0000-0000-000000000002', 's0000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'claude-3-opus', 8000, 4000, 0.42, NOW() - INTERVAL '30 minutes'),
    ('c0000000-0000-0000-0000-000000000003', 's0000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001', 'claude-3-sonnet', 15000, 7500, 0.1575, NOW() - INTERVAL '23 hours'),
    ('c0000000-0000-0000-0000-000000000004', 's0000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000002', 'gpt-4', 5000, 2500, 0.25, NOW() - INTERVAL '2 days'),
    ('c0000000-0000-0000-0000-000000000005', 's0000000-0000-0000-0000-000000000004', 'a0000000-0000-0000-0000-000000000003', 'gpt-4-turbo', 3000, 1500, 0.075, NOW() - INTERVAL '15 minutes')
ON CONFLICT DO NOTHING;

-- Insert sample models for testing
INSERT INTO models (id, provider, name, input_cost_per_1k, output_cost_per_1k, context_window, max_output_tokens, capabilities) VALUES
    ('m0000000-0000-0000-0000-000000000001', 'anthropic', 'claude-3-opus', 0.015, 0.075, 200000, 4096, ARRAY['text_generation', 'code_completion', 'vision']),
    ('m0000000-0000-0000-0000-000000000002', 'anthropic', 'claude-3-sonnet', 0.003, 0.015, 200000, 4096, ARRAY['text_generation', 'code_completion', 'vision']),
    ('m0000000-0000-0000-0000-000000000003', 'anthropic', 'claude-3-haiku', 0.00025, 0.00125, 200000, 4096, ARRAY['text_generation', 'code_completion']),
    ('m0000000-0000-0000-0000-000000000004', 'openai', 'gpt-4', 0.03, 0.06, 128000, 4096, ARRAY['text_generation', 'code_completion', 'function_calling']),
    ('m0000000-0000-0000-0000-000000000005', 'openai', 'gpt-4-turbo', 0.01, 0.03, 128000, 4096, ARRAY['text_generation', 'code_completion', 'vision', 'function_calling']),
    ('m0000000-0000-0000-0000-000000000006', 'google', 'gemini-pro', 0.00025, 0.0005, 32000, 8192, ARRAY['text_generation', 'code_completion'])
ON CONFLICT DO NOTHING;

-- Insert sample plugins for testing
INSERT INTO plugins (id, name, version, author, description, status, capabilities) VALUES
    ('p0000000-0000-0000-0000-000000000001', 'json-exporter', '1.0.0', 'Rimuru Team', 'Export data to JSON format', 'enabled', ARRAY['exporter']),
    ('p0000000-0000-0000-0000-000000000002', 'csv-exporter', '1.0.0', 'Rimuru Team', 'Export data to CSV format', 'enabled', ARRAY['exporter']),
    ('p0000000-0000-0000-0000-000000000003', 'slack-notifier', '1.0.0', 'Rimuru Team', 'Send notifications to Slack', 'disabled', ARRAY['notifier']),
    ('p0000000-0000-0000-0000-000000000004', 'discord-notifier', '1.0.0', 'Rimuru Team', 'Send notifications to Discord', 'loaded', ARRAY['notifier']),
    ('p0000000-0000-0000-0000-000000000005', 'webhook-notifier', '1.0.0', 'Rimuru Team', 'Send notifications via webhooks', 'enabled', ARRAY['notifier'])
ON CONFLICT DO NOTHING;

-- Insert sample hooks for testing
INSERT INTO hooks (id, name, hook_type, handler_type, enabled, priority) VALUES
    ('h0000000-0000-0000-0000-000000000001', 'session-start-log', 'session_start', 'session_log', true, 10),
    ('h0000000-0000-0000-0000-000000000002', 'session-end-log', 'session_end', 'session_log', true, 10),
    ('h0000000-0000-0000-0000-000000000003', 'cost-alert', 'cost_threshold', 'cost_alert', true, 5),
    ('h0000000-0000-0000-0000-000000000004', 'metrics-export', 'metrics_collected', 'metrics_export', false, 0)
ON CONFLICT DO NOTHING;

-- Insert sample metrics for testing
INSERT INTO metrics (id, timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions) VALUES
    (uuid_generate_v4(), NOW() - INTERVAL '5 minutes', 45.5, 8192, 32768, 2),
    (uuid_generate_v4(), NOW() - INTERVAL '10 minutes', 62.3, 8500, 32768, 3),
    (uuid_generate_v4(), NOW() - INTERVAL '15 minutes', 38.1, 7800, 32768, 1),
    (uuid_generate_v4(), NOW() - INTERVAL '20 minutes', 55.0, 8000, 32768, 2),
    (uuid_generate_v4(), NOW() - INTERVAL '25 minutes', 71.2, 9200, 32768, 4)
ON CONFLICT DO NOTHING;

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_sessions_agent_id ON sessions(agent_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_cost_records_session_id ON cost_records(session_id);
CREATE INDEX IF NOT EXISTS idx_cost_records_agent_id ON cost_records(agent_id);
CREATE INDEX IF NOT EXISTS idx_cost_records_recorded_at ON cost_records(recorded_at);
CREATE INDEX IF NOT EXISTS idx_models_provider ON models(provider);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_sync_history_provider ON sync_history(provider);

-- Grant permissions (for test user)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO rimuru;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO rimuru;
