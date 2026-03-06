// MoDa Browser Core 主入口文件
// 基于最小权限原则的现代模块化浏览器引擎

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod core;
mod sandbox;
mod security;

fn main() {
    // 初始化日志系统
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("MoDa Browser Core v0.1.0 启动");
    tracing::info!("基于最小权限原则的现代模块化浏览器引擎");

    // 初始化核心组件
    tracing::debug!("正在初始化核心组件...");

    // 创建核心架构实例
    let core = core::CoreArchitecture::new();
    tracing::debug!("核心架构初始化完成");

    // 初始化安全框架
    let security = security::SecurityFramework::new();
    tracing::debug!("安全框架初始化完成");

    // 初始化沙箱管理机制
    let sandbox = sandbox::SandboxManager::new();
    tracing::debug!("沙箱管理机制初始化完成");

    tracing::info!("所有核心组件初始化完成，准备运行");

    // 运行核心组件
    core.run();
    security.run();
    sandbox.run();

    tracing::info!("MoDa Browser Core 运行中...");

    // 等待退出信号
    let (_tx, rx) = std::sync::mpsc::channel();
    signal_hook::flag::register(signal_hook::consts::SIGINT, rx.clone()).unwrap();
    signal_hook::flag::register(signal_hook::consts::SIGTERM, rx.clone()).unwrap();

    rx.recv().unwrap();

    tracing::info!("收到退出信号，正在关闭 MoDa Browser Core...");

    // 关闭核心组件
    core.shutdown();
    security.shutdown();
    sandbox.shutdown();

    tracing::info!("MoDa Browser Core 已安全关闭");
}
