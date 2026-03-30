use crate::domain::entities::{Issue, Suggestion};
use crate::domain::ports::AnalysisInput;

/// 本地分析规则 trait。
/// 实现此 trait 即可添加新的检测能力，无需修改其他代码。
pub trait LocalRule: Send + Sync {
    /// 规则名称，用于日志和报告标注来源
    fn name(&self) -> &str;

    /// 执行检测，返回发现的问题列表
    fn check(&self, input: &AnalysisInput) -> Vec<Issue>;

    /// 根据检测结果生成优化建议（可选，默认返回空）
    fn suggestions(&self, input: &AnalysisInput) -> Vec<Suggestion> {
        let _ = input;
        vec![]
    }
}
