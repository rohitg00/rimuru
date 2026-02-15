-- Create cost_records table for tracking API usage costs
CREATE TABLE IF NOT EXISTS cost_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    model_name VARCHAR(255) NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_usd DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for common query patterns
CREATE INDEX idx_cost_records_session_id ON cost_records(session_id);
CREATE INDEX idx_cost_records_agent_id ON cost_records(agent_id);
CREATE INDEX idx_cost_records_recorded_at ON cost_records(recorded_at DESC);
CREATE INDEX idx_cost_records_model_name ON cost_records(model_name);

-- Composite index for date range queries by agent
CREATE INDEX idx_cost_records_agent_date ON cost_records(agent_id, recorded_at DESC);

-- Index for cost aggregation queries
CREATE INDEX idx_cost_records_recorded_at_cost ON cost_records(recorded_at, cost_usd);

-- Add comment for documentation
COMMENT ON TABLE cost_records IS 'Tracks API usage costs per model invocation';
COMMENT ON COLUMN cost_records.input_tokens IS 'Number of tokens in the input/prompt';
COMMENT ON COLUMN cost_records.output_tokens IS 'Number of tokens in the output/completion';
COMMENT ON COLUMN cost_records.cost_usd IS 'Calculated cost in USD for this invocation';
