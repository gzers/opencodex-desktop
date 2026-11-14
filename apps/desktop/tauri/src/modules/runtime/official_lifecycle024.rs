//! Shared admission input for the unchanged official 2.99 Windows owner.
//! No native handle, runtime-record schema, OS ownership or exit proof lives here.
use super::{program_layout, program_selection};
// One public shared boundary for the platform adapter. Transport internals and
// record secrets remain private to attested_status.
pub use crate::infrastructure::attested_status::official024::{
    observe_official024, request_stop_official024, RetainedService, ServiceObservation,
    StopAccepted,
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{platform, runtime_executable::SharedRuntimeExecutable},
    modules::process::{EnvironmentPolicy, LifecycleAction, ProcessCommand},
};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
fn refused() -> AppError {
    AppError::RuntimeManaged {
        code: "official024_launch_unconfirmed".into(),
        detail: "官方运行程序、用户目录或动作预算未确认；未启动新进程".into(),
    }
}
/// Construct once at the public action boundary. A nested owner consumes the
/// inherited absolute deadlines, never creates a replacement 30/10s window.
#[derive(Clone, Copy, Debug)]
pub struct ActionWindow {
    deadline: Instant,
    stop_deadline: Instant,
}
impl ActionWindow {
    pub fn begin(action: LifecycleAction, started: Instant, parent: Instant) -> AppResult<Self> {
        let duration = match action {
            LifecycleAction::Start => crate::modules::process::runner_start_timeout(),
            LifecycleAction::Stop => crate::modules::process::stop_timeout(),
            LifecycleAction::Restart => crate::modules::process::restart_limit(),
        };
        let value = Self {
            deadline: parent.min(started + duration),
            stop_deadline: parent
                .min(started + duration)
                .min(started + crate::modules::process::stop_timeout()),
        };
        value.remaining()?;
        Ok(value)
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn stop_deadline(&self) -> Instant {
        self.stop_deadline
    }
    pub fn remaining(&self) -> AppResult<Duration> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            Err(AppError::Timeout)
        } else {
            Ok(remaining)
        }
    }
}
/// Private fields, no Clone/serde. This is admitted executable input, not native
/// permission to resume, kill, claim an existing PID, or restore client files.
pub struct LaunchPlan {
    runtime: SharedRuntimeExecutable,
    program: program_selection::SelectedProgram,
    root: PathBuf,
    profile: PathBuf,
    home: PathBuf,
    args: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
    deadline: Instant,
}
impl LaunchPlan {
    pub fn prepare(
        runtime: &SharedRuntimeExecutable,
        command: &ProcessCommand,
        deadline: Instant,
    ) -> AppResult<Self> {
        if Instant::now() >= deadline {
            return Err(AppError::Timeout);
        }
        command.validate()?;
        if command.action != LifecycleAction::Start
            || command.environment.home.as_deref() != Some(command.working_directory.as_os_str())
        {
            return Err(refused());
        }
        for path in [
            &command.working_directory,
            &command.environment.opencodex_home,
        ] {
            if !program_layout::clean_absolute(path) {
                return Err(refused());
            }
        }
        let root = runtime.managed_installation_root().ok_or_else(refused)?;
        let (target, platform) = program_layout::host_target()?;
        let program =
            program_selection::read_complete(&root, platform, target)?.ok_or_else(refused)?;
        if program.selection.version != "2.99.0"
            || runtime.executable().as_ref() != Some(&program.stable_entry)
            || command.executable != program.stable_entry
        {
            return Err(refused());
        }
        // Launch Node by its exact admitted path; the .mjs is its argument.
        // The original launcher may start Bun descendants; the native owner must
        // keep the whole Job, not equate launcher exit0 with service exit.
        let args = vec![
            program.script.as_os_str().to_owned(),
            "start".into(),
            "--port".into(),
            command.requested_port.unwrap_or(0).to_string().into(),
        ];
        let environment = environment(
            &command.environment,
            &command.working_directory,
            &program.paths.node,
        )?;
        let value = Self {
            runtime: runtime.clone(),
            program,
            root,
            profile: command.working_directory.clone(),
            home: command.environment.opencodex_home.clone(),
            args,
            environment,
            deadline,
        };
        value.check_current(runtime)?;
        Ok(value)
    }
    pub fn executable(&self) -> &Path {
        &self.program.paths.node
    }
    pub fn arguments(&self) -> &[OsString] {
        &self.args
    }
    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }
    pub fn user_profile(&self) -> &Path {
        &self.profile
    }
    pub fn opencodex_home(&self) -> &Path {
        &self.home
    }
    pub fn program_root(&self) -> &Path {
        &self.root
    }
    pub fn version(&self) -> &str {
        &self.program.selection.version
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn check_current(&self, runtime: &SharedRuntimeExecutable) -> AppResult<()> {
        if Instant::now() >= self.deadline {
            return Err(AppError::Timeout);
        }
        if !Arc::ptr_eq(runtime, &self.runtime)
            || runtime.managed_installation_root().as_deref() != Some(&self.root)
            || runtime.executable().as_ref() != Some(&self.program.stable_entry)
        {
            return Err(refused());
        }
        program_selection::verify_current(&self.root, &self.program)?;
        if Instant::now() >= self.deadline {
            return Err(AppError::Timeout);
        }
        Ok(())
    }
    pub fn verify_full(&self, runtime: &SharedRuntimeExecutable) -> AppResult<()> {
        self.check_current(runtime)?;
        super::program_image::verify(&self.program)?;
        self.check_current(runtime)
    }
}
fn environment(
    policy: &EnvironmentPolicy,
    profile: &Path,
    node: &Path,
) -> AppResult<BTreeMap<OsString, OsString>> {
    let mut cmd = std::process::Command::new(node);
    cmd.env_clear();
    platform::apply_user_environment(&mut cmd, Some(profile.as_os_str()));
    cmd.env("HOME", profile)
        .env("OPENCODEX_HOME", &policy.opencodex_home);
    cmd.env(
        "PATH",
        platform::controlled_path(node.parent().map(Path::to_path_buf), policy.path.as_deref()),
    );
    for (k, v) in [
        ("LANG", &policy.lang),
        ("LC_ALL", &policy.lc_all),
        ("HTTP_PROXY", &policy.http_proxy),
        ("HTTPS_PROXY", &policy.https_proxy),
        ("NO_PROXY", &policy.no_proxy),
    ] {
        if let Some(v) = v {
            cmd.env(k, v);
        }
    }
    let mut values = BTreeMap::new();
    for (k, v) in cmd.get_envs() {
        let key = k.to_str().ok_or_else(refused)?;
        if !policy.allowed_keys().contains(&key) {
            return Err(refused());
        }
        let v = v.ok_or_else(refused)?;
        if k.as_encoded_bytes().contains(&0) || v.as_encoded_bytes().contains(&0) {
            return Err(refused());
        }
        values.insert(k.to_owned(), v.to_owned());
    }
    Ok(values)
}
#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "official_lifecycle024_tests.rs"]
mod tests;
