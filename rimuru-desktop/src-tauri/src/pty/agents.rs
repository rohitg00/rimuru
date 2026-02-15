pub struct AgentLaunchConfig {
    pub binary: String,
    pub default_args: Vec<String>,
    pub prompt_flag: Option<String>,
}

pub fn get_agent_config(agent_type: &str) -> Option<AgentLaunchConfig> {
    match agent_type {
        "claude_code" => Some(AgentLaunchConfig {
            binary: "claude".to_string(),
            default_args: vec![],
            prompt_flag: Some("--prompt".to_string()),
        }),
        "codex" => Some(AgentLaunchConfig {
            binary: "codex".to_string(),
            default_args: vec![],
            prompt_flag: None,
        }),
        "goose" => Some(AgentLaunchConfig {
            binary: "goose".to_string(),
            default_args: vec!["session".to_string()],
            prompt_flag: None,
        }),
        "open_code" => Some(AgentLaunchConfig {
            binary: "opencode".to_string(),
            default_args: vec![],
            prompt_flag: None,
        }),
        _ => None,
    }
}
