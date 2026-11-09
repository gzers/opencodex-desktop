//! Candidate protocol reader, not registered as a production Stop authority.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Binding {
    schema: String,
    home: PathBuf,
    sid: String,
    pid: u32,
    created: String,
    generation: String,
    job: String,
    lease_volume: u32,
    lease_index: String,
}
fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}
#[derive(Clone, Copy)]
struct Expected<'a> {
    home: &'a Path,
    sid: &'a str,
    pid: u32,
    created: &'a str,
    secret: &'a str,
    challenge: &'a str,
    port: u16,
    proof: &'a str,
}
impl Binding {
    fn authenticates(&self, expected: &Expected<'_>) -> bool {
        let Expected {
            home: expected_home,
            sid: expected_sid,
            pid: expected_pid,
            created: expected_created,
            secret,
            challenge,
            port,
            proof,
        } = *expected;
        let valid_key =
            |v: &str| v.len() == 43 && URL_SAFE_NO_PAD.decode(v).is_ok_and(|b| b.len() == 32);
        let home = key(&self.home);
        let mut job_digest = Sha256::new();
        job_digest.update(self.sid.as_bytes());
        job_digest.update([0]);
        job_digest.update(home.as_bytes());
        let expected_job = format!(
            "Global{}opencodex-runtime-{:x}",
            char::from(92),
            job_digest.finalize()
        );
        if self.schema != "ocxd-runtime-lifetime/1"
            || !self.home.is_absolute()
            || home != key(expected_home)
            || self.sid != expected_sid
            || self.pid == 0
            || self.pid != expected_pid
            || self.created != expected_created
            || self.created.parse::<u64>().is_err()
            || self.lease_index.parse::<u64>().is_err()
            || uuid::Uuid::parse_str(&self.generation).is_err()
            || self.job != expected_job
            || port == 0
            || !valid_key(secret)
            || !valid_key(challenge)
        {
            return false;
        }
        let tuple = serde_json::json!([
            self.schema,
            self.home,
            self.sid,
            self.pid,
            self.created,
            self.generation,
            self.job,
            self.lease_volume,
            self.lease_index
        ]);
        let Ok(tuple) = serde_json::to_string(&tuple) else {
            return false;
        };
        let Ok(actual) = URL_SAFE_NO_PAD.decode(proof) else {
            return false;
        };
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(
            format!("opencodex-runtime-lifetime-v1\n{challenge}\n{port}\n{tuple}").as_bytes(),
        );
        mac.verify_slice(&actual).is_ok()
    }
}

pub(crate) fn verify_candidate_file(path: &Path, home: &Path, sid: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    if bytes.len() > 8192 {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(pid) = value["pid"].as_u64().and_then(|p| u32::try_from(p).ok()) else {
        return false;
    };
    let Some(created) = value["created"].as_str() else {
        return false;
    };
    let Some(proof) = value["candidateProof"].as_str() else {
        return false;
    };
    let Ok(binding) = serde_json::from_value::<Binding>(value["candidateBinding"].clone()) else {
        return false;
    };
    let secret = "A".repeat(43);
    let challenge = URL_SAFE_NO_PAD.encode([1u8; 32]);
    let expected = Expected {
        home,
        sid,
        pid,
        created,
        secret: &secret,
        challenge: &challenge,
        port: 1,
        proof,
    };
    if !binding.authenticates(&expected) {
        return false;
    }
    for (field, bad) in [
        ("home", serde_json::json!(home.join("other"))),
        ("sid", serde_json::json!(format!("{sid}-other"))),
        ("pid", serde_json::json!(pid.wrapping_add(1))),
        ("created", serde_json::json!("0")),
        (
            "generation",
            serde_json::json!(uuid::Uuid::new_v4().to_string()),
        ),
        ("job", serde_json::json!("Global\\other-job")),
        (
            "leaseVolume",
            serde_json::json!(binding.lease_volume.wrapping_add(1)),
        ),
        ("leaseIndex", serde_json::json!("0")),
    ] {
        let mut bad_value = value["candidateBinding"].clone();
        bad_value[field] = bad;
        let Ok(changed) = serde_json::from_value::<Binding>(bad_value) else {
            return false;
        };
        if changed.authenticates(&expected) {
            return false;
        }
    }
    !binding.authenticates(&Expected {
        secret: &"C".repeat(43),
        ..expected
    }) && !binding.authenticates(&Expected {
        challenge: &"D".repeat(43),
        ..expected
    }) && !binding.authenticates(&Expected {
        port: 2,
        ..expected
    })
}
