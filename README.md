# MoDa Browser Core -一个基于最小权限原则的现代模块化浏览器引擎

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://img.shields.io/github/actions/workflow/status/MoDa-Browser/MoDa-Core/ci.yml?branch=main)](https://github.com/MoDa-Browser/MoDa-Core/actions)
[![Contributors](https://img.shields.io/github/contributors/MoDa-Browser/MoDa-Core)](https://github.com/MoDa-Browser/MoDa-Core/graphs/contributors)

**MoDa Browser Core** 是一个从零设计的现代浏览器引擎，致力于通过**最小权限架构、进程级隔离和内存安全语言**构建下一代安全的网络平台。我们相信浏览器的安全性应内建于架构，而非事后附加。

> **当前状态**：MoDa 正处于**概念验证与核心开发阶段**。我们正在构建基础架构，欢迎开发者、安全研究人员和浏览器爱好者参与这一开源项目。

## ✨ 核心理念

- **安全即架构**：每个组件默认运行在独立沙箱中，遵循最小权限原则
- **内存安全基础**：关键路径使用 Rust 实现，C++ 组件采用严格的安全子集
- **模块化设计**：清晰的组件边界与接口定义，支持独立开发与测试
- **面向未来**：专注现代 Web 标准，为未来十年的网络应用提供安全基础

## 🏗 架构概览

```
┌─────────────────────────────────────────┐
│           用户界面 / 扩展层              │
├─────────────────────────────────────────┤
│        进程间通信 (能力验证)             │
├──────┬──────┬──────┬──────┬─────────────┤
│渲染  │网络  │存储  │脚本  │ 媒体/其他   │
│进程  │进程  │进程  │进程  │ 组件        │
├──────┴──────┴──────┴──────┴─────────────┤
│     系统安全层 (沙箱/命名空间)           │
└─────────────────────────────────────────┘
```

## 🚀 快速开始

### 系统要求
- **操作系统**：Linux 5.10+ (其他平台规划中)
- **内存**：4GB RAM
- **磁盘空间**：10GB
- **编译器**：Clang 16+ 或 GCC 13+
- **Rust**：1.75+

### 从源码构建
```bash
# 1. 克隆仓库
git clone https://github.com/MoDa-Browser/MoDa-Core.git
cd MoDa-Core

# 2. 安装依赖
./scripts/setup.sh  # 支持 Ubuntu/Debian/Fedora

# 3. 配置与构建
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo \
      -DMODA_BUILD_TESTS=ON \
      -DMODA_BUILD_EXAMPLES=ON ..
make -j$(nproc)

# 4. 运行示例
./examples/minimal-browser
```

### 使用 Docker 开发
```bash
# 获取开发环境镜像
docker pull modabrowser/dev:latest

# 运行开发容器
docker run -it --rm -v $(pwd):/workspace modabrowser/dev:latest
cd /workspace && ./scripts/build.sh
```

## 📁 项目结构

```
MoDa-Core/
├── src/
│   ├── security/          # 安全框架 (Rust)
│   ├── sandbox/           # 沙箱管理
│   ├── ipc/               # 进程间通信
│   ├── render/            # 渲染引擎 (C++)
│   ├── network/           # 网络栈 (Rust)
│   ├── storage/           # 安全存储
│   └── platform/          # 平台抽象层
├── include/               # 公共头文件
├── examples/              # 示例程序
├── tests/                 # 测试套件
├── docs/                  # 文档
└── tools/                 # 开发工具
```

## 🛡 安全特性

### 多层防御架构
1. **编译时保护**
   - 所有 C++ 代码启用 `-fsanitize=address,undefined`
   - Rust 强制内存安全保证
   - 自定义静态分析规则

2. **运行时隔离**
   - 每个组件在独立 Linux 命名空间中运行
   - 通过 seccomp BPF 限制系统调用
   - 能力(Capabilities)最小化授予

3. **进程间通信安全**
   ```rust
   // 所有 IPC 消息必须携带能力证明
   struct IPCMessage {
       capability: CapabilityToken,
       payload: SecureBuffer,
       signature: MessageSignature,
   }
   ```

### 安全开发实践
- 所有代码必须通过静态分析检查
- 模糊测试覆盖所有解析器组件
- 定期第三方安全审计
- 漏洞奖励计划（规划中）

## 🤝 参与贡献

MoDa 是一个社区驱动的开源项目，我们欢迎各种形式的贡献。

### 贡献流程
1. **发现议题**：查看 https://github.com/MoDa-Browser/MoDa-Core/issues?q=is:open+is:issue+label:"good+first+issue" 或 https://github.com/orgs/MoDa-Browser/projects/1
2. **讨论设计**：在相关 Issue 或 Discussion 中提出你的方案
3. **提交代码**：
   ```bash
   # Fork 并创建分支
   git checkout -b feature/your-feature
   
   # 提交更改
   git commit -s -m "feat: 描述你的变更"
   
   # 推送并创建 PR
   git push origin feature/your-feature
   ```
4. **代码审查**：至少需要一名核心维护者批准
5. **合并部署**：通过所有检查后合并到主分支

### 开发指南
- 代码风格：遵循项目中的 `.clang-format` 和 `rustfmt.toml`
- 提交信息：使用 https://www.conventionalcommits.org/
- 测试要求：新功能必须包含单元测试
- 文档更新：API 变更需同步更新文档

### 社区角色
- **贡献者**：提交过被合并的 PR
- **维护者**：拥有特定模块的审查与合并权限
- **核心团队**：负责项目方向与架构决策

## 📖 文档

- **docs/architecture/overview.md** - 详细技术架构
- **docs/development/getting-started.md** - 开发环境设置与工作流
- **docs/api/** - 公共 API 文档
- **docs/security/model.md** - 安全设计与威胁模型
- **docs/performance/** - 优化与基准测试

## 🔧 开发工具

### 预提交检查
```bash
# 安装预提交钩子
pip install pre-commit
pre-commit install

# 手动运行所有检查
./scripts/run-checks.sh
```

### 调试与测试
```bash
# 运行完整测试套件
./scripts/test-all.sh

# 模糊测试特定组件
./scripts/fuzz-html-parser.sh --timeout=3600

# 内存泄漏检查
valgrind --leak-check=full ./tests/sandbox-tests
```

### 性能分析
```bash
# CPU 性能分析
perf record ./examples/minimal-browser
perf report

# 内存使用分析
heaptrack ./tests/render-benchmark
```

## 📊 项目状态

### 组件完成度
| 组件 | 状态 | 完成度 |
|------|------|--------|
| 安全框架 | 🟡 开发中 | 0% |
| 进程管理 | 🟡 开发中 | 0% |
| IPC 系统 | 🟡 开发中 | 0% |
| 网络栈 | 🟡 开发中 | 0% |
| 渲染引擎 | 🟡 原型阶段 | 0% |
| JavaScript 引擎 | 🔴 规划中 | 0% |

### 近期里程碑
- **v0.1.0** (规划中): 基础进程框架与安全模型
- **v0.2.0** (规划中): 最小可运行浏览器 (HTML/CSS)
- **v0.3.0** (规划中): 网络协议支持 (HTTP/1.1, TLS)
- **v0.4.0** (规划中): JavaScript 引擎基础

## 🌍 社区

### 交流渠道
- **GitHub Discussions**: 功能讨论与设计决策
- **飞书群**: 实时交流与协作
- **邮件列表**: 公告与深度讨论
- **社区会议**: 月度开发者会议 (线上)

### 相关项目 (计划中)
- https://github.com/Ink-dark/MoDa-Browser-Core/web-platform-tests - Web 平台测试套件
- https://github.com/Ink-dark/MoDa-Browser-Core/devtools - 开发者工具
- https://github.com/Ink-dark/MoDa-Browser-Core/bindings - 语言绑定 (Python, Node.js 等)

## 📄 许可证

MoDa Browser Core 使用 **Apache License 2.0** 开源。

```
Copyright 2024-2026 MoDa Browser Core Community

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

第三方依赖的许可证信息详见 LICENSE-THIRD-PARTY。

## 🤗 致谢

感谢所有为项目做出贡献的开发者，特别感谢：

- **Ink-dark (墨染柒DarkSeven)** - 项目发起人与架构设计
- 所有代码贡献者 (https://github.com/Ink-dark/MoDa-Browser-Core/graphs/contributors)
- 安全研究人员与审计人员
- 开源浏览器社区 (Chromium, Firefox, Servo) 的启发与贡献

## 🆘 支持与联系

- **问题反馈**: https://github.com/Ink-dark/MoDa-Browser-Core/issues
- **安全漏洞**: SECURITY.md
- **合作咨询**: contact@modabrowser.hallochat.cn
- **项目官网**: https://modabrowser.hallochat.cn (规划中)
- **加入我们**: 想加入我们？联系墨染柒DarkSeven（QQ：3773704332，验证问题全部填写MoDaBrowser）

---

> **我们的使命**: 构建一个安全、高效、开放的浏览器核心，为下一代网络应用奠定可信基础。无论你是开发者、研究者还是爱好者，我们都欢迎你加入这个旅程。

*MoDa - 现代浏览，安全守护。*