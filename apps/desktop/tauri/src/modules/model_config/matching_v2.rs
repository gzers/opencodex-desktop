//! Data-driven family, numeric-slot and name matching; no IO or implicit adoption.
use super::schema::{ChannelMode, ModelRules, ModelSelection};
use super::schema_v2::*;
use std::collections::{BTreeMap, BTreeSet};

fn folded(s: &str, sensitive: bool) -> String {
    if sensitive {
        s.into()
    } else {
        s.to_lowercase()
    }
}
fn glob(pattern: &str, input: &str) -> bool {
    let p: Vec<_> = pattern.chars().collect();
    let s: Vec<_> = input.chars().collect();
    let (mut i, mut j) = (0, 0);
    let mut star = None;
    let mut retry = 0;
    while j < s.len() {
        if i < p.len() && (p[i] == '?' || p[i] == s[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == '*' {
            star = Some(i);
            i += 1;
            retry = j;
        } else if let Some(k) = star {
            retry += 1;
            j = retry;
            i = k + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == '*' {
        i += 1;
    }
    i == p.len()
}
fn pattern_matches(p: &PatternV2, s: &str, sensitive: bool) -> bool {
    let pvalue = folded(&p.value, sensitive);
    let s = folded(s, sensitive);
    match p.kind {
        PatternKindV2::Exact => s == pvalue,
        PatternKindV2::Prefix => s.starts_with(&pvalue),
        PatternKindV2::Glob => glob(&pvalue, &s),
        PatternKindV2::AsciiNumericPrefix => s
            .strip_prefix(&pvalue)
            .and_then(|s| s.as_bytes().first())
            .is_some_and(u8::is_ascii_digit),
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedNameV2 {
    pub components: Vec<String>,
    pub tail: String,
    pub version_start: usize,
    pub version_end: usize,
}
pub fn parse_name_v2(layout: &NameLayoutV2, input: &str, sensitive: bool) -> Option<ParsedNameV2> {
    let s = folded(input, sensitive);
    let prefix = folded(&layout.literal_prefix, sensitive);
    let body = s.strip_prefix(&prefix)?;
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let mut components: Vec<String> = vec![body[..i].into()];
    let mut joiners = vec![];
    let mut starts = vec![0];
    while i + 1 < bytes.len() && b".-".contains(&bytes[i]) && bytes[i + 1].is_ascii_digit() {
        joiners.push(bytes[i]);
        i += 1;
        let start = i;
        starts.push(start);
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        components.push(body[start..i].into());
    }
    if components.len() >= 3
        && joiners.last() == Some(&b'-')
        && components.last()?.len() == 8
        && chrono::NaiveDate::parse_from_str(components.last()?, "%Y%m%d").is_ok()
    {
        i = starts.pop()? - 1;
        components.pop();
        joiners.pop();
    }
    let tail = body[i..].to_string();
    if tail.starts_with('.')
        || (components.len() >= 3
            && [4, 6].contains(&components.last()?.len())
            && joiners.contains(&b'-'))
    {
        return None;
    }
    Some(ParsedNameV2 {
        components,
        tail,
        version_start: prefix.len(),
        version_end: prefix.len() + i,
    })
}
pub fn selector_matches_v2(
    content: &TemplateContentV2,
    selector: &SelectorV2,
    input: &str,
) -> bool {
    let sensitive = content.matching.case_sensitive;
    let s = folded(input, sensitive);
    let parsed = if let Some(id) = &selector.layout_id {
        let Some(layout) = content.name_layouts.iter().find(|l| &l.id == id) else {
            return false;
        };
        let Some(p) = parse_name_v2(layout, input, sensitive) else {
            return false;
        };
        if selector
            .literal_tail
            .as_ref()
            .is_none_or(|t| folded(t, sensitive) != p.tail)
        {
            return false;
        }
        Some(p)
    } else {
        None
    };
    selector.all.iter().all(|atom| match atom {
        SelectorAtomV2::ExactId { value } => s == folded(value, sensitive),
        SelectorAtomV2::NumericVersion { components } => {
            parsed.as_ref().is_some_and(|p| &p.components == components)
        }
        SelectorAtomV2::LiteralSegment {
            value, position, ..
        } => {
            let value = folded(value, sensitive);
            s.match_indices(&value).any(|(start, _)| {
                let end = start + value.len();
                let bounded = (start == 0 || s.as_bytes()[start - 1] == b'-')
                    && (end == s.len() || s.as_bytes()[end] == b'-');
                bounded
                    && match position {
                        LiteralPositionV2::Any => true,
                        LiteralPositionV2::BeforeVersion => {
                            parsed.as_ref().is_some_and(|p| end <= p.version_start)
                        }
                        LiteralPositionV2::AfterVersion => {
                            parsed.as_ref().is_some_and(|p| start >= p.version_end)
                        }
                    }
            })
        }
    })
}
pub fn validate_content_v2(content: &TemplateContentV2) -> Result<()> {
    let matching = &content.matching;
    if content.description.len() > 32768
        || content.sources.len() > 128
        || matching.include.len() + matching.exclude.len() > 256
        || content.name_layouts.len() > 256
        || content.condition_overrides.len() > 256
    {
        return Err(error("model_limit"));
    }
    if matching.include.is_empty() {
        return Err(error("model_selector_invalid"));
    }
    for p in matching.include.iter().chain(&matching.exclude) {
        if p.value.is_empty() || p.value.len() > 1024 {
            return Err(error("model_selector_invalid"));
        }
    }
    for id in &matching.channels {
        check_id(id)?;
    }
    validate_rules(&content.common_rules, false)?;
    let mut layout_ids = BTreeSet::new();
    for l in &content.name_layouts {
        if l.id.is_empty()
            || !layout_ids.insert(&l.id)
            || l.literal_prefix.is_empty()
            || l.literal_prefix.len() > 1024
            || l.version_capture != "complete_numeric_components"
            || l.numeric_joiners != [".", "-"]
            || l.tail_capture != "literal_remainder"
            || l.trailing_date_policy != "retain_valid_YYYYMMDD_after_hyphen"
            || l.ambiguous_numeric_tail != "no_equivalence"
        {
            return Err(error("model_selector_invalid"));
        }
    }
    let locks: Vec<_> = RULE_KEYS
        .iter()
        .map(|s| format!("rules.{s}"))
        .chain(["description", "matching", "rules", "variants"].map(String::from))
        .collect();
    if content.locked_fields.iter().any(|k| !locks.contains(k)) {
        return Err(error("model_selector_invalid"));
    }
    for src in &content.sources {
        if src.note.len() > 8192
            || src.field.len() > 256
            || chrono::DateTime::parse_from_rfc3339(&src.recorded_at).is_err()
        {
            return Err(error("model_source_invalid"));
        }
        if let Some(url) = &src.url {
            let u = reqwest::Url::parse(url).map_err(|_| error("model_source_invalid"))?;
            if !["http", "https"].contains(&u.scheme())
                || !u.username().is_empty()
                || u.password().is_some()
            {
                return Err(error("model_source_invalid"));
            }
        }
    }
    let mut ids = BTreeSet::new();
    let mut selectors = BTreeSet::new();
    for condition in &content.condition_overrides {
        check_id(&condition.id)?;
        if !ids.insert(&condition.id)
            || condition.scope != "known_identity_only"
            || condition.source_exact_id.is_empty()
            || condition
                .locked_fields
                .iter()
                .any(|k| !RULE_KEYS.contains(&k.as_str()))
        {
            return Err(error("model_selector_invalid"));
        }
        validate_rules(&condition.rules, false)?;
        validate_rules(&condition.rules.overlay(&content.common_rules), false)?;
        let sel = &condition.selector;
        if sel.all.is_empty()
            || sel.all.len() > 32
            || sel.layout_id.is_some() != sel.literal_tail.is_some()
        {
            return Err(error("model_selector_invalid"));
        }
        let layout = sel
            .layout_id
            .as_ref()
            .map(|id| {
                content
                    .name_layouts
                    .iter()
                    .find(|l| &l.id == id)
                    .ok_or_else(|| error("model_selector_invalid"))
            })
            .transpose()?;
        let mut numeric = vec![];
        let mut exact = vec![];
        for atom in &sel.all {
            match atom {
                SelectorAtomV2::NumericVersion { components } => {
                    if layout.is_none()
                        || components.is_empty()
                        || components
                            .iter()
                            .any(|c| c.is_empty() || !c.bytes().all(|b| b.is_ascii_digit()))
                    {
                        return Err(error("model_selector_invalid"));
                    }
                    numeric.push(components);
                }
                SelectorAtomV2::ExactId { value } => {
                    if value.is_empty() || value.len() > 2048 {
                        return Err(error("model_selector_invalid"));
                    }
                    exact.push(folded(value, matching.case_sensitive));
                }
                SelectorAtomV2::LiteralSegment {
                    value,
                    position,
                    boundary,
                } => {
                    if value.is_empty()
                        || value.len() > 1024
                        || boundary != "hyphen_or_string_edge"
                        || (layout.is_none() && *position != LiteralPositionV2::Any)
                    {
                        return Err(error("model_selector_invalid"));
                    }
                }
            }
        }
        if layout.is_some() && numeric.len() != 1 {
            return Err(error("model_selector_invalid"));
        }
        let key = if let Some(layout) = layout {
            canonical_bytes(&(
                folded(&layout.literal_prefix, matching.case_sensitive),
                numeric[0],
                folded(
                    sel.literal_tail.as_deref().unwrap(),
                    matching.case_sensitive,
                ),
            ))?
        } else {
            exact.sort();
            exact.dedup();
            if exact.is_empty() {
                let mut atoms = sel
                    .all
                    .iter()
                    .map(canonical_bytes)
                    .collect::<Result<Vec<_>>>()?;
                atoms.sort();
                atoms.dedup();
                canonical_bytes(&atoms)?
            } else {
                canonical_bytes(&exact)?
            }
        };
        if !selectors.insert(key) {
            return Err(error("model_selector_duplicate"));
        }
    }
    // Exact IDs and pinned source identities are concrete overlap witnesses.
    // General selector overlap is still checked for every actual resolution.
    let witnesses: BTreeSet<_> = content
        .condition_overrides
        .iter()
        .flat_map(|c| {
            std::iter::once(c.source_exact_id.as_str()).chain(c.selector.all.iter().filter_map(
                |a| match a {
                    SelectorAtomV2::ExactId { value } => Some(value.as_str()),
                    _ => None,
                },
            ))
        })
        .collect();
    for witness in witnesses {
        condition_v2(content, witness)?;
    }
    Ok(())
}
pub fn matching_view_v2(channel: &ChannelV2, raw: &str) -> Result<String> {
    if raw.is_empty() || raw.len() > 2048 {
        return Err(error("model_limit"));
    }
    if let Some(ns) = &channel.namespace {
        if ns.channel_id != channel.id
            || ns
                .remove_leading_prefixes
                .iter()
                .any(|p| p.is_empty() || !p.is_ascii())
        {
            return Err(error("model_selector_invalid"));
        }
        let lower = raw.to_ascii_lowercase();
        if let Some(prefix) = ns
            .remove_leading_prefixes
            .iter()
            .filter(|p| lower.starts_with(&p.to_ascii_lowercase()))
            .max_by_key(|p| p.len())
        {
            return Ok(raw[prefix.len()..].into());
        }
    }
    Ok(raw.into())
}
pub fn family_matches_v2(content: &TemplateContentV2, channel_id: &str, view: &str) -> bool {
    let r = &content.matching;
    if match r.channel_mode {
        ChannelMode::All => false,
        ChannelMode::Whitelist => !r.channels.contains(channel_id),
        ChannelMode::Blacklist => r.channels.contains(channel_id),
    } {
        return false;
    }
    let s = folded(view, r.case_sensitive);
    if r.reject_repeated_literal_prefix
        && r.include
            .iter()
            .filter(|p| p.kind == PatternKindV2::Prefix)
            .any(|p| {
                let p = folded(&p.value, r.case_sensitive);
                s.starts_with(&format!("{p}{p}"))
            })
    {
        return false;
    }
    !r.exclude
        .iter()
        .any(|p| pattern_matches(p, view, r.case_sensitive))
        && r.include
            .iter()
            .any(|p| pattern_matches(p, view, r.case_sensitive))
}
pub fn condition_v2<'a>(
    content: &'a TemplateContentV2,
    view: &str,
) -> Result<Option<&'a ConditionOverrideV2>> {
    let hits: Vec<_> = content
        .condition_overrides
        .iter()
        .filter(|c| selector_matches_v2(content, &c.selector, view))
        .collect();
    if hits.len() > 1 {
        let mut e = error("model_condition_overlap");
        e.issues[0].condition_ids = hits.iter().map(|c| c.id.clone()).collect();
        return Err(e);
    }
    Ok(hits.first().copied())
}
pub fn select_reference_v2(
    library: &LibraryV2,
    policy: &PolicyV2,
    channel: &ChannelV2,
    raw: &str,
) -> Result<Option<RefV2>> {
    let view = matching_view_v2(channel, raw)?;
    let mut refs = BTreeMap::new();
    for r in policy.template_refs.iter().chain(&channel.template_refs) {
        refs.insert(r.template_id.clone(), r.clone());
    }
    for template in &library.templates {
        if !refs.contains_key(&template.id) {
            if let Some(id) = &template.default_revision {
                let r = template
                    .revisions
                    .iter()
                    .find(|r| &r.id == id)
                    .ok_or_else(|| error("model_dependency_missing"))?;
                refs.insert(
                    template.id.clone(),
                    RefV2 {
                        template_id: template.id.clone(),
                        revision_id: r.id.clone(),
                        digest: r.digest.clone(),
                    },
                );
            }
        }
    }
    let mut hits = vec![];
    for reference in refs.values() {
        let r = revision(library, reference)?;
        if family_matches_v2(&r.content, &channel.id, &view) {
            hits.push((r.content.matching.priority, reference.clone()));
        }
    }
    let Some(priority) = hits.iter().map(|(p, _)| *p).max() else {
        return Ok(None);
    };
    let winners: Vec<_> = hits.into_iter().filter(|(p, _)| *p == priority).collect();
    if winners.len() > 1 {
        return Err(error("model_matching_conflict"));
    }
    Ok(Some(winners[0].1.clone()))
}
pub fn resolve_effective_model_v2(
    library: &LibraryV2,
    channel: &ChannelV2,
    model: &ModelConfigV2,
) -> Result<EffectiveModelV2> {
    validate_non_protocol(&channel.defaults)?;
    for rules in channel.family_overrides.values() {
        validate_non_protocol(rules)?;
    }
    if model.raw_model_id != model.source.raw_model_id {
        return Err(error("model_source_invalid"));
    }
    let mut rules = super::source_v2::source_rules_v2(&model.source)?;
    let mut origins = BTreeMap::new();
    for (key, field) in &model.source.fields {
        origins.insert(
            key.clone(),
            FieldOriginV2::Source {
                evidence: field.evidence.clone(),
            },
        );
    }
    fn overlay(
        rules: &mut ModelRules,
        origins: &mut BTreeMap<String, FieldOriginV2>,
        higher: &ModelRules,
        origin: FieldOriginV2,
    ) -> Result<()> {
        for key in rules_values(higher)?.keys() {
            origins.insert(key.clone(), origin.clone());
        }
        *rules = higher.overlay(rules);
        Ok(())
    }
    let mut matched_condition_id = None;
    if let Some(binding) = &model.binding {
        let reference = binding.reference();
        let r = revision(library, reference)?;
        overlay(
            &mut rules,
            &mut origins,
            &r.content.common_rules,
            FieldOriginV2::TemplateCommon {
                reference: reference.clone(),
            },
        )?;
        if let Some(c) = condition_v2(&r.content, &matching_view_v2(channel, &model.raw_model_id)?)?
        {
            matched_condition_id = Some(c.id.clone());
            overlay(
                &mut rules,
                &mut origins,
                &c.rules,
                FieldOriginV2::TemplateCondition {
                    reference: reference.clone(),
                    condition_id: c.id.clone(),
                },
            )?;
        }
    }
    overlay(
        &mut rules,
        &mut origins,
        &channel.defaults,
        FieldOriginV2::ChannelDefault,
    )?;
    if let Some(binding) = &model.binding {
        if let Some(value) = channel
            .family_overrides
            .get(&binding.reference().template_id)
        {
            overlay(
                &mut rules,
                &mut origins,
                value,
                FieldOriginV2::ChannelFamily {
                    template_id: binding.reference().template_id.clone(),
                },
            )?;
        }
    }
    validate_rules(&model.explicit_overrides, false)?;
    overlay(
        &mut rules,
        &mut origins,
        &model.explicit_overrides,
        FieldOriginV2::ExplicitModel,
    )?;
    validate_rules(&rules, true)?;
    let origin = origins
        .get("protocol")
        .cloned()
        .unwrap_or(FieldOriginV2::Unknown);
    let protocol = match &rules.protocol {
        Field::Inherit => ProtocolStateV2::Missing,
        Field::Custom(value) => match ProtocolV2::parse(value) {
            Some(value) => ProtocolStateV2::Resolved { value, origin },
            None => ProtocolStateV2::Unsupported {
                value: value.clone(),
                origin,
            },
        },
    };
    let selected = match &channel.selection {
        ModelSelection::All => model.observed,
        ModelSelection::None => false,
        ModelSelection::Whitelist(ids) => ids.contains(&model.raw_model_id),
    };
    Ok(EffectiveModelV2 {
        channel_id: channel.id.clone(),
        raw_model_id: model.raw_model_id.clone(),
        selected,
        enabled: channel.connection.enabled,
        observed: model.observed,
        binding: model.binding.clone(),
        rules,
        protocol,
        field_origins: origins,
        matched_condition_id,
        notices: vec![],
    })
}
