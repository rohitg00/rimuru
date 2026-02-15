use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

fn get_rimuru_binary() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let workspace_root = PathBuf::from(&manifest_dir).join("..");
    let binary_path = workspace_root.join("target").join("debug").join("rimuru");

    if binary_path.exists() {
        return binary_path;
    }

    let direct = PathBuf::from(&manifest_dir)
        .join("target")
        .join("debug")
        .join("rimuru");
    if direct.exists() {
        return direct;
    }

    PathBuf::from("target/debug/rimuru")
}

fn run_rimuru(args: &[&str]) -> Output {
    Command::new(get_rimuru_binary())
        .args(args)
        .output()
        .expect("Failed to execute rimuru command")
}

fn run_rimuru_with_env(args: &[&str], env_vars: Vec<(&str, &str)>) -> Output {
    let mut cmd = Command::new(get_rimuru_binary());
    cmd.args(args);
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    cmd.output().expect("Failed to execute rimuru command")
}

fn output_to_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr_to_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

mod version_command_tests {
    use super::*;

    #[test]
    fn test_version_command_basic() {
        let output = run_rimuru(&["version"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "version command should succeed");
        assert!(stdout.contains("rimuru"), "output should contain 'rimuru'");
        assert!(
            stdout.contains("0.1.0"),
            "output should contain version number"
        );
    }

    #[test]
    fn test_version_command_detailed() {
        let output = run_rimuru(&["version", "--detailed"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "version --detailed should succeed");
        assert!(
            stdout.contains("Version"),
            "output should contain 'Version'"
        );
        assert!(
            stdout.contains("License"),
            "output should contain 'License'"
        );
        assert!(
            stdout.contains("Apache-2.0"),
            "output should contain license type"
        );
        assert!(
            stdout.contains("Supported Agents"),
            "output should list supported agents"
        );
        assert!(
            stdout.contains("Claude Code"),
            "output should mention Claude Code"
        );
    }
}

mod help_command_tests {
    use super::*;

    #[test]
    fn test_help_command() {
        let output = run_rimuru(&["--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "--help should succeed");
        assert!(stdout.contains("Rimuru"), "help should mention Rimuru");
        assert!(stdout.contains("init"), "help should mention init command");
        assert!(
            stdout.contains("status"),
            "help should mention status command"
        );
        assert!(
            stdout.contains("agents"),
            "help should mention agents command"
        );
        assert!(
            stdout.contains("sessions"),
            "help should mention sessions command"
        );
        assert!(
            stdout.contains("costs"),
            "help should mention costs command"
        );
    }

    #[test]
    fn test_agents_help() {
        let output = run_rimuru(&["agents", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "agents --help should succeed");
        assert!(
            stdout.contains("agents") || stdout.contains("Agents"),
            "should mention agents"
        );
    }

    #[test]
    fn test_sessions_help() {
        let output = run_rimuru(&["sessions", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "sessions --help should succeed");
        assert!(
            stdout.contains("sessions") || stdout.contains("Sessions"),
            "should mention sessions"
        );
    }

    #[test]
    fn test_costs_help() {
        let output = run_rimuru(&["costs", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "costs --help should succeed");
        assert!(
            stdout.contains("costs") || stdout.contains("Costs"),
            "should mention costs"
        );
    }

    #[test]
    fn test_models_help() {
        let output = run_rimuru(&["models", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "models --help should succeed");
        assert!(
            stdout.contains("models") || stdout.contains("Models"),
            "should mention models"
        );
    }

    #[test]
    fn test_plugins_help() {
        let output = run_rimuru(&["plugins", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "plugins --help should succeed");
        assert!(
            stdout.contains("plugins") || stdout.contains("Plugins"),
            "should mention plugins"
        );
    }

    #[test]
    fn test_hooks_help() {
        let output = run_rimuru(&["hooks", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "hooks --help should succeed");
        assert!(
            stdout.contains("hooks") || stdout.contains("Hooks"),
            "should mention hooks"
        );
    }

    #[test]
    fn test_sync_help() {
        let output = run_rimuru(&["sync", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "sync --help should succeed");
        assert!(
            stdout.contains("sync") || stdout.contains("Sync"),
            "should mention sync"
        );
    }

    #[test]
    fn test_skills_help() {
        let output = run_rimuru(&["skills", "--help"]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "skills --help should succeed");
        assert!(
            stdout.contains("skills") || stdout.contains("Skills"),
            "should mention skills"
        );
    }
}

mod invalid_command_tests {
    use super::*;

    #[test]
    fn test_invalid_command() {
        let output = run_rimuru(&["nonexistent-command"]);

        assert!(!output.status.success(), "invalid command should fail");
    }

    #[test]
    fn test_invalid_subcommand() {
        let output = run_rimuru(&["agents", "invalid-subcommand"]);

        assert!(!output.status.success(), "invalid subcommand should fail");
    }
}

mod verbose_flag_tests {
    use super::*;

    #[test]
    fn test_verbose_flag_accepted() {
        let output = run_rimuru(&["-v", "version"]);

        assert!(output.status.success(), "-v flag should be accepted");
    }

    #[test]
    fn test_verbose_long_flag_accepted() {
        let output = run_rimuru(&["--verbose", "version"]);

        assert!(output.status.success(), "--verbose flag should be accepted");
    }
}

mod database_connection_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_init_without_database_fails_gracefully() {
        let output = run_rimuru_with_env(
            &["init"],
            vec![(
                "DATABASE_URL",
                "postgres://invalid:invalid@localhost:5432/nonexistent",
            )],
        );

        let stderr = stderr_to_string(&output);
        assert!(
            !output.status.success() || stderr.contains("Error"),
            "init should fail gracefully without valid database"
        );
    }

    #[test]
    #[ignore]
    fn test_status_without_database_fails_gracefully() {
        let output = run_rimuru_with_env(
            &["status"],
            vec![(
                "DATABASE_URL",
                "postgres://invalid:invalid@localhost:5432/nonexistent",
            )],
        );

        assert!(
            !output.status.success(),
            "status should fail without database"
        );
    }
}

#[cfg(feature = "integration_with_db")]
mod database_integration_tests {
    use super::*;

    fn get_test_database_url() -> String {
        env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://rimuru:rimuru@localhost:5432/rimuru_test".to_string())
    }

    #[test]
    fn test_init_creates_database_schema() {
        let db_url = get_test_database_url();
        let output = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        assert!(
            output.status.success(),
            "init should succeed with valid database"
        );

        let stdout = output_to_string(&output);
        assert!(
            stdout.contains("initialized") || stdout.contains("success"),
            "should indicate successful initialization"
        );
    }

    #[test]
    fn test_status_shows_system_metrics() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["status"], vec![("DATABASE_URL", &db_url)]);
        let stdout = output_to_string(&output);

        assert!(output.status.success(), "status should succeed");
        assert!(
            stdout.contains("CPU") || stdout.contains("cpu"),
            "should show CPU metrics"
        );
        assert!(
            stdout.contains("Memory") || stdout.contains("memory"),
            "should show memory metrics"
        );
    }

    #[test]
    fn test_status_json_format() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(
            &["status", "--format", "json"],
            vec![("DATABASE_URL", &db_url)],
        );
        let stdout = output_to_string(&output);

        assert!(
            output.status.success(),
            "status --format json should succeed"
        );

        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        assert!(parsed.is_ok(), "output should be valid JSON");
    }

    #[test]
    fn test_agents_list_empty() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["agents", "list"], vec![("DATABASE_URL", &db_url)]);

        assert!(
            output.status.success(),
            "agents list should succeed even when empty"
        );
    }

    #[test]
    fn test_sessions_list_empty() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["sessions", "list"], vec![("DATABASE_URL", &db_url)]);

        assert!(
            output.status.success(),
            "sessions list should succeed even when empty"
        );
    }

    #[test]
    fn test_costs_summary_empty() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["costs", "summary"], vec![("DATABASE_URL", &db_url)]);

        assert!(
            output.status.success(),
            "costs summary should succeed even when empty"
        );
    }

    #[test]
    fn test_models_list() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["models", "list"], vec![("DATABASE_URL", &db_url)]);

        assert!(output.status.success(), "models list should succeed");
    }

    #[test]
    fn test_plugins_list() {
        let db_url = get_test_database_url();

        let _ = run_rimuru_with_env(&["init"], vec![("DATABASE_URL", &db_url)]);

        let output = run_rimuru_with_env(&["plugins", "list"], vec![("DATABASE_URL", &db_url)]);

        assert!(output.status.success(), "plugins list should succeed");
    }
}

mod cli_output_format_tests {
    use super::*;

    #[test]
    fn test_version_output_is_clean() {
        let output = run_rimuru(&["version"]);
        let stdout = output_to_string(&output);

        assert!(
            !stdout.contains("error"),
            "version output should not contain errors"
        );
        assert!(
            !stdout.contains("panic"),
            "version output should not contain panics"
        );
    }

    #[test]
    fn test_help_output_structure() {
        let output = run_rimuru(&["--help"]);
        let stdout = output_to_string(&output);

        assert!(
            stdout.contains("Usage:") || stdout.contains("USAGE:"),
            "help should show usage section"
        );
        assert!(
            stdout.contains("Commands:")
                || stdout.contains("COMMANDS:")
                || stdout.contains("Subcommands:"),
            "help should show commands section"
        );
    }
}

mod exit_code_tests {
    use super::*;

    #[test]
    fn test_success_returns_zero() {
        let output = run_rimuru(&["version"]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "successful command should return 0"
        );
    }

    #[test]
    fn test_invalid_args_returns_nonzero() {
        let output = run_rimuru(&["--invalid-flag-that-does-not-exist"]);
        assert_ne!(
            output.status.code(),
            Some(0),
            "invalid args should return non-zero"
        );
    }
}
