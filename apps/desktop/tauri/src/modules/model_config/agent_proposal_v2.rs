//! Strict v2 Agent proposal boundary. All identities, time, authority and save
//! origin come from trusted backend context; no network, model, CLI or IO here.
use super::{schema::SaveOrigin, schema_v2::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
const MAX_PROPOSALS: usize = 32;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentProposalV2 {
    pub target: Option<RefV2>,
    pub label: String,
    pub content_version: u32,
    pub content: TemplateContentV2,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentProposalV2 {
    pub contract_version: u32,
    pub expected_library: String,
    pub proposals: Vec<ContentProposalV2>,
}
/// Not deserializable: construct from the authorized executor/task state, never
/// from the model response, browser DTO, proposal content or prompt text.
#[derive(Debug, Clone)]
pub enum TrustedAgentOriginV2 {
    InternalAgent { task_id: String },
    ExternalTask { task_id: String },
}
impl TrustedAgentOriginV2 {
    fn save_origin(&self) -> Result<SaveOrigin> {
        match self {
            Self::InternalAgent { task_id } => {
                check_id(task_id)?;
                Ok(SaveOrigin::InternalAgent(task_id.clone()))
            }
            Self::ExternalTask { task_id } => {
                check_id(task_id)?;
                Ok(SaveOrigin::ExternalTask(task_id.clone()))
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct AssignedRevisionV2 {
    pub template_id: String,
    pub revision_id: String,
    pub created_at: String,
}
#[derive(Debug, Clone)]
pub struct AgentProposalContextV2 {
    pub expected_library: String,
    pub allowed_targets: Vec<RefV2>,
    pub allow_create: bool,
    pub origin: TrustedAgentOriginV2,
    pub assignments: Vec<AssignedRevisionV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalDiffV2 {
    pub field_path: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalResultV2 {
    pub index: u32,
    pub proposal: ContentProposalV2,
    pub planned_revision: Option<RefV2>,
    pub differences: Vec<ProposalDiffV2>,
    pub dependencies: Vec<RefV2>,
    pub issues: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalReviewV2 {
    pub dto_version: u32,
    pub expected_library: String,
    pub valid: bool,
    pub proposals: Vec<ProposalResultV2>,
    pub issues: Vec<MatchIssueV2>,
}
#[derive(Debug, Clone)]
pub struct AgentSavePlanV2 {
    pub library: LibraryV2,
    pub review: ProposalReviewV2,
}
fn diff(
    before: Option<&Value>,
    after: Option<&Value>,
    path: &str,
    out: &mut Vec<ProposalDiffV2>,
) -> Result<()> {
    if before == after {
        return Ok(());
    }
    if let (Some(Value::Object(before)), Some(Value::Object(after))) = (before, after) {
        let keys = before.keys().chain(after.keys()).collect::<BTreeSet<_>>();
        for key in keys {
            let child = format!("{path}/{}", key.replace('~', "~0").replace('/', "~1"));
            diff(before.get(key), after.get(key), &child, out)?;
        }
    } else {
        if out.len() >= 10_000 {
            return Err(error("model_limit"));
        }
        out.push(ProposalDiffV2 {
            field_path: path.into(),
            before: before.cloned(),
            after: after.cloned(),
        });
    }
    Ok(())
}
fn parse(bytes: &[u8]) -> Result<AgentProposalV2> {
    let value = parse_strict_json_v2(bytes)?;
    if value["contractVersion"] != 2 {
        return Err(error("model_schema_unsupported"));
    }
    let proposal: AgentProposalV2 = serde_json::from_value(value).map_err(json_error)?;
    if proposal.proposals.is_empty() || proposal.proposals.len() > MAX_PROPOSALS {
        return Err(error("model_limit"));
    }
    Ok(proposal)
}
fn prepare(
    library: &LibraryV2,
    response_bytes: &[u8],
    context: &AgentProposalContextV2,
) -> Result<(ProposalReviewV2, Option<LibraryV2>)> {
    validate_library_v2(library)?;
    let submission = parse(response_bytes)?;
    if context.expected_library != digest(&canonical_bytes(library)?)
        || submission.expected_library != context.expected_library
    {
        return Err(error("model_save_stale"));
    }
    let origin = context.origin.save_origin()?;
    if context.assignments.len() != submission.proposals.len()
        || context.allowed_targets.len() > 4096
    {
        return Err(error("model_agent_assignment"));
    }
    for target in &context.allowed_targets {
        revision(library, target)?;
    }
    let mut assigned_templates = BTreeSet::new();
    let mut assigned_revisions = BTreeSet::new();
    for assignment in &context.assignments {
        check_id(&assignment.template_id)?;
        check_id(&assignment.revision_id)?;
        chrono::DateTime::parse_from_rfc3339(&assignment.created_at)
            .map_err(|_| error("model_agent_assignment"))?;
        if !assigned_templates.insert(&assignment.template_id)
            || !assigned_revisions.insert(&assignment.revision_id)
        {
            return Err(error("model_agent_assignment"));
        }
    }
    let mut candidate = library.clone();
    let mut results = vec![];
    let mut all_issues = vec![];
    for (index, (proposal, assignment)) in submission
        .proposals
        .iter()
        .zip(&context.assignments)
        .enumerate()
    {
        let mut dependencies = vec![];
        let mut differences = vec![];
        let target_content = if let Some(target) = &proposal.target {
            library
                .templates
                .iter()
                .find(|t| t.id == target.template_id)
                .and_then(|t| t.revisions.iter().find(|r| r.id == target.revision_id))
        } else {
            None
        };
        let old =
            target_content.map(|r| serde_json::json!({"label":r.saved_label,"content":r.content}));
        let new = serde_json::json!({"label":proposal.label,"content":proposal.content});
        diff(old.as_ref(), Some(&new), "", &mut differences)?;
        let result = (|| {
            if proposal.content_version != CONTENT_VERSION {
                return Err(error("model_schema_unsupported"));
            }
            match &proposal.target {
                Some(target) => {
                    revision(library, target)?;
                    if !context.allowed_targets.contains(target)
                        || target.template_id != assignment.template_id
                    {
                        return Err(error("model_agent_target_scope"));
                    }
                    dependencies.push(target.clone());
                }
                None => {
                    if !context.allow_create
                        || library
                            .templates
                            .iter()
                            .any(|t| t.id == assignment.template_id)
                    {
                        return Err(error("model_agent_target_scope"));
                    }
                }
            }
            // The same production function checks AST, immutable identities,
            // exact base revision and protected fields with trusted origin.
            save_revision_plan_v2(
                &candidate,
                &SaveRevisionRequestV2 {
                    expected_library: digest(&canonical_bytes(&candidate)?),
                    template_id: assignment.template_id.clone(),
                    base_revision: proposal.target.as_ref().map(|r| r.revision_id.clone()),
                    revision_id: assignment.revision_id.clone(),
                    created_at: assignment.created_at.clone(),
                    label: proposal.label.clone(),
                    content: proposal.content.clone(),
                    make_default: false,
                    origin: origin.clone(),
                },
            )
        })();
        let (planned_revision, issues) = match result {
            Ok(next) => {
                let r = next
                    .templates
                    .iter()
                    .find(|t| t.id == assignment.template_id)
                    .and_then(|t| t.revisions.iter().find(|r| r.id == assignment.revision_id))
                    .ok_or_else(|| error("model_agent_assignment"))?;
                let reference = RefV2 {
                    template_id: assignment.template_id.clone(),
                    revision_id: r.id.clone(),
                    digest: r.digest.clone(),
                };
                candidate = next;
                (Some(reference), vec![])
            }
            Err(mut err) => {
                for issue in &mut err.issues {
                    issue.reference = proposal.target.clone();
                    if issue.field_path.is_none() {
                        issue.field_path = Some(format!("/proposals/{index}"));
                    }
                }
                all_issues.extend(err.issues.clone());
                (None, err.issues)
            }
        };
        results.push(ProposalResultV2 {
            index: index as u32,
            proposal: proposal.clone(),
            planned_revision,
            differences,
            dependencies,
            issues,
        });
    }
    let valid = all_issues.is_empty();
    let review = ProposalReviewV2 {
        dto_version: 2,
        expected_library: context.expected_library.clone(),
        valid,
        proposals: results,
        issues: all_issues,
    };
    Ok((review, valid.then_some(candidate)))
}
pub fn parse_validate_proposal_v2(
    library: &LibraryV2,
    response_bytes: &[u8],
    context: &AgentProposalContextV2,
) -> Result<ProposalReviewV2> {
    Ok(prepare(library, response_bytes, context)?.0)
}
pub fn plan_agent_save_v2(
    library: &LibraryV2,
    response_bytes: &[u8],
    context: &AgentProposalContextV2,
) -> Result<AgentSavePlanV2> {
    let (review, candidate) = prepare(library, response_bytes, context)?;
    match candidate {
        Some(library) => Ok(AgentSavePlanV2 { library, review }),
        None => Err(MatchErrorV2 {
            code: "model_agent_proposal_invalid".into(),
            issues: review.issues,
        }),
    }
}
#[cfg(test)]
#[path = "agent_proposal_v2_tests.rs"]
mod tests;
