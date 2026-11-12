//! Pure, bounded lineage from the independent canonical event stream. No
//! directory observations may supply an origin, generation or predecessor.
use super::*;
pub(super) struct Generation<'a> {
    pub(super) binding: Vec<u8>,
    pub(super) origin: super::root_creation::References,
    pub(super) heads: &'a [Head],
}
pub(in crate::infrastructure::macos_client_originals) struct Lineage<'a> {
    generations: Vec<Generation<'a>>,
}
pub(in crate::infrastructure::macos_client_originals) struct IndexedNamespace {
    slots: Vec<String>,
}
impl IndexedNamespace {
    pub(in crate::infrastructure::macos_client_originals) fn names(&self) -> &[String] {
        &self.slots
    }
}
impl Lineage<'_> {
    pub(super) fn generations(&self) -> &[Generation<'_>] {
        &self.generations
    }
    pub(super) fn namespace(&self) -> IndexedNamespace {
        IndexedNamespace {
            slots: self.slots(),
        }
    }
    pub(super) fn bind_pending(
        &mut self,
        transfer: &super::root_creation::Transfer,
    ) -> io::Result<()> {
        let slots = self.slots();
        for generation in &mut self.generations {
            transfer.bind_predecessor_namespace(&mut generation.origin, &slots)?;
        }
        Ok(())
    }
    pub(super) fn slots(&self) -> Vec<String> {
        let mut slots = self
            .generations
            .iter()
            .skip(1)
            .map(|g| g.origin.id.clone())
            .collect::<Vec<_>>();
        slots.sort();
        slots
    }
    pub(super) fn current(&self) -> &Generation<'_> {
        self.generations.last().expect("nonempty validated lineage")
    }
}
fn scope_value(binding: &[u8]) -> io::Result<Vec<serde_json::Value>> {
    let raw = root_binding::scope_bytes(binding)?;
    let value: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|_| denied("invalid generation scope"))?;
    let parts = value
        .as_array()
        .filter(|p| p.len() == 9)
        .ok_or_else(|| denied("generation scope not full tuple"))?;
    Ok(parts.clone())
}
fn validate_segment(heads: &[Head], origin: &super::root_creation::References) -> io::Result<()> {
    let mut completion = None;
    for (i, h) in heads.iter().enumerate() {
        if h.ticket != origin.id
            || i != 0 && h.root_generation.is_some()
            || h.references
                .as_ref()
                .is_some_and(|r| r.attempt != origin.id)
        {
            return Err(denied("generation events rebound source"));
        }
        if h.state == State::RootAbsent {
            if completion.is_some_and(|old| Some(old) != h.completion_sha256.as_ref()) {
                return Err(denied(
                    "generation reclose changes immutable completion receipt",
                ));
            }
            completion = h.completion_sha256.as_ref();
        }
    }
    for pair in heads.windows(2) {
        let before = &pair[0];
        let after = &pair[1];
        if !matches!(
            (before.state, after.state),
            (State::Preparing, State::CaptureReserved)
                | (State::CaptureReserved, State::Captured)
                | (State::Captured, State::Ready)
                | (State::Ready, State::StopPrepared)
                | (State::StopPrepared, State::Stopped)
                | (State::Stopped, State::RestorePrepared)
                | (
                    State::Captured | State::Stopped | State::RestorePrepared,
                    State::RootRetirementPrepared
                )
                | (State::RootRetirementPrepared, State::RootAbsent)
                | (State::RootAbsent, State::RootRetirementPrepared)
                | (
                    State::Captured | State::Ready | State::StopPrepared,
                    State::Failed
                )
        ) || before.references.as_ref().is_some_and(|refs| {
            after.references.as_ref().is_none_or(|next| {
                next.attempt != refs.attempt
                    || next.original_sha256 != refs.original_sha256
                    || next.scope_sha256 != refs.scope_sha256
                    || !next.phases.starts_with(&refs.phases)
                    || refs.live_sha256.is_some() && next.live_sha256 != refs.live_sha256
            })
        }) || before.root_retirement.is_some() && before.root_retirement != after.root_retirement
            || before.restore_plan.is_some() && before.restore_plan != after.restore_plan
        {
            return Err(denied(
                "generation phase chain differs from registered lifecycle",
            ));
        }
    }
    Ok(())
}
pub(super) fn parse<'a>(binding: &[u8], history: &'a [Head]) -> io::Result<Lineage<'a>> {
    if history.is_empty() || history.len() > 4096 {
        return Err(denied("root lineage empty or over bound"));
    }
    let mut payload = 0usize;
    for (i, h) in history.iter().enumerate() {
        validate(h)?;
        let bytes = encode(h)?.len();
        payload = payload
            .checked_add(bytes)
            .filter(|n| *n <= 4 * 1024 * 1024)
            .ok_or_else(|| denied("root lineage payload over bound"))?;
        if bytes > 65536 {
            return Err(denied("root lineage event over bound"));
        }
        if h.revision != i as u64 + 1 {
            return Err(denied("root lineage is not contiguous"));
        }
    }
    let first = root_binding::original_root(binding)?;
    if first.generation_slot.is_some() {
        return Err(denied("singleton binding is not initial root"));
    }
    let first_scope = scope_value(binding)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut identities = Vec::new();
    let mut generations = Vec::new();
    let mut start = 0;
    let mut active_binding = binding.to_vec();
    while start < history.len() {
        if generations.len() >= 512 {
            return Err(denied("root generation count over bound"));
        }
        let initial = &history[start];
        let origin = root_binding::original_root(&active_binding)?;
        if initial.state != State::Preparing
            || initial.ticket != origin.id
            || !seen.insert(origin.id.clone())
            || [
                &origin.intent_sha256,
                &origin.created_sha256,
                &origin.published_sha256,
            ]
            .iter()
            .any(|h| !valid_hash(h))
            || start == 0 && initial.root_generation.is_some()
            || start != 0 && origin.generation_slot.as_deref() != Some(origin.id.as_str())
        {
            return Err(denied("root generation initial source invalid or reused"));
        }
        let scope = scope_value(&active_binding)?;
        if first_scope[..8] != scope[..8] || identities.contains(&scope[8]) {
            return Err(denied(
                "root generation changes context or reuses directory identity",
            ));
        }
        identities.push(scope[8].clone());
        let end = (start + 1..history.len())
            .find(|i| history[*i].ticket != origin.id)
            .unwrap_or(history.len());
        let heads = &history[start..end];
        validate_segment(heads, &origin)?;
        generations.push(Generation {
            binding: active_binding,
            origin,
            heads,
        });
        if end < history.len() {
            let closed = &history[end - 1];
            let next = &history[end];
            let source = next
                .root_generation
                .as_ref()
                .ok_or_else(|| denied("successor lacks atomic initial source"))?;
            if closed.state != State::RootAbsent
                || source.preparing_revision != next.revision
                || source.predecessor_revision != closed.revision
                || source.predecessor_sha256 != digest(&encode(closed)?)
            {
                return Err(denied("successor predecessor is not exact adjacent close"));
            }
            active_binding = source.binding.clone();
        } else {
            break;
        }
        start = end;
    }
    let mut lineage = Lineage { generations };
    let namespace = lineage.namespace();
    for generation in &mut lineage.generations {
        generation.origin.bind_indexed_slots(&namespace);
    }
    Ok(lineage)
}
