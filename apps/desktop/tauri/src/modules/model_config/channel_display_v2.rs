//! Official display-only names for logical channels whose runtime key differs.
//! Generated labels live in the official config, so official refresh owns their
//! publication. No catalog rewriting, polling writer or routing alias is used.
use super::{schema_v2::*, source_v2::MaterializationLedgerV2};
use serde_json::Value;

pub(super) fn generated(
    channel: &ChannelV2,
    provider: &str,
    model: &str,
) -> Result<Option<String>> {
    if channel.label == provider {
        return Ok(None);
    }
    // U+FF0F is presentation only. Native model IDs and routing slugs are never
    // rewritten; official2.99 rejects U+002F in modelDisplayNames values.
    let value = format!("{} · {}", channel.label, model).replace('/', "／");
    if value.encode_utf16().count() > 128
        || value.chars().any(|c| {
            c <= '\u{1f}'
                || ('\u{7f}'..='\u{9f}').contains(&c)
                || matches!(c, '\u{2028}' | '\u{2029}')
        })
    {
        return Err(error("model_display_name_limit"));
    }
    Ok(Some(value))
}

/// Only a previous deliberate auto projection, exact pointer and actual digest
/// can be classified as ours. An external/manual label is never inferred from
/// a string that merely resembles the channel name.
pub(super) fn is_own_projection(
    channel: &ChannelV2,
    provider: &str,
    model: &str,
    actual: Option<&str>,
    ledger: Option<&MaterializationLedgerV2>,
    expected_policy: &str,
) -> Result<bool> {
    if channel
        .naming
        .get(model)
        .and_then(|n| n.display_name.as_ref())
        .is_some_and(|value| !value.is_empty())
    {
        return Ok(false);
    }
    let expected = match generated(channel, provider, model) {
        Ok(Some(value)) => value,
        // An invalid auto label could not have been our successful previous
        // publication. Keep an external valid manual label as manual.
        _ => return Ok(false),
    };
    if actual != Some(expected.as_str()) {
        return Ok(false);
    }
    let pointer = format!(
        "/providers/{}/modelDisplayNames/{}",
        provider.replace('~', "~0").replace('/', "~1"),
        model.replace('~', "~0").replace('/', "~1")
    );
    let Some(ledger) = ledger.filter(|l| l.source_policy_digest == expected_policy) else {
        return Ok(false);
    };
    let entries = ledger
        .fields
        .iter()
        .filter(|e| {
            e.channel_id == channel.id && e.raw_model_id == model && e.field == "displayName"
        })
        .collect::<Vec<_>>();
    if entries.len() != 1 {
        return Ok(false);
    }
    let entry = entries[0];
    Ok(entry.present
        && entry.target_pointer == pointer
        && entry.value_digest == digest(&canonical_bytes(&Value::String(expected))?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn official_utf16_limit_rejects_overflow_and_controls_without_truncation() {
        let channel: ChannelV2 = serde_json::from_value(serde_json::json!({
            "id":"11111111-1111-4111-8111-111111111111","label":"中",
            "connection":{"providerId":"channel-fixture","nodes":[],"activeNodeId":"","credentialRef":null,"enabled":true,"allowPrivateNetwork":false,"authMode":"key","sourceReadId":null,"importedConfig":null},
            "selection":{"mode":"all"},"modelOrder":[],"models":[],"templateRefs":[],"defaults":super::super::schema::ModelRules::default(),
            "familyOverrides":{},"naming":{},"namespace":null
        })).unwrap();
        assert!(generated(&channel, "different", &"a".repeat(124))
            .unwrap()
            .is_some());
        assert!(generated(&channel, "different", &"a".repeat(125)).is_err());
        assert!(generated(&channel, "different", &"😀".repeat(63)).is_err());
        assert!(!is_own_projection(
            &channel,
            "different",
            &"😀".repeat(63),
            Some("人工名称"),
            None,
            "unknown"
        )
        .unwrap());
        assert!(generated(&channel, "different", "model\u{2028}suffix").is_err());
        assert_eq!(generated(&channel, "中", &"a".repeat(1000)).unwrap(), None);
    }
}
