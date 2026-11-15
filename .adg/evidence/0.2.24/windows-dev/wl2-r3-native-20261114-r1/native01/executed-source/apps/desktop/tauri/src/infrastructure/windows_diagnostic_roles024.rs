//! Strict source-bound query roles. No public/serde constructor for either the
//! role or NormalExit. Raw process outcomes never become zero in this adapter.
use super::{
    windows_official_lifecycle024::Owner,
    windows_official_runtime024::{ProcessIdentity, ProcessState},
    windows_supervisor024::{Absent, WindowsSupervisor},
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::hash,
    modules::runtime::windows_lifecycle024_contract::{
        ChildExit, ChildRole, DiagnosticRoles, NativeSession, SupervisorAbsence,
        OFFICIAL_PACKAGE_SHA256,
    },
};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
fn refuse(detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_diagnostic_role_unknown".into(),
        detail: detail.into(),
    }
}
/// Retained identity data is only a lookup key; classify independently finds
/// the original handle and its command/parent history in the owner's Job.
pub(crate) struct RetainedChild {
    identity: ProcessIdentity,
}
pub(crate) struct ProvenRole {
    scope: String,
    child: ProcessIdentity,
    role: ChildRole,
    exit: ChildExit,
    argv: Vec<String>,
    parent: ProcessIdentity,
    source_sha: String,
    phase: &'static str,
}
pub(crate) struct NormalExit {
    scope: String,
    deadline: Instant,
    raw: Vec<(u32, u64, ChildRole, ChildExit)>,
}
impl NormalExit {
    pub(crate) fn check(&self, owner: &mut Owner, deadline: Instant) -> AppResult<()> {
        owner.check_current(deadline)?;
        if self.scope != owner.scope()? || self.deadline != deadline || self.raw.is_empty() {
            return Err(refuse("退出证明不是当前原动作/代次"));
        }
        Ok(())
    }
}
fn exact_query(argv: &[String], image: &Path, system: &Path) -> Option<ChildRole> {
    let same = |a: &Path, b: &Path| {
        dunce::simplified(a)
            .to_string_lossy()
            .replace('/', "\\")
            .eq_ignore_ascii_case(&dunce::simplified(b).to_string_lossy().replace('/', "\\"))
    };
    if argv.is_empty() || !same(Path::new(&argv[0]), image) {
        return None;
    }
    let args: Vec<_> = argv[1..].iter().map(|s| s.to_ascii_lowercase()).collect();
    if same(image, &system.join("sc.exe")) && args == ["query", "opencodex-proxy-native"] {
        Some(ChildRole::ScmRegistrationQuery)
    } else if same(image, &system.join("schtasks.exe"))
        && args == ["/query", "/tn", "opencodex-proxy", "/xml"]
    {
        Some(ChildRole::SchedulerRegistrationQuery)
    } else {
        None
    }
}
fn system_dir() -> AppResult<PathBuf> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut b = vec![0u16; 32768];
    let n = unsafe { GetSystemDirectoryW(b.as_mut_ptr(), b.len() as u32) };
    if n == 0 || n as usize >= b.len() {
        return Err(refuse("原生System32目录不可读"));
    }
    Ok(PathBuf::from(
        String::from_utf16(&b[..n as usize]).map_err(|_| refuse("System32 Unicode"))?,
    ))
}
fn call_site(owner: &Owner, role: &ChildRole) -> AppResult<String> {
    let selected = owner.selected_program();
    if selected.image.official_archive_sha256 != OFFICIAL_PACKAGE_SHA256 {
        return Err(refuse("官方包来源不匹配"));
    }
    let (relative, expected) = match role {
        ChildRole::ScmRegistrationQuery => (
            "src/lib/winsw.ts",
            "e71fa285ae983bc5ce55adf9b2d437a33b5fc942f84b2bb8aa194d804cd245e0",
        ),
        ChildRole::SchedulerRegistrationQuery => (
            "src/service/windows-guarded-manager.ts",
            "04d0498217a31ba4791183f79ff2ba17ba5c2301484fb70ff113f14b6e6927dd",
        ),
        _ => return Err(refuse("没有固定诊断调用点")),
    };
    let actual = hash::sha256_file(&selected.paths.package_root.join(relative))
        .map_err(|e| refuse(e.to_string()))?;
    if actual != expected {
        return Err(refuse("官方诊断call-site字节改变"));
    }
    Ok(actual)
}
pub(crate) struct WindowsDiagnostics;
impl DiagnosticRoles for WindowsDiagnostics {
    type Owner = Owner;
    type RetainedChild = RetainedChild;
    type ProvenRole = ProvenRole;
    type NormalExit = NormalExit;
    type SupervisorAbsent = Absent;
    fn classify(
        &mut self,
        owner: &mut Owner,
        child: &RetainedChild,
        deadline: Instant,
    ) -> AppResult<ProvenRole> {
        owner.check_current(deadline)?;
        let scope = owner.scope()?;
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| refuse(e.to_string()))?;
        let (native, state) = family
            .retained_members
            .iter()
            .find(|(id, _)| id == &child.identity)
            .ok_or_else(|| refuse("子代次不在原Job完整历史"))?;
        let native = native.clone();
        let state = *state;
        let command = owner
            .original_native()
            .captured_command(native.pid)
            .map_err(|e| refuse(e.to_string()))?
            .clone();
        let system = system_dir()?;
        let role = exact_query(&command.argv, &native.executable, &system)
            .ok_or_else(|| refuse("非准确只读查询语法/目标/镜像"))?;
        owner
            .original_native()
            .verify_member_image(native.pid, &native.executable)
            .map_err(|e| refuse(e.to_string()))?;
        let source_sha = call_site(owner, &role)?;
        // The original parent command and phase must be captured before child
        // exit and retained independently. A bare admitted Bun name is not enough.
        let parent_command = owner
            .original_native()
            .captured_command(command.parent.pid)
            .map_err(|e| refuse(format!("调用点父参数缺失：{e}")))?
            .clone();
        let package = owner.selected_program().paths.package_root.clone();
        let cli = package.join("src/cli/index.ts");
        let phase = owner.command_phase(command.captured_at)?;
        let bun = owner
            .selected_program()
            .paths
            .package_root
            .join("node_modules/bun/bin/bun.exe");
        let other_bun = owner.selected_program().paths.bun.clone();
        if owner
            .original_native()
            .verify_member_image(command.parent.pid, &bun)
            .is_err()
        {
            owner
                .original_native()
                .verify_member_image(command.parent.pid, &other_bun)
                .map_err(|e| refuse(e.to_string()))?;
        }
        if !parent_command.argv.iter().any(|a| Path::new(a) == cli)
            || parent_command
                .argv
                .iter()
                .any(|a| ["-e", "--eval", "-p", "--print"].contains(&a.as_str()))
            || !parent_command.argv.iter().any(|a| a == phase)
            || parent_command.captured_at > command.captured_at
        {
            return Err(refuse("原父进程没有准确官方cli入口/启动阶段证明"));
        }
        // Recheck the full admitted immutable program and pinned call-site at
        // this boundary. The source is fixed; no alternate script/eval/target
        // may create a diagnostic role. These facts are never accepted from IPC.
        crate::modules::runtime::program_image::verify(owner.selected_program())?;
        if source_sha != call_site(owner, &role)? {
            return Err(refuse("查询源码读回漂移"));
        }
        owner.check_current(deadline)?;
        let exit = match state {
            ProcessState::Exited { code } => ChildExit::Exited(code),
            ProcessState::Running => ChildExit::StillRunning,
        };
        Ok(ProvenRole {
            scope,
            child: native,
            role,
            exit,
            argv: command.argv,
            parent: command.parent,
            source_sha,
            phase,
        })
    }
    fn complete_family(
        &mut self,
        owner: &mut Owner,
        supervisor: &Absent,
        deadline: Instant,
    ) -> AppResult<NormalExit> {
        supervisor.check(owner, deadline)?;
        WindowsSupervisor.recheck_absent(owner, supervisor, deadline)?;
        let scope = owner.scope()?;
        let root = owner.original_native().binding().process.clone();
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| refuse(e.to_string()))?;
        if !family.complete_exit()
            || family.total_processes as usize != family.retained_members.len() + 1
            || family.root != (ProcessState::Exited { code: 0 })
        {
            return Err(refuse("原Job未完整正常结束/root非零"));
        }
        let mut raw = vec![(
            root.pid,
            root.creation_filetime,
            ChildRole::Launcher,
            ChildExit::Exited(0),
        )];
        for (identity, state) in family.retained_members {
            let proven = self.classify(owner, &RetainedChild { identity }, deadline)?;
            let exit = match state {
                ProcessState::Exited { code } => ChildExit::Exited(code),
                ProcessState::Running => ChildExit::StillRunning,
            };
            match (&proven.role, &exit) {
                (
                    ChildRole::ScmRegistrationQuery | ChildRole::SchedulerRegistrationQuery,
                    ChildExit::Exited(_),
                ) => {}
                (
                    ChildRole::Launcher | ChildRole::Service | ChildRole::OtherRequired,
                    ChildExit::Exited(0),
                ) => {}
                _ => return Err(refuse("required非零/Unknown/仍活/不可读否决NormalExit")),
            }
            raw.push((
                proven.child.pid,
                proven.child.creation_filetime,
                proven.role,
                exit,
            ));
        }
        WindowsSupervisor.recheck_absent(owner, supervisor, deadline)?;
        owner.check_current(deadline)?;
        Ok(NormalExit {
            scope,
            deadline,
            raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn wl2_r3_roles_exact_query_only_not_name_or_exit_allowlist() {
        let system = Path::new("C:/Windows/System32");
        let sc = system.join("sc.exe");
        let task = system.join("schtasks.exe");
        assert_eq!(
            exact_query(
                &a(&[
                    "C:/Windows/System32/sc.exe",
                    "query",
                    "opencodex-proxy-native"
                ]),
                &sc,
                system
            ),
            Some(ChildRole::ScmRegistrationQuery)
        );
        assert_eq!(
            exact_query(
                &a(&[
                    "C:/Windows/System32/schtasks.exe",
                    "/QUERY",
                    "/TN",
                    "opencodex-proxy",
                    "/XML"
                ]),
                &task,
                system
            ),
            Some(ChildRole::SchedulerRegistrationQuery)
        );
        for bad in [
            a(&["C:/Windows/System32/sc.exe", "query", "foreign"]),
            a(&[
                "C:/Windows/System32/sc.exe",
                "delete",
                "opencodex-proxy-native",
            ]),
            a(&[
                "C:/Windows/System32/sc.exe",
                "query",
                "opencodex-proxy-native",
                "&&",
                "echo",
            ]),
            a(&["C:/Windows/System32/sc.exe", "qc", "opencodex-proxy-native"]),
            a(&["sc.exe", "query", "opencodex-proxy-native"]),
        ] {
            assert!(exact_query(&bad, &sc, system).is_none());
        }
        for bad in [
            a(&[
                "C:/Windows/System32/schtasks.exe",
                "/query",
                "/tn",
                "foreign",
                "/xml",
            ]),
            a(&[
                "C:/Windows/System32/schtasks.exe",
                "/delete",
                "/tn",
                "opencodex-proxy",
                "/xml",
            ]),
            a(&[
                "C:/Windows/System32/schtasks.exe",
                "/query",
                "/tn",
                "opencodex-proxy",
            ]),
        ] {
            assert!(exact_query(&bad, &task, system).is_none());
        }
        assert!(exact_query(
            &a(&["C:/other/sc.exe", "query", "opencodex-proxy-native"]),
            Path::new("C:/other/sc.exe"),
            system
        )
        .is_none());
    }
}
