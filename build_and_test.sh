#!/bin/bash
# MoDa Browser Core 构建和测试脚本

set -e

echo "========================================="
echo "MoDa Browser Core 构建和测试脚本"
echo "========================================="

# 检查 Rust 工具链
if ! command -v cargo &> /dev/null; then
    echo "错误: 未找到 cargo 命令，请先安装 Rust 工具链"
    echo "访问 https://rustup.rs/ 安装 Rust"
    exit 1
fi

echo "Rust 版本: $(rustc --version)"
echo "Cargo 版本: $(cargo --version)"
echo ""

# 清理之前的构建
echo "清理之前的构建..."
cargo clean

# 检查代码
echo "检查代码..."
cargo check --all-features

# 运行单元测试
echo "运行单元测试..."
cargo test --lib --all-features

# 运行文档测试
echo "运行文档测试..."
cargo test --doc

# 生成文档
echo "生成文档..."
cargo doc --no-deps

# 代码格式检查
echo "检查代码格式..."
cargo fmt -- --check

# 代码质量检查
echo "运行 Clippy 检查..."
cargo clippy --all-features -- -D warnings

# 构建发布版本
echo "构建发布版本..."
cargo build --release --all-features

echo ""
echo "========================================="
echo "构建和测试完成！"
echo "========================================="
echo ""
echo "生成的文件位置:"
echo "  - 可执行文件: target/release/moda-browser"
echo "  - 库文件: target/release/libmoda_core.*"
echo "  - 文档: target/doc/"
echo ""
echo "运行程序:"
echo "  cargo run --release"
echo ""
