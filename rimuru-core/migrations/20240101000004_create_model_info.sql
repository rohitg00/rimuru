-- Create model_info table for storing LLM pricing and metadata
CREATE TABLE IF NOT EXISTS model_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(100) NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    input_price_per_1k DOUBLE PRECISION NOT NULL,
    output_price_per_1k DOUBLE PRECISION NOT NULL,
    context_window INTEGER NOT NULL,
    last_synced TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure unique provider/model combination
    CONSTRAINT uq_model_info_provider_model UNIQUE (provider, model_name)
);

-- Create indexes for common query patterns
CREATE INDEX idx_model_info_provider ON model_info(provider);
CREATE INDEX idx_model_info_model_name ON model_info(model_name);
CREATE INDEX idx_model_info_last_synced ON model_info(last_synced DESC);

-- Seed with common model pricing data
INSERT INTO model_info (provider, model_name, input_price_per_1k, output_price_per_1k, context_window) VALUES
    -- Anthropic Claude models
    ('anthropic', 'claude-opus-4-5-20251101', 0.015, 0.075, 200000),
    ('anthropic', 'claude-sonnet-4-20250514', 0.003, 0.015, 200000),
    ('anthropic', 'claude-3-5-haiku-20241022', 0.001, 0.005, 200000),
    -- OpenAI models
    ('openai', 'gpt-4o', 0.005, 0.015, 128000),
    ('openai', 'gpt-4o-mini', 0.00015, 0.0006, 128000),
    ('openai', 'o1', 0.015, 0.06, 200000),
    ('openai', 'o1-mini', 0.003, 0.012, 128000),
    -- Google models
    ('google', 'gemini-2.0-flash', 0.0001, 0.0004, 1000000),
    ('google', 'gemini-1.5-pro', 0.00125, 0.005, 2000000)
ON CONFLICT (provider, model_name) DO UPDATE SET
    input_price_per_1k = EXCLUDED.input_price_per_1k,
    output_price_per_1k = EXCLUDED.output_price_per_1k,
    context_window = EXCLUDED.context_window,
    last_synced = NOW();

-- Add comment for documentation
COMMENT ON TABLE model_info IS 'Stores LLM model pricing and capability information';
COMMENT ON COLUMN model_info.input_price_per_1k IS 'Price in USD per 1000 input tokens';
COMMENT ON COLUMN model_info.output_price_per_1k IS 'Price in USD per 1000 output tokens';
COMMENT ON COLUMN model_info.context_window IS 'Maximum context window size in tokens';
