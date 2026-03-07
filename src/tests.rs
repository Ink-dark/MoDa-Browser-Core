// MoDa Browser Core 进程隔离功能测试
// 验证沙箱隔离机制和安全验证体系

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_sandbox_manager_creation() {
        let manager = sandbox::SandboxManager::new();
        assert_eq!(manager.get_sandbox("test").is_none(), true);
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = sandbox::SandboxConfig::builder()
            .name("test-sandbox")
            .process_name("notepad.exe")
            .arg("--test")
            .max_memory_mb(512)
            .max_cpu_percent(80)
            .max_runtime_seconds(1800)
            .build();

        assert_eq!(config.name, "test-sandbox");
        assert_eq!(config.process_name, "notepad.exe");
        assert_eq!(config.args, vec!["--test"]);
        assert_eq!(config.resource_limits.max_memory_mb, Some(512));
        assert_eq!(config.resource_limits.max_cpu_percent, Some(80));
        assert_eq!(config.resource_limits.max_runtime_seconds, Some(1800));
    }

    #[test]
    fn test_sandbox_creation() {
        let manager = sandbox::SandboxManager::new();
        let config = sandbox::SandboxConfig::builder()
            .name("test-sandbox")
            .process_name("notepad.exe")
            .build();

        let sandbox = manager.create_sandbox(config);
        assert!(sandbox.is_ok());

        let sandbox = sandbox.unwrap();
        assert_eq!(sandbox.name(), "test-sandbox");
        assert_eq!(sandbox.pid(), None);
    }

    #[test]
    fn test_sandbox_state_transitions() {
        let manager = sandbox::SandboxManager::new();
        let config = sandbox::SandboxConfig::builder()
            .name("test-sandbox")
            .process_name("notepad.exe")
            .build();

        let sandbox = manager.create_sandbox(config).unwrap();

        // 初始状态应该是 Created
        assert!(matches!(sandbox.state(), sandbox::SandboxState::Created));

        // 尝试暂停应该失败
        assert!(sandbox.pause().is_err());

        // 尝试恢复应该失败
        assert!(sandbox.resume().is_err());

        // 终止应该成功
        assert!(sandbox.terminate().is_ok());

        // 状态应该是 Terminated
        assert!(matches!(sandbox.state(), sandbox::SandboxState::Terminated));
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = sandbox::ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, Some(1024));
        assert_eq!(limits.max_processes, Some(10));
        assert_eq!(limits.max_runtime_seconds, Some(3600));
        assert_eq!(limits.max_network_connections, Some(100));
        assert_eq!(limits.max_disk_write_mb, Some(1024));
    }

    #[test]
    fn test_process_monitor_data() {
        let monitor_data = sandbox::ProcessMonitorData {
            pid: 1234,
            cpu_usage: 50.5,
            memory_usage_mb: 256,
            runtime_seconds: 60,
            file_descriptors: 10,
            network_connections: 5,
            disk_write_mb: 100,
            last_update: std::time::Instant::now(),
        };

        assert_eq!(monitor_data.pid, 1234);
        assert_eq!(monitor_data.cpu_usage, 50.5);
        assert_eq!(monitor_data.memory_usage_mb, 256);
        assert_eq!(monitor_data.runtime_seconds, 60);
    }

    #[test]
    fn test_security_framework_creation() {
        let security = security::SecurityFramework::new();
        assert_eq!(security.capability_manager.issued_tokens.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_capability_token_issuance() {
        let security = security::SecurityFramework::new();
        let token = security.capability_manager.issue_token(
            "/test/resource",
            vec![security::Permission::Read, security::Permission::Write],
            "test-subject",
        );

        assert_eq!(token.resource, "/test/resource");
        assert_eq!(token.subject, "test-subject");
        assert!(token.permissions.contains(&security::Permission::Read));
        assert!(token.permissions.contains(&security::Permission::Write));
    }

    #[test]
    fn test_capability_token_verification() {
        let security = security::SecurityFramework::new();
        let token = security.capability_manager.issue_token(
            "/test/resource",
            vec![security::Permission::Read],
            "test-subject",
        );

        let result = security.verify_capability(&token, "/test/resource", &security::Permission::Read);
        assert!(result);

        // 测试权限不足的情况
        let result = security.verify_capability(&token, "/test/resource", &security::Permission::Write);
        assert!(!result);
    }

    #[test]
    fn test_capability_token_revocation() {
        let security = security::SecurityFramework::new();
        let token = security.capability_manager.issue_token(
            "/test/resource",
            vec![security::Permission::Read],
            "test-subject",
        );

        let revoked = security.capability_manager.revoke_token(&token.id);
        assert!(revoked);

        // 再次撤销应该失败
        let revoked = security.capability_manager.revoke_token(&token.id);
        assert!(!revoked);
    }

    #[test]
    fn test_ipc_manager_creation() {
        let security = Arc::new(security::SecurityFramework::new());
        let ipc = ipc::IpcManager::new(security);
        assert_eq!(ipc.get_channel("test-channel").is_none(), true);
    }

    #[test]
    fn test_ipc_channel_creation() {
        let security = Arc::new(security::SecurityFramework::new());
        let ipc = Arc::new(ipc::IpcManager::new(security));

        let key_pair = ipc::IpcManager::generate_key_pair();
        assert!(key_pair.is_ok());

        let key_pair = key_pair.unwrap();
        let channel = ipc.create_channel(
            "test-channel".to_string(),
            "sender-1".to_string(),
            "receiver-1".to_string(),
            key_pair,
        );

        assert!(channel.is_ok());

        let channel = channel.unwrap();
        assert_eq!(channel.id(), "test-channel");
        assert_eq!(channel.sender_id(), "sender-1");
        assert_eq!(channel.receiver_id(), "receiver-1");
    }

    #[test]
    fn test_ipc_channel_retrieval() {
        let security = Arc::new(security::SecurityFramework::new());
        let ipc = Arc::new(ipc::IpcManager::new(security));

        let key_pair = ipc::IpcManager::generate_key_pair().unwrap();
        ipc.create_channel(
            "test-channel".to_string(),
            "sender-1".to_string(),
            "receiver-1".to_string(),
            key_pair,
        )
        .unwrap();

        let channel = ipc.get_channel("test-channel");
        assert!(channel.is_some());
        assert_eq!(channel.unwrap().id(), "test-channel");
    }

    #[test]
    fn test_ipc_channel_removal() {
        let security = Arc::new(security::SecurityFramework::new());
        let ipc = Arc::new(ipc::IpcManager::new(security));

        let key_pair = ipc::IpcManager::generate_key_pair().unwrap();
        ipc.create_channel(
            "test-channel".to_string(),
            "sender-1".to_string(),
            "receiver-1".to_string(),
            key_pair,
        )
        .unwrap();

        let removed = ipc.remove_channel("test-channel");
        assert!(removed.is_ok());

        let channel = ipc.get_channel("test-channel");
        assert!(channel.is_none());
    }

    #[test]
    fn test_ipc_message_creation() {
        let message = ipc::IpcMessage {
            id: "msg-1".to_string(),
            sender: "sender-1".to_string(),
            receiver: "receiver-1".to_string(),
            message_type: ipc::MessageType::Request,
            payload: vec![1, 2, 3, 4],
            capability_token: None,
            timestamp: 1234567890,
            signature: None,
        };

        assert_eq!(message.id, "msg-1");
        assert_eq!(message.sender, "sender-1");
        assert_eq!(message.receiver, "receiver-1");
        assert!(matches!(message.message_type, ipc::MessageType::Request));
        assert_eq!(message.payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_sandbox_manager_shutdown() {
        let manager = sandbox::SandboxManager::new();
        manager.run();
        manager.shutdown();

        // 应该能够安全地多次调用 shutdown
        manager.shutdown();
    }

    #[test]
    fn test_ipc_manager_shutdown() {
        let security = Arc::new(security::SecurityFramework::new());
        let ipc = ipc::IpcManager::new(security);
        ipc.run();
        ipc.shutdown();

        // 应该能够安全地多次调用 shutdown
        ipc.shutdown();
    }

    #[test]
    fn test_security_framework_shutdown() {
        let security = security::SecurityFramework::new();
        security.run();
        security.shutdown();

        // 应该能够安全地多次调用 shutdown
        security.shutdown();
    }

    #[test]
    fn test_sandbox_clone() {
        let manager = sandbox::SandboxManager::new();
        let config = sandbox::SandboxConfig::builder()
            .name("test-sandbox")
            .process_name("notepad.exe")
            .build();

        let sandbox1 = manager.create_sandbox(config).unwrap();
        let sandbox2 = sandbox1.clone();

        assert_eq!(sandbox1.name(), sandbox2.name());
        assert_eq!(sandbox1.state(), sandbox2.state());
    }

    #[test]
    fn test_multiple_sandboxes() {
        let manager = sandbox::SandboxManager::new();

        let config1 = sandbox::SandboxConfig::builder()
            .name("sandbox-1")
            .process_name("notepad.exe")
            .build();

        let config2 = sandbox::SandboxConfig::builder()
            .name("sandbox-2")
            .process_name("calc.exe")
            .build();

        let sandbox1 = manager.create_sandbox(config1).unwrap();
        let sandbox2 = manager.create_sandbox(config2).unwrap();

        assert_eq!(sandbox1.name(), "sandbox-1");
        assert_eq!(sandbox2.name(), "sandbox-2");

        // 应该能够通过名称获取沙箱
        let retrieved = manager.get_sandbox("sandbox-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "sandbox-1");
    }

    #[test]
    fn test_policy_rule_addition() {
        let security = security::SecurityFramework::new();
        let rule = security::PolicyRule {
            resource_pattern: "/test/*".to_string(),
            allowed_permissions: vec![security::Permission::Read],
            condition: None,
        };

        security.policy_manager.add_policy_rule(rule);

        // 验证规则已添加
        let rules = security.policy_manager.rules.lock().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].resource_pattern, "/test/*");
    }
}
