// MoDa Browser Core - IPC Channel Module
// 实现通道管理和消息传递

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::sandbox::SandboxId;

/// 通道ID类型
pub type ChannelId = u64;

/// 通道结构体
pub struct Channel {
    /// 发送端
    sender: mpsc::UnboundedSender<Vec<u8>>,
    /// 接收端
    receiver: mpsc::UnboundedReceiver<Vec<u8>>,
    /// 发送方沙盒ID
    from: SandboxId,
    /// 接收方沙盒ID
    to: SandboxId,
    /// 通道状态
    status: ChannelStatus,
}

/// 通道状态枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelStatus {
    /// 未连接
    Disconnected,
    /// 已连接
    Connected,
    /// 正在关闭
    Closing,
}

impl Channel {
    /// 创建新通道
    pub fn new(from: SandboxId, to: SandboxId) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver,
            from,
            to,
            status: ChannelStatus::Connected,
        }
    }

    /// 发送消息
    pub async fn send_message(&self, msg: Vec<u8>) -> Result<(), String> {
        if self.status != ChannelStatus::Connected {
            return Err("Channel is not connected".to_string());
        }
        
        match self.sender.send(msg) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to send message: {}", e)),
        }
    }

    /// 接收消息
    pub async fn receive_message(&mut self) -> Result<Vec<u8>, String> {
        if self.status != ChannelStatus::Connected {
            return Err("Channel is not connected".to_string());
        }
        
        match self.receiver.recv().await {
            Some(msg) => Ok(msg),
            None => Err("Channel closed".to_string()),
        }
    }

    /// 关闭通道
    pub fn close(&mut self) {
        self.status = ChannelStatus::Closing;
    }
}

/// 通道管理器结构体
pub struct ChannelManager {
    /// 通道映射
    channels: HashMap<ChannelId, Channel>,
    /// 下一个通道ID
    next_channel_id: ChannelId,
}

impl ChannelManager {
    /// 创建新的通道管理器
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            next_channel_id: 0,
        }
    }

    /// 初始化通道管理器
    pub fn init(&mut self) -> Result<(), String> {
        // 初始化通道管理器，目前不需要特殊操作
        Ok(())
    }

    /// 创建通信通道
    pub fn create_channel(&mut self, from: SandboxId, to: SandboxId) -> Result<ChannelId, String> {
        let channel_id = self.next_channel_id;
        self.next_channel_id += 1;
        
        let channel = Channel::new(from, to);
        self.channels.insert(channel_id, channel);
        
        Ok(channel_id)
    }

    /// 发送消息
    pub async fn send_message(&mut self, channel_id: ChannelId, msg: Vec<u8>) -> Result<(), String> {
        match self.channels.get(channel_id) {
            Some(channel) => channel.send_message(msg).await,
            None => Err(format!("Channel {} not found", channel_id)),
        }
    }

    /// 接收消息
    pub async fn receive_message(&mut self, channel_id: ChannelId) -> Result<Vec<u8>, String> {
        match self.channels.get_mut(channel_id) {
            Some(channel) => channel.receive_message().await,
            None => Err(format!("Channel {} not found", channel_id)),
        }
    }

    /// 关闭通道
    pub fn close_channel(&mut self, channel_id: ChannelId) -> Result<(), String> {
        match self.channels.get_mut(channel_id) {
            Some(channel) => {
                channel.close();
                self.channels.remove(&channel_id);
                Ok(())
            },
            None => Err(format!("Channel {} not found", channel_id)),
        }
    }
}
