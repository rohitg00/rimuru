-- Create system_metrics table for storing system resource snapshots
-- Uses BRIN index for time-series data which is more efficient than B-tree for sequential timestamps
CREATE TABLE IF NOT EXISTS system_metrics (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    cpu_percent REAL NOT NULL,
    memory_used_mb BIGINT NOT NULL,
    memory_total_mb BIGINT NOT NULL,
    active_sessions INTEGER NOT NULL DEFAULT 0,

    -- Use timestamp as primary key since we don't need a separate UUID
    PRIMARY KEY (timestamp)
);

-- BRIN index is highly efficient for time-series data with sequential inserts
CREATE INDEX idx_system_metrics_timestamp_brin ON system_metrics USING BRIN (timestamp);

-- Standard B-tree index for recent data queries
CREATE INDEX idx_system_metrics_timestamp_btree ON system_metrics(timestamp DESC);

-- Create a function to clean up old metrics (keep last 30 days by default)
CREATE OR REPLACE FUNCTION cleanup_old_system_metrics(days_to_keep INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM system_metrics
    WHERE timestamp < NOW() - (days_to_keep || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Add comment for documentation
COMMENT ON TABLE system_metrics IS 'Time-series data for system resource monitoring (CPU, RAM, sessions)';
COMMENT ON COLUMN system_metrics.cpu_percent IS 'CPU utilization percentage (0-100)';
COMMENT ON COLUMN system_metrics.memory_used_mb IS 'Memory currently in use in megabytes';
COMMENT ON COLUMN system_metrics.memory_total_mb IS 'Total system memory in megabytes';
COMMENT ON COLUMN system_metrics.active_sessions IS 'Count of currently active agent sessions';
COMMENT ON FUNCTION cleanup_old_system_metrics IS 'Removes metrics older than specified days (default 30)';
