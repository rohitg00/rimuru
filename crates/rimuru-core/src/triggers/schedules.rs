use iii_sdk::{III, RegisterTriggerInput};
use serde_json::json;
use tracing::info;

struct Schedule {
    cron: &'static str,
    function_id: &'static str,
}

const SCHEDULES: &[Schedule] = &[
    Schedule {
        cron: "0 */5 * * * *",
        function_id: "rimuru.metrics.collect",
    },
    Schedule {
        cron: "0 0 */6 * * *",
        function_id: "rimuru.models.sync",
    },
    Schedule {
        cron: "0 0 0 * * *",
        function_id: "rimuru.costs.daily_rollup",
    },
    Schedule {
        cron: "0 0 1 * * *",
        function_id: "rimuru.sessions.cleanup",
    },
];

pub fn register(iii: &III) {
    for schedule in SCHEDULES {
        match iii.register_trigger(RegisterTriggerInput {
            trigger_type: "cron".to_string(),
            function_id: schedule.function_id.to_string(),
            config: json!({"expression": schedule.cron}),
        }) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(
                    "Failed to register cron trigger {}: {}",
                    schedule.function_id,
                    e
                );
            }
        }
    }

    info!("Registered {} cron triggers", SCHEDULES.len());
}
