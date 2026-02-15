#[macro_export]
macro_rules! rimuru_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        author: $author:expr,
        description: $description:expr,
        capabilities: [$($cap:expr),* $(,)?]
        $(, homepage: $homepage:expr)?
        $(, repository: $repository:expr)?
        $(, license: $license:expr)?
    ) => {
        fn plugin_info() -> $crate::PluginInfo {
            $crate::PluginInfo {
                name: $name.to_string(),
                version: $version.to_string(),
                author: $author.to_string(),
                description: $description.to_string(),
                capabilities: vec![$($cap),*],
                homepage: rimuru_plugin!(@opt $($homepage)?),
                repository: rimuru_plugin!(@opt $($repository)?),
                license: rimuru_plugin!(@opt $($license)?),
            }
        }
    };

    (@opt) => { None };
    (@opt $val:expr) => { Some($val.to_string()) };
}

#[macro_export]
macro_rules! define_exporter {
    (
        $struct_name:ident,
        name: $name:expr,
        version: $version:expr,
        format: $format:expr,
        extension: $extension:expr
        $(, author: $author:expr)?
        $(, description: $description:expr)?
    ) => {
        pub struct $struct_name {
            info: $crate::PluginInfo,
            initialized: bool,
            config: $crate::PluginConfig,
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    info: $crate::PluginInfo::new($name, $version)
                        .with_capability($crate::PluginCapability::Exporter)
                        $(.with_author($author))?
                        $(.with_description($description))?,
                    initialized: false,
                    config: $crate::PluginConfig::default(),
                }
            }

            pub fn format_name(&self) -> &'static str {
                $format
            }

            pub fn extension(&self) -> &'static str {
                $extension
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

#[macro_export]
macro_rules! define_notifier {
    (
        $struct_name:ident,
        name: $name:expr,
        version: $version:expr,
        notification_type: $notif_type:expr
        $(, author: $author:expr)?
        $(, description: $description:expr)?
    ) => {
        pub struct $struct_name {
            info: $crate::PluginInfo,
            initialized: bool,
            config: $crate::PluginConfig,
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    info: $crate::PluginInfo::new($name, $version)
                        .with_capability($crate::PluginCapability::Notifier)
                        $(.with_author($author))?
                        $(.with_description($description))?,
                    initialized: false,
                    config: $crate::PluginConfig::default(),
                }
            }

            pub fn notif_type(&self) -> &'static str {
                $notif_type
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

#[macro_export]
macro_rules! define_agent {
    (
        $struct_name:ident,
        name: $name:expr,
        version: $version:expr,
        agent_type: $agent_type:expr
        $(, author: $author:expr)?
        $(, description: $description:expr)?
    ) => {
        pub struct $struct_name {
            info: $crate::PluginInfo,
            initialized: bool,
            connected: bool,
            config: $crate::PluginConfig,
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    info: $crate::PluginInfo::new($name, $version)
                        .with_capability($crate::PluginCapability::Agent)
                        $(.with_author($author))?
                        $(.with_description($description))?,
                    initialized: false,
                    connected: false,
                    config: $crate::PluginConfig::default(),
                }
            }

            pub fn agent_type_name(&self) -> &'static str {
                $agent_type
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_plugin_base {
    ($struct_name:ty) => {
        #[$crate::async_trait]
        impl $crate::Plugin for $struct_name {
            fn info(&self) -> &$crate::PluginInfo {
                &self.info
            }

            async fn init(&mut self, _ctx: &$crate::PluginContext) -> $crate::RimuruResult<()> {
                self.initialized = true;
                Ok(())
            }

            async fn shutdown(&mut self) -> $crate::RimuruResult<()> {
                self.initialized = false;
                Ok(())
            }

            fn is_initialized(&self) -> bool {
                self.initialized
            }

            fn configure(&mut self, config: $crate::PluginConfig) -> $crate::RimuruResult<()> {
                self.config = config;
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! hook_handler {
    (
        $struct_name:ident,
        name: $name:expr,
        hook: $hook:expr
        $(, priority: $priority:expr)?
        $(, description: $description:expr)?
    ) => {
        pub struct $struct_name {
            name: &'static str,
            hook: $crate::Hook,
            #[allow(dead_code)]
            priority: i32,
            #[allow(dead_code)]
            description: Option<&'static str>,
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    name: $name,
                    hook: $hook,
                    priority: hook_handler!(@priority $($priority)?),
                    description: hook_handler!(@desc $($description)?),
                }
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    (@priority) => { 0 };
    (@priority $val:expr) => { $val };
    (@desc) => { None };
    (@desc $val:expr) => { Some($val) };
}

#[macro_export]
macro_rules! impl_hook_handler {
    ($struct_name:ty, |$ctx:ident| $body:expr) => {
        #[$crate::async_trait]
        impl $crate::HookHandler for $struct_name {
            fn name(&self) -> &str {
                self.name
            }

            fn hook(&self) -> $crate::Hook {
                self.hook.clone()
            }

            fn priority(&self) -> i32 {
                self.priority
            }

            fn description(&self) -> Option<&str> {
                self.description
            }

            async fn handle(
                &self,
                $ctx: &$crate::HookContext,
            ) -> $crate::RimuruResult<$crate::HookResult> {
                $body
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_trait;
    use crate::HookHandler;
    use crate::{
        Hook, HookContext, HookData, HookResult, PluginCapability, PluginInfo, RimuruResult,
    };

    #[test]
    fn test_rimuru_plugin_macro() {
        rimuru_plugin!(
            name: "test-plugin",
            version: "1.0.0",
            author: "Test Author",
            description: "A test plugin",
            capabilities: [PluginCapability::Exporter],
            homepage: "https://example.com",
            repository: "https://github.com/example/plugin",
            license: "MIT"
        );

        let info = plugin_info();
        assert_eq!(info.name, "test-plugin");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.author, "Test Author");
        assert!(info.capabilities.contains(&PluginCapability::Exporter));
    }

    #[test]
    fn test_define_exporter_macro() {
        define_exporter!(
            TestExporter,
            name: "test-exporter",
            version: "0.1.0",
            format: "test",
            extension: "tst",
            author: "Test",
            description: "Test exporter"
        );

        let exporter = TestExporter::new();
        assert_eq!(exporter.format_name(), "test");
        assert_eq!(exporter.extension(), "tst");
    }

    #[test]
    fn test_define_notifier_macro() {
        define_notifier!(
            TestNotifier,
            name: "test-notifier",
            version: "0.1.0",
            notification_type: "test",
            author: "Test",
            description: "Test notifier"
        );

        let notifier = TestNotifier::new();
        assert_eq!(notifier.notif_type(), "test");
    }

    #[test]
    fn test_define_agent_macro() {
        define_agent!(
            TestAgent,
            name: "test-agent",
            version: "0.1.0",
            agent_type: "test",
            author: "Test",
            description: "Test agent"
        );

        let agent = TestAgent::new();
        assert_eq!(agent.agent_type_name(), "test");
    }

    #[test]
    fn test_hook_handler_macro() {
        hook_handler!(
            TestHookHandler,
            name: "test-handler",
            hook: Hook::PreSessionStart,
            priority: 10,
            description: "Test handler"
        );

        let handler = TestHookHandler::new();
        assert_eq!(handler.name, "test-handler");
        assert_eq!(handler.priority, 10);
    }
}
