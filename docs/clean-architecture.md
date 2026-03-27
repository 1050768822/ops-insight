# Clean Architecture — ops-insight

## 层次结构

```
interfaces/cli
      │
      ▼
application/use_cases
      │
      ▼
    domain          ◄──── infrastructure
  (entities, ports,        (newrelic, serilog,
   value_objects)           claude, output)
```

依赖方向：外层依赖内层，内层不知道外层的存在。

---

## 依赖规则

依赖只能由外向内指向，绝不能反向。

- `interfaces` 可以依赖 `application`
- `application` 可以依赖 `domain`
- `infrastructure` 可以依赖 `domain`
- `domain` 不依赖任何其他层

---

## 各层职责

| 路径 | 职责 |
|------|------|
| `domain/entities/` | 纯数据结构，无外部依赖，代表业务核心概念 |
| `domain/ports/` | 仅定义 trait，不包含任何实现，是外部系统的抽象接口 |
| `domain/value_objects/` | 值对象（如 `SecretKey`），封装业务规则，不可变 |
| `application/dtos/` | 用例的输入/输出结构体，与外部交互的数据契约 |
| `application/use_cases/` | 业务流程编排，每个文件对应一个用例，对外暴露唯一的 `execute()` 方法 |
| `infrastructure/newrelic/` | 实现 `DataSource` trait，从 New Relic 获取数据 |
| `infrastructure/serilog/` | 实现 `DataSource` trait，从本地日志文件获取数据 |
| `infrastructure/claude/` | 实现 `Analyzer` trait，调用 Claude API 进行分析 |
| `infrastructure/output/` | 实现 `ReportWriter` trait，负责报告的输出与格式化 |
| `interfaces/cli/` | 使用 clap 定义命令行参数，仅调用 use case，不直接调用 infrastructure |

---

## 扩展点

项目通过三个核心 trait 支持扩展，新增功能只需在 `infrastructure/` 中添加实现，其他层无需改动。

### `DataSource` trait

定义位置：`domain/ports/`

用于接入新的数据来源（如 Datadog、CloudWatch 等）。

扩展方式：在 `infrastructure/xxx/source.rs` 中新建文件并实现该 trait。

### `Analyzer` trait

定义位置：`domain/ports/`

用于接入新的分析引擎（如 GPT 等）。

扩展方式：在 `infrastructure/xxx/analyzer.rs` 中新建文件并实现该 trait。

### `ReportWriter` trait

定义位置：`domain/ports/`

用于接入新的输出渠道（如 Slack、邮件等）。

扩展方式：在 `infrastructure/output/xxx.rs` 中新建文件并实现该 trait。

**规则：扩展时只新增 infrastructure 层的实现文件，不修改其他任何层。**

---

## 规则

1. **禁止跨层调用**：`interfaces` 层不得直接调用 `infrastructure` 层，必须通过 `application/use_cases/` 中转。

2. **trait 定义与实现分离**：trait 定义只存放于 `domain/ports/`，具体实现只存放于 `infrastructure/`。

3. **use case 返回 DTO**：`application/use_cases/` 中的 `execute()` 方法返回 `application/dtos/` 中定义的类型，不返回 domain entities。

4. **domain 层禁止引入外部 crate**：`domain` 层不得导入 `reqwest`、数据库相关 crate、HTTP 类型等任何基础设施依赖。
