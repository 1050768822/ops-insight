use std::fmt;

use zeroize::ZeroizeOnDrop;

/// API Key 的安全包装类型。
/// - 内部值私有，无法直接读取
/// - Debug/Display 输出 [REDACTED]
/// - 只能通过 use_key() 访问，每次访问自动记录审计日志
/// - Drop 时自动清零内存
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecretKey {
    inner: String,
    label: String,
}

impl SecretKey {
    pub fn new(label: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            inner: key.into(),
        }
    }

    /// 唯一允许访问 Key 值的入口，访问时记录审计日志。
    pub fn use_key<F, R>(&self, purpose: &str, f: F) -> R
    where
        F: FnOnce(&str) -> R,
    {
        tracing::info!(
            key_label = %self.label,
            purpose = %purpose,
            "api_key_accessed"
        );
        f(&self.inner)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey({}=[REDACTED])", self.label)
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}
