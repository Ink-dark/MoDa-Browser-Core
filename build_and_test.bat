@echo off
REM MoDa Browser Core 构建和测试脚本 (Windows)

echo =========================================
echo MoDa Browser Core 构建和测试脚本
echo =========================================

REM 检查 Rust 工具链
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo 错误: 未找到 cargo 命令，请先安装 Rust 工具链
    echo 访问 https://rustup.rs/ 安装 Rust
    exit /b 1
)

echo Rust 版本:
rustc --version
echo Cargo 版本:
cargo --version
echo.

REM 清理之前的构建
echo 清理之前的构建...
cargo clean

REM 检查代码
echo 检查代码...
cargo check --all-features
if %errorlevel% neq 0 (
    echo 错误: 代码检查失败
    exit /b 1
)

REM 运行单元测试
echo 运行单元测试...
cargo test --lib --all-features
if %errorlevel% neq 0 (
    echo 错误: 单元测试失败
    exit /b 1
)

REM 运行文档测试
echo 运行文档测试...
cargo test --doc
if %errorlevel% neq 0 (
    echo 错误: 文档测试失败
    exit /b 1
)

REM 生成文档
echo 生成文档...
cargo doc --no-deps
if %errorlevel% neq 0 (
    echo 错误: 文档生成失败
    exit /b 1
)

REM 代码格式检查
echo 检查代码格式...
cargo fmt -- --check
if %errorlevel% neq 0 (
    echo 警告: 代码格式不符合规范，运行 cargo fmt 修复
)

REM 代码质量检查
echo 运行 Clippy 检查...
cargo clippy --all-features -- -D warnings
if %errorlevel% neq 0 (
    echo 错误: Clippy 检查失败
    exit /b 1
)

REM 构建发布版本
echo 构建发布版本...
cargo build --release --all-features
if %errorlevel% neq 0 (
    echo 错误: 构建失败
    exit /b 1
)

echo.
echo =========================================
echo 构建和测试完成！
echo =========================================
echo.
echo 生成的文件位置:
echo   - 可执行文件: target\release\moda-browser.exe
echo   - 库文件: target\release\moda_core.lib
echo   - 文档: target\doc\
echo.
echo 运行程序:
echo   cargo run --release
echo.

exit /b 0
