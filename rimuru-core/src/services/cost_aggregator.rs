use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::adapters::{AdapterRegistry, UsageStats};
use crate::error::RimuruResult;
use crate::models::AgentType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeRange {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    ThisMonth,
    LastMonth,
    AllTime,
    Custom,
}

impl TimeRange {
    pub fn to_datetime(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match self {
            TimeRange::Today => Some(now.date_naive().and_hms_opt(0, 0, 0)?.and_utc()),
            TimeRange::Yesterday => {
                let yesterday = now - Duration::days(1);
                Some(yesterday.date_naive().and_hms_opt(0, 0, 0)?.and_utc())
            }
            TimeRange::Last7Days => Some(now - Duration::days(7)),
            TimeRange::Last30Days => Some(now - Duration::days(30)),
            TimeRange::ThisMonth => {
                let first_of_month = now.date_naive().with_day(1)?;
                Some(first_of_month.and_hms_opt(0, 0, 0)?.and_utc())
            }
            TimeRange::LastMonth => {
                let first_of_this_month = now.date_naive().with_day(1)?;
                let last_month = first_of_this_month - Duration::days(1);
                let first_of_last_month = last_month.with_day(1)?;
                Some(first_of_last_month.and_hms_opt(0, 0, 0)?.and_utc())
            }
            TimeRange::AllTime => None,
            TimeRange::Custom => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub agent_name: String,
    pub agent_type: AgentType,
    pub model_name: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub requests: i64,
}

impl CostBreakdown {
    pub fn new(agent_name: String, agent_type: AgentType) -> Self {
        Self {
            agent_name,
            agent_type,
            model_name: None,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            input_cost: 0.0,
            output_cost: 0.0,
            total_cost: 0.0,
            requests: 0,
        }
    }

    pub fn add_usage(&mut self, usage: &UsageStats, input_cost_rate: f64, output_cost_rate: f64) {
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.total_tokens = self.input_tokens + self.output_tokens;
        self.requests += usage.requests;

        let input_cost = usage.input_tokens as f64 * input_cost_rate / 1_000_000.0;
        let output_cost = usage.output_tokens as f64 * output_cost_rate / 1_000_000.0;

        self.input_cost += input_cost;
        self.output_cost += output_cost;
        self.total_cost = self.input_cost + self.output_cost;

        if self.model_name.is_none() {
            self.model_name = usage.model_name.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedCostReport {
    pub time_range: TimeRange,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub total_input_cost: f64,
    pub total_output_cost: f64,
    pub total_cost: f64,
    pub total_requests: i64,
    pub by_agent: Vec<CostBreakdown>,
    pub by_agent_type: HashMap<AgentType, CostBreakdown>,
    pub by_model: HashMap<String, CostBreakdown>,
}

impl AggregatedCostReport {
    pub fn new(time_range: TimeRange, start_time: Option<DateTime<Utc>>) -> Self {
        Self {
            time_range,
            start_time,
            end_time: Utc::now(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            total_input_cost: 0.0,
            total_output_cost: 0.0,
            total_cost: 0.0,
            total_requests: 0,
            by_agent: Vec::new(),
            by_agent_type: HashMap::new(),
            by_model: HashMap::new(),
        }
    }

    pub fn add_breakdown(&mut self, breakdown: CostBreakdown) {
        self.total_input_tokens += breakdown.input_tokens;
        self.total_output_tokens += breakdown.output_tokens;
        self.total_tokens += breakdown.total_tokens;
        self.total_input_cost += breakdown.input_cost;
        self.total_output_cost += breakdown.output_cost;
        self.total_cost += breakdown.total_cost;
        self.total_requests += breakdown.requests;

        let type_entry = self
            .by_agent_type
            .entry(breakdown.agent_type)
            .or_insert_with(|| {
                CostBreakdown::new(breakdown.agent_type.to_string(), breakdown.agent_type)
            });

        type_entry.input_tokens += breakdown.input_tokens;
        type_entry.output_tokens += breakdown.output_tokens;
        type_entry.total_tokens += breakdown.total_tokens;
        type_entry.input_cost += breakdown.input_cost;
        type_entry.output_cost += breakdown.output_cost;
        type_entry.total_cost += breakdown.total_cost;
        type_entry.requests += breakdown.requests;

        if let Some(ref model_name) = breakdown.model_name {
            let model_entry = self.by_model.entry(model_name.clone()).or_insert_with(|| {
                let mut b = CostBreakdown::new(model_name.clone(), breakdown.agent_type);
                b.model_name = Some(model_name.clone());
                b
            });

            model_entry.input_tokens += breakdown.input_tokens;
            model_entry.output_tokens += breakdown.output_tokens;
            model_entry.total_tokens += breakdown.total_tokens;
            model_entry.input_cost += breakdown.input_cost;
            model_entry.output_cost += breakdown.output_cost;
            model_entry.total_cost += breakdown.total_cost;
            model_entry.requests += breakdown.requests;
        }

        self.by_agent.push(breakdown);
    }

    pub fn average_cost_per_request(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_cost / self.total_requests as f64
        } else {
            0.0
        }
    }

    pub fn average_tokens_per_request(&self) -> f64 {
        if self.total_requests > 0 {
            self.total_tokens as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CostFilter {
    pub agent_names: Option<Vec<String>>,
    pub agent_types: Option<Vec<AgentType>>,
    pub models: Option<Vec<String>>,
    pub min_cost: Option<f64>,
    pub max_cost: Option<f64>,
}

impl CostFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_agents(mut self, agents: Vec<String>) -> Self {
        self.agent_names = Some(agents);
        self
    }

    pub fn with_agent_types(mut self, types: Vec<AgentType>) -> Self {
        self.agent_types = Some(types);
        self
    }

    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = Some(models);
        self
    }

    pub fn with_min_cost(mut self, min: f64) -> Self {
        self.min_cost = Some(min);
        self
    }

    pub fn with_max_cost(mut self, max: f64) -> Self {
        self.max_cost = Some(max);
        self
    }

    pub fn matches(&self, breakdown: &CostBreakdown) -> bool {
        if let Some(ref agents) = self.agent_names {
            if !agents.contains(&breakdown.agent_name) {
                return false;
            }
        }

        if let Some(ref types) = self.agent_types {
            if !types.contains(&breakdown.agent_type) {
                return false;
            }
        }

        if let Some(ref models) = self.models {
            if let Some(ref model) = breakdown.model_name {
                if !models.contains(model) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(min) = self.min_cost {
            if breakdown.total_cost < min {
                return false;
            }
        }

        if let Some(max) = self.max_cost {
            if breakdown.total_cost > max {
                return false;
            }
        }

        true
    }
}

pub struct CostAggregator {
    registry: Arc<AdapterRegistry>,
}

impl CostAggregator {
    pub fn new(registry: Arc<AdapterRegistry>) -> Self {
        Self { registry }
    }

    pub async fn get_total_cost(&self, time_range: TimeRange) -> RimuruResult<f64> {
        let since = time_range.to_datetime();
        self.registry.get_aggregated_cost(since).await
    }

    pub async fn get_total_cost_since(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        self.registry.get_aggregated_cost(since).await
    }

    pub async fn get_usage(&self, time_range: TimeRange) -> RimuruResult<UsageStats> {
        let since = time_range.to_datetime();
        self.registry.get_aggregated_usage(since).await
    }

    pub async fn get_usage_since(&self, since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
        self.registry.get_aggregated_usage(since).await
    }

    pub async fn get_cost_report(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<AggregatedCostReport> {
        let since = time_range.to_datetime();
        self.get_cost_report_since(time_range, since).await
    }

    pub async fn get_cost_report_since(
        &self,
        time_range: TimeRange,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<AggregatedCostReport> {
        let mut report = AggregatedCostReport::new(time_range, since);

        let adapter_names = self.registry.list_names().await;

        for name in adapter_names {
            if let Some(adapter) = self.registry.get(&name).await {
                let adapter_guard = adapter.read().await;
                let agent_type = adapter_guard.agent_type();

                if let Ok(usage) = adapter_guard.get_usage(since).await {
                    let mut breakdown = CostBreakdown::new(name.clone(), agent_type);

                    let (input_rate, output_rate) = if let Some(ref model_name) = usage.model_name {
                        if let Ok(Some(model_info)) = adapter_guard.get_model_info(model_name).await
                        {
                            (
                                model_info.input_price_per_1k * 1000.0,
                                model_info.output_price_per_1k * 1000.0,
                            )
                        } else {
                            (15.0, 75.0)
                        }
                    } else {
                        (15.0, 75.0)
                    };

                    breakdown.add_usage(&usage, input_rate, output_rate);
                    report.add_breakdown(breakdown);
                }
            }
        }

        Ok(report)
    }

    pub async fn get_filtered_cost_report(
        &self,
        time_range: TimeRange,
        filter: CostFilter,
    ) -> RimuruResult<AggregatedCostReport> {
        let full_report = self.get_cost_report(time_range).await?;

        let mut filtered_report = AggregatedCostReport::new(time_range, full_report.start_time);

        for breakdown in full_report.by_agent {
            if filter.matches(&breakdown) {
                filtered_report.add_breakdown(breakdown);
            }
        }

        Ok(filtered_report)
    }

    pub async fn get_cost_by_agent(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<Vec<CostBreakdown>> {
        let report = self.get_cost_report(time_range).await?;
        Ok(report.by_agent)
    }

    pub async fn get_cost_by_agent_type(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<HashMap<AgentType, CostBreakdown>> {
        let report = self.get_cost_report(time_range).await?;
        Ok(report.by_agent_type)
    }

    pub async fn get_cost_by_model(
        &self,
        time_range: TimeRange,
    ) -> RimuruResult<HashMap<String, CostBreakdown>> {
        let report = self.get_cost_report(time_range).await?;
        Ok(report.by_model)
    }

    pub async fn get_agent_cost(
        &self,
        agent_name: &str,
        time_range: TimeRange,
    ) -> RimuruResult<f64> {
        let since = time_range.to_datetime();

        if let Some(adapter) = self.registry.get(agent_name).await {
            let adapter_guard = adapter.read().await;
            adapter_guard.get_total_cost(since).await
        } else {
            Ok(0.0)
        }
    }

    pub async fn compare_costs(
        &self,
        time_range1: TimeRange,
        time_range2: TimeRange,
    ) -> RimuruResult<(AggregatedCostReport, AggregatedCostReport, f64)> {
        let report1 = self.get_cost_report(time_range1).await?;
        let report2 = self.get_cost_report(time_range2).await?;

        let diff = report2.total_cost - report1.total_cost;
        let percent_change = if report1.total_cost > 0.0 {
            (diff / report1.total_cost) * 100.0
        } else {
            0.0
        };

        Ok((report1, report2, percent_change))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_to_datetime() {
        let today = TimeRange::Today.to_datetime();
        assert!(today.is_some());

        let all_time = TimeRange::AllTime.to_datetime();
        assert!(all_time.is_none());

        let last_7 = TimeRange::Last7Days.to_datetime();
        assert!(last_7.is_some());
    }

    #[test]
    fn test_cost_breakdown_add_usage() {
        let mut breakdown = CostBreakdown::new("test".to_string(), AgentType::ClaudeCode);

        let usage = UsageStats {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            requests: 10,
            model_name: Some("claude-3-opus".to_string()),
            period_start: None,
            period_end: None,
        };

        breakdown.add_usage(&usage, 15.0, 75.0);

        assert_eq!(breakdown.input_tokens, 1_000_000);
        assert_eq!(breakdown.output_tokens, 500_000);
        assert_eq!(breakdown.total_tokens, 1_500_000);
        assert_eq!(breakdown.requests, 10);
        assert!((breakdown.input_cost - 15.0).abs() < 0.01);
        assert!((breakdown.output_cost - 37.5).abs() < 0.01);
        assert!((breakdown.total_cost - 52.5).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_cost_report() {
        let mut report = AggregatedCostReport::new(TimeRange::Today, Some(Utc::now()));

        let mut breakdown1 = CostBreakdown::new("agent-1".to_string(), AgentType::ClaudeCode);
        breakdown1.input_tokens = 1000;
        breakdown1.output_tokens = 500;
        breakdown1.total_tokens = 1500;
        breakdown1.total_cost = 0.10;
        breakdown1.requests = 5;
        breakdown1.model_name = Some("claude-3-opus".to_string());

        let mut breakdown2 = CostBreakdown::new("agent-2".to_string(), AgentType::OpenCode);
        breakdown2.input_tokens = 2000;
        breakdown2.output_tokens = 1000;
        breakdown2.total_tokens = 3000;
        breakdown2.total_cost = 0.05;
        breakdown2.requests = 10;
        breakdown2.model_name = Some("gpt-4".to_string());

        report.add_breakdown(breakdown1);
        report.add_breakdown(breakdown2);

        assert_eq!(report.total_tokens, 4500);
        assert!((report.total_cost - 0.15).abs() < 0.01);
        assert_eq!(report.total_requests, 15);
        assert_eq!(report.by_agent.len(), 2);
        assert_eq!(report.by_agent_type.len(), 2);
        assert_eq!(report.by_model.len(), 2);
    }

    #[test]
    fn test_cost_report_averages() {
        let mut report = AggregatedCostReport::new(TimeRange::Today, None);
        report.total_cost = 1.0;
        report.total_tokens = 10000;
        report.total_requests = 100;

        assert!((report.average_cost_per_request() - 0.01).abs() < 0.0001);
        assert!((report.average_tokens_per_request() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_cost_filter() {
        let filter = CostFilter::new()
            .with_agents(vec!["agent-1".to_string()])
            .with_min_cost(0.01);

        let mut breakdown1 = CostBreakdown::new("agent-1".to_string(), AgentType::ClaudeCode);
        breakdown1.total_cost = 0.10;

        let mut breakdown2 = CostBreakdown::new("agent-2".to_string(), AgentType::OpenCode);
        breakdown2.total_cost = 0.10;

        let mut breakdown3 = CostBreakdown::new("agent-1".to_string(), AgentType::ClaudeCode);
        breakdown3.total_cost = 0.001;

        assert!(filter.matches(&breakdown1));
        assert!(!filter.matches(&breakdown2));
        assert!(!filter.matches(&breakdown3));
    }

    #[test]
    fn test_cost_filter_with_agent_types() {
        let filter = CostFilter::new().with_agent_types(vec![AgentType::ClaudeCode]);

        let breakdown1 = CostBreakdown::new("agent-1".to_string(), AgentType::ClaudeCode);
        let breakdown2 = CostBreakdown::new("agent-2".to_string(), AgentType::OpenCode);

        assert!(filter.matches(&breakdown1));
        assert!(!filter.matches(&breakdown2));
    }

    #[test]
    fn test_cost_filter_with_models() {
        let filter = CostFilter::new().with_models(vec!["claude-3-opus".to_string()]);

        let mut breakdown1 = CostBreakdown::new("agent-1".to_string(), AgentType::ClaudeCode);
        breakdown1.model_name = Some("claude-3-opus".to_string());

        let mut breakdown2 = CostBreakdown::new("agent-2".to_string(), AgentType::OpenCode);
        breakdown2.model_name = Some("gpt-4".to_string());

        let breakdown3 = CostBreakdown::new("agent-3".to_string(), AgentType::ClaudeCode);

        assert!(filter.matches(&breakdown1));
        assert!(!filter.matches(&breakdown2));
        assert!(!filter.matches(&breakdown3));
    }

    #[tokio::test]
    async fn test_cost_aggregator_creation() {
        let registry = Arc::new(AdapterRegistry::new());
        let aggregator = CostAggregator::new(registry);

        let cost = aggregator.get_total_cost(TimeRange::AllTime).await.unwrap();
        assert_eq!(cost, 0.0);
    }

    #[tokio::test]
    async fn test_cost_aggregator_empty_report() {
        let registry = Arc::new(AdapterRegistry::new());
        let aggregator = CostAggregator::new(registry);

        let report = aggregator.get_cost_report(TimeRange::Today).await.unwrap();
        assert_eq!(report.total_cost, 0.0);
        assert_eq!(report.by_agent.len(), 0);
    }
}
