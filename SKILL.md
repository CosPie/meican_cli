---
name: meican-order
description: '美餐点餐助手。Use when: 帮用户在美餐(Meican)平台点餐、取消订单、查看今日菜单、浏览可点菜品、查看历史订单、根据历史偏好智能规划一周午餐晚餐。Examples: "帮我点午餐", "看看今天有什么菜", "取消我的晚餐", "最近都点了什么", "帮我规划一周午餐", "plan weekly meals", "根据偏好帮我点这周的饭", "order lunch", "what can I eat today".'
argument-hint: '告诉我你想做什么，例如：帮我点午餐 / 帮我规划一周午餐晚餐'
---

# 美餐点餐助手 (Meican Order Assistant)

## 概述

通过 `meican` CLI 工具帮助用户完成美餐平台的点餐操作。**所有命令都不加 `--table` 以输出 JSON 供解析；展示给用户时加 `--table` 以输出可读表格。**

---

## 前置检查

每次启动前先确认环境就绪：

```bash
# 1. 确认工具已安装
# 如果用户指定了自定义路径（如 /path/to/meican），直接用该路径
which meican || echo "NOT INSTALLED"

# 2. 确认已登录
meican status
```

> **Fish Shell 语法提示**：在 fish 中设置自定义二进制路径时，用 `set MEICAN /path/to/binary`，而非 `MEICAN=/path/to/binary`（bash 语法在 fish 下会报错）。

- **未安装** → 告知用户安装命令：`curl -fsSL https://raw.githubusercontent.com/CosPie/meican_cli/master/install.sh | bash`
- **未登录** → 引导用户执行 `meican login <email>`，密码会交互式提示

---

## 核心工作流

### 流程 A：帮用户点餐（最常用）

```bash
# Step 1：查看今天餐次状态
meican --table today

# Step 2：列出可选菜品，获取 Dish ID
meican --table dishes lunch      # 或 breakfast / dinner

# Step 3：确认用户选择后下单
meican --table order lunch --dish <DISH_ID>
```

**关键逻辑：**
- `Status = AVAILABLE` → 可以点餐
- `Status = ORDER` → 已点过，询问用户是否要取消重点
- `Status = CLOSED` → 已关闭，无法点餐
- 下单时系统自动选择第一个配送地址，无需用户手动指定

---

### 流程 B：取消订单

```bash
# Step 1：查看今日订单确认有哪些已点
meican --table today

# Step 2：取消指定餐次
meican cancel lunch      # 或 breakfast / dinner
```

---

### 流程 C：查看可选菜品 / 餐厅

```bash
# 查看菜品（含价格和 Dish ID）
meican --table dishes lunch

# 查看餐厅列表
meican --table restaurants lunch
```

---

### 流程 D：查看历史订单

```bash
meican --table history --days 30
```

---

### 流程 E：智能周餐规划（基于历史偏好批量点一周午餐晚餐）
没有特别说明，默认为本周周一至周五的午餐和晚餐。如果今天是周六日，则默认为下周周一至周五。

> **说明：** `meican order --dish <ID> --date <YYYY-MM-DD>` 支持提前下单，无需当天执行。`meican dishes --date <YYYY-MM-DD>` 可查看指定日期可选菜品，`meican calendar` 可查看未来日期餐次状态。本流程可在任意一天完成整周下单。

#### Step 1：获取过去 30 天历史订单（JSON 模式，不加 --table）

```bash
meican history --days 30
```

返回 JSON 数组，每条记录包含：
- `date` — 日期
- `meal_time` — 餐次（`BREAKFAST` / `LUNCH` / `DINNER`）
- `dish_name` — 菜品名
- `restaurant_name` — 餐厅名
- `price_in_cent` — 价格（分）

#### Step 2：分析用户偏好（Agent 侧计算）

从 Step 1 的 JSON 数据中提取偏好画像：

1. **按餐次分组**：只取 `meal_time = LUNCH` 和 `DINNER` 的记录
2. **菜品偏好 Top N**：统计每道 `dish_name` 的出现次数，按频次降序排列
3. **餐厅偏好 Top N**：统计每家 `restaurant_name` 的出现次数，按频次降序排列
4. **价格区间**：计算 `price_in_cent` 的平均值和 P25-P75 区间
5. **近期趋势**：对比最近 7 天 vs 更早的偏好，是否有新增/减少的菜品，体现口味变化
6. **午餐/晚餐分别分析**：两个餐次的偏好可能不同，分别生成 Top N

将偏好画像以表格形式展示给用户确认，例如：
```
🍽️ 你的午餐偏好 Top 5：
1. 黄焖鸡米饭 (7次, 平均¥22) — 来自「味美香」
2. 宫保鸡丁饭 (5次, 平均¥25) — 来自「川香阁」
...
```

#### Step 3：查看本周日历获取可用餐次（JSON 模式）

```bash
# 查看本周一到周五（根据实际日期调整）
meican calendar 2026-04-13 2026-04-17
```

从返回的 JSON 中，对每天的 `dateList[].calendarItemList[]` 逐项检查：
- 用 `title` 字段判断餐次类型（包含「午餐」→ lunch，包含「晚餐」→ dinner）
- 检查 `status` 字段：
  - `AVAILABLE` → 可以规划，记录 `userTab.uniqueId` 用于后续查菜品/下单
  - `ORDER` → **已点过，默认跳过**（不询问用户是否取消）
  - `CLOSED` → 已关闭，跳过
- 生成一周可用时段列表，例如：
  ```
  周一 午餐: AVAILABLE ✅  晚餐: AVAILABLE ✅
  周二 午餐: ORDER (已点) ⏭️  晚餐: AVAILABLE ✅
  ...
  ```

#### Step 4：获取每日可选菜品（JSON 模式，逐天查询）

对 Step 3 中每个 `AVAILABLE` 的日期+餐次，使用 `--date` 参数查询该日可选菜品：

```bash
# 查周一午餐可选菜品
meican dishes lunch --date 2026-04-13

# 查周一晚餐可选菜品
meican dishes dinner --date 2026-04-13

# 查周二晚餐可选菜品（午餐已点过，跳过）
meican dishes dinner --date 2026-04-14

# ... 逐天逐餐次查询
```

从 JSON 中提取 `othersRegularDishList`（camelCase）中每道菜的：
- `id` — 下单时需要的 Dish ID
- `name` — 菜品名
- `priceInCent` — 价格（分，÷100 = 元）
- `restaurant.name` — 所属餐厅

> **注意：** 不同日期的可选菜品可能不同。必须逐天查询，不能用今天的菜单代替未来日期。

#### Step 5：智能匹配生成一周计划（Agent 侧推理）

将每天的可选菜品与 Step 2 的偏好画像做匹配，生成推荐：

**匹配优先级（从高到低）：**
1. **菜品名完全匹配**：可选菜品的 `name` 与偏好 Top N 的 `dish_name` 一致
2. **偏好餐厅匹配**：可选菜品来自偏好 Top N 的 `restaurant_name`
3. **价格区间匹配**：菜品价格在历史平均 P25-P75 区间内
4. **兜底推荐**：以上都不匹配时，按价格适中原则推荐

**多样性规则：**
- 连续两天不推荐同一道菜
- 午餐和晚餐不推荐同一道菜
- 尽量在偏好 Top N 中轮换

生成完整计划表展示给用户：

```
📋 本周点餐计划：

| 日期 | 餐次 | 推荐菜品 | 餐厅 | 价格 | 匹配原因 |
|------|------|----------|------|------|----------|
| 周一 4/13 | 午餐 | 黄焖鸡米饭 | 味美香 | ¥22 | 历史最爱 #1 |
| 周一 4/13 | 晚餐 | 酸菜鱼 | 鱼你在一起 | ¥28 | 偏好餐厅 |
| 周二 4/14 | 午餐 | — | — | — | 已点过，跳过 |
| 周二 4/14 | 晚餐 | 宫保鸡丁饭 | 川香阁 | ¥25 | 历史最爱 #2 |
| ... | ... | ... | ... | ... | ... |
```

#### Step 6：用户确认并调整

- 展示完整计划表，等待用户确认
- 用户可以说「周三午餐换一个」「不想吃XX」「换个便宜的」等
- 根据调整需求从该天的可选菜品中重新推荐
- 用户确认「OK」或「就这样」后进入执行阶段

#### Step 7：批量执行所有 AVAILABLE 餐次下单（支持提前下单）

对计划中所有 status=AVAILABLE 的餐次，使用 `--date` 参数提前下单：

```bash
# 下单时指定日期（加 --table 展示给用户看）
meican --table order lunch --dish <DISH_ID> --date 2026-04-16
meican --table order dinner --dish <DISH_ID> --date 2026-04-16
meican --table order dinner --dish <DISH_ID> --date 2026-04-17
```

- 逐个餐次下单，每次下完确认输出包含 `Order placed successfully!` 后再下一个
- 如果下单失败（输出 `Error:`），告知用户错误信息并询问是否换一道菜
- **今天的餐次**不需要 `--date` 参数（或传今日日期均可）

#### Step 8：下单完成汇总

全部下单完成后：

1. **汇总展示**本次已成功下单的所有餐次：
   ```
   ✅ 本周下单完成：
   周四 4/16 午餐: 明炉烧鸭拼... ¥35
   周四 4/16 晚餐: 芥兰牛肉饭 ¥30
   周五 4/17 晚餐: 三杯鸡拼... ¥30
   ```
2. **提醒**：`cancel` 命令仅支持取消**今天**的订单（不支持 `--date`）。如需取消未来某天的订单，需在当天执行 `meican cancel <meal>`

---

## 命令速查

| 目标 | 命令 |
|------|------|
| 查看今天所有餐次 | `meican --table today` |
| 查看指定餐次菜品 | `meican --table dishes <meal>` |
| 查看指定日期菜品 | `meican --table dishes <meal> --date YYYY-MM-DD` |
| 查看指定餐次餐厅 | `meican --table restaurants <meal>` |
| 查看指定日期餐厅 | `meican --table restaurants <meal> --date YYYY-MM-DD` |
| 查看配送地址 | `meican --table addresses` |
| 下单（今天） | `meican --table order <meal> --dish <ID>` |
| 提前下单（指定日期） | `meican --table order <meal> --dish <ID> --date YYYY-MM-DD` |
| 取消订单 | `meican cancel <meal>` |
| 查看历史 | `meican --table history --days <N>` |
| 查看日历范围 | `meican --table calendar 2024-01-01 2024-01-07` |
| 检查登录状态 | `meican status` |
| 更新工具 | `meican update` |

**`<meal>` 可选值：** `breakfast` 早餐 / `lunch` 午餐 / `dinner` 晚餐

---

## 错误处理

| 错误信息 | 处理方式 |
|----------|----------|
| `Not logged in` | 执行 `meican login <email>` |
| `No available meal slot` | 当天该餐次已关闭或不存在 |
| `No delivery address found` | 需要用户在美餐网页端先设置地址 |
| `API error (HTTP 401/302)` | 会话过期，需重新 `meican login` |
| `Login failed: Invalid username or password` | 账号密码错误 |
| `Failed to get latest version` | 网络问题，稍后重试 |

---

## 交互原则

1. **先查后点**：下单前始终先展示今日菜品让用户确认，不要直接执行 `order`
2. **展示价格**：从 `dishes` 输出中取 Price 字段告知用户
3. **确认操作**：取消订单前向用户二次确认
4. **JSON 解析**：需要提取数据时去掉 `--table`；展示给用户时加 `--table`
