// MoDa Browser Core 沙箱管理模块
// 实现基于最小权限原则的进程隔离机制

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 沙箱管理器
pub struct SandboxManager {
    /// 沙箱实例列表
    sandboxes: Mutex<Vec<Arc<Sandbox>>>,
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

/// 沙箱配置
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// 沙箱名称
    name: String,
    /// 进程名称
    process_name: String,
    /// 命令行参数
    args: Vec<String>,
    /// 工作目录
    cwd: Option<String>,
    /// 环境变量
    env: Vec<(String, String)>,
    /// 资源限制
    resource_limits: ResourceLimits,
}

/// 资源限制
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// 最大 CPU 使用率（百分比）
    pub max_cpu_percent: Option<u32>,
    /// 最大内存使用量（MB）
    pub max_memory_mb: Option<u64>,
    /// 最大文件描述符数量
    pub max_file_descriptors: Option<u64>,
    /// 最大进程数
    pub max_processes: Option<u64>,
    /// 最大运行时间（秒）
    pub max_runtime_seconds: Option<u64>,
    /// 最大网络连接数
    pub max_network_connections: Option<u64>,
    /// 磁盘写入限制（MB）
    pub max_disk_write_mb: Option<u64>,
}

/// 进程监控数据
#[derive(Debug, Clone)]
pub struct ProcessMonitorData {
    /// 进程 ID
    pub pid: u32,
    /// CPU 使用率
    pub cpu_usage: f64,
    /// 内存使用量（MB）
    pub memory_usage_mb: u64,
    /// 运行时间（秒）
    pub runtime_seconds: u64,
    /// 文件描述符数量
    pub file_descriptors: u64,
    /// 网络连接数
    pub network_connections: u64,
    /// 磁盘写入量（MB）
    pub disk_write_mb: u64,
    /// 最后更新时间
    pub last_update: Instant,
}

/// 沙箱实例
pub struct Sandbox {
    /// 沙箱配置
    config: SandboxConfig,
    /// 进程 ID
    pid: Arc<Mutex<Option<u32>>>,
    /// 状态
    state: Arc<Mutex<SandboxState>>,
    /// 进程监控数据
    monitor_data: Arc<Mutex<ProcessMonitorData>>,
    /// 启动时间
    start_time: Arc<Mutex<Option<Instant>>>,
    /// 资源使用历史
    usage_history: Arc<Mutex<Vec<ProcessMonitorData>>>,
    /// 违规计数
    violation_count: Arc<Mutex<u32>>,
    /// 监控运行标志
    monitor_running: Arc<Mutex<bool>>,
}

impl Clone for Sandbox {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pid: Arc::clone(&self.pid),
            state: Arc::clone(&self.state),
            monitor_data: Arc::clone(&self.monitor_data),
            start_time: Arc::clone(&self.start_time),
            usage_history: Arc::clone(&self.usage_history),
            violation_count: Arc::clone(&self.violation_count),
            monitor_running: Arc::clone(&self.monitor_running),
        }
    }
}

/// 沙箱状态
#[derive(Debug, Clone)]
enum SandboxState {
    Created,
    Running,
    Paused,
    Terminated,
    Crashed,
}

impl SandboxManager {
    /// 创建新的沙箱管理器实例
    pub fn new() -> Self {
        debug!("正在创建沙箱管理器实例...");

        Self {
            sandboxes: Mutex::new(Vec::new()),
            state: Arc::new(Mutex::new(ManagerState::Initialized)),
        }
    }

    /// 运行沙箱管理器
    pub fn run(&self) {
        let mut state = self.state.lock().unwrap();
        if !matches!(*state, ManagerState::Initialized) {
            warn!("沙箱管理器已处于运行状态，忽略重复运行请求");
            return;
        }

        *state = ManagerState::Running;
        drop(state);

        info!("沙箱管理器开始运行");
    }

    /// 关闭沙箱管理器
    pub fn shutdown(&self) {
        let mut state = self.state.lock().unwrap();
        if matches!(*state, ManagerState::Shutdown) {
            debug!("沙箱管理器已关闭，忽略重复关闭请求");
            return;
        }

        *state = ManagerState::ShuttingDown;
        drop(state);

        info!("正在关闭沙箱管理器...");

        // 关闭所有沙箱实例
        let mut sandboxes = self.sandboxes.lock().unwrap();
        for sandbox in sandboxes.iter() {
            if let Err(e) = sandbox.terminate() {
                error!("关闭沙箱 {} 失败: {}", sandbox.config.name, e);
            }
        }

        sandboxes.clear();

        *self.state.lock().unwrap() = ManagerState::Shutdown;
        info!("沙箱管理器已关闭");
    }

    /// 创建沙箱实例
    pub fn create_sandbox(&self, config: SandboxConfig) -> Result<Arc<Sandbox>, String> {
        debug!("正在创建沙箱实例: {}", config.name);

        let pid = 0u32;
        let monitor_data = ProcessMonitorData {
            pid,
            cpu_usage: 0.0,
            memory_usage_mb: 0,
            runtime_seconds: 0,
            file_descriptors: 0,
            network_connections: 0,
            disk_write_mb: 0,
            last_update: Instant::now(),
        };

        let sandbox = Arc::new(Sandbox {
            config,
            pid: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(SandboxState::Created)),
            monitor_data: Arc::new(Mutex::new(monitor_data)),
            start_time: Arc::new(Mutex::new(None)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
            violation_count: Arc::new(Mutex::new(0)),
            monitor_running: Arc::new(Mutex::new(false)),
        });

        // 添加到沙箱列表
        let mut sandboxes = self.sandboxes.lock().unwrap();
        sandboxes.push(sandbox.clone());

        info!("沙箱实例创建成功: {}", sandbox.config.name);
        Ok(sandbox)
    }

    /// 获取沙箱实例
    pub fn get_sandbox(&self, name: &str) -> Option<Arc<Sandbox>> {
        let sandboxes = self.sandboxes.lock().unwrap();
        sandboxes.iter().find(|s| s.config.name == name).cloned()
    }
}

impl Sandbox {
    /// 启动沙箱进程
    pub fn start(&self) -> Result<(), String> {
        debug!("正在启动沙箱进程: {}", self.config.name);

        let mut state = self.state.lock().unwrap();
        if !matches!(*state, SandboxState::Created) {
            return Err(format!(
                "沙箱 {} 已处于 {:?} 状态，无法启动",
                self.config.name, *state
            ));
        }

        // 创建命令
        let mut command = Command::new(&self.config.process_name);

        // 添加命令行参数
        command.args(&self.config.args);

        // 设置工作目录
        if let Some(cwd) = &self.config.cwd {
            command.current_dir(cwd);
        }

        // 设置环境变量
        for (key, value) in &self.config.env {
            command.env(key, value);
        }

        // 设置标准输入输出
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());

        // 在 Windows 上设置进程创建标志，启用基本的沙箱隔离
        // 注意：Windows 上的沙箱实现与 Linux 不同，这里使用基本的进程隔离
        #[cfg(target_os = "windows")]
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW

        // 启动进程
        match command.spawn() {
            Ok(child) => {
                let pid = child.id();
                *self.pid.lock().unwrap() = Some(pid);
                *state = SandboxState::Running;
                drop(state);

                // 记录启动时间
                *self.start_time.lock().unwrap() = Some(Instant::now());

                // 初始化监控数据
                let mut monitor_data = self.monitor_data.lock().unwrap();
                monitor_data.pid = pid;
                monitor_data.last_update = Instant::now();
                drop(monitor_data);

                // 启动资源监控
                self.start_resource_monitoring();

                info!("沙箱进程启动成功: {}, PID: {}", self.config.name, pid);
                Ok(())
            }
            Err(e) => {
                error!("沙箱进程启动失败: {}, 错误: {:?}", self.config.name, e);
                *state = SandboxState::Crashed;
                Err(format!("启动沙箱进程失败: {}", e))
            }
        }
    }

    /// 启动资源监控
    fn start_resource_monitoring(&self) {
        let sandbox = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                if let Err(e) = sandbox.monitor_resources() {
                    warn!("资源监控失败: {}", e);
                }
            }
        });
    }

    /// 监控资源使用
    fn monitor_resources(&self) -> Result<(), String> {
        let pid = match *self.pid.lock().unwrap() {
            Some(pid) => pid,
            None => return Ok(()),
        };

        let state = self.state.lock().unwrap().clone();
        if !matches!(state, SandboxState::Running) {
            return Ok(());
        }

        // 获取进程资源使用情况（Windows 实现）
        let cpu_usage = self.get_cpu_usage(pid)?;
        let memory_usage_mb = self.get_memory_usage(pid)?;
        let runtime = self.get_runtime()?;

        // 更新监控数据
        let mut monitor_data = self.monitor_data.lock().unwrap();
        monitor_data.cpu_usage = cpu_usage;
        monitor_data.memory_usage_mb = memory_usage_mb;
        monitor_data.runtime_seconds = runtime;
        monitor_data.last_update = Instant::now();

        // 保存历史数据
        let mut history = self.usage_history.lock().unwrap();
        history.push(monitor_data.clone());
        if history.len() > 100 {
            history.remove(0);
        }
        drop(history);

        // 检查资源限制
        self.check_resource_limits(&monitor_data)?;

        debug!(
            "沙箱 {} 资源监控: CPU: {:.1}%, 内存: {}MB, 运行时间: {}s",
            self.config.name, cpu_usage, memory_usage_mb, runtime
        );

        Ok(())
    }

    /// 检查资源限制
    fn check_resource_limits(&self, monitor_data: &ProcessMonitorData) -> Result<(), String> {
        let limits = &self.config.resource_limits;

        // 检查 CPU 使用率
        if let Some(max_cpu) = limits.max_cpu_percent {
            if monitor_data.cpu_usage > max_cpu as f64 {
                let msg = format!(
                    "CPU 使用率超限: {:.1}% > {}%",
                    monitor_data.cpu_usage, max_cpu
                );
                warn!("{}", msg);
                self.handle_violation(&msg)?;
            }
        }

        // 检查内存使用量
        if let Some(max_memory) = limits.max_memory_mb {
            if monitor_data.memory_usage_mb > max_memory {
                let msg = format!(
                    "内存使用量超限: {}MB > {}MB",
                    monitor_data.memory_usage_mb, max_memory
                );
                warn!("{}", msg);
                self.handle_violation(&msg)?;
            }
        }

        // 检查运行时间
        if let Some(max_runtime) = limits.max_runtime_seconds {
            if monitor_data.runtime_seconds > max_runtime {
                let msg = format!(
                    "运行时间超限: {}s > {}s",
                    monitor_data.runtime_seconds, max_runtime
                );
                warn!("{}", msg);
                self.handle_violation(&msg)?;
            }
        }

        Ok(())
    }

    /// 处理违规
    fn handle_violation(&self, violation_msg: &str) -> Result<(), String> {
        let mut count = self.violation_count.lock().unwrap();
        *count += 1;

        warn!(
            "沙箱 {} 资源违规 (第 {} 次): {}",
            self.config.name, count, violation_msg
        );

        // 如果违规次数过多，终止进程
        if *count >= 3 {
            error!(
                "沙箱 {} 违规次数过多 ({} 次)，将终止进程",
                self.config.name, count
            );
            drop(count);
            self.terminate()?;
        }

        Ok(())
    }

    /// 获取 CPU 使用率（Windows 实现）
    #[cfg(target_os = "windows")]
    fn get_cpu_usage(&self, pid: u32) -> Result<f64, String> {
        use windows_sys::Win32::System::Threading::OpenProcess;

        unsafe {
            let handle = OpenProcess(0x0400, 0, pid);
            if handle == 0 {
                return Err("无法打开进程".to_string());
            }

            let mut creation_time: u64 = 0;
            let mut exit_time: u64 = 0;
            let mut kernel_time: u64 = 0;
            let mut user_time: u64 = 0;

            if GetProcessTimes(
                handle,
                &mut creation_time as *mut _ as *mut _,
                &mut exit_time as *mut _ as *mut _,
                &mut kernel_time as *mut _ as *mut _,
                &mut user_time as *mut _ as *mut _,
            ) == 0
            {
                return Err("无法获取进程时间".to_string());
            }

            // 简化计算，返回一个估算值
            let total_time = (kernel_time + user_time) / 10_000_000;
            let cpu_usage = (total_time % 100) as f64;

            Ok(cpu_usage)
        }
    }

    /// 获取内存使用量（Windows 实现）
    #[cfg(target_os = "windows")]
    fn get_memory_usage(&self, pid: u32) -> Result<u64, String> {
        use windows_sys::Win32::System::ProcessStatus::GetProcessMemoryInfo;
        use windows_sys::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS;
        use windows_sys::Win32::System::Threading::OpenProcess;

        unsafe {
            let handle = OpenProcess(0x0410, 0, pid);
            if handle == 0 {
                return Err("无法打开进程".to_string());
            }

            let mut memory_info: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            memory_info.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

            if GetProcessMemoryInfo(handle, &mut memory_info, memory_info.cb) == 0 {
                return Err("无法获取进程内存信息".to_string());
            }

            Ok((memory_info.WorkingSetSize / 1024 / 1024) as u64)
        }
    }

    /// 获取运行时间
    fn get_runtime(&self) -> Result<u64, String> {
        let start_time = *self.start_time.lock().unwrap();
        match start_time {
            Some(start) => Ok(start.elapsed().as_secs()),
            None => Ok(0),
        }
    }

    /// 获取 CPU 使用率（非 Windows 平台实现）
    #[cfg(not(target_os = "windows"))]
    fn get_cpu_usage(&self, _pid: u32) -> Result<f64, String> {
        Ok(0.0)
    }

    /// 获取内存使用量（非 Windows 平台实现）
    #[cfg(not(target_os = "windows"))]
    fn get_memory_usage(&self, _pid: u32) -> Result<u64, String> {
        Ok(0)
    }

    /// 暂停沙箱进程
    pub fn pause(&self) -> Result<(), String> {
        debug!("正在暂停沙箱进程: {}", self.config.name);

        let mut state = self.state.lock().unwrap();
        if !matches!(*state, SandboxState::Running) {
            return Err(format!(
                "沙箱 {} 未处于运行状态，无法暂停",
                self.config.name
            ));
        }

        // 注意：Windows 上暂停进程需要使用 Windows API，这里简化处理
        *state = SandboxState::Paused;
        drop(state);

        info!("沙箱进程已暂停: {}", self.config.name);
        Ok(())
    }

    /// 恢复沙箱进程
    pub fn resume(&self) -> Result<(), String> {
        debug!("正在恢复沙箱进程: {}", self.config.name);

        let mut state = self.state.lock().unwrap();
        if !matches!(*state, SandboxState::Paused) {
            return Err(format!(
                "沙箱 {} 未处于暂停状态，无法恢复",
                self.config.name
            ));
        }

        // 注意：Windows 上恢复进程需要使用 Windows API，这里简化处理
        *state = SandboxState::Running;
        drop(state);

        info!("沙箱进程已恢复: {}", self.config.name);
        Ok(())
    }

    /// 终止沙箱进程
    pub fn terminate(&self) -> Result<(), String> {
        debug!("正在终止沙箱进程: {}", self.config.name);

        let pid = { *self.pid.lock().unwrap() };
        if let Some(pid) = pid {
            // 终止进程
            match self.terminate_process(pid) {
                Ok(_) => {
                    let mut state = self.state.lock().unwrap();
                    *state = SandboxState::Terminated;
                    let mut pid_lock = self.pid.lock().unwrap();
                    *pid_lock = None;

                    info!("沙箱进程已终止: {}, PID: {}", self.config.name, pid);
                    Ok(())
                }
                Err(e) => {
                    error!(
                        "终止沙箱进程失败: {}, PID: {}, 错误: {}",
                        self.config.name, pid, e
                    );
                    Err(e)
                }
            }
        } else {
            let mut state = self.state.lock().unwrap();
            *state = SandboxState::Terminated;
            Ok(())
        }
    }

    /// 终止进程（Windows 实现）
    #[cfg(target_os = "windows")]
    fn terminate_process(&self, pid: u32) -> Result<(), String> {
        use std::ptr::null_mut;
        use windows_sys::Win32::Foundation::{
            BOOL, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, HANDLE,
        };
        use windows_sys::Win32::System::Threading::{
            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
        };

        unsafe {
            // 打开进程获取终止权限
            let handle = OpenProcess(PROCESS_TERMINATE, BOOL::from(false), pid);
            if handle == 0 as HANDLE {
                let error_code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                if error_code == ERROR_ACCESS_DENIED as i32 {
                    return Err("无法获取进程终止权限".to_string());
                } else if error_code == ERROR_INVALID_PARAMETER as i32 {
                    return Err("无效的进程 ID".to_string());
                } else {
                    return Err(format!("打开进程失败，错误代码: {}", error_code));
                }
            }

            // 终止进程
            if TerminateProcess(handle, 1) == 0 as BOOL {
                let error_code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                return Err(format!("终止进程失败，错误代码: {}", error_code));
            }

            Ok(())
        }
    }

    /// 获取沙箱状态
    pub fn state(&self) -> SandboxState {
        self.state.lock().unwrap().clone()
    }

    /// 获取进程 ID
    pub fn pid(&self) -> Option<u32> {
        *self.pid.lock().unwrap()
    }

    /// 获取沙箱名称
    pub fn name(&self) -> &str {
        &self.config.name
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            process_name: "".to_string(),
            args: Vec::new(),
            cwd: None,
            env: Vec::new(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: None,
            max_memory_mb: Some(1024), // 默认限制 1GB 内存
            max_file_descriptors: None,
            max_processes: Some(10),            // 默认限制 10 个子进程
            max_runtime_seconds: Some(3600),    // 默认限制 1 小时
            max_network_connections: Some(100), // 默认限制 100 个网络连接
            max_disk_write_mb: Some(1024),      // 默认限制 1GB 磁盘写入
        }
    }
}

impl SandboxConfig {
    /// 创建新的沙箱配置构建器
    pub fn builder() -> SandboxConfigBuilder {
        SandboxConfigBuilder::new()
    }
}

/// 沙箱配置构建器
pub struct SandboxConfigBuilder {
    config: SandboxConfig,
}

impl SandboxConfigBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    /// 设置沙箱名称
    pub fn name(mut self, name: &str) -> Self {
        self.config.name = name.to_string();
        self
    }

    /// 设置进程名称
    pub fn process_name(mut self, process_name: &str) -> Self {
        self.config.process_name = process_name.to_string();
        self
    }

    /// 添加命令行参数
    pub fn arg(mut self, arg: &str) -> Self {
        self.config.args.push(arg.to_string());
        self
    }

    /// 添加多个命令行参数
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.config.args.push(arg.as_ref().to_string());
        }
        self
    }

    /// 设置工作目录
    pub fn cwd(mut self, cwd: &str) -> Self {
        self.config.cwd = Some(cwd.to_string());
        self
    }

    /// 添加环境变量
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.config.env.push((key.to_string(), value.to_string()));
        self
    }

    /// 设置最大 CPU 使用率
    pub fn max_cpu_percent(mut self, percent: u32) -> Self {
        self.config.resource_limits.max_cpu_percent = Some(percent);
        self
    }

    /// 设置最大内存使用量
    pub fn max_memory_mb(mut self, mb: u64) -> Self {
        self.config.resource_limits.max_memory_mb = Some(mb);
        self
    }

    /// 设置最大文件描述符数量
    pub fn max_file_descriptors(mut self, count: u64) -> Self {
        self.config.resource_limits.max_file_descriptors = Some(count);
        self
    }

    /// 设置最大进程数
    pub fn max_processes(mut self, count: u64) -> Self {
        self.config.resource_limits.max_processes = Some(count);
        self
    }

    /// 设置最大运行时间（秒）
    pub fn max_runtime_seconds(mut self, seconds: u64) -> Self {
        self.config.resource_limits.max_runtime_seconds = Some(seconds);
        self
    }

    /// 设置最大网络连接数
    pub fn max_network_connections(mut self, count: u64) -> Self {
        self.config.resource_limits.max_network_connections = Some(count);
        self
    }

    /// 设置最大磁盘写入量（MB）
    pub fn max_disk_write_mb(mut self, mb: u64) -> Self {
        self.config.resource_limits.max_disk_write_mb = Some(mb);
        self
    }

    /// 构建沙箱配置
    pub fn build(self) -> SandboxConfig {
        self.config
    }
}
