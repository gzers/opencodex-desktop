use super::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn folded(value: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        value.into()
    } else {
        value.to_lowercase()
    }
}

/// Literal glob: only '*' and '?' have special meanings. No regex execution.
pub fn glob(pattern: &str, input: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = input.chars().collect();
    let (mut pi, mut si) = (0, 0);
    let mut star = None;
    let mut retry = 0;
    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            pi += 1;
            retry = si;
        } else if let Some(index) = star {
            retry += 1;
            si = retry;
            pi = index + 1;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}
fn matches(pattern: &Pattern, input: &str, case_sensitive: bool) -> bool {
    let value = folded(&pattern.value, case_sensitive);
    let input = folded(input, case_sensitive);
    match pattern.kind {
        MatchKind::Exact => input == value,
        MatchKind::Prefix => input.starts_with(&value),
        MatchKind::Glob => glob(&value, &input),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchPreview {
    pub matched: bool,
    pub reason: String,
    pub variant_id: Option<String>,
    pub rules: ModelRules,
}
pub fn preview(content: &TemplateContent, channel: &str, model: &str) -> ModelResult<MatchPreview> {
    content.validate()?;
    if model.len() > 2048 {
        return Err(invalid("limit", "模型ID过长"));
    }
    let allowed = match content.matching.channel_mode {
        ChannelMode::All => true,
        ChannelMode::Whitelist => content.matching.channels.contains(channel),
        ChannelMode::Blacklist => !content.matching.channels.contains(channel),
    };
    let reason = if !allowed {
        "不在渠道范围"
    } else if content
        .matching
        .exclude
        .iter()
        .any(|p| matches(p, model, content.matching.case_sensitive))
    {
        "命中排除规则"
    } else if !content
        .matching
        .include
        .iter()
        .any(|p| matches(p, model, content.matching.case_sensitive))
    {
        "未命中包含规则"
    } else {
        "已匹配"
    };
    if reason != "已匹配" {
        return Ok(MatchPreview {
            matched: false,
            reason: reason.into(),
            variant_id: None,
            rules: content.rules.clone(),
        });
    }
    let (variant_id, rules) = composed(content, model)?;
    Ok(MatchPreview {
        matched: true,
        reason: reason.into(),
        variant_id,
        rules,
    })
}
pub(super) fn composed(
    content: &TemplateContent,
    model: &str,
) -> ModelResult<(Option<String>, ModelRules)> {
    let model = folded(model, content.matching.case_sensitive);
    let variant = content
        .variants
        .iter()
        .filter(|v| model.ends_with(&folded(&v.suffix, content.matching.case_sensitive)))
        .max_by_key(|v| v.suffix.len());
    let rules = variant
        .map(|v| v.rules.overlay(&content.rules))
        .unwrap_or_else(|| content.rules.clone());
    rules.validate()?;
    Ok((variant.map(|v| v.id.clone()), rules))
}
pub fn revision<'a>(library: &'a Library, reference: &TemplateRef) -> ModelResult<&'a Revision> {
    library
        .templates
        .iter()
        .find(|t| t.id == reference.template_id)
        .and_then(|t| t.revisions.iter().find(|r| r.id == reference.revision_id))
        .ok_or_else(|| invalid("missing_dependency", "缺少精确采用的模版版本"))
}

/// Policy selection is authoritative: candidates/defaults never enter the resolver.
pub fn resolve(
    library: &Library,
    policy: &Policy,
    channel: &str,
    model: &str,
) -> ModelResult<Option<(TemplateRef, ModelRules)>> {
    let local = policy.channels.iter().find(|c| c.id == channel);
    if let Some(reference) = local.and_then(|c| c.model_refs.get(model)) {
        let value = revision(library, reference)?;
        let (_, rules) = composed(&value.content, model)?;
        return Ok(Some((reference.clone(), rules)));
    }
    let mut adopted = BTreeMap::new();
    for reference in &policy.template_refs {
        adopted.insert(reference.template_id.clone(), reference);
    }
    if let Some(local) = local {
        for reference in &local.template_refs {
            adopted.insert(reference.template_id.clone(), reference);
        }
    }
    let mut highest = None;
    let mut result = None;
    let mut conflict = false;
    for reference in adopted.values() {
        let value = revision(library, reference)?;
        let matched = preview(&value.content, channel, model)?;
        if !matched.matched {
            continue;
        }
        let priority = value.content.matching.priority;
        match highest {
            Some(previous) if priority < previous => {}
            Some(previous) if priority == previous => {
                conflict = true;
            }
            _ => {
                highest = Some(priority);
                conflict = false;
                result = Some(((*reference).clone(), matched.rules));
            }
        }
    }
    if conflict {
        return Err(invalid(
            "matching_conflict",
            "多个已采用模版以相同最高优先级匹配，请明确选择",
        ));
    }
    Ok(result)
}
