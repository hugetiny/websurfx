# WebSurfX CI/CD 工作流

## 📁 工作流文件

| 文件 | 用途 | 触发条件 |
|------|------|----------|
| `ci.yml` | 构建、测试、代码质量检查 | Push, PR, 手动 |
| `release.yml` | 跨平台构建和发布 | 标签 `v*` 或手动 |

## 🚀 本地测试

### 方式一：PowerShell 脚本 (推荐，无需 Docker)

```powershell
cd src-tauri/websurfx
.\test-local.ps1
```

这会运行:
- ✅ Rust 工具链检查
- ✅ 代码格式检查 (cargo fmt)
- ✅ Clippy 检查
- ✅ Debug 构建
- ✅ 单元测试
- ✅ Release 构建

### 方式二：Act + Docker

需要先安装 Docker Desktop:

```powershell
# 1. 安装 Docker Desktop (二选一)
winget install Docker.DockerDesktop
# 或从 https://www.docker.com/products/docker-desktop 下载

# 2. 启动 Docker Desktop 并等待就绪

# 3. 运行 act 测试
cd src-tauri/websurfx
act -j build                    # 只运行 build job
act -j build -j api-test        # 运行 build 和 api-test
act push                        # 模拟 push 事件
```

### Docker vs Podman 对比

| 特性 | Docker Desktop | Podman |
|------|----------------|--------|
| Act 兼容性 | ✅ 原生支持 | ⚠️ 需额外配置 |
| Windows 支持 | ✅ 开箱即用 | ❌ 需要 WSL2 |
| 资源占用 | ~2GB RAM | 较低 |
| 推荐程度 | ⭐⭐⭐ | ⭐⭐ |

**推荐**: Windows 用户使用 Docker Desktop

## 🔧 CI 工作流 Jobs

| Job | 说明 | 平台 |
|-----|------|------|
| `build` | 核心构建和测试 | Linux |
| `api-test` | API 集成测试 | Linux |
| `cross-build-linux` | Linux 构建 | Linux |
| `cross-build-windows` | Windows 构建 | Windows |
| `cross-build-macos` | macOS 构建 | macOS |

## 🏷️ 发布工作流

### 触发发布

```bash
git tag v1.0.0
git push origin v1.0.0
```

### 构建产物

| 平台 | 文件 |
|------|------|
| Linux x64 | `websurfx-linux-x64.tar.gz` |
| Windows x64 | `websurfx-windows-x64.zip` |
| macOS x64 | `websurfx-macos-x64.tar.gz` |
| macOS ARM | `websurfx-macos-arm64.tar.gz` |

## 📊 状态徽章

```markdown
[![CI](https://github.com/<owner>/websurfx/actions/workflows/ci.yml/badge.svg)](https://github.com/<owner>/websurfx/actions/workflows/ci.yml)
```

## 🐛 故障排除

### Act 报错 "no DOCKER_HOST"
```powershell
# 确保 Docker Desktop 正在运行
docker version
```

### 构建失败
```powershell
# 使用正确的包名
cargo build -p websurfx
```
