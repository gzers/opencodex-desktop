//! Original-host memory authority. All connection capabilities are private,
//! non-Clone/non-serde and contain actual native pipe/process handles.
use super::*;
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        windows_official_lifecycle024::Owner,
        windows_official_runtime024::{PrivatePipe, RetainedPeer},
    },
    modules::runtime::windows_lifecycle024_contract::{NativeSession, OwnerContinuity},
};
use std::{collections::BTreeSet, path::PathBuf};
fn refused(detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_owner_attach_unknown".into(),
        detail: detail.into(),
    }
}
pub(crate) struct ManagerConnection {
    pipe: PrivatePipe,
    peer: RetainedPeer,
    challenge: String,
    generation: u64,
    deadline: Instant,
}
pub(crate) struct AttachedManager {
    connection: ManagerConnection,
    scope: String,
    request: Request,
    window: ValidatedDeadline,
    action_used: std::cell::Cell<bool>,
}
pub(crate) struct WindowsOwnerHost {
    owner: Owner,
    broker: RetainedPeer,
    manager_image: PathBuf,
    instance: uuid::Uuid,
    generation: u64,
    used: BTreeSet<String>,
}
impl WindowsOwnerHost {
    /// Only the original creator may move its in-memory Owner into the host.
    /// No JSON/rendezvous/PID constructor for this type exists.
    pub(crate) fn new(
        mut owner: Owner,
        broker_image: &std::path::Path,
        manager_image: PathBuf,
        deadline: Instant,
    ) -> AppResult<Self> {
        owner.check_current(deadline)?;
        let broker = RetainedPeer::current().map_err(|e| refused(e.to_string()))?;
        let token = &broker.identity().token;
        if token.elevated || token.elevation_type == 2 {
            return Err(refused("broker必须为原普通令牌"));
        }
        broker
            .verify_image(broker_image)
            .map_err(|e| refused(e.to_string()))?;
        broker.no_outer_job().map_err(|e| refused(e.to_string()))?;
        Ok(Self {
            owner,
            broker,
            manager_image,
            instance: uuid::Uuid::new_v4(),
            generation: 0,
            used: BTreeSet::new(),
        })
    }
    pub(crate) fn challenge(
        &mut self,
        pipe: PrivatePipe,
        deadline: Instant,
    ) -> AppResult<ManagerConnection> {
        self.owner.check_current(deadline)?;
        self.broker
            .no_outer_job()
            .map_err(|e| refused(e.to_string()))?;
        let peer = pipe.accept(deadline).map_err(|e| refused(e.to_string()))?;
        peer.verify_image(&self.manager_image)
            .map_err(|e| refused(e.to_string()))?;
        let a = &peer.identity().token;
        let b = &self.broker.identity().token;
        if a.sid != b.sid || a.session_id != b.session_id || a.elevated || a.elevation_type == 2 {
            return Err(refused("pipe原peer令牌/SID/session不匹配"));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| refused("manager generation overflow"))?;
        let random: [u8; 32] = rand::random();
        let challenge = hash_hex(&random);
        let response = serde_json::json!({"schema":1,"instanceId":self.instance.to_string(),"managerGeneration":generation.to_string(),"challenge":challenge,"brokerPid":self.broker.identity().pid,"brokerFiletime":self.broker.identity().creation_filetime.to_string(),"scopeDigest":self.owner.scope()?});
        pipe.write_frame(
            &serde_json::to_vec(&response).map_err(|e| refused(e.to_string()))?,
            deadline,
        )
        .map_err(|e| refused(e.to_string()))?;
        Ok(ManagerConnection {
            pipe,
            peer,
            challenge,
            generation,
            deadline,
        })
    }
}
fn hash_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|n| format!("{n:02x}")).collect()
}
impl OwnerContinuity for WindowsOwnerHost {
    type Owner = Owner;
    type ManagerConnection = ManagerConnection;
    type AttachedManager = AttachedManager;
    fn authenticate_manager(
        &mut self,
        connection: ManagerConnection,
        deadline: Instant,
    ) -> AppResult<AttachedManager> {
        let deadline = deadline.min(connection.deadline);
        self.owner.check_current(deadline)?;
        self.broker
            .no_outer_job()
            .map_err(|e| refused(e.to_string()))?;
        connection
            .peer
            .verify_image(&self.manager_image)
            .map_err(|e| refused(e.to_string()))?;
        let actual = connection.pipe.peer().map_err(|e| refused(e.to_string()))?;
        if actual.identity() != connection.peer.identity() {
            return Err(refused("同一连接peer代次改变"));
        }
        let frame = connection
            .pipe
            .read_frame(deadline)
            .map_err(|e| refused(e.to_string()))?;
        let request = decode(&frame).map_err(|e| refused(e.to_string()))?;
        if request.instance_id != self.instance.to_string()
            || request.manager_generation != connection.generation.to_string()
            || request.challenge != connection.challenge
            || connection.generation != self.generation + 1
        {
            return Err(refused("challenge/instance/generation不匹配或旧连接重放"));
        }
        let (now, frequency) = qpc().map_err(|e| refused(e.to_string()))?;
        let window = request
            .deadline_at(now, frequency, Instant::now())
            .map_err(|e| refused(e.to_string()))?;
        let replay = format!(
            "{}:{}:{}",
            request.request_id, window.action_id, request.challenge
        );
        // Retain all request IDs, action IDs and nonce separately; no eviction
        // that would make old actions valid again in this original host lifetime.
        let keys = [
            replay,
            request.request_id.clone(),
            window.action_id.to_string(),
            request.challenge.clone(),
        ];
        if self.used.len() + keys.len() > 16384 || keys.iter().any(|k| self.used.contains(k)) {
            return Err(refused("请求/action/nonce重放或表上限"));
        }
        for key in keys {
            self.used.insert(key);
        }
        self.owner.check_current(deadline.min(window.deadline))?;
        let scope = self.owner.scope()?;
        // Single host mutex/loop owns this transition; old connection objects
        // remain native objects but cannot execute after this generation change.
        self.generation = connection.generation;
        Ok(AttachedManager {
            connection,
            scope,
            request,
            window,
            action_used: std::cell::Cell::new(false),
        })
    }
    fn with_owner<T>(
        &mut self,
        manager: &AttachedManager,
        deadline: Instant,
        operation: impl FnOnce(&mut Owner) -> AppResult<T>,
    ) -> AppResult<T> {
        let deadline = deadline.min(manager.window.deadline);
        if Instant::now() >= deadline
            || manager.connection.generation != self.generation
            || manager.scope != self.owner.scope()?
        {
            return Err(refused("动作过期/旧manager代次/来源改变"));
        }
        if matches!(manager.request.operation, Operation::Detach) {
            return Err(refused("detach没有服务动作权限"));
        }
        if manager.action_used.replace(true) {
            return Err(refused("同一已认证action已消费"));
        }
        self.broker
            .no_outer_job()
            .map_err(|e| refused(e.to_string()))?;
        manager
            .connection
            .peer
            .verify_image(&self.manager_image)
            .map_err(|e| refused(e.to_string()))?;
        self.owner.check_current(deadline)?;
        let result = operation(&mut self.owner)?;
        self.owner.check_current(deadline)?;
        self.broker
            .no_outer_job()
            .map_err(|e| refused(e.to_string()))?;
        Ok(result)
    }
    fn detach_manager(&mut self, manager: AttachedManager) -> AppResult<()> {
        if manager.connection.generation != self.generation {
            return Err(refused("旧manager不能detach当前代次"));
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| refused("manager generation overflow"))?;
        drop(manager); // only pipe/peer handles, never the original Job/Owner.
        Ok(())
    }
}
