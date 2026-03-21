# Development Guide

## 环境要求

- Node.js 18+
- pnpm (推荐) 或 npm
- Rust 1.70+
- Tauri CLI

## 快速开始

```bash
# 安装依赖
pnpm install

# 启动开发模式
pnpm dev

# 构建生产版本
pnpm tauri build
```

## 配置文件路径

### 生产模式

配置文件统一存放在用户主目录下：

| 平台 | 路径 |
|------|------|
| Windows | `C:\Users\{username}\.config\pranslator\settings.toml` |
| macOS | `~/.config/pranslator/settings.toml` |
| Linux | `~/.config/pranslator/settings.toml` |

### 开发模式

为避免污染正式配置，开发时可设置 `PRANSLATOR_CONFIG_PATH` 环境变量指定临时配置目录：

#### Windows PowerShell

```powershell
$env:PRANSLATOR_CONFIG_PATH = ".\dev-config"
pnpm dev
```

#### Windows CMD

```cmd
set PRANSLATOR_CONFIG_PATH=.\dev-config && pnpm dev
```

#### Linux / macOS

```bash
PRANSLATOR_CONFIG_PATH=./dev-config pnpm dev
```

配置文件将保存在项目目录的 `dev-config/settings.toml`。

> 建议将 `dev-config/` 添加到 `.gitignore`，避免提交开发配置。

## 项目结构

```
pranslator/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── stores/             # Zustand 状态管理
│   └── App.tsx             # 主应用
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── config/         # 配置管理
│   │   ├── llm/            # LLM API 调用
│   │   └── main.rs         # 入口
│   └── tauri.conf.json     # Tauri 配置
└── package.json
```

## 代码规范

项目使用 ESLint 和 Prettier 进行代码检查：

```bash
# 检查代码
pnpm lint

# 自动修复
pnpm lint:fix
```

## IDE 推荐

- [VS Code](https://code.visualstudio.com/)
- [Tauri 扩展](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
