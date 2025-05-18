# Yjs 协作编辑服务器

中文 | [English](README.md)

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

基于 Rust、Yrs (Yjs 的 Rust 实现) 和 Volo HTTP 构建的高性能实时协作文档编辑服务器。

## ✨ 功能特性

- 🚀 **实时协作**：多用户同时编辑文档
- 🔄 **基于 CRDT**：无冲突复制数据类型确保一致性
- ⚡ **高性能**：使用 Rust 构建，性能卓越
- 🌐 **WebSocket 支持**：实时双向通信
- 🏗️ **整洁架构**：代码结构清晰，易于维护
- 🔒 **类型安全**：利用 Rust 强大的类型系统

## 📦 依赖

- **volo**：核心 HTTP/RPC 框架
- **volo-http**：路由和中间件的 HTTP 工具
- **tokio**：用于并发的异步运行时
- **futures-util**：异步 futures 的工具库
- **yrs**：Yjs CRDT 协议的 Rust 实现
- **serde**：数据序列化框架，支持 derive
- **sonic-rs**：高性能 JSON 序列化/反序列化
- **once_cell**：支持静态数据的延迟初始化
- **uuid**：生成唯一文档标识符
- **base64**：CRDT 更新载荷的编码/解码
- **tracing**：结构化日志和埋点框架
- **tracing-subscriber**：`tracing` 的订阅者，支持格式化输出和过滤

## 🚧 路线图

- [ ] 基础设施
  - [ ] 缓存
    - [ ] 支持 Redis
    - [ ] 支持多级缓存
  - [ ] 存储
    - [ ] 支持 MySQL
    - [ ] 支持 MongoDB
- [ ] 网络连接
  - [ ] 心跳检测
  - [ ] 限流
- [ ] 日志与监控
  - [ ] 动态调整日志级别
  - [ ] 集成监控采集（Prometheus 等）
- [ ] 部署
  - [ ] Docker 镜像
  - [ ] Kubernetes 部署
- [ ] 测试
  - [ ] 单元测试覆盖
  - [ ] 集成测试
- [ ] 性能优化
  - [ ] 算法优化
    - [ ] 减少不必要的增量更新
    - [ ] 压缩连续小更新
  - [ ] 并发优化
    - [ ] 无锁或细粒度锁
    - [ ] 减少上下文切换
  - [ ] 资源复用

## 🏗️ 系统架构

项目遵循整洁架构原则，关注点分离明确：

- **领域层**：核心业务逻辑与实体

  - `entities/`：核心领域模型
  - `repositories/`：仓储接口
  - `services/`：领域服务

- **应用层**：用例与应用服务

  - `use_cases/`：应用特定业务规则

- **基础设施层**：外部实现

  - `adapters/`：仓储具体实现

- **适配器层**：Web/API 适配器
  - `http/`：HTTP 服务器和路由
  - `websocket/`：WebSocket 处理器

## 🚀 快速开始

### 环境要求

- Rust 1.60+ (通过 [rustup](https://rustup.rs/) 安装)
- Cargo (Rust 包管理器)

### 安装

1. 克隆仓库：

   ```bash
   git clone https://github.com/Wenrh2004/yjs-collaboration-server.git
   cd yjs-collaboration-server
   ```

2. 构建项目：
   ```bash
   cargo build --release
   ```

### 运行服务器

```bash
cargo run --release
```

服务器将启动在 `http://localhost:8080`

## 📚 API 文档

### 接口端点

- `GET /` - 健康检查

  - 返回: `200 OK` 包含服务器状态

- `GET /ws` - 实时协作 WebSocket 端点
  - 协议: 基于 WebSocket 的 Yjs 同步协议

### WebSocket 协议

服务器实现了 Yjs 同步协议。客户端应连接到 WebSocket 端点并遵循协议进行文档同步。

#### 消息格式

```json
{
  "type": "sync|update|sv",
  "data": "...",
  "update": "base64_encoded_update"
}
```

## 🧪 测试

运行测试套件：

```bash
cargo test
```

测试覆盖率：

```bash
cargo tarpaulin --ignore-tests
```

## 🛠️ 开发

### 代码风格

使用 `rustfmt` 进行代码格式化：

```bash
cargo fmt
```

### 代码检查

运行 clippy：

```bash
cargo clippy -- -D warnings
```

### 文档

生成文档：

```bash
cargo doc --open
```

## 🤝 贡献指南

欢迎贡献代码！请随时提交 Pull Request。

1. Fork 本仓库至您的 GitHub 账户。
2. Clone 您 fork 的仓库到本地并添加上游仓库：
   ```bash
   git clone https://github.com/<您的用户名>/yjs-collaboration-server.git
   cd yjs-collaboration-server
   git remote add upstream https://github.com/Wenrh2004/yjs-collaboration-server.git
   ```
3. 创建特性分支：
   ```bash
   git checkout -b feature/amazing-feature
   ```
4. 提交更改：
   ```bash
   git add .
   git commit -m "feat: 添加了很棒的功能"
   ```
5. 从上游仓库同步分支状态:
   ```
   git fetch upstream
   git rebase upstream/master
   ```
6. 在 GitHub 上向上游仓库提交 Pull Request，并在 PR 描述中说明您的改动目的。

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

- [Yjs](https://yjs.dev/) - 用于协作应用的 CRDT 框架
- [Yrs](https://github.com/y-crdt/y-crdt) - Yjs 的 Rust 实现
- [Volo](https://www.cloudwego.io/volo/) - 高性能 HTTP/RPC 框架
