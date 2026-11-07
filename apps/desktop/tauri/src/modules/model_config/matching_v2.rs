//! Data-driven family, numeric-slot and name matching; no IO or implicit adoption.
use super::schema::{ChannelMode, ModelRules, ModelSelection};
use super::schema_v2::*;
use std::collections::{BTreeMap, BTreeSet};

/// Stable explanations emitted only by the same predicates used for matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchReasonV2 {
    FamilyMatched,
    ChannelScopeMismatch,
    RepeatedLiteralPrefix,
    ExcludedPattern,
    NoIncludedPattern,
    FamilyNotApplicable,
    LayoutMatched,
    LayoutMissing,
    LayoutParseFailed,
    LiteralTailMatched,
    LiteralTailMismatch,
    SelectorPreconditionFailed,
    ExactIdMatched,
    ExactIdMismatch,
    NumericComponentsMatched,
    NumericComponentsMismatch,
    NumericRequiresLayout,
    LiteralSegmentMatched,
    LiteralSegmentMissing,
    SegmentBoundaryMismatch,
    PositionRequiresLayout,
    SegmentPositionMismatch,
    ConditionMatched,
    ConditionAndFailed,
    NoConditionMatched,
    NoConditions,
}
impl MatchReasonV2 {
    pub fn text(self) -> &'static str {
        match self {
            Self::FamilyMatched => "命中族包含范围，且未被渠道范围、重复前缀或排除规则拒绝",
            Self::ChannelScopeMismatch => "当前渠道不在此模版允许的 UUID 范围内",
            Self::RepeatedLiteralPrefix => "模型 ID 以重复的已声明字面族前缀开头",
            Self::ExcludedPattern => "命中排除规则；排除优先于包含",
            Self::NoIncludedPattern => "未命中任何族包含规则",
            Self::FamilyNotApplicable => "族范围未命中，此条件及原子不参与采用",
            Self::LayoutMatched => "名称结构解析成功，保留完整数字段和精确剩余文本",
            Self::LayoutMissing => "所引用的名称结构不存在",
            Self::LayoutParseFailed => "名称不符合声明前缀和完整数字版本结构，或数字尾部存在歧义",
            Self::LiteralTailMatched => "结构后的精确剩余文本与声明相同",
            Self::LiteralTailMismatch => "结构后的精确剩余文本或规格与声明不同",
            Self::SelectorPreconditionFailed => "名称结构或精确尾部前置条件失败；原子未执行",
            Self::ExactIdMatched => "完整模型 ID 按声明的大小写规则相同",
            Self::ExactIdMismatch => "完整模型 ID 不同",
            Self::NumericComponentsMatched => "声明数字槽位的完整数字段相同",
            Self::NumericComponentsMismatch => "完整数字段不同；段数、前导零和构建数字不自动折叠",
            Self::NumericRequiresLayout => "数字版本条件没有可用的名称结构",
            Self::LiteralSegmentMatched => "完整名称片段的边界与声明位置均满足",
            Self::LiteralSegmentMissing => "模型 ID 中没有此字面名称片段",
            Self::SegmentBoundaryMismatch => "仅出现子串，未形成连字符或字符串边界包围的完整片段",
            Self::PositionRequiresLayout => "版本前或版本后的位置条件需要可用的名称结构",
            Self::SegmentPositionMismatch => "完整名称片段存在，但不在声明的版本前或版本后位置",
            Self::ConditionMatched => "此覆盖项的全部前置条件及 AND 原子均满足",
            Self::ConditionAndFailed => "前置条件通过，但至少一个 AND 原子不满足",
            Self::NoConditionMatched => "命中族范围但没有条件覆盖命中，使用族通用值",
            Self::NoConditions => "命中族范围，模版未声明条件覆盖，使用族通用值",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreconditionKindV2 {
    Layout,
    LiteralTail,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AtomKindV2 {
    ExactId,
    NumericVersion,
    LiteralSegment,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PredicateTraceV2 {
    pub kind: PreconditionKindV2,
    pub matched: Option<bool>,
    pub reason_code: MatchReasonV2,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AtomTraceV2 {
    pub index: usize,
    pub kind: AtomKindV2,
    pub matched: Option<bool>,
    pub reason_code: MatchReasonV2,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FamilyTraceV2 {
    pub matched: bool,
    pub reason_code: MatchReasonV2,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConditionMatchTraceV2 {
    pub condition_id: String,
    pub applicable: bool,
    pub matched: Option<bool>,
    pub applied: bool,
    pub reason_code: MatchReasonV2,
    pub reason: String,
    pub preconditions: Vec<PredicateTraceV2>,
    pub atoms: Vec<AtomTraceV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MatchingTraceV2 {
    pub matching_view: String,
    pub family: FamilyTraceV2,
    pub reason_code: MatchReasonV2,
    pub reason: String,
    pub conditions: Vec<ConditionMatchTraceV2>,
}
#[derive(Clone, Copy)]
struct Check {
    matched: bool,
    reason: MatchReasonV2,
}
impl Check {
    fn decide(matched: bool, yes: MatchReasonV2, no: MatchReasonV2) -> Self {
        Self {
            matched,
            reason: if matched { yes } else { no },
        }
    }
    fn fail(reason: MatchReasonV2) -> Self {
        Self {
            matched: false,
            reason,
        }
    }
}
fn atom_kind(atom: &SelectorAtomV2) -> AtomKindV2 {
    match atom {
        SelectorAtomV2::ExactId { .. } => AtomKindV2::ExactId,
        SelectorAtomV2::NumericVersion { .. } => AtomKindV2::NumericVersion,
        SelectorAtomV2::LiteralSegment { .. } => AtomKindV2::LiteralSegment,
    }
}
struct SelectorTrace {
    preconditions: Vec<PredicateTraceV2>,
    atoms: Vec<AtomTraceV2>,
}
impl SelectorTrace {
    fn precondition(
        &mut self,
        kind: PreconditionKindV2,
        check: Option<Check>,
        skipped: MatchReasonV2,
    ) {
        let reason = check.map_or(skipped, |c| c.reason);
        self.preconditions.push(PredicateTraceV2 {
            kind,
            matched: check.map(|c| c.matched),
            reason_code: reason,
            reason: reason.text().into(),
        });
    }
    fn atom(
        &mut self,
        index: usize,
        atom: &SelectorAtomV2,
        check: Option<Check>,
        skipped: MatchReasonV2,
    ) {
        let reason = check.map_or(skipped, |c| c.reason);
        self.atoms.push(AtomTraceV2 {
            index,
            kind: atom_kind(atom),
            matched: check.map(|c| c.matched),
            reason_code: reason,
            reason: reason.text().into(),
        });
    }
    fn skipped_atoms(&mut self, selector: &SelectorV2, reason: MatchReasonV2) {
        for (index, atom) in selector.all.iter().enumerate() {
            self.atom(index, atom, None, reason);
        }
    }
}
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
fn atom_check(
    atom: &SelectorAtomV2,
    s: &str,
    sensitive: bool,
    parsed: Option<&ParsedNameV2>,
) -> Check {
    use MatchReasonV2 as R;
    match atom {
        SelectorAtomV2::ExactId { value } => Check::decide(
            s == folded(value, sensitive),
            R::ExactIdMatched,
            R::ExactIdMismatch,
        ),
        SelectorAtomV2::NumericVersion { components } => match parsed {
            Some(p) => Check::decide(
                &p.components == components,
                R::NumericComponentsMatched,
                R::NumericComponentsMismatch,
            ),
            None => Check::fail(R::NumericRequiresLayout),
        },
        SelectorAtomV2::LiteralSegment {
            value, position, ..
        } => {
            let value = folded(value, sensitive);
            let mut found = false;
            let mut bounded_found = false;
            for (start, _) in s.match_indices(&value) {
                found = true;
                let end = start + value.len();
                let bounded = (start == 0 || s.as_bytes()[start - 1] == b'-')
                    && (end == s.len() || s.as_bytes()[end] == b'-');
                if !bounded {
                    continue;
                }
                bounded_found = true;
                let positioned = match position {
                    LiteralPositionV2::Any => true,
                    LiteralPositionV2::BeforeVersion => {
                        parsed.is_some_and(|p| end <= p.version_start)
                    }
                    LiteralPositionV2::AfterVersion => {
                        parsed.is_some_and(|p| start >= p.version_end)
                    }
                };
                if positioned {
                    return Check {
                        matched: true,
                        reason: R::LiteralSegmentMatched,
                    };
                }
            }
            Check::fail(if !found {
                R::LiteralSegmentMissing
            } else if !bounded_found {
                R::SegmentBoundaryMismatch
            } else if parsed.is_none() && *position != LiteralPositionV2::Any {
                R::PositionRequiresLayout
            } else {
                R::SegmentPositionMismatch
            })
        }
    }
}
/// Both normal matching and trace use this evaluator. Normal callers allocate
/// no trace and retain short-circuit behavior; trace evaluates all AND atoms.
fn selector_check(
    content: &TemplateContentV2,
    selector: &SelectorV2,
    input: &str,
    mut trace: Option<&mut SelectorTrace>,
) -> Check {
    use MatchReasonV2 as R;
    let sensitive = content.matching.case_sensitive;
    let s = folded(input, sensitive);
    let parsed = if let Some(id) = &selector.layout_id {
        let parsed = content
            .name_layouts
            .iter()
            .find(|l| &l.id == id)
            .ok_or(R::LayoutMissing)
            .and_then(|l| parse_name_v2(l, input, sensitive).ok_or(R::LayoutParseFailed));
        let parsed = match parsed {
            Ok(p) => {
                if let Some(t) = trace.as_mut() {
                    t.precondition(
                        PreconditionKindV2::Layout,
                        Some(Check {
                            matched: true,
                            reason: R::LayoutMatched,
                        }),
                        R::SelectorPreconditionFailed,
                    );
                }
                p
            }
            Err(reason) => {
                if let Some(t) = trace.as_mut() {
                    t.precondition(
                        PreconditionKindV2::Layout,
                        Some(Check::fail(reason)),
                        reason,
                    );
                    t.precondition(
                        PreconditionKindV2::LiteralTail,
                        None,
                        R::SelectorPreconditionFailed,
                    );
                    t.skipped_atoms(selector, R::SelectorPreconditionFailed);
                }
                return Check::fail(reason);
            }
        };
        let tail = Check::decide(
            selector
                .literal_tail
                .as_ref()
                .is_some_and(|t| folded(t, sensitive) == parsed.tail),
            R::LiteralTailMatched,
            R::LiteralTailMismatch,
        );
        if let Some(t) = trace.as_mut() {
            t.precondition(
                PreconditionKindV2::LiteralTail,
                Some(tail),
                R::SelectorPreconditionFailed,
            );
        }
        if !tail.matched {
            if let Some(t) = trace.as_mut() {
                t.skipped_atoms(selector, R::SelectorPreconditionFailed);
            }
            return tail;
        }
        Some(parsed)
    } else {
        None
    };
    let mut matched = true;
    for (index, atom) in selector.all.iter().enumerate() {
        let check = atom_check(atom, &s, sensitive, parsed.as_ref());
        if let Some(t) = trace.as_mut() {
            t.atom(index, atom, Some(check), R::SelectorPreconditionFailed);
        }
        matched &= check.matched;
        if !matched && trace.is_none() {
            break;
        }
    }
    Check::decide(matched, R::ConditionMatched, R::ConditionAndFailed)
}
pub fn selector_matches_v2(
    content: &TemplateContentV2,
    selector: &SelectorV2,
    input: &str,
) -> bool {
    selector_check(content, selector, input, None).matched
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
fn family_check(content: &TemplateContentV2, channel_id: &str, view: &str) -> Check {
    use MatchReasonV2 as R;
    let r = &content.matching;
    if match r.channel_mode {
        ChannelMode::All => false,
        ChannelMode::Whitelist => !r.channels.contains(channel_id),
        ChannelMode::Blacklist => r.channels.contains(channel_id),
    } {
        return Check::fail(R::ChannelScopeMismatch);
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
        return Check::fail(R::RepeatedLiteralPrefix);
    }
    if r.exclude
        .iter()
        .any(|p| pattern_matches(p, view, r.case_sensitive))
    {
        return Check::fail(R::ExcludedPattern);
    }
    Check::decide(
        r.include
            .iter()
            .any(|p| pattern_matches(p, view, r.case_sensitive)),
        R::FamilyMatched,
        R::NoIncludedPattern,
    )
}
pub fn family_matches_v2(content: &TemplateContentV2, channel_id: &str, view: &str) -> bool {
    family_check(content, channel_id, view).matched
}
/// Explain actual predicate outcomes without selecting or saving a revision.
/// Namespace processing is exactly the same matching view used by resolve.
pub fn trace_matching_v2(
    content: &TemplateContentV2,
    channel: &ChannelV2,
    raw: &str,
) -> Result<MatchingTraceV2> {
    use MatchReasonV2 as R;
    let view = matching_view_v2(channel, raw)?;
    let family = family_check(content, &channel.id, &view);
    let mut conditions = Vec::with_capacity(content.condition_overrides.len());
    for condition in &content.condition_overrides {
        let mut trace = SelectorTrace {
            preconditions: vec![],
            atoms: Vec::with_capacity(condition.selector.all.len()),
        };
        let check = if family.matched {
            Some(selector_check(
                content,
                &condition.selector,
                &view,
                Some(&mut trace),
            ))
        } else {
            if condition.selector.layout_id.is_some() {
                trace.precondition(PreconditionKindV2::Layout, None, R::FamilyNotApplicable);
                trace.precondition(
                    PreconditionKindV2::LiteralTail,
                    None,
                    R::FamilyNotApplicable,
                );
            }
            trace.skipped_atoms(&condition.selector, R::FamilyNotApplicable);
            None
        };
        let reason = check.map_or(R::FamilyNotApplicable, |c| c.reason);
        conditions.push(ConditionMatchTraceV2 {
            condition_id: condition.id.clone(),
            applicable: family.matched,
            matched: check.map(|c| c.matched),
            applied: check.is_some_and(|c| c.matched),
            reason_code: reason,
            reason: reason.text().into(),
            preconditions: trace.preconditions,
            atoms: trace.atoms,
        });
    }
    let hits = conditions
        .iter()
        .filter(|c| c.applied)
        .map(|c| c.condition_id.clone())
        .collect::<Vec<_>>();
    if hits.len() > 1 {
        let mut e = error("model_condition_overlap");
        e.issues[0].condition_ids = hits;
        return Err(e);
    }
    let reason = if !family.matched {
        family.reason
    } else if !hits.is_empty() {
        R::ConditionMatched
    } else if conditions.is_empty() {
        R::NoConditions
    } else {
        R::NoConditionMatched
    };
    Ok(MatchingTraceV2 {
        matching_view: view,
        family: FamilyTraceV2 {
            matched: family.matched,
            reason_code: family.reason,
            reason: family.reason.text().into(),
        },
        reason_code: reason,
        reason: reason.text().into(),
        conditions,
    })
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

#[cfg(test)]
#[path = "matching_v2_trace_tests.rs"]
mod trace_tests;
