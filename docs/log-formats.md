# 日志格式说明

## 格式自动检测

检测逻辑位于 `src/infrastructure/serilog/source.rs` 的 `parse_file()` 函数中。

检测依据是**文件内容**，而非文件扩展名。规则如下：

- 若文件第一个非空行以 `{` 开头 → 识别为 JSON/CLEF 格式
- 否则 → 识别为管道分隔格式

---

## 支持的格式

### 格式一：Serilog JSON / CLEF

CLEF（Compact Log Event Format）是 Serilog 的紧凑 JSON 日志格式，每行一个 JSON 对象。

示例：

```json
{"@t":"2026-03-27T09:11:49Z","@l":"Warning","@mt":"User {Id} logged in","@m":"User 42 logged in"}
```

字段映射规则：

| 日志字段 | JSON 键（按优先级） |
|----------|---------------------|
| 时间戳   | `@t`、`Timestamp` |
| 日志级别 | `@l`、`Level` |
| 消息内容 | `@m`、`@mt`、`RenderedMessage`、`MessageTemplate` |
| 主机名   | `Properties.MachineName`、`Properties.HostName` |
| 服务名   | `Properties.Application`、`Properties.SourceContext` |

消息字段按上表优先级取第一个存在的值。

---

### 格式二：管道分隔格式

基于自定义 Serilog 输出模板生成，字段间使用 `||` 分隔。

单行示例：

```
09:11:49 || Warning || MyApp.Service || 用户登录成功 ||  ||end
```

多行示例（含异常堆栈）：

```
09:11:49 || Error || MyApp.Service || 发生未处理异常 || System.NullReferenceException: ...
   at MyApp.Service.Run() in Service.cs:line 42
 ||end
```

格式规则：

- 每条日志条目以 `||end` 结尾，支持跨行
- 字段顺序：`时间 || 级别 || 来源 || 消息 || 异常详情`
- 异常部分可为空：`... || 消息内容 ||  ||end`
- 日志条目不含日期，**日期从文件名中推断**

文件名日期解析规则：

| 文件名示例           | 解析结果   |
|----------------------|------------|
| `log_20260311.log`   | 2026-03-11 |
| `app_2026-03-11.log` | 2026-03-11 |

支持的日期格式：`YYYYMMDD` 或 `YYYY-MM-DD`。

---

## 日志级别映射

两种格式均使用以下映射规则（大小写不敏感）：

| 原始值                  | 统一级别  |
|-------------------------|-----------|
| `verbose`、`debug`      | Debug     |
| `information`、`info`   | Info      |
| `warning`、`warn`       | Warning   |
| `error`                 | Error     |
| `fatal`、`critical`     | Fatal     |

---

## 错误事件聚合规则

以下级别的日志会被聚合为 `ErrorEvent`：Warning、Error、Fatal。

聚合逻辑：

1. 按 `(消息内容, 主机名)` 组合分组
2. 统计每组出现次数
3. 按出现次数降序排列
4. 取前 50 条发送给 Claude 分析器

---

## 如何添加新格式

所有改动仅限于 `src/infrastructure/serilog/source.rs`，无需修改其他层。

**第一步：添加检测条件**

在 `parse_file()` 函数中，根据文件内容特征添加新的判断分支：

```rust
// 示例：根据首行特征检测新格式
if first_non_empty_line.starts_with("some_marker") {
    return parse_new_format(content, hostname, date);
}
```

**第二步：实现解析函数**

添加新的解析函数，签名参考现有函数：

```rust
fn parse_new_format(content: &str, hostname: &str, date: NaiveDate) -> Vec<LogEntry> {
    // 解析逻辑
}
```

**注意事项：**

- 检测必须基于**文件内容**，不得依赖文件扩展名
- 不得修改 `DataSource` trait 或任何其他层的代码
- 不得在领域层或接口层引入基础设施相关逻辑
