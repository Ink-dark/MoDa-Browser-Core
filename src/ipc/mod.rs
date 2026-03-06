// MoDa Browser Core - IPC Module
// M004 - 进程间通信机制

pub mod channel;
pub mod message;
pub mod security;
pub mod serialization;

use crate::sandbox::SandboxId;

/// IPC模块的核心结构体
pub struct IPCManager {
    channels: channel::ChannelManager,
    security: security::SecurityManager,
}

impl IPCManager {
    /// 创建新的IPC管理器
    pub fn new() -> Self {
        Self {
            channels: channel::ChannelManager::new(),
            security: security::SecurityManager::new(),
        }
    }

    /// 初始化IPC管理器
    pub fn init(&mut self) -> Result<(), String> {
        self.channels.init()?;
        self.security.init()?;
        Ok(())
    }

    /// 创建通信通道
    pub fn create_channel(
        &mut self,
        from: SandboxId,
        to: SandboxId,
    ) -> Result<channel::ChannelId, String> {
        self.channels.create_channel(from, to)
    }

    /// 发送消息
    pub async fn send_message(
        &mut self,
        channel_id: channel::ChannelId,
        msg: message::Message,
    ) -> Result<(), String> {
        // 序列化消息
        let serialized_msg = serialization::serialize(&msg)?;

        // 加密消息
        let encrypted_msg = self.security.encrypt_message(serialized_msg)?;

        // 发送消息
        self.channels.send_message(channel_id, encrypted_msg).await
    }

    /// 接收消息
    pub async fn receive_message(
        &mut self,
        channel_id: channel::ChannelId,
    ) -> Result<message::Message, String> {
        // 接收消息
        let encrypted_msg = self.channels.receive_message(channel_id).await?;

        // 解密消息
        let serialized_msg = self.security.decrypt_message(encrypted_msg)?;

        // 反序列化消息
        let msg = serialization::deserialize(&serialized_msg)?;

        Ok(msg)
    }
}
