use iii_sdk::{III, RegisterTriggerInput};
use serde_json::json;
use tracing::info;

struct Route {
    method: &'static str,
    path: &'static str,
    function_id: &'static str,
}

const ROUTES: &[Route] = &[
    Route {
        method: "GET",
        path: "api/agents",
        function_id: "rimuru.agents.list",
    },
    Route {
        method: "GET",
        path: "api/agents/:id",
        function_id: "rimuru.agents.get",
    },
    Route {
        method: "POST",
        path: "api/agents",
        function_id: "rimuru.agents.create",
    },
    Route {
        method: "POST",
        path: "api/agents/connect",
        function_id: "rimuru.agents.connect",
    },
    Route {
        method: "POST",
        path: "api/agents/:id/disconnect",
        function_id: "rimuru.agents.disconnect",
    },
    Route {
        method: "GET",
        path: "api/agents/detect",
        function_id: "rimuru.agents.detect",
    },
    Route {
        method: "GET",
        path: "api/sessions",
        function_id: "rimuru.sessions.list",
    },
    Route {
        method: "GET",
        path: "api/sessions/:id",
        function_id: "rimuru.sessions.get",
    },
    Route {
        method: "GET",
        path: "api/sessions/active",
        function_id: "rimuru.sessions.active",
    },
    Route {
        method: "GET",
        path: "api/sessions/history",
        function_id: "rimuru.sessions.history",
    },
    Route {
        method: "GET",
        path: "api/costs/summary",
        function_id: "rimuru.costs.summary",
    },
    Route {
        method: "GET",
        path: "api/costs/daily",
        function_id: "rimuru.costs.daily",
    },
    Route {
        method: "GET",
        path: "api/costs/agent/:id",
        function_id: "rimuru.costs.by_agent",
    },
    // Alias: api/costs is a shorthand for api/costs/summary
    Route {
        method: "GET",
        path: "api/costs",
        function_id: "rimuru.costs.summary",
    },
    Route {
        method: "POST",
        path: "api/costs",
        function_id: "rimuru.costs.record",
    },
    Route {
        method: "GET",
        path: "api/system",
        function_id: "rimuru.hardware.get",
    },
    Route {
        method: "POST",
        path: "api/system/detect",
        function_id: "rimuru.hardware.detect",
    },
    Route {
        method: "GET",
        path: "api/models",
        function_id: "rimuru.models.list",
    },
    Route {
        method: "GET",
        path: "api/models/advisor",
        function_id: "rimuru.advisor.assess",
    },
    Route {
        method: "GET",
        path: "api/models/catalog",
        function_id: "rimuru.advisor.catalog",
    },
    // Alias: /runnable filters in the handler via query params
    Route {
        method: "GET",
        path: "api/models/catalog/runnable",
        function_id: "rimuru.advisor.catalog",
    },
    Route {
        method: "POST",
        path: "api/models/sync",
        function_id: "rimuru.models.sync",
    },
    Route {
        method: "GET",
        path: "api/models/:id",
        function_id: "rimuru.models.get",
    },
    Route {
        method: "GET",
        path: "api/metrics",
        function_id: "rimuru.metrics.current",
    },
    Route {
        method: "GET",
        path: "api/metrics/history",
        function_id: "rimuru.metrics.history",
    },
    Route {
        method: "GET",
        path: "api/health",
        function_id: "rimuru.health.check",
    },
    Route {
        method: "GET",
        path: "api/context/breakdown/:session_id",
        function_id: "rimuru.context.breakdown",
    },
    Route {
        method: "GET",
        path: "api/context/breakdowns",
        function_id: "rimuru.context.breakdown_by_session",
    },
    Route {
        method: "GET",
        path: "api/context/utilization",
        function_id: "rimuru.context.utilization",
    },
    Route {
        method: "GET",
        path: "api/context/waste",
        function_id: "rimuru.context.waste",
    },
    Route {
        method: "POST",
        path: "api/mcp/proxy/connect",
        function_id: "rimuru.mcp.proxy.connect",
    },
    Route {
        method: "GET",
        path: "api/mcp/proxy/tools",
        function_id: "rimuru.mcp.proxy.tools",
    },
    Route {
        method: "POST",
        path: "api/mcp/proxy/call",
        function_id: "rimuru.mcp.proxy.call",
    },
    Route {
        method: "GET",
        path: "api/mcp/proxy/search",
        function_id: "rimuru.mcp.proxy.search",
    },
    Route {
        method: "GET",
        path: "api/mcp/proxy/stats",
        function_id: "rimuru.mcp.proxy.stats",
    },
    Route {
        method: "POST",
        path: "api/mcp/proxy/disconnect",
        function_id: "rimuru.mcp.proxy.disconnect",
    },
    Route {
        method: "POST",
        path: "api/hooks/register",
        function_id: "rimuru.hooks.register",
    },
    Route {
        method: "POST",
        path: "api/hooks/dispatch",
        function_id: "rimuru.hooks.dispatch",
    },
    Route {
        method: "POST",
        path: "api/plugins/install",
        function_id: "rimuru.plugins.install",
    },
    Route {
        method: "DELETE",
        path: "api/plugins/:id",
        function_id: "rimuru.plugins.uninstall",
    },
    Route {
        method: "POST",
        path: "api/plugins/:id/enable",
        function_id: "rimuru.plugins.start",
    },
    Route {
        method: "POST",
        path: "api/plugins/:id/disable",
        function_id: "rimuru.plugins.stop",
    },
    Route {
        method: "GET",
        path: "api/config",
        function_id: "rimuru.config.get",
    },
    Route {
        method: "PUT",
        path: "api/config",
        function_id: "rimuru.config.set",
    },
    Route {
        method: "POST",
        path: "api/config",
        function_id: "rimuru.config.set",
    },
    // Budget engine
    Route {
        method: "POST",
        path: "api/budget/check",
        function_id: "rimuru.budget.check",
    },
    Route {
        method: "GET",
        path: "api/budget/status",
        function_id: "rimuru.budget.status",
    },
    Route {
        method: "POST",
        path: "api/budget/set",
        function_id: "rimuru.budget.set",
    },
    Route {
        method: "GET",
        path: "api/budget/alerts",
        function_id: "rimuru.budget.alerts",
    },
    // Runaway detection
    Route {
        method: "POST",
        path: "api/runaway/analyze",
        function_id: "rimuru.runaway.analyze",
    },
    Route {
        method: "GET",
        path: "api/runaway/scan",
        function_id: "rimuru.runaway.scan",
    },
    Route {
        method: "POST",
        path: "api/runaway/configure",
        function_id: "rimuru.runaway.configure",
    },
    Route {
        method: "GET",
        path: "api/runaway/configure",
        function_id: "rimuru.runaway.configure",
    },
    // Guard wrapper
    Route {
        method: "GET",
        path: "api/guard",
        function_id: "rimuru.guard.list",
    },
    Route {
        method: "POST",
        path: "api/guard/register",
        function_id: "rimuru.guard.register",
    },
    Route {
        method: "POST",
        path: "api/guard/complete",
        function_id: "rimuru.guard.complete",
    },
    Route {
        method: "GET",
        path: "api/guard/history",
        function_id: "rimuru.guard.history",
    },
];

/// HTTP API version. Bumped when we make a breaking change to any
/// route's request or response schema. Adding new fields or routes is
/// backwards compatible and stays on the current version.
pub const API_VERSION: &str = "v1";

fn register_one(iii: &III, method: &str, path: &str, function_id: &str) {
    if let Err(e) = iii.register_trigger(RegisterTriggerInput {
        trigger_type: "http".to_string(),
        function_id: function_id.to_string(),
        config: json!({
            "api_path": path,
            "http_method": method,
        }),
    }) {
        tracing::error!(
            "Failed to register HTTP trigger {} -> {}: {}",
            path,
            function_id,
            e
        );
    }
}

pub fn register(iii: &III) {
    // Every route is registered twice: once at /api/v1/<path> as the
    // versioned canonical route, and once at /api/<path> as a
    // backward-compatible alias. The unversioned alias is preserved
    // until external clients have migrated; new schemas should land
    // under /api/v1 and only break the alias on a future major bump.
    for route in ROUTES {
        // route.path always starts with "api/" — strip it and rebuild
        let suffix = route.path.strip_prefix("api/").unwrap_or(route.path);
        let v1_path = format!("api/{}/{}", API_VERSION, suffix);
        register_one(iii, route.method, &v1_path, route.function_id);
        register_one(iii, route.method, route.path, route.function_id);
    }

    info!(
        "Registered {} HTTP API triggers ({} unique routes × 2: /api/v1 canonical + /api alias)",
        ROUTES.len() * 2,
        ROUTES.len()
    );
}
