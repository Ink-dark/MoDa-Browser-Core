// MoDa Browser Core - IPC Message Module
// 定义消息格式和协议

use std::time::SystemTime;

/// 消息类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// 控制消息
    Control(ControlMessage),
    /// 数据消息
    Data(DataMessage),
    /// 事件消息
    Event(EventMessage),
}

/// 控制消息枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlMessage {
    /// 连接请求
    Connect,
    /// 连接确认
    ConnectAck,
    /// 断开连接
    Disconnect,
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// 数据消息结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMessage {
    pub data_type: String,
    pub data: Vec<u8>,
}

/// 事件消息结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventMessage {
    pub event_type: String,
    pub event_data: Vec<u8>,
}

/// 消息结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// 消息ID
    pub id: u64,
    /// 发送时间
    pub timestamp: SystemTime,
    /// 消息类型
    pub msg_type: MessageType,
    /// 优先级
    pub priority: Priority,
}

/// 消息优先级枚举
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// 低优先级
    Low,
    /// 中优先级
    Medium,
    /// 高优先级
    High,
    /// 最高优先级（紧急）
    Critical,
}

impl Message {
    /// 创建新的控制消息
    pub fn new_control(id: u64, control_msg: ControlMessage, priority: Priority) -> Self {
        Self {
            id,
            timestamp: SystemTime::now(),
            msg_type: MessageType::Control(control_msg),
            priority,
        }
    }

    /// 创建新的数据消息
    pub fn new_data(id: u64, data_type: String, data: Vec<u8>, priority: Priority) -> Self {
        Self {
            id,
            timestamp: SystemTime::now(),
            msg_type: MessageType::Data(DataMessage { data_type, data }),
            priority,
        }
    }

    /// 创建新的事件消息
    pub fn new_event(id: u64, event_type: String, event_data: Vec<u8>, priority: Priority) -> Self {
        Self {
            id,
            timestamp: SystemTime::now(),
            msg_type: MessageType::Event(EventMessage {
                event_type,
                event_data,
            }),
            priority,
        }
    }
}
