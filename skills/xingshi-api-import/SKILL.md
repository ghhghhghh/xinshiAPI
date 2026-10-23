---
name: xingshi-api-import
description: 把当前项目的接口导入本地「信使 API 工作台」桌面应用，供用户直接点击发送测试。当用户说「导入信使」「把接口放进信使」「同步到 API 工作台」「生成可调试接口」时使用；会扫描路由/DTO，填写项目/模块/接口、参数表、Body、鉴权、设计视图返回字段，并调用 xingshi MCP。
---

# 信使接口导入

## 目标

把当前项目里已有的接口整理成「项目 → 模块 → 接口」，通过 `xingshi` MCP 写入本地信使数据库，让用户切回信使后直接点击发送。不要用 UI 自动化，也不要直接改数据库。

## 前置

- MCP 服务名通常是 `xingshi`，工具可能显示为 `mcp__xingshi__import_api_tree`、`mcp__xingshi__list_workspace`。
- 如果找不到这些工具：不要手写 SQL，提示用户新开一个 Qoder 会话（MCP 配置在会话启动时加载）。
- 只在用户明确要导入时写入；默认不删除、不改动无关接口。

## 流程

1. **确认范围**：项目名、环境前置 URL（例如 `http://localhost:3000`）、要导入的模块/接口。能从代码、README、`.env.example` 推断就直接做；拿不准再问。
2. **扫描接口**：从路由注册、Controller、OpenAPI/Swagger、前端请求封装、DTO/类型定义中提取 method、路径、参数、请求体、响应结构。
3. **先查已有**：调用 `list_workspace`，复用已有项目/模块名称，避免重复目录；不要把已有接口改名或删除。
4. **映射字段**：
   - 参数名用代码里的真实字段名；类型只使用 `string` / `number` / `boolean` / `enum` / `file`。
   - 枚举必须有真实候选值，写入该参数的 `options`；调试时信使会显示为下拉框。
   - `value` 填一个可发送的示例值；说明写中文，标注必填/单位/范围。
   - 路径参数、Query、Header 都写进 `params` / `headers`；不要漏掉分页、排序、租户、语言等公共参数。
   - 请求体按项目选择 `json` / `form-data` / `urlencoded` / `xml` / `text`；JSON 写一段完整示例。
   - 鉴权优先写进环境变量或请求的 `auth`，不要粘贴真实 token；如果 token 来自登录接口，配置 `extractors`。
   - `responseLabels` 对应设计视图：从 DTO/示例响应提取 `path`（数组用 `data.list[].name`）、`type`、中文名 `label`、说明和示例。
5. **一次导入**：调用 `import_api_tree`，可带 `environment` 设置 `baseUrl` 和变量。先用 `dryRun: true` 看计划，确认无误后去掉 `dryRun` 真正写入。
6. **回报**：列出导入了哪些项目/模块/接口，告诉用户「切回信使，窗口聚焦会自动刷新；点接口即可发送」。

## 数据结构示例

```json
{
  "environment": {
    "name": "本地开发",
    "baseUrl": "http://localhost:3000",
    "activate": true,
    "variables": [{ "name": "token", "value": "", "description": "留空，登录后提取" }]
  },
  "projects": [
    {
      "name": "示例项目",
      "folders": [
        {
          "name": "用户管理",
          "requests": [
            {
              "name": "获取用户列表",
              "method": "GET",
              "url": "/api/users",
              "description": "分页获取用户",
              "params": [
                { "name": "page", "type": "number", "value": "1", "description": "页码，必填" },
                {
                  "name": "role",
                  "type": "enum",
                  "value": "admin",
                  "options": ["admin", "editor", "viewer"],
                  "description": "角色筛选"
                }
              ],
              "headers": [
                { "name": "Authorization", "type": "string", "value": "Bearer {{token}}", "description": "登录令牌" }
              ],
              "body": { "mode": "none" },
              "responseLabels": [
                { "path": "data.list[].name", "label": "姓名", "type": "string", "example": "张三" },
                { "path": "data.total", "label": "总数", "type": "number", "example": "10" }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## 工具约定

- `list_workspace`：读取现有项目/模块/接口和环境；返回体里没有完整参数，避免把大响应塞进上下文。
- `import_api_tree`：按路径 + 名称幂等 upsert；不会删除。`dryRun` 只返回创建/更新清单。
- `upsert_environment`：单独维护环境前置 URL、变量和当前环境。
- 导入成功后 MCP 会更新工作台版本号，信使在重新获得焦点或轮询时自动刷新。

## 边界

- 不生成无法发送的空接口；每个接口至少要有 method + url。
- 不臆造接口；以代码/文档为准。找不到响应结构时，`responseLabels` 可以留空。
- 不把真实密码、生产 token 写进参数或示例值。
- 用户只要求部分接口时，只导入选中的接口。
