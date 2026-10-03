//! FZ-12/FZ-14 Doctor 前端 DTO；只传输脱敏后的只读结果。

use serde::{Deserialize, Serialize};

use crate::modules::doctor::{DoctorMode, DoctorReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DoctorModeDto {
    ReadOnly,
}

impl From<DoctorMode> for DoctorModeDto {
    fn from(_value: DoctorMode) -> Self {
        Self::ReadOnly
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorDto {
    pub mode: DoctorModeDto,
    pub lines: Vec<String>,
    pub truncated: bool,
}

impl From<DoctorReport> for DoctorDto {
    fn from(value: DoctorReport) -> Self {
        Self {
            mode: value.mode.into(),
            lines: value.lines,
            truncated: value.truncated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::doctor::DoctorOutput;

    #[test]
    fn doctor_dto_uses_camel_case_and_read_only_mode() {
        let report = DoctorReport::from_output(&DoctorOutput {
            stdout: b"safe\n".to_vec(),
        })
        .unwrap();
        let payload = serde_json::to_value(DoctorDto::from(report)).unwrap();
        assert_eq!(payload["mode"], "readOnly");
        assert_eq!(payload["truncated"], false);
        assert_eq!(payload["lines"][0], "safe");
    }
}
