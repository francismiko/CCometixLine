# Quota Feature 开发记录

> 创建: 2025-11-27 | 标签: #开发 #ClaudeCode | 状态: 进行中

## 概述

为 CCometixLine 添加第三方平台订阅额度显示功能，支持从 API 获取并在状态栏显示日/月额度信息。

## 已完成工作

### 1. 核心文件修改

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/core/segments/quota.rs` | 新建 | Quota segment 核心实现 |
| `src/core/segments/mod.rs` | 修改 | 添加 quota 模块导出 |
| `src/config/types.rs` | 修改 | 添加 `SegmentId::Quota` |
| `src/core/statusline.rs` | 修改 | 集成 QuotaSegment |
| `Cargo.toml` | 修改 | 添加 `quota` feature |

### 2. UI 文件修改（TUI 支持）

| 文件 | 修改内容 |
|------|----------|
| `src/ui/app.rs` | 3 处 match 添加 Quota 分支 |
| `src/ui/components/segment_list.rs` | 添加 Quota 分支 |
| `src/ui/components/settings.rs` | 添加 Quota 分支 |
| `src/ui/components/preview.rs` | 添加 Quota mock 数据 |

### 3. 编译状态

```bash
cargo build --release  # ✅ 成功
```

## 待完成工作

### 1. 配置文件设置

需要在 `~/.claude/ccline/` 下创建：

**quota_token** - 存储 API 认证 token：
```
fbe0cdbb-3382-4755-bca8-10e30304782f
```

**quota.toml** (可选) - 自定义配置：
```toml
api_url = "https://cc.yhlxj.com/8081/api/applet/claude/code/get/dashboard"
cache_ttl = 60
timeout = 3
show_requests = false
warning_threshold = 0.15
```

### 2. 主题预设更新

需要在主题文件中添加 Quota segment 配置，或在 TUI 中手动添加。

### 3. 测试验证

1. 替换现有 ccline 二进制
2. 验证 API 调用正常
3. 验证缓存机制工作
4. 验证状态栏显示

## 技术细节

### Quota Segment 功能

- **缓存机制**: 60 秒 TTL，避免频繁 API 请求
- **降级处理**: API 失败时返回缓存数据
- **显示格式**: `日 12.4/70 | 月 1162/2100`
- **低额度警告**: 剩余 <15% 时显示 ⚠️

### 配置文件位置

```
~/.claude/ccline/
├── quota_token      # API token
├── quota.toml       # 配置 (可选)
└── quota_cache.json # 缓存 (自动生成)
```

### API 响应格式

```json
{
  "data": {
    "daily_remaining": 12.43,
    "daily_total": 70,
    "month_remaining": 1162.53,
    "month_total": 2100,
    "today_requests": 1365
  },
  "success": true
}
```

## 后续优化

- [ ] 添加到默认主题预设
- [ ] 支持更多第三方平台 API
- [ ] TUI 中添加 Quota 配置界面
- [ ] 添加刷新快捷键

## 相关链接

- [[CCometixLine]] 原项目
- Fork: https://github.com/francismiko/CCometixLine
