# 银河录像局 API 文档

> 官方提供的 API 文档，用于第三方订阅额度显示功能

## 基础信息

- **Base URL**: `https://relay.nf.video`
- **认证方式**: `Authorization: Bearer {API_KEY}`

---

## 获取使用量

```
GET /v1/usage
```

**请求**
```bash
curl https://relay.nf.video/v1/usage \
  -H "Authorization: Bearer $API_KEY"
```

**响应**
```json
{
  "usage": {
    "totalCostUSD": 9.05,
    "requestCount": 150,
    "dailyLimitUSD": 20.0,
    "remainingUSD": 10.95,
    "canMakeRequest": true
  },
  "limits": {
    "dailyUSD": 20.0
  },
  "timestamp": "2025-11-28T10:00:00Z"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `usage.totalCostUSD` | number | 今日已用费用 (USD) |
| `usage.requestCount` | integer | 今日请求次数 |
| `usage.dailyLimitUSD` | number | 每日限额 (USD) |
| `usage.remainingUSD` | number | 今日剩余额度 (USD) |
| `usage.canMakeRequest` | boolean | 是否可继续请求 |
| `limits.dailyUSD` | number | 每日限额 (USD) |
| `timestamp` | string | 响应时间戳 (可选) |

---

## 获取订阅信息

```
GET /api/subscription
```

> ⚠️ 此端点可能需要不同的认证方式，API Key 方式暂不支持

**请求**
```bash
curl https://relay.nf.video/api/subscription \
  -H "Authorization: Bearer $API_KEY"
```

**响应**
```json
[
  {
    "subscriptionPlanName": "Pro Plan",
    "cost": 99.0,
    "endDate": "2025-12-31",
    "subscriptionStatus": "active",
    "remainingDays": 33,
    "billingCycleDesc": "月",
    "resetTimes": 5,
    "isActive": true
  }
]
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `subscriptionPlanName` | string | 订阅计划名称 |
| `cost` | number | 订阅费用 |
| `endDate` | string \| null | 到期日期 |
| `subscriptionStatus` | string | 订阅状态 (active/expired/cancelled/pending) |
| `remainingDays` | integer | 剩余天数 |
| `billingCycleDesc` | string | 计费周期 (月/年) |
| `resetTimes` | integer | 可用重置次数 |
| `isActive` | boolean | 是否激活 |

---

## 健康检查

```
GET /
```

**请求**
```bash
curl https://relay.nf.video/
```

**响应**: HTTP 状态码 `200-299` 表示服务正常

---

## 使用说明

### 获取 API Key

登录 [银河录像局](https://nf.video) 平台获取 API Key。

### 配置 CCometixLine

将 API Key 保存到配置文件：

```bash
echo "your-api-key" > ~/.claude/ccline/quota_token
```

### 相关链接

- 银河录像局官网: https://nf.video
- CCometixLine: https://github.com/francismiko/CCometixLine
