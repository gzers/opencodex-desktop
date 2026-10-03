//! 清理结果前端 DTO；只传输脱敏计数与摘要。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalCleanupResultDto {
    pub cleaned_logs: usize,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_result_uses_camel_case() {
        let value = LocalCleanupResultDto {
            cleaned_logs: 2,
            summary: "ok".to_string(),
        };
        let payload = serde_json::to_value(&value).expect("dto");
        assert_eq!(payload["cleanedLogs"], 2);
        assert_eq!(payload["summary"], "ok");
    }
}
