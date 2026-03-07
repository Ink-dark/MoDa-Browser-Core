// MoDa Browser Core 进程间通信模块
// 实现安全的跨进程通信机制

use crate::security::{CapabilityToken, Permission, SecurityFramework};
use ring::rand::SecureRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// IPC 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// 消息 ID
    pub id: String,
    /// 发送者 ID
    pub sender: String,
    /// 接收者 ID
    pub receiver: String,
    /// 消息类型
    pub message_type: MessageType,
    /// 消息负载
    pub payload: Vec<u8>,
    /// 能力令牌
    pub capability_token: Option<CapabilityToken>,
    /// 时间戳
    pub timestamp: u64,
    /// 签名
    pub signature: Option<String>,
}

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 请求
    Request,
    /// 响应
    Response,
    /// 通知
    Notification,
    /// 错误
    Error,
}

/// IPC 通道
pub struct IpcChannel {
    /// 通道 ID
    id: String,
    /// 发送者 ID
    sender_id: String,
    /// 接收者 ID
    receiver_id: String,
    /// 发送通道
    tx: mpsc::UnboundedSender<IpcMessage>,
    /// 接收通道
    rx: Arc<Mutex<mpsc::UnboundedReceiver<IpcMessage>>>,
    /// 安全框架
    security: Arc<SecurityFramework>,
    /// 密钥对
    key_pair: Arc<Ed25519KeyPair>,
    /// 消息计数器
    message_counter: Arc<Mutex<u64>>,
}

/// IPC 管理器
pub struct IpcManager {
    /// 通道列表
    channels: Arc<Mutex<HashMap<String, Arc<IpcChannel>>>>,
    /// 安全框架
    security: Arc<SecurityFramework>,
    /// 状态
    state: Arc<Mutex<ManagerState>>,
}

/// 管理器状态
enum ManagerState {
    Initialized,
    Running,
    ShuttingDown,
    Shutdown,
}

impl IpcChannel {
    /// 创建新的 IPC 通道
    pub fn new(
        id: String,
        sender_id: String,
        receiver_id: String,
        security: Arc<SecurityFramework>,
        key_pair: Arc<Ed25519KeyPair>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        debug!(
            "创建 IPC 通道: {} -> {}, 通道 ID: {}",
            sender_id, receiver_id, id
        );

        Self {
            id,
            sender_id,
            receiver_id,
            tx,
            rx: Arc::new(Mutex::new(rx)),
            security,
            key_pair,
            message_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// 发送消息
    pub async fn send(&self, mut message: IpcMessage) -> Result<(), String> {
        debug!("发送 IPC 消息: {} -> {}", message.sender, message.receiver);

        // 验证消息
        if let Err(e) = self.validate_message(&message) {
            error!("消息验证失败: {}", e);
            return Err(e);
        }

        // 添加时间戳
        message.timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 签名消息
        message.signature = Some(self.sign_message(&message)?);

        // 发送消息
        if let Err(e) = self.tx.send(message.clone()) {
            error!("发送消息失败: {}", e);
            return Err(format!("发送消息失败: {}", e));
        }

        // 更新消息计数器
        *self.message_counter.lock().unwrap() += 1;

        debug!("IPC 消息发送成功: {}", message.id);
        Ok(())
    }

    /// 接收消息
    pub async fn receive(&self) -> Result<IpcMessage, String> {
        let mut rx = self.rx.lock().unwrap();
        match rx.recv().await {
            Some(message) => {
                debug!("接收 IPC 消息: {} -> {}", message.sender, message.receiver);

                // 验证消息
                if let Err(e) = self.validate_message(&message) {
                    error!("接收消息验证失败: {}", e);
                    return Err(e);
                }

                // 验证签名
                if let Err(e) = self.verify_signature(&message) {
                    error!("签名验证失败: {}", e);
                    return Err(e);
                }

                debug!("IPC 消息接收成功: {}", message.id);
                Ok(message)
            }
            None => Err("通道已关闭".to_string()),
        }
    }

    /// 验证消息
    fn validate_message(&self, message: &IpcMessage) -> Result<(), String> {
        // 检查发送者和接收者
        if message.sender != self.sender_id {
            return Err(format!(
                "发送者不匹配: 期望 {}, 实际 {}",
                self.sender_id, message.sender
            ));
        }

        if message.receiver != self.receiver_id {
            return Err(format!(
                "接收者不匹配: 期望 {}, 实际 {}",
                self.receiver_id, message.receiver
            ));
        }

        // 验证能力令牌
        if let Some(ref token) = message.capability_token {
            if !self
                .security
                .verify_capability(token, &message.receiver, &Permission::Write)
            {
                return Err("能力令牌验证失败".to_string());
            }
        }

        Ok(())
    }

    /// 签名消息
    fn sign_message(&self, message: &IpcMessage) -> Result<String, String> {
        let message_bytes = self.serialize_message(message)?;
        let signature = self.key_pair.sign(&message_bytes);
        Ok(base64::encode(signature.as_ref()))
    }

    /// 验证签名
    fn verify_signature(&self, message: &IpcMessage) -> Result<(), String> {
        let signature = match &message.signature {
            Some(sig) => sig,
            None => return Err("消息缺少签名".to_string()),
        };

        let message_bytes = self.serialize_message(message)?;
        let signature_bytes = base64::decode(signature)?;

        // 这里应该使用发送者的公钥验证签名
        // 简化实现，实际需要从密钥管理器获取发送者的公钥
        let public_key = UnparsedPublicKey::new(&ED25519, self.key_pair.public_key().as_ref());

        if public_key.verify(&message_bytes, &signature_bytes).is_err() {
            return Err("签名验证失败".to_string());
        }

        Ok(())
    }

    /// 序列化消息
    fn serialize_message(&self, message: &IpcMessage) -> Result<Vec<u8>, String> {
        serde_json::to_vec(message).map_err(|e| format!("序列化消息失败: {}", e))
    }

    /// 获取通道 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取发送者 ID
    pub fn sender_id(&self) -> &str {
        &self.sender_id
    }

    /// 获取接收者 ID
    pub fn receiver_id(&self) -> &str {
        &self.receiver_id
    }
}

impl IpcManager {
    /// 创建新的 IPC 管理器
    pub fn new(security: Arc<SecurityFramework>) -> Self {
        debug!("正在创建 IPC 管理器...");

        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            security,
            state: Arc::new(Mutex::new(ManagerState::Initialized)),
        }
    }

    /// 生成密钥对
    pub fn generate_key_pair() -> Result<Arc<Ed25519KeyPair>, String> {
        let rng = ring::rand::SystemRandom::new();
        let mut seed = [0u8; 32];
        rng.fill(&mut seed)
            .map_err(|e| format!("生成随机种子失败: {}", e))?;
        let key_pair = Ed25519KeyPair::from_seed_unchecked(&seed)?;
        Ok(Arc::new(key_pair))
    }

    /// 运行 IPC 管理器
    pub fn run(&self) {
        let mut state = self.state.lock().unwrap();
        if !matches!(*state, ManagerState::Initialized) {
            warn!("IPC 管理器已处于运行状态，忽略重复运行请求");
            return;
        }

        *state = ManagerState::Running;
        drop(state);

        info!("IPC 管理器开始运行");
    }

    /// 关闭 IPC 管理器
    pub fn shutdown(&self) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, ManagerState::Shutdown) {
            debug!("IPC 管理器已关闭，忽略重复关闭请求");
            return;
        }

        *state = ManagerState::ShuttingDown;
        drop(state);

        info!("正在关闭 IPC 管理器...");

        // 关闭所有通道
        let mut channels = self.channels.lock().unwrap();
        channels.clear();

        *self.state.lock().unwrap() = ManagerState::Shutdown;
        info!("IPC 管理器已关闭");
    }

    /// 创建 IPC 通道
    pub fn create_channel(
        &self,
        id: String,
        sender_id: String,
        receiver_id: String,
        key_pair: Arc<Ed25519KeyPair>,
    ) -> Result<Arc<IpcChannel>, String> {
        debug!("正在创建 IPC 通道: {}", id);

        let channel = Arc::new(IpcChannel::new(
            id.clone(),
            sender_id,
            receiver_id,
            Arc::clone(&self.security),
            key_pair,
        ));

        // 添加到通道列表
        let mut channels = self.channels.lock().unwrap();
        if channels.contains_key(&id) {
            return Err(format!("IPC 通道 {} 已存在", id));
        }
        channels.insert(id.clone(), channel.clone());

        info!("IPC 通道创建成功: {}", id);
        Ok(channel)
    }

    /// 获取 IPC 通道
    pub fn get_channel(&self, id: &str) -> Option<Arc<IpcChannel>> {
        let channels = self.channels.lock().unwrap();
        channels.get(id).cloned()
    }

    /// 删除 IPC 通道
    pub fn remove_channel(&self, id: &str) -> Result<(), String> {
        debug!("正在删除 IPC 通道: {}", id);

        let mut channels = self.channels.lock().unwrap();
        if channels.remove(id).is_some() {
            info!("IPC 通道删除成功: {}", id);
            Ok(())
        } else {
            Err(format!("IPC 通道 {} 不存在", id))
        }
    }
}

// 简化的 base64 编码/解码实现
mod base64 {
    pub fn encode(bytes: &[u8]) -> String {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::with_capacity((bytes.len() * 4 + 2) / 3);

        for chunk in bytes.chunks(3) {
            let a = chunk[0] as u32;
            let b = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let c = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

            let triplet = (a << 16) | (b << 8) | c;

            result.push(chars[((triplet >> 18) & 0x3F) as usize] as char);
            result.push(chars[((triplet >> 12) & 0x3F) as usize] as char);
            result.push(chars[((triplet >> 6) & 0x3F) as usize] as char);
            result.push(chars[(triplet & 0x3F) as usize] as char);
        }

        // 添加填充
        let padding = 4 - (result.len() % 4);
        if padding < 4 {
            for _ in 0..padding {
                result.pop();
                result.push('=');
            }
        }

        result
    }

    pub fn decode(input: &str) -> Result<Vec<u8>, String> {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = Vec::with_capacity(input.len() * 3 / 4);

        let input = input.trim_end_matches('=');

        for chunk in input.as_bytes().chunks(4) {
            let a = chars.iter().position(|&c| c == chunk[0]).unwrap() as u32;
            let b = if chunk.len() > 1 {
                chars.iter().position(|&c| c == chunk[1]).unwrap() as u32
            } else {
                0
            };
            let c = if chunk.len() > 2 {
                chars.iter().position(|&c| c == chunk[2]).unwrap() as u32
            } else {
                0
            };
            let d = if chunk.len() > 3 {
                chars.iter().position(|&c| c == chunk[3]).unwrap() as u32
            } else {
                0
            };

            let triplet = (a << 18) | (b << 12) | (c << 6) | d;

            result.push((triplet >> 16) as u8);
            if chunk.len() > 2 {
                result.push((triplet >> 8) as u8);
            }
            if chunk.len() > 3 {
                result.push(triplet as u8);
            }
        }

        Ok(result)
    }
}
