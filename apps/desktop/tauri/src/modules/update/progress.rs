//! Operation-scoped observations; percent exists only when the server provides bytes.
use super::UpdateChannel;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub operation_id: String,
    pub target: &'static str,
    pub channel: UpdateChannel,
    pub candidate_version: String,
    pub generation: u64,
    pub sequence: u64,
    pub stage: &'static str,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

impl UpdateProgress {
    pub fn new(channel: UpdateChannel, candidate: String, generation: u64) -> Self {
        Self {
            operation_id: format!("manager-{generation}-{}", rand::random::<u64>()),
            target: "manager",
            channel,
            candidate_version: candidate,
            generation,
            sequence: 0,
            stage: "checking",
            downloaded_bytes: 0,
            total_bytes: None,
        }
    }
    pub fn step(&mut self, stage: &'static str) -> Self {
        self.sequence += 1;
        self.stage = stage;
        self.clone()
    }
    pub fn chunk(&mut self, bytes: usize, total: Option<u64>) {
        self.downloaded_bytes = self.downloaded_bytes.saturating_add(bytes as u64);
        self.total_bytes = total.filter(|total| *total > 0 && *total >= self.downloaded_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operation_keeps_identity_and_does_not_invent_totals() {
        let mut progress = UpdateProgress::new(UpdateChannel::Beta, "0.1.10".into(), 4);
        progress.chunk(100, None);
        assert_eq!(progress.total_bytes, None);
        let first = progress.step("downloading");
        progress.chunk(100, Some(1000));
        let second = progress.step("downloading");
        assert_eq!(first.operation_id, second.operation_id);
        assert!(second.sequence > first.sequence);
        assert_eq!(second.total_bytes, Some(1000));
        progress.chunk(1000, Some(1000));
        assert_eq!(progress.total_bytes, None);
    }
}
