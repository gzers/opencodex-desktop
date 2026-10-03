//! 设置页工作台前端 DTO；路径、打开结果与清理摘要分离。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedPathKey {
    ManagerState,
    Backups,
    Logs,
    Exports,
    Cache,
    SyncState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentPathKind {
    #[serde(rename = "skills-source")]
    SkillsSource,
    #[serde(rename = "skills")]
    Skills,
    #[serde(rename = "mcp")]
    Mcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedPathDto {
    pub key: ManagedPathKey,
    pub path: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPathDto {
    pub kind: AgentPathKind,
    pub client: String,
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedPathTargetsDto {
    pub managed: Vec<ManagedPathDto>,
    pub agent: Vec<AgentPathDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenResultDto {
    pub opened: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_path_dto_uses_camel_case() {
        let value = ManagedPathTargetsDto {
            managed: vec![ManagedPathDto {
                key: ManagedPathKey::ManagerState,
                path: "/fixtures/manager-state".to_string(),
                label: "管理器设置目录".to_string(),
            }],
            agent: vec![AgentPathDto {
                kind: AgentPathKind::SkillsSource,
                client: "preferred".to_string(),
                label: "Skills 源目录".to_string(),
                path: "/fixtures/skills".to_string(),
            }],
        };
        let payload = serde_json::to_value(&value).expect("dto");
        assert_eq!(payload["managed"][0]["key"], "manager_state");
        assert_eq!(payload["agent"][0]["kind"], "skills-source");
    }

    #[test]
    fn open_result_is_explicit() {
        let value = serde_json::to_value(OpenResultDto { opened: true }).expect("dto");
        assert_eq!(value["opened"], true);
    }
}
