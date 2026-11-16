//! Source-bound original launcher/service/query roles. No public/serde constructor for the
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
/// A successfully checked old Stop fact retained in the original broker. The
/// original Stop deadline is immutable. A later user action reobserves the old
/// original handles/history under its own budget, never extends the old Stop.
pub(crate) struct ClosedExitHistory {
    proof: NormalExit,
    completed_at: Instant,
}
impl ClosedExitHistory {
    pub(crate) fn check(&self, owner: &mut Owner, new_deadline: Instant) -> AppResult<()> {
        owner.check_current(new_deadline)?;
        if self.completed_at >= self.proof.deadline {
            return Err(refuse("原Stop窗口内闭合历史缺席"));
        }
        self.proof.check_raw_history(owner, new_deadline)
    }
}
impl NormalExit {
    /// Actual retained generations from the original successfully checked Job
    /// history. PIDs are only content observations; they never qualify a path,
    /// caller process or reconstructed Owner. Reject ambiguous reused PIDs.
    pub(crate) fn original_content_processes(
        &self,
        owner: &mut Owner,
        end: Instant,
    ) -> AppResult<Vec<ProcessIdentity>> {
        self.check(owner, end)?;
        let root = owner.original_native().binding().process.clone();
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| refuse(e.to_string()))?;
        if !family.complete_exit()
            || family.total_processes as usize != self.raw.len()
            || family.total_processes as usize != family.retained_members.len() + 1
        {
            return Err(refuse("原内容审计进程集合不完整"));
        }
        let mut processes = vec![root];
        processes.extend(family.retained_members.into_iter().map(|(p, _)| p));
        processes.sort_by_key(|p| (p.pid, p.creation_filetime));
        if processes.windows(2).any(|p| p[0].pid == p[1].pid)
            || processes.iter().any(|p| {
                !self
                    .raw
                    .iter()
                    .any(|(pid, time, _, _)| *pid == p.pid && *time == p.creation_filetime)
            })
        {
            return Err(refuse("原审计PID/创建代次歧义"));
        }
        self.check(owner, end)?;
        Ok(processes)
    }
    pub(crate) fn check(&self, owner: &mut Owner, deadline: Instant) -> AppResult<()> {
        owner.check_current(deadline)?;
        if self.scope != owner.scope()? || self.deadline != deadline || self.raw.is_empty() {
            return Err(refuse("退出证明不是当前原动作/代次"));
        }
        self.check_raw_history(owner, deadline)
    }
    pub(crate) fn into_closed_history(
        self,
        owner: &mut Owner,
        deadline: Instant,
    ) -> AppResult<ClosedExitHistory> {
        self.check(owner, deadline)?;
        let completed_at = Instant::now();
        if completed_at >= deadline {
            return Err(refuse("原Stop截止已过；不得补签历史"));
        }
        Ok(ClosedExitHistory {
            proof: self,
            completed_at,
        })
    }
    fn check_raw_history(&self, owner: &mut Owner, deadline: Instant) -> AppResult<()> {
        owner.check_current(deadline)?;
        if self.scope != owner.scope()? || self.raw.is_empty() {
            return Err(refuse("保留退出历史原代次不匹配"));
        }
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| refuse(e.to_string()))?;
        if owner.original_native().forced_cleanup_requested()
            || !family.complete_exit()
            || family.total_processes as usize != self.raw.len()
            || family.total_processes as usize != family.retained_members.len() + 1
        {
            return Err(refuse("原退出证明后Job历史/活动代次改变"));
        }
        let root = &owner.original_native().binding().process;
        let mut seen = std::collections::HashSet::new();
        for (pid, created, role, exit) in &self.raw {
            if !seen.insert((*pid, *created)) || !role_exit_allowed(role, exit) {
                return Err(refuse("退出证明重复代次/角色或原始退出码不成立"));
            }
            let state = if (*pid, *created) == (root.pid, root.creation_filetime) {
                if role != &ChildRole::Launcher {
                    return Err(refuse("原launcher角色改变"));
                }
                family.root
            } else {
                family
                    .retained_members
                    .iter()
                    .find(|(id, _)| (id.pid, id.creation_filetime) == (*pid, *created))
                    .ok_or_else(|| refuse("原始退出代次缺失"))?
                    .1
            };
            if raw_exit(state) != *exit {
                return Err(refuse("原始退出码读回改变"));
            }
        }
        owner.check_current(deadline)?;
        Ok(())
    }
}
fn raw_exit(state: ProcessState) -> ChildExit {
    match state {
        ProcessState::Exited { code } => ChildExit::Exited(code),
        ProcessState::Running => ChildExit::StillRunning,
    }
}
fn role_exit_allowed(role: &ChildRole, exit: &ChildExit) -> bool {
    matches!(
        (role, exit),
        (
            ChildRole::ScmRegistrationQuery | ChildRole::SchedulerRegistrationQuery,
            ChildExit::Exited(_)
        ) | (
            ChildRole::Launcher | ChildRole::Service | ChildRole::OtherRequired,
            ChildExit::Exited(0)
        )
    )
}
fn exact_root(argv: &[String], image: &Path, script: &Path) -> bool {
    argv.len() == 5
        && Path::new(&argv[0]) == image
        && Path::new(&argv[1]) == script
        && argv[2] == "start"
        && argv[3] == "--port"
        && argv[4]
            .parse::<u16>()
            .is_ok_and(|port| port.to_string() == argv[4])
}
fn exact_bun_chain(argv: &[String], image: &Path, cli: &Path, root: &[String]) -> bool {
    // Shape validation only, never validation of the official launch capability.
    // Authority additionally requires the retained original Node parent, exact
    // current script/source objects, native Job history and original phase.
    let proof = argv
        .get(2)
        .and_then(|v| v.strip_prefix("--ocx-internal-launch-proof="));
    argv.len() == 6
        && root.len() == 5
        && Path::new(&argv[0]) == image
        && Path::new(&argv[1]) == cli
        && argv[3..] == root[2..]
        && proof.is_some_and(|v| {
            v.len() == 43
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
}
fn root_source(owner: &mut Owner, deadline: Instant) -> AppResult<()> {
    owner.check_current(deadline)?;
    let root = owner.original_native().binding().process.clone();
    let args = owner.original_native().original_arguments().clone();
    let node = owner.selected_program().paths.node.clone();
    let script = owner.selected_program().script.clone();
    if !exact_root(&args.argv, &node, &script) {
        return Err(refuse("原Node实际参数不是准确StartPlan"));
    }
    owner
        .original_native()
        .verify_retained_image(&root, &node)
        .map_err(|e| refuse(e.to_string()))?;
    if call_site(owner, &ChildRole::Launcher)? != owner.selected_program().image.bin_sha256 {
        return Err(refuse("原Node启动脚本不是固定官方来源"));
    }
    Ok(())
}
fn bun_source(
    owner: &mut Owner,
    process: &ProcessIdentity,
    deadline: Instant,
) -> AppResult<super::windows_official_runtime024::CommandEvidence> {
    root_source(owner, deadline)?;
    let command = owner
        .original_native()
        .captured_command(process.pid)
        .map_err(|e| refuse(e.to_string()))?
        .clone();
    let root = owner.original_native().binding().process.clone();
    let args = owner.original_native().original_arguments().clone();
    if command.parent != root || args.captured_at > command.captured_at {
        return Err(refuse("Bun父代次不是原Node创建者或捕获阶段倒置"));
    }
    let selected = owner.selected_program();
    let cli = selected.paths.package_root.join("src/cli/index.ts");
    let bun = selected
        .paths
        .package_root
        .join("node_modules/bun/bin/bun.exe");
    let other_bun = selected.paths.bun.clone();
    if !exact_bun_chain(&command.argv, &process.executable, &cli, &args.argv) {
        return Err(refuse("Bun实际参数未逐项绑定原Node入口/port"));
    }
    if owner
        .original_native()
        .verify_retained_image(process, &bun)
        .is_err()
    {
        owner
            .original_native()
            .verify_retained_image(process, &other_bun)
            .map_err(|e| refuse(e.to_string()))?;
    }
    owner.check_current(deadline)?;
    Ok(command)
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
fn exact_sentinel(argv: &[String], image: &Path, parent_image: &Path, package: &Path) -> bool {
    // The unchanged installed (not compiled standalone) package dispatches this
    // exact TS entry. Neither __health-sentinel on an arbitrary executable nor
    // another script containing this filename can identify this child.
    argv.len() == 2
        && Path::new(&argv[0]) == image
        && image == parent_image
        && Path::new(&argv[1]) == package.join("src/service/health-sentinel-main.ts")
}
fn sentinel_source(owner: &mut Owner, child: &ProcessIdentity, deadline: Instant) -> AppResult<()> {
    let command = owner
        .original_native()
        .captured_command(child.pid)
        .map_err(|e| refuse(e.to_string()))?
        .clone();
    let original_service = owner
        .confirmed_service_generation()?
        .ok_or_else(|| refuse("sentinel缺少原typedReady/pre-stop Service证明"))?;
    if command.parent != original_service || owner.command_phase(command.captured_at)? != "start" {
        return Err(refuse("sentinel父代次/原启动阶段不匹配"));
    }
    let parent_command = bun_source(owner, &original_service, deadline)?;
    let package = owner.selected_program().paths.package_root.clone();
    if parent_command.captured_at > command.captured_at
        || !exact_sentinel(
            &command.argv,
            &child.executable,
            &original_service.executable,
            &package,
        )
    {
        return Err(refuse("sentinel实际入口/父镜像/捕获顺序不匹配"));
    }
    // Identity is checked on each retained original object. Equal names,
    // equal bytes at another fileID, synthetic argv or a source-only fixture
    // cannot issue a role or make its raw exit irrelevant.
    owner
        .original_native()
        .verify_retained_image(child, &original_service.executable)
        .map_err(|e| refuse(e.to_string()))?;
    call_site(owner, &ChildRole::OtherRequired)?;
    owner.check_current(deadline)
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
        ChildRole::Launcher => (
            "bin/ocx.mjs",
            "21b88153740ec422033b39caaa7735ba2c166dbf5d436c732ae3c5e01275cc7f",
        ),
        ChildRole::Service => (
            "src/cli/index.ts",
            "8d176afa6d6a5aa65c4e6b2e964401fadbfeb9b11b14386c503a321aecc17206",
        ),
        ChildRole::OtherRequired => {
            let sources = [
                (
                    "src/service/health-sentinel-launch.ts",
                    "0f2b70693cb06039b53e83aa6e08ad644fe36d62052b911f8653a67575112de3",
                ),
                (
                    "src/service/health-sentinel-main.ts",
                    "408d8bc501115132caec7190cd78fbeca64f1105f9184e518f6da1b8621477f0",
                ),
                (
                    "src/service/health-sentinel-windows.ts",
                    "a312da3eef2e45bd337181971f726074fc56916be7f345ddd9e76236d8717e6e",
                ),
                (
                    "src/cli/index.ts",
                    "8d176afa6d6a5aa65c4e6b2e964401fadbfeb9b11b14386c503a321aecc17206",
                ),
            ];
            let mut seal = Vec::new();
            for (relative, expected) in sources {
                if hash::sha256_file(&selected.paths.package_root.join(relative))
                    .map_err(|e| refuse(e.to_string()))?
                    != expected
                {
                    return Err(refuse("sentinel原入口/启动/原生父句柄源码改变"));
                }
                seal.extend_from_slice(relative.as_bytes());
                seal.push(0);
                seal.extend_from_slice(expected.as_bytes());
                seal.push(0);
            }
            return Ok(hash::sha256_hex(&seal));
        }
        _ => return Err(refuse("没有固定角色调用点")),
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
        let phase = owner.command_phase(command.captured_at)?;
        let verified_state = owner
            .original_native()
            .verify_retained_image(&native, &native.executable)
            .map_err(|e| refuse(e.to_string()))?;
        if verified_state != state {
            return Err(refuse("原子代次退出状态在分类窗口改变"));
        }
        let role = if let Some(query) =
            exact_query(&command.argv, &native.executable, &system_dir()?)
        {
            let parent = bun_source(owner, &command.parent, deadline)?;
            if parent.captured_at > command.captured_at {
                return Err(refuse("查询捕获早于原父代次参数"));
            }
            query
        } else if exact_sentinel(
            &command.argv,
            &native.executable,
            &command.parent.executable,
            &owner.selected_program().paths.package_root,
        ) {
            sentinel_source(owner, &native, deadline)?;
            ChildRole::OtherRequired
        } else {
            let original_command = bun_source(owner, &native, deadline)?;
            if original_command.argv != command.argv || original_command.parent != command.parent {
                return Err(refuse("原Bun命令历史读回改变"));
            }
            // Service is bound to the original pre-stop listener generation,
            // obtained only after the typed Ready/current verification. Any
            // other directly proven original Node->Bun start wrapper remains
            // Launcher and still requires raw0. Unknown helpers are not granted
            // OtherRequired merely because they share this Job or exit0. Only
            // the separately source/parent-bound sentinel above has that role.
            match owner.confirmed_service_generation()? {
                Some(service) if service == native => ChildRole::Service,
                Some(_) => return Err(refuse("已确认Service与原直接Bun代次冲突")),
                None => ChildRole::Launcher,
            }
        };
        let source_sha = call_site(owner, &role)?;
        // Recheck the full admitted immutable program and pinned call-site at
        // this boundary. The source is fixed; no alternate script/eval/target
        // may create a diagnostic role. These facts are never accepted from IPC.
        crate::modules::runtime::program_image::verify(owner.selected_program())?;
        if source_sha != call_site(owner, &role)? {
            return Err(refuse("角色源码读回漂移"));
        }
        owner.check_current(deadline)?;
        let exit = raw_exit(state);
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
        root_source(owner, deadline)?;
        let root = owner.original_native().binding().process.clone();
        let family = owner
            .original_native()
            .family_state()
            .map_err(|e| refuse(e.to_string()))?;
        if owner.original_native().forced_cleanup_requested()
            || !family.complete_exit()
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
            let exit = raw_exit(state);
            if proven.scope != scope
                || proven.exit != exit
                || !role_exit_allowed(&proven.role, &exit)
            {
                return Err(refuse("required非零/Unknown/仍活/不可读否决NormalExit"));
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
        let proof = NormalExit {
            scope,
            deadline,
            raw,
        };
        proof.check(owner, deadline)?;
        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn wl2_helper_sentinel_exact_entry_is_required_and_never_diagnostic() {
        let package = Path::new("C:/task/I/package");
        let bun = package.join("node_modules/bun/bin/bun.exe");
        let entry = package.join("src/service/health-sentinel-main.ts");
        let good = a(&[bun.to_str().unwrap(), entry.to_str().unwrap()]);
        assert!(exact_sentinel(&good, &bun, &bun, package));
        assert!(!exact_sentinel(
            &good,
            &bun,
            Path::new("C:/foreign/bun.exe"),
            package
        ));
        for bad in [
            a(&[bun.to_str().unwrap(), "__health-sentinel"]),
            a(&[bun.to_str().unwrap(), "C:/foreign/health-sentinel-main.ts"]),
            a(&[bun.to_str().unwrap(), "--eval", entry.to_str().unwrap()]),
            a(&[bun.to_str().unwrap(), entry.to_str().unwrap(), "start"]),
            a(&["C:/foreign/bun.exe", entry.to_str().unwrap()]),
        ] {
            assert!(!exact_sentinel(&bad, &bun, &bun, package));
        }
        assert!(role_exit_allowed(
            &ChildRole::OtherRequired,
            &ChildExit::Exited(0)
        ));
        for code in [1, 5, 242, 1060, u32::MAX] {
            assert!(!role_exit_allowed(
                &ChildRole::OtherRequired,
                &ChildExit::Exited(code)
            ));
        }
        assert!(exact_query(&good, &bun, Path::new("C:/Windows/System32")).is_none());
        // This pure shape predicate grants no role. Production additionally
        // requires original Ready service parent, start phase and four fixed
        // script hashes, all checked only from the retained original Owner.
    }
    #[test]
    fn wl2_next_d_raw_outcomes_cannot_erase_required_failure_or_unknown() {
        for role in [
            ChildRole::Launcher,
            ChildRole::Service,
            ChildRole::OtherRequired,
        ] {
            assert!(role_exit_allowed(&role, &ChildExit::Exited(0)));
            for code in [1, 5, 242, 1060, u32::MAX] {
                assert!(!role_exit_allowed(&role, &ChildExit::Exited(code)));
            }
            assert!(!role_exit_allowed(&role, &ChildExit::StillRunning));
            assert!(!role_exit_allowed(&role, &ChildExit::Unreadable));
        }
        for role in [
            ChildRole::ScmRegistrationQuery,
            ChildRole::SchedulerRegistrationQuery,
        ] {
            for code in [0, 1, 5, 1060, u32::MAX] {
                let original = raw_exit(ProcessState::Exited { code });
                assert_eq!(original, ChildExit::Exited(code));
                assert!(role_exit_allowed(&role, &original));
            }
            assert!(!role_exit_allowed(&role, &ChildExit::StillRunning));
            assert!(!role_exit_allowed(&role, &ChildExit::Unreadable));
        }
        for exit in [
            ChildExit::Exited(0),
            ChildExit::Exited(1),
            ChildExit::StillRunning,
            ChildExit::Unreadable,
        ] {
            assert!(!role_exit_allowed(&ChildRole::Unknown, &exit));
        }
        // This exit rule issues neither ProvenRole nor NormalExit. Exact native
        // argv/parent/source and independent pre/post Absent remain mandatory.
    }
    #[test]
    fn wl2_family_original_node_bun_grammar_is_bound_not_merely_containing_phase() {
        let node = Path::new("C:/task/I/node.exe");
        let bin = Path::new("C:/task/I/package/bin/ocx.mjs");
        let bun = Path::new("C:/task/I/bun.exe");
        let cli = Path::new("C:/task/I/package/src/cli/index.ts");
        let root = a(&[
            node.to_str().unwrap(),
            bin.to_str().unwrap(),
            "start",
            "--port",
            "0",
        ]);
        let proof = format!("--ocx-internal-launch-proof={}", "A".repeat(43));
        let good = a(&[
            bun.to_str().unwrap(),
            cli.to_str().unwrap(),
            &proof,
            "start",
            "--port",
            "0",
        ]);
        assert!(exact_root(&root, node, bin));
        assert!(exact_bun_chain(&good, bun, cli, &root));
        for port in ["+0", "00", "65536", "-1", "1e3"] {
            let mut bad = root.clone();
            bad[4] = port.into();
            assert!(!exact_root(&bad, node, bin));
        }
        for index in 0..good.len() {
            let mut bad = good.clone();
            bad[index] = "foreign".into();
            assert!(
                !exact_bun_chain(&bad, bun, cli, &root),
                "changed arg {index}"
            );
        }
        let mut missing = good.clone();
        missing.remove(2);
        assert!(!exact_bun_chain(&missing, bun, cli, &root));
        let mut extra = good.clone();
        extra.push("--eval".into());
        assert!(!exact_bun_chain(&extra, bun, cli, &root));
        let mut other_root = root.clone();
        other_root[4] = "10100".into();
        assert!(!exact_bun_chain(&good, bun, cli, &other_root));
        // These pure predicates cannot construct any native/source/role proof;
        // the synthetic capability above is not accepted as real authentication.
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
