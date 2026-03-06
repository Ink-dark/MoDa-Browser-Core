// MoDa Browser Core 核心架构模块
// 实现基于最小权限原则的模块化架构

use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// 核心架构组件
pub struct CoreArchitecture {
    /// 组件状态
    state: Arc<Mutex<ComponentState>>,
    /// 组件列表
    components: Vec<Arc<dyn Component>>,
}

/// 组件状态
enum ComponentState {
    Initialized,
    Running,
    ShuttingDown,
    Shutdown,
}

/// 组件接口
trait Component {
    /// 组件名称
    fn name(&self) -> &'static str;

    /// 初始化组件
    fn init(&self) -> Result<(), String>;

    /// 运行组件
    fn run(&self) -> Result<(), String>;

    /// 关闭组件
    fn shutdown(&self) -> Result<(), String>;
}

impl CoreArchitecture {
    /// 创建新的核心架构实例
    pub fn new() -> Self {
        debug!("正在创建核心架构实例...");

        let mut components: Vec<Arc<dyn Component>> = Vec::new();

        // 可以在这里添加默认组件

        Self {
            state: Arc::new(Mutex::new(ComponentState::Initialized)),
            components,
        }
    }

    /// 运行核心架构
    pub fn run(&self) {
        let mut state = self.state.lock().unwrap();
        if !matches!(*state, ComponentState::Initialized) {
            warn!("核心架构已处于运行状态，忽略重复运行请求");
            return;
        }

        *state = ComponentState::Running;
        drop(state);

        info!("核心架构开始运行");

        // 运行所有组件
        for component in &self.components {
            if let Err(e) = component.run() {
                warn!("组件 {} 运行失败: {}", component.name(), e);
            }
        }
    }

    /// 关闭核心架构
    pub fn shutdown(&self) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, ComponentState::Shutdown) {
            debug!("核心架构已关闭，忽略重复关闭请求");
            return;
        }

        *state = ComponentState::ShuttingDown;
        drop(state);

        info!("正在关闭核心架构...");

        // 关闭所有组件
        for component in &self.components {
            if let Err(e) = component.shutdown() {
                warn!("组件 {} 关闭失败: {}", component.name(), e);
            }
        }

        *self.state.lock().unwrap() = ComponentState::Shutdown;
        info!("核心架构已关闭");
    }

    /// 添加组件
    pub fn add_component(&mut self, component: Arc<dyn Component>) -> Result<(), String> {
        debug!("正在添加组件: {}", component.name());

        // 初始化组件
        component.init()?;

        // 添加到组件列表
        self.components.push(component);

        Ok(())
    }
}

/// 架构配置
pub struct ArchitectureConfig {
    /// 启用调试模式
    pub debug_mode: bool,
    /// 启用性能监控
    pub performance_monitoring: bool,
    /// 组件启动超时时间（毫秒）
    pub component_timeout_ms: u64,
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self {
            debug_mode: true,
            performance_monitoring: true,
            component_timeout_ms: 5000,
        }
    }
}
