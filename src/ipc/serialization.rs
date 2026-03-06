// MoDa Browser Core - IPC Serialization Module
// 实现消息序列化和反序列化

use crate::ipc::message::Message;

/// 序列化消息
pub fn serialize(msg: &Message) -> Result<Vec<u8>, String> {
    // 这里使用简单的序列化方式，实际应用中应该使用更高效的序列化库如bincode或protobuf
    let mut serialized = Vec::new();
    
    // 序列化消息ID
    serialized.extend_from_slice(&msg.id.to_le_bytes());
    
    // 序列化时间戳（简化处理，实际应该使用更精确的时间序列化）
    let timestamp_bytes = [0u8; 8]; // 简化处理，实际应该序列化SystemTime
    serialized.extend_from_slice(&timestamp_bytes);
    
    // 序列化消息类型
    match &msg.msg_type {
        message::MessageType::Control(control_msg) => {
            serialized.push(0x00); // 控制消息标记
            match control_msg {
                message::ControlMessage::Connect => serialized.push(0x00),
                message::ControlMessage::ConnectAck => serialized.push(0x01),
                message::ControlMessage::Disconnect => serialized.push(0x02),
                message::ControlMessage::Ping => serialized.push(0x03),
                message::ControlMessage::Pong => serialized.push(0x04),
            }
        },
        message::MessageType::Data(data_msg) => {
            serialized.push(0x01); // 数据消息标记
            // 序列化数据类型长度
            let data_type_len = data_msg.data_type.len() as u32;
            serialized.extend_from_slice(&data_type_len.to_le_bytes());
            // 序列化数据类型
            serialized.extend_from_slice(data_msg.data_type.as_bytes());
            // 序列化数据长度
            let data_len = data_msg.data.len() as u32;
            serialized.extend_from_slice(&data_len.to_le_bytes());
            // 序列化数据
            serialized.extend_from_slice(&data_msg.data);
        },
        message::MessageType::Event(event_msg) => {
            serialized.push(0x02); // 事件消息标记
            // 序列化事件类型长度
            let event_type_len = event_msg.event_type.len() as u32;
            serialized.extend_from_slice(&event_type_len.to_le_bytes());
            // 序列化事件类型
            serialized.extend_from_slice(event_msg.event_type.as_bytes());
            // 序列化事件数据长度
            let event_data_len = event_msg.event_data.len() as u32;
            serialized.extend_from_slice(&event_data_len.to_le_bytes());
            // 序列化事件数据
            serialized.extend_from_slice(&event_msg.event_data);
        },
    }
    
    // 序列化优先级
    match msg.priority {
        message::Priority::Low => serialized.push(0x00),
        message::Priority::Medium => serialized.push(0x01),
        message::Priority::High => serialized.push(0x02),
        message::Priority::Critical => serialized.push(0x03),
    }
    
    Ok(serialized)
}

/// 反序列化消息
pub fn deserialize(data: &[u8]) -> Result<Message, String> {
    // 这里使用简单的反序列化方式，实际应用中应该使用更高效的序列化库如bincode或protobuf
    if data.len() < 11 { // 最小长度检查
        return Err("Invalid message length".to_string());
    }
    
    let mut offset = 0;
    
    // 反序列化消息ID
    let id = u64::from_le_bytes(data[offset..offset+8].try_into().unwrap());
    offset += 8;
    
    // 反序列化时间戳（简化处理）
    offset += 8; // 跳过时间戳
    
    // 反序列化消息类型
    let msg_type = match data[offset] {
        0x00 => {
            offset += 1;
            let control_msg = match data[offset] {
                0x00 => message::ControlMessage::Connect,
                0x01 => message::ControlMessage::ConnectAck,
                0x02 => message::ControlMessage::Disconnect,
                0x03 => message::ControlMessage::Ping,
                0x04 => message::ControlMessage::Pong,
                _ => return Err("Invalid control message type".to_string()),
            };
            offset += 1;
            message::MessageType::Control(control_msg)
        },
        0x01 => {
            offset += 1;
            // 反序列化数据类型长度
            let data_type_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;
            // 反序列化数据类型
            let data_type = String::from_utf8_lossy(&data[offset..offset+data_type_len]).to_string();
            offset += data_type_len;
            // 反序列化数据长度
            let data_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;
            // 反序列化数据
            let data = data[offset..offset+data_len].to_vec();
            offset += data_len;
            
            message::MessageType::Data(message::DataMessage {
                data_type,
                data,
            })
        },
        0x02 => {
            offset += 1;
            // 反序列化事件类型长度
            let event_type_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;
            // 反序列化事件类型
            let event_type = String::from_utf8_lossy(&data[offset..offset+event_type_len]).to_string();
            offset += event_type_len;
            // 反序列化事件数据长度
            let event_data_len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;
            // 反序列化事件数据
            let event_data = data[offset..offset+event_data_len].to_vec();
            offset += event_data_len;
            
            message::MessageType::Event(message::EventMessage {
                event_type,
                event_data,
            })
        },
        _ => return Err("Invalid message type".to_string()),
    };
    
    // 反序列化优先级
    let priority = match data[offset] {
        0x00 => message::Priority::Low,
        0x01 => message::Priority::Medium,
        0x02 => message::Priority::High,
        0x03 => message::Priority::Critical,
        _ => return Err("Invalid priority".to_string()),
    };
    
    Ok(Message {
        id,
        timestamp: std::time::SystemTime::UNIX_EPOCH, // 简化处理，实际应该从序列化数据中恢复
        msg_type,
        priority,
    })
}
