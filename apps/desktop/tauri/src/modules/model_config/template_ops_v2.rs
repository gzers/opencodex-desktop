//! Pure exact-revision library operations. The caller owns snapshots, locks and
//! atomic library/version writes. No file/history discovery or implicit latest.
use super::{
    matching_v2,
    schema_v2::*,
    source_v2::{policy_semantic_digest_v2, validate_policy_v2},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
const MAX_USAGE: usize = 20_000;
fn fail(code: &str, action: &str) -> MatchErrorV2 {
    let mut err = error(code);
    err.issues[0].recovery_actions = vec![action.into()];
    err
}
fn check_expected(library: &LibraryV2, expected: &str) -> Result<()> {
    validate_library_v2(library)?;
    if digest(&canonical_bytes(library)?) != expected {
        return Err(error("model_save_stale"));
    }
    Ok(())
}
fn exact(library: &LibraryV2, template_id: &str, revision_id: &str) -> Result<RefV2> {
    check_id(template_id)?;
    check_id(revision_id)?;
    let r = library
        .templates
        .iter()
        .find(|t| t.id == template_id)
        .and_then(|t| t.revisions.iter().find(|r| r.id == revision_id))
        .ok_or_else(|| error("model_dependency_missing"))?;
    Ok(RefV2 {
        template_id: template_id.into(),
        revision_id: revision_id.into(),
        digest: r.digest.clone(),
    })
}
pub fn set_default(
    library: &LibraryV2,
    expected_library: &str,
    reference: &RefV2,
) -> Result<LibraryV2> {
    check_expected(library, expected_library)?;
    revision(library, reference)?;
    let mut next = library.clone();
    next.templates
        .iter_mut()
        .find(|t| t.id == reference.template_id)
        .unwrap()
        .default_revision = Some(reference.revision_id.clone());
    Ok(next)
}
/// Cancellation names the exact currently selected default; it cannot clear a
/// different default merely because another saved revision was highlighted.
pub fn clear_default(
    library: &LibraryV2,
    expected_library: &str,
    reference: &RefV2,
) -> Result<LibraryV2> {
    check_expected(library, expected_library)?;
    revision(library, reference)?;
    let mut next = library.clone();
    let target = next
        .templates
        .iter_mut()
        .find(|t| t.id == reference.template_id)
        .unwrap();
    if target
        .default_revision
        .as_ref()
        .is_some_and(|id| id != &reference.revision_id)
    {
        return Err(fail("model_template_default_changed", "refresh_and_retry"));
    }
    target.default_revision = None;
    Ok(next)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageStateV2 {
    Saved,
    Draft,
    Additional,
    Library,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UsageKindV2 {
    GlobalReference,
    ChannelReference,
    Manual,
    AutoRead,
    AdditionalExact,
    BaseRevision,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RevisionUseV2 {
    pub state: UsageStateV2,
    pub kind: UsageKindV2,
    pub reference: RefV2,
    pub channel_id: Option<String>,
    pub raw_model_id: Option<String>,
    pub dependent_revision: Option<RefV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageModelV2 {
    pub state: UsageStateV2,
    pub channel_id: String,
    pub channel_label: String,
    pub raw_model_id: String,
    pub matched: bool,
    pub matched_condition_id: Option<String>,
    pub binding_kind: Option<UsageKindV2>,
    pub using_queried_revision: bool,
    pub issues: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageReportV2 {
    pub dto_version: u32,
    pub reference: RefV2,
    pub library_digest: String,
    pub saved_policy_digest: String,
    pub draft_policy_digest: Option<String>,
    pub is_default: bool,
    pub builtin_initial: bool,
    pub references: Vec<RevisionUseV2>,
    pub models: Vec<UsageModelV2>,
}
fn builtin_initial(template_id: &str, revision_id: &str) -> Result<bool> {
    Ok(seed_library_v2()?
        .templates
        .iter()
        .any(|t| t.id == template_id && t.revisions.iter().any(|r| r.id == revision_id)))
}
pub fn usage(
    library: &LibraryV2,
    saved_policy: &PolicyV2,
    draft_policy: Option<&PolicyV2>,
    template_id: &str,
    revision_id: &str,
    additional_exact_refs: &[RefV2],
) -> Result<UsageReportV2> {
    validate_library_v2(library)?;
    validate_policy_v2(saved_policy)?;
    if let Some(draft) = draft_policy {
        validate_policy_v2(draft)?;
    }
    if additional_exact_refs.len() > MAX_USAGE {
        return Err(error("model_limit"));
    }
    let reference = exact(library, template_id, revision_id)?;
    let target = revision(library, &reference)?;
    let mut uses = vec![];
    let mut models = vec![];
    let states = std::iter::once((UsageStateV2::Saved, saved_policy)).chain(
        draft_policy
            .filter(|p| *p != saved_policy)
            .map(|p| (UsageStateV2::Draft, p)),
    );
    for (state, policy) in states {
        for r in &policy.template_refs {
            revision(library, r)?;
            if r == &reference {
                uses.push(RevisionUseV2 {
                    state: state.clone(),
                    kind: UsageKindV2::GlobalReference,
                    reference: r.clone(),
                    channel_id: None,
                    raw_model_id: None,
                    dependent_revision: None,
                });
            }
        }
        for channel in &policy.channels {
            for r in &channel.template_refs {
                revision(library, r)?;
                if r == &reference {
                    uses.push(RevisionUseV2 {
                        state: state.clone(),
                        kind: UsageKindV2::ChannelReference,
                        reference: r.clone(),
                        channel_id: Some(channel.id.clone()),
                        raw_model_id: None,
                        dependent_revision: None,
                    });
                }
            }
            for model in &channel.models {
                if models.len() >= MAX_USAGE || uses.len() >= MAX_USAGE {
                    return Err(error("model_limit"));
                }
                let kind = model.binding.as_ref().map(|binding| match binding {
                    ModelBindingV2::Manual { .. } => UsageKindV2::Manual,
                    ModelBindingV2::AutoRead { .. } => UsageKindV2::AutoRead,
                });
                let used = if let Some(binding) = &model.binding {
                    revision(library, binding.reference())?;
                    binding.reference() == &reference
                } else {
                    false
                };
                if used {
                    uses.push(RevisionUseV2 {
                        state: state.clone(),
                        kind: kind.clone().unwrap(),
                        reference: reference.clone(),
                        channel_id: Some(channel.id.clone()),
                        raw_model_id: Some(model.raw_model_id.clone()),
                        dependent_revision: None,
                    });
                }
                let view = matching_v2::matching_view_v2(channel, &model.raw_model_id)?;
                let matched = matching_v2::family_matches_v2(&target.content, &channel.id, &view);
                let (matched_condition_id, issues) = if matched {
                    match matching_v2::condition_v2(&target.content, &view) {
                        Ok(condition) => (condition.map(|c| c.id.clone()), vec![]),
                        Err(err) => (None, at(err, &channel.id, &model.raw_model_id).issues),
                    }
                } else {
                    (None, vec![])
                };
                models.push(UsageModelV2 {
                    state: state.clone(),
                    channel_id: channel.id.clone(),
                    channel_label: channel.label.clone(),
                    raw_model_id: model.raw_model_id.clone(),
                    matched,
                    matched_condition_id,
                    binding_kind: kind,
                    using_queried_revision: used,
                    issues,
                });
            }
        }
    }
    for r in additional_exact_refs {
        revision(library, r)?;
        if r == &reference {
            uses.push(RevisionUseV2 {
                state: UsageStateV2::Additional,
                kind: UsageKindV2::AdditionalExact,
                reference: r.clone(),
                channel_id: None,
                raw_model_id: None,
                dependent_revision: None,
            });
        }
    }
    let template = library
        .templates
        .iter()
        .find(|t| t.id == template_id)
        .unwrap();
    for r in &template.revisions {
        if r.base_revision.as_deref() == Some(revision_id) {
            uses.push(RevisionUseV2 {
                state: UsageStateV2::Library,
                kind: UsageKindV2::BaseRevision,
                reference: reference.clone(),
                channel_id: None,
                raw_model_id: None,
                dependent_revision: Some(RefV2 {
                    template_id: template_id.into(),
                    revision_id: r.id.clone(),
                    digest: r.digest.clone(),
                }),
            });
        }
    }
    if uses.len() > MAX_USAGE {
        return Err(error("model_limit"));
    }
    Ok(UsageReportV2 {
        dto_version: 2,
        reference,
        library_digest: digest(&canonical_bytes(library)?),
        saved_policy_digest: policy_semantic_digest_v2(saved_policy)?,
        draft_policy_digest: draft_policy.map(policy_semantic_digest_v2).transpose()?,
        is_default: template.default_revision.as_deref() == Some(revision_id),
        builtin_initial: builtin_initial(template_id, revision_id)?,
        references: uses,
        models,
    })
}
/// All dependencies are passed as exact snapshots by L0 under its transaction.
/// Explicit dependency snapshots avoid a hidden or stale file/history reader.
pub fn delete_revisions(
    library: &LibraryV2,
    expected_library: &str,
    template_id: &str,
    revision_ids: &[String],
    saved_policy: &PolicyV2,
    draft_policy: Option<&PolicyV2>,
    additional_exact_refs: &[RefV2],
) -> Result<LibraryV2> {
    check_expected(library, expected_library)?;
    let ids: BTreeSet<_> = revision_ids.iter().collect();
    if ids.is_empty() || ids.len() != revision_ids.len() || ids.len() > 4096 {
        return Err(fail("model_revision_selection_invalid", "select_revisions"));
    }
    for id in &ids {
        let report = usage(
            library,
            saved_policy,
            draft_policy,
            template_id,
            id,
            additional_exact_refs,
        )?;
        if report.builtin_initial {
            return Err(fail(
                "model_revision_builtin_protected",
                "create_new_revision",
            ));
        }
        if report.is_default {
            return Err(fail("model_revision_is_default", "clear_or_change_default"));
        }
        if report
            .references
            .iter()
            .any(|use_| match (&use_.kind, &use_.dependent_revision) {
                (UsageKindV2::BaseRevision, Some(dependent)) => {
                    !ids.contains(&dependent.revision_id)
                }
                _ => true,
            })
        {
            return Err(fail("model_revision_in_use", "remove_exact_dependencies"));
        }
    }
    let mut next = library.clone();
    next.templates
        .iter_mut()
        .find(|t| t.id == template_id)
        .ok_or_else(|| error("model_dependency_missing"))?
        .revisions
        .retain(|r| !ids.contains(&r.id));
    next.templates.retain(|t| !t.revisions.is_empty());
    validate_library_v2(&next)?;
    Ok(next)
}
#[cfg(test)]
#[path = "template_ops_v2_tests.rs"]
mod tests;
