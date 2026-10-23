# 信使 · API 工作台

本地优先的桌面 API 调试客户端。用 Tauri 2 + Rust 做请求引擎、MCP 服务与本地存储，用 Vue 3 + shadcn-vue 做界面；接口、参数、环境和历史都只保存在本机。

版本：1.0.0

## 功能

- **调试 / 设计双视图**：调试视图发送请求；设计视图定义参数、Body 示例和返回字段
- **参数表**：名称、类型、值、说明一屏看清；支持 string / number / boolean / enum / file
- **自定义枚举**：enum 类型的值列直接下拉，选项由每个字段自己维护
- **项目 / 模块 / 接口**：多层文件夹树，支持搜索、重命名、复制、删除
- **文件夹鉴权继承**：子请求选「继承」即可复用上级目录的 API Key / Bearer / Basic
- **环境**：前置 URL、变量、自动保存、当前环境切换
- **Body**：none / form-data / x-www-form-urlencoded / JSON / XML / Text；JSON 支持原文与字段表格双模式
- **响应**：JSON 折叠树、中文返回字段名、原始文本、响应头、实际请求回看、图片预览、8 MiB 截断保护
- **后置操作**：从响应 JSON 按路径提取值，自动写回环境变量（适合登录 token）
- **AI 接入**：安装包内置 Rust MCP 服务与 Skill，Codex / Qoder 可直接把接口导入信使
- **其他**：请求取消、历史记录、命令面板、亮暗主题、12 种主色、窗口尺寸位置记忆、cURL / OpenAPI 导出

首次启动是空工作台，只带一个「本地开发」环境，不预置示例接口。

## AI 接入（安装版）

MCP 服务和 Skill 都随安装包分发，不需要 Node、源码仓库或手写路径。

1. 安装并启动信使。
2. 打开 **设置 → AI 接入**。
3. 复制对应工具配置：
   - **Codex**：把 `[mcp_servers.xingshi]` 追加到 `%USERPROFILE%\.codex\config.toml`
   - **Qoder**：把 JSON 合并到 `%USERPROFILE%\.qoder\settings.json` 的 `mcpServers`
4. 点「安装 Skill 到 Qoder」（其他支持技能目录的工具，复制面板里的 Skill 路径，把整个文件夹放进技能目录）。
5. 重启 AI 工具。

配置长这样，`<信使安装目录>` 由面板自动替换成当前电脑的实际位置，不用手填：

**Codex**

```toml
[mcp_servers.xingshi]
command = "<信使安装目录>/xingshi-mcp.exe"
args = []
```

**Qoder**

```json
{
  "mcpServers": {
    "xingshi": {
      "command": "<信使安装目录>/xingshi-mcp.exe",
      "args": []
    }
  }
}
```

它不是 npm 包，所以 `command` 直接指向安装包里的 `xingshi-mcp.exe`，不是 `npx`，`args` 为空。其他支持 stdio MCP 的工具同理：`command` 填这个 exe，`args` 留空。

换电脑或重装后，重新打开面板复制一次即可。

之后在任意项目里说「把这些接口导入信使」。导入成功后切回信使，窗口获得焦点会自动刷新；也可以在侧栏点刷新图标。找到新接口直接点发送。

## 开发者快速开始

前置：Node.js 22.5+（仅开发/构建需要）、Rust 与 Tauri 2 的系统依赖。

```bash
npm install
npm run tauri dev
```

`tauri dev/build` 会自动执行 `npm run ai:prepare`：编译 Rust MCP sidecar，并把 `skills/xingshi-api-import` 复制进 Tauri 资源。

打包安装包（Windows 主目标，NSIS）：

```bash
npm run tauri build
```

只在浏览器预览界面（mock 模式，数据存 localStorage，不含 MCP sidecar）：

```bash
npm run dev
```

## 数据位置

桌面端使用本地 SQLite：

- Windows：`%APPDATA%\com.zx.rustone\messenger.db`
- macOS：`~/Library/Application Support/com.zx.rustone/messenger.db`
- Linux：`${XDG_DATA_HOME:-~/.local/share}/com.zx.rustone/messenger.db`

浏览器 mock 模式使用 `localStorage`，清掉 `messenger.workspace.v1` 等键即可回到空工作台。

## 测试与检查

```bash
npm run build                 # vue-tsc + vite build
cd src-tauri && cargo test    # 请求引擎、数据库、MCP sidecar
npm run ai:prepare            # 单独重建 sidecar 与资源
```

## 目录结构

```text
src/                              Vue 界面、stores、请求组装
src-tauri/                        Rust 后端、SQLite、Tauri 命令
src-tauri/src/mcp.rs              MCP stdio 协议与接口导入
src-tauri/src/bin/xingshi-mcp.rs  MCP sidecar 入口
skills/xingshi-api-import/        Skill 源，随安装包作为资源分发
scripts/prepare-ai.mjs            构建 sidecar、复制 Skill 资源
```

## 开发约定

- 新增/缺失 UI 组件一律通过 shadcn-vue CLI 引入：`npm run ui -- <组件名>`，不要手写替代组件。
- 新增功能后至少跑 `npm run build`；改到 Rust、数据库结构或 MCP 时同时跑 `cargo test`。
- MCP 直接读写 SQLite；数据库表结构变更时同步修改 `src-tauri/src/mcp.rs` 的导入逻辑。
- sidecar 与 Skill 资源是构建产物，不要提交 `src-tauri/binaries/` 和 `src-tauri/resources/skills/`。
