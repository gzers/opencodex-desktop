#![cfg(unix)]
//! 扩展只读发现契约集成测试：数据源、多客户端 MCP、脱敏与失败保留。

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use opencodex_desktop_lib::commands::extensions::{
    execute_extension_write_with_paths, extension_config_with_paths, list_extensions_with_paths,
    read_mcp_detail_with_paths, read_skill_detail_with_paths,
    set_extension_client_enabled_with_paths, ExtensionWriteCommand,
};
use opencodex_desktop_lib::errors::AppError;
use opencodex_desktop_lib::modules::data_root;
use opencodex_desktop_lib::modules::extensions::discovery::read_server_definition;
use opencodex_desktop_lib::modules::extensions::projection;
use opencodex_desktop_lib::modules::extensions::{ClientId, ClientTarget, CLIENT_IDS};
use opencodex_desktop_lib::types::extensions::{ExtensionSkillDto, ExtensionsDto};

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let root = tempfile::tempdir().expect("create temp fixture");
    let data_root_path = root.path().join("data-root");
    let home = root.path().join("home");
    fs::create_dir_all(&home).expect("create home");
    data_root::initialize(&data_root_path).expect("initialize data root");
    (root, data_root_path, home)
}

fn codex_mcp_payload() -> String {
    r#"
[mcp_servers.node_repl]
type = "stdio"
command = "node"
args = ["--foo"]
env = { TOKEN = "secret-value", SAFE_KEY = "x" }
description = "https://user:token@example.com/v1?api_key=secret"
"#
    .to_string()
}

fn skill_manifest(description: &str) -> String {
    format!("---\nname: design-studio\ndescription: {description}\n---\n# 标题\n\n- 列表项\n")
}

#[test]
fn reads_skill_detail_with_bounded_body_and_directory_stats() {
    let (_root, data_root, home) = fixture();
    let skill_dir = home.join(".agents/skills/design-studio");
    fs::create_dir_all(skill_dir.join("references")).expect("create skill fixture");
    fs::write(skill_dir.join("SKILL.md"), skill_manifest("设计与实现。")).expect("write manifest");
    fs::write(skill_dir.join("references/guide.md"), "guide").expect("write reference");
    fs::write(skill_dir.join("assets.bin"), [0u8, 1, 2, 3]).expect("write asset");

    let detail = read_skill_detail_with_paths("design-studio", &data_root, &home).expect("detail");
    assert_eq!(detail.name, "design-studio");
    assert_eq!(detail.entry_file, "SKILL.md");
    let body = detail.body.as_deref().unwrap_or_default();
    assert!(body.contains("# 标题"));
    // frontmatter 里的 name / description 已由信息区展示，正文里不再重复渲染。
    assert!(
        !body.contains("description:"),
        "正文不应再带 frontmatter 描述：{body}"
    );
    assert!(
        !body.starts_with("---"),
        "正文不应以 frontmatter 分隔符开头"
    );
    assert!(!detail.body_truncated);
    // SKILL.md + references/guide.md + assets.bin
    assert_eq!(detail.file_count, Some(3));
    assert!(detail.total_bytes.unwrap_or(0) > 0);
    assert_eq!(detail.stats_note, None);
    assert!(std::path::Path::new(&detail.source_path).starts_with(home.join(".agents/skills")));
}

#[test]
fn rejects_unsafe_skill_names_and_unknown_skills() {
    let (_root, data_root, home) = fixture();
    fs::create_dir_all(home.join(".agents/skills/design-studio")).expect("create skill fixture");
    fs::write(
        home.join(".agents/skills/design-studio/SKILL.md"),
        skill_manifest("设计与实现。"),
    )
    .expect("write manifest");
    // 详情入口来自界面，但仍按不可信输入处理：路径穿越与缺失条目都必须失败。
    for name in ["../design-studio", "design-studio/..", "", "missing-skill"] {
        let error = read_skill_detail_with_paths(name, &data_root, &home).expect_err("must fail");
        assert!(
            matches!(error, AppError::NotFound { .. }),
            "unexpected error for {name:?}: {error:?}"
        );
    }
}

#[test]
fn truncates_oversized_skill_body_instead_of_reading_it_unbounded() {
    let (_root, data_root, home) = fixture();
    let skill_dir = home.join(".agents/skills/huge-skill");
    fs::create_dir_all(&skill_dir).expect("create skill fixture");
    // 无 frontmatter 的正文：截断后长度应恰为上限，便于断言截断生效。
    let payload = "x".repeat(300 * 1024);
    fs::write(skill_dir.join("SKILL.md"), payload).expect("write manifest");

    let detail = read_skill_detail_with_paths("huge-skill", &data_root, &home).expect("detail");
    assert!(detail.body_truncated, "超上限正文必须标记截断");
    assert_eq!(detail.body.as_deref().unwrap_or_default().len(), 256 * 1024);
}

#[test]
fn mcp_detail_masks_sensitive_args_and_reports_client_landings() {
    let (_root, data_root, home) = fixture();
    fs::create_dir_all(home.join(".codex")).expect("create codex dir");
    fs::write(
        home.join(".codex/config.toml"),
        r#"
[mcp_servers.node_repl]
type = "stdio"
command = "node"
args = ["--token=abc123", "--flag"]
env = { TOKEN = "secret-value", SAFE_KEY = "x" }
description = "本地工具"
"#,
    )
    .expect("write codex mcp");

    let detail = read_mcp_detail_with_paths("node_repl", &data_root, &home).expect("detail");
    assert_eq!(detail.name, "node_repl");
    assert_eq!(detail.transport, "stdio");
    assert_eq!(detail.command.as_deref(), Some("node"));
    assert_eq!(detail.args, vec!["--token=••••", "--flag"]);
    assert!(detail.env_keys.contains(&"TOKEN".to_string()));
    // 落点覆盖全部客户端；只有 Codex 真的有这条配置。
    assert_eq!(detail.landings.len(), CLIENT_IDS.len());
    let codex = detail
        .landings
        .iter()
        .find(|landing| landing.client == ClientId::Codex)
        .expect("codex landing");
    assert!(codex.present);
    assert!(codex.config_path.ends_with(".codex/config.toml"));
    assert_eq!(codex.key, "mcp_servers");
    // 展示用 JSON 与发现结果同一套掩码口径：不得出现原始敏感值。
    let json = detail.config_json.expect("config json");
    assert!(json.contains("••••"), "掩码占位符缺失");
    assert!(!json.contains("abc123"), "原始令牌泄漏：{json}");
    assert!(!json.contains("secret-value"), "环境变量原值泄漏：{json}");
}

#[test]
fn lists_source_skills_and_codex_mcp_with_sanitized_description() {
    let (_root, data_root, home) = fixture();
    // 默认读源 = Agent Skills 共享目录。
    let skill_dir = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&skill_dir).expect("create skill fixture");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: design-studio\ndescription: 设计与实现。\n---\n# sample\n",
    )
    .expect("write manifest");

    fs::create_dir_all(home.join(".codex")).expect("create codex dir");
    fs::write(home.join(".codex/config.toml"), codex_mcp_payload()).expect("write codex mcp");
    fs::create_dir_all(home.join(".codex/skills/design-studio")).expect("create target");

    let result = list_extensions_with_paths(&data_root, &home).expect("discover");
    assert_eq!(result.skills.len(), 1);
    let skill = &result.skills[0];
    assert_eq!(skill.name, "design-studio");
    assert_eq!(
        opencodex_desktop_lib::modules::extensions::discovery::SkillSourceKind::Agents,
        skill.source
    );
    assert_eq!(skill.description, "设计与实现。");
    assert_eq!(skill.clients, vec![ClientId::Codex]);
    assert!(skill.updated_at.is_some());

    assert_eq!(result.servers.len(), 1);
    let server = &result.servers[0];
    assert_eq!(server.name, "node_repl");
    assert_eq!(server.transport, "stdio");
    assert_eq!(server.command.as_deref(), Some("node"));
    assert_eq!(server.args, vec!["--foo"]);
    assert_eq!(server.env_keys, vec!["SAFE_KEY", "TOKEN"]);
    assert_eq!(server.description, "https://example.com/v1");
    assert_eq!(server.clients, vec![ClientId::Codex]);
}

#[test]
fn empty_sources_render_explicit_empty_states_without_writing() {
    let (_root, data_root, home) = fixture();
    let result: ExtensionsDto = list_extensions_with_paths(&data_root, &home).expect("discover");
    assert!(result.skills.is_empty());
    assert!(result.servers.is_empty());
    assert!(!home.join(".agents/skills").exists());
    assert!(!home.join(".codex").exists());
    assert!(!home.join(".claude.json").exists());
}

#[test]
fn merges_source_and_store_and_prefers_agents_source() {
    let (_root, data_root, home) = fixture();
    // 管理器写入区（导入 / 恢复产物）。
    let store = data_root.join("manager-state/skills-store/shared");
    fs::create_dir_all(&store).expect("store");
    fs::write(
        store.join("SKILL.md"),
        "---\ndescription: manager-store\n---\n",
    )
    .expect("store manifest");
    // 默认读源（Agent Skills 共享目录）。
    let source = home.join(".agents/skills/shared");
    fs::create_dir_all(&source).expect("source");
    fs::write(
        source.join("SKILL.md"),
        "---\ndescription: agents-source\n---\n",
    )
    .expect("source manifest");
    let store_only = data_root.join("manager-state/skills-store/imported-only");
    fs::create_dir_all(&store_only).expect("store only");

    let result = list_extensions_with_paths(&data_root, &home).expect("discover");
    assert_eq!(result.skills.len(), 2);
    let shared = result
        .skills
        .iter()
        .find(|skill| skill.name == "shared")
        .expect("shared");
    // 同名时默认读源胜出。
    assert_eq!(shared.description, "agents-source");
    assert_eq!(
        opencodex_desktop_lib::modules::extensions::discovery::SkillSourceKind::Agents,
        shared.source
    );
    let imported = result
        .skills
        .iter()
        .find(|skill| skill.name == "imported-only")
        .expect("imported");
    assert_eq!(
        opencodex_desktop_lib::modules::extensions::discovery::SkillSourceKind::Store,
        imported.source
    );
}

#[test]
fn dto_contract_uses_camel_case_and_masks_raw_env_values() {
    let skill = ExtensionSkillDto {
        id: "sample".into(),
        name: "sample".into(),
        source: opencodex_desktop_lib::modules::extensions::discovery::SkillSourceKind::Agents,
        updated_at: Some("2026-09-15T00:00:00Z".into()),
        description: "safe".into(),
        clients: vec![ClientId::Codex],
    };
    let payload = serde_json::to_value(&skill).expect("serialize skill");
    assert_eq!(payload["updatedAt"], "2026-09-15T00:00:00Z");
    assert_eq!(payload["clients"][0], "codex");

    let source = codex_mcp_payload();
    assert!(source.contains("secret-value"));
    let serialized = serde_json::to_string(&skill).expect("serialize skill");
    assert!(!serialized.contains("secret-value"));
}

#[expect(dead_code)]
fn client_targets(home: &std::path::Path) -> Vec<ClientTarget> {
    CLIENT_IDS
        .map(|client| ClientTarget::user_target(client, home, true, true))
        .to_vec()
}

#[test]
fn unified_config_write_creates_locked_backed_projection_and_toggles_once() {
    let (_root, data_root, home) = fixture();
    std::fs::create_dir_all(home.join(".codex")).expect("create explicit context");
    std::fs::create_dir_all(home.join(".codex/skills")).expect("create skills context");

    let result = set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, false)
        .expect("write projection");
    assert_eq!(result.revision, 2);
    assert_eq!(result.enablement.get(&ClientId::Codex), Some(&false));
    assert!(!result.backed_up);

    let path = data_root.join("manager-state/extension-config.json");
    assert_eq!(
        std::fs::metadata(&path)
            .expect("projection metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let loaded = extension_config_with_paths(&data_root, &home).expect("load projection");
    assert_eq!(loaded.revision, 2);
    assert_eq!(loaded.conflict, None);
    assert_eq!(loaded.document_sha256, result.document_sha256);

    let second = set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("second write");
    assert_eq!(second.revision, 3);
    assert!(second.backed_up);
    let backups = data_root.join("backups");
    let mut found = false;
    for year in std::fs::read_dir(&backups).expect("backup years") {
        let year = year.expect("year").path();
        for month in std::fs::read_dir(&year).expect("backup months") {
            let action = month.expect("month").path().join("extension-write");
            if action.is_dir() {
                found = std::fs::read_dir(&action).expect("action dir").count() > 0;
            }
        }
    }
    assert!(found);
}

#[test]
fn unified_config_rejects_modified_projection_before_write() {
    let (_root, data_root, home) = fixture();
    std::fs::create_dir_all(home.join(".codex")).expect("create explicit context");
    std::fs::create_dir_all(home.join(".codex/skills")).expect("create skills context");

    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("create projection");
    let path = data_root.join("manager-state/extension-config.json");
    std::fs::write(&path, b"not projection").expect("modify projection");

    let error = set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect_err("reject modified projection");
    assert!(error.to_string().contains("projection is invalid"));
}

#[test]
fn unified_config_rejects_inconsistent_or_missing_target_context() {
    let (_root, data_root, home) = fixture();
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("create projection");

    let path = data_root.join("manager-state/extension-config.json");
    let mut payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("read projection"))
            .expect("parse projection");
    payload["config"]["enablement"]
        .as_object_mut()
        .expect("enablement")
        .remove("gemini");
    payload["config"]["revision"] =
        serde_json::Value::from(payload["config"]["revision"].as_u64().expect("revision") + 1);
    payload["fingerprint"]["fingerprint"] = serde_json::Value::from("0".repeat(64));
    std::fs::write(&path, serde_json::to_vec(&payload).expect("serialize")).expect("mutate");

    let error = set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect_err("reject inconsistent targets");
    assert!(error.to_string().contains("external target changed"));
}

fn write_all_clients(home: &std::path::Path) {
    for client in CLIENT_IDS {
        let target = ClientTarget::user_target(client, home, true, true);
        if let Some(parent) = target.mcp_config_path.parent() {
            fs::create_dir_all(parent).expect("create client parent");
        }
    }
}

fn codex_toml() -> String {
    r#"
[mcp_servers.existing]
type = "stdio"
command = "keep"
args = ["--keep"]
env = { KEEP = "secret" }

[mcp_servers.shared]
type = "local"
command = "node"
args = ["--foo", "--stdio"]
env = { TOKEN = "secret-value" }
"#
    .to_string()
}

fn claude_json() -> String {
    r#"{"other":{"enabled":true},"mcp_servers":{"existing":{"type":"stdio","command":"keep","args":["--keep"],"environment":{"KEEP":"secret"}}}}"#.to_string()
}

fn grok_json() -> String {
    r#"{"mcp_servers":[{"name":"existing","type":"stdio","command":"keep"}]}"#.to_string()
}

fn hermes_yaml() -> String {
    "mcp_servers:\n  existing:\n    type: stdio\n    command: keep\n".to_string()
}

fn populate_live_configs(home: &std::path::Path) {
    fs::write(home.join(".codex/config.toml"), codex_toml()).expect("codex config");
    fs::write(home.join(".claude.json"), claude_json()).expect("claude config");
    fs::write(home.join(".gemini/settings.json"), claude_json()).expect("gemini config");
    fs::write(home.join(".grok/user-settings.json"), grok_json()).expect("grok config");
    fs::write(home.join(".config/opencode/opencode.json"), claude_json()).expect("opencode config");
    fs::write(home.join(".hermes/config.yaml"), hermes_yaml()).expect("hermes config");
}

/// 回归（2026-10-01 真机发现）：MCP 客户端配置的「投影前备份」曾用
/// `skills_dir.ancestors().nth(4)` 当备份根，而不是数据根——各客户端 skills_dir 深度不同
/// （`.config/opencode/skills` 比 `.claude/skills` 深一层），于是备份被写到
/// `$HOME/..`（隔离根下）或系统临时目录（测试下），真实 `$HOME` 下更会退化到 `/`。
#[test]
fn mcp_projection_backups_land_in_the_data_root_not_next_to_home() {
    let (root, data_root, home) = fixture();
    write_all_clients(&home);
    populate_live_configs(&home);
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("initialize projection");

    let added = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::AddMcp {
            definition: projection::RawServerDefinition {
                name: "local_repl".into(),
                value: serde_json::json!({ "command": "node", "args": ["--stdio"] }),
            },
        },
    )
    .expect("add mcp to every enabled client");
    assert!(added.mcp_written.contains(&"opencode".to_string()));
    assert!(added.mcp_written.contains(&"claude".to_string()));

    fn collect(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else if path
                .file_name()
                .is_some_and(|name| name == "backup-manifest.json")
            {
                out.push(path);
            }
        }
    }

    let mut manifests = Vec::new();
    collect(&data_root.join("backups"), &mut manifests);
    let mcp_backups: Vec<String> = manifests
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter(|text| text.contains("before extension MCP projection write"))
        .collect();
    assert!(
        mcp_backups.len() >= 2,
        "每个被写客户端都应有一份投影前备份，实际 {}",
        mcp_backups.len()
    );
    assert!(mcp_backups.iter().any(|text| text.contains(".claude.json")));
    assert!(mcp_backups
        .iter()
        .any(|text| text.contains("opencode/opencode.json")));

    // 数据根之外不得再出现备份树：旧实现的 opencode 分支会落到 <fixture>/backups。
    assert!(
        !root.path().join("backups").exists(),
        "备份不得落在数据根之外"
    );
    assert!(!home.join("backups").exists());
}

/// 逐客户端连接：图标开关的语义就是「把该 Skill 连到这一个客户端」。
fn link_skill_for(
    data_root: &std::path::Path,
    home: &std::path::Path,
    name: &str,
    client: ClientId,
) -> Vec<String> {
    execute_extension_write_with_paths(
        data_root,
        home,
        ExtensionWriteCommand::LinkSkill {
            name: name.into(),
            client,
        },
    )
    .expect("link skill for one client")
    .skills_linked
}

/// 逐个客户端连接（等价旧「全部启用客户端」行为，用来说明逐客户端语义）。
fn link_skill_all(data_root: &std::path::Path, home: &std::path::Path, name: &str) -> Vec<String> {
    let mut linked = Vec::new();
    for client in CLIENT_IDS {
        linked.extend(link_skill_for(data_root, home, name, client));
    }
    linked
}

fn unlink_skill_for(
    data_root: &std::path::Path,
    home: &std::path::Path,
    name: &str,
    client: ClientId,
) -> Vec<String> {
    execute_extension_write_with_paths(
        data_root,
        home,
        ExtensionWriteCommand::UnlinkSkill {
            name: name.into(),
            client,
            confirm: true,
        },
    )
    .expect("unlink skill for one client")
    .skills_linked
}

fn write_mcp_for(
    data_root: &std::path::Path,
    home: &std::path::Path,
    name: &str,
    client: ClientId,
) -> Vec<String> {
    execute_extension_write_with_paths(
        data_root,
        home,
        ExtensionWriteCommand::WriteMcp {
            name: name.into(),
            client,
            confirm: true,
        },
    )
    .expect("write mcp for one client")
    .mcp_written
}

#[test]
fn links_skill_from_agents_source_without_manager_import() {
    // 默认读源是 Agent Skills 共享目录：只存在于 `~/.agents/skills` 的 Skill
    // 也应能直接同步，不需要先导入管理器写入区（此前会报「同步失败」）。
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let source = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&source).expect("source skill");
    fs::write(source.join("SKILL.md"), "demo").expect("manifest");
    let store_copy = data_root.join("manager-state/skills-store/design-studio");
    assert!(!store_copy.exists());

    // 先建立权威投影（保持 Codex 启用）。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");

    let result = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::LinkSkill {
            name: "design-studio".into(),
            client: ClientId::Codex,
        },
    )
    .expect("link from agents source");
    assert!(
        result
            .skills_linked
            .iter()
            .any(|entry| entry.contains("codex") && entry.contains("design-studio")),
        "codex 应从默认读源完成链接：{:?}",
        result.skills_linked
    );
    // 逐客户端语义：只连被点的那一个，其余客户端保持未同步。
    for relative in [
        ".claude/skills/design-studio",
        ".gemini/skills/design-studio",
        ".grok/skills/design-studio",
        ".config/opencode/skills/design-studio",
    ] {
        assert!(
            !home.join(relative).exists(),
            "未点选的客户端不应被写：{relative}"
        );
    }

    let link = home.join(".codex/skills/design-studio");
    assert_eq!(
        fs::read_link(&link).expect("symlink"),
        source.canonicalize().expect("canonical source")
    );
    // 读源保持只读语义：没有把内容复制进管理器写入区。
    assert!(!store_copy.exists());

    // 卸载只断开客户端链接，不删除默认读源里的内容。
    execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::UninstallSkill {
            name: "design-studio".into(),
            confirm: true,
        },
    )
    .expect("uninstall source-only skill");
    assert!(!link.exists());
    assert!(source.join("SKILL.md").exists());
}

#[test]
fn skill_link_skips_existing_real_directory_without_overwriting() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let source = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&source).expect("source skill");
    fs::write(source.join("SKILL.md"), "demo").expect("manifest");
    // Claude 目录里已有同名真实目录（用户自有内容，不归管理器）。
    let owned = home.join(".claude/skills/design-studio");
    fs::create_dir_all(&owned).expect("owned dir");
    fs::write(owned.join("KEEP.md"), "mine").expect("owned file");

    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");
    // 先连 Codex，再连 Claude（该目录被用户自有内容占着）。
    let codex_linked = link_skill_for(&data_root, &home, "design-studio", ClientId::Codex);
    let result = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::LinkSkill {
            name: "design-studio".into(),
            client: ClientId::Claude,
        },
    )
    .expect("link must not fail because the target holds a real directory");

    // 真实目录保持原样，没有被链接或删除。
    assert!(owned.join("KEEP.md").exists());
    assert!(!fs::symlink_metadata(&owned).expect("owned").is_symlink());
    let linked = result.skills_linked.join(",");
    assert!(!linked.contains("claude"), "冲突目标应被跳过：{linked}");
    assert!(
        codex_linked.iter().any(|entry| entry.contains("codex")),
        "codex 链接应已建立：{codex_linked:?}"
    );
    assert!(
        fs::symlink_metadata(home.join(".codex/skills/design-studio"))
            .expect("codex link")
            .is_symlink()
    );
}

#[test]
fn extension_write_projects_skills_and_mcp_across_clients() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    populate_live_configs(&home);
    let skill = data_root.join("manager-state/skills-store/design-studio");
    fs::create_dir_all(&skill).expect("skill source");
    fs::write(skill.join("SKILL.md"), "demo").expect("skill manifest");

    let result = set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, false)
        .expect("initialize projection");
    assert_eq!(result.revision, 2);
    assert!(!home.join(".codex/skills/design-studio").exists());

    // 逐客户端连接：只连 Claude，其余客户端保持未同步，Codex 也仍保持停用。
    let linked = link_skill_for(&data_root, &home, "design-studio", ClientId::Claude);
    assert!(linked.contains(&"claude/design-studio".to_string()));
    for relative in [
        ".codex/skills/design-studio",
        ".gemini/skills/design-studio",
        ".grok/skills/design-studio",
        ".config/opencode/skills/design-studio",
        ".hermes/skills/design-studio",
    ] {
        assert!(
            !home.join(relative).exists(),
            "未点选的客户端不应被写：{relative}"
        );
    }

    // MCP 也是逐客户端写入：逐个客户端各写一次，六份配置都应有该定义。
    let mut written = Vec::new();
    for client in CLIENT_IDS {
        written.extend(write_mcp_for(&data_root, &home, "shared", client));
    }
    assert_eq!(written.len(), 6, "{written:?}");

    let codex = fs::read_to_string(home.join(".codex/config.toml")).expect("codex output");
    assert!(codex.contains("[mcp_servers.shared]"));
    assert!(codex.contains("command = \"node\""));
    assert!(codex.contains("TOKEN = \"secret-value\""));
    assert!(codex.contains("[mcp_servers.existing]"));

    let claude: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".claude.json")).expect("claude output"))
            .expect("parse claude");
    assert_eq!(
        claude["mcp_servers"]["shared"]["type"],
        serde_json::Value::String("stdio".into())
    );
    assert_eq!(
        claude["mcp_servers"]["shared"]["env"]["TOKEN"],
        serde_json::Value::String("secret-value".into())
    );
    assert_eq!(claude["other"]["enabled"], serde_json::Value::Bool(true));

    let grok: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".grok/user-settings.json")).expect("grok"))
            .expect("parse grok");
    assert_eq!(grok["mcp_servers"].as_array().expect("array").len(), 2);
    assert_eq!(
        grok["mcp_servers"][0]["name"],
        serde_json::Value::String("existing".into())
    );
    assert_eq!(
        grok["mcp_servers"][1]["name"],
        serde_json::Value::String("shared".into())
    );

    let hermes = fs::read_to_string(home.join(".hermes/config.yaml")).expect("hermes");
    assert!(hermes.contains("shared:"));
    assert!(hermes.contains("env:"));
    assert!(hermes.contains("existing:"));
}

#[test]
fn extension_write_requires_confirmation_for_new_conflicts_and_backs_up_targets() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    populate_live_configs(&home);
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, false)
        .expect("initialize projection");

    let before = fs::read(home.join(".claude.json")).expect("before");
    let error = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::WriteMcp {
            name: "shared".into(),
            client: ClientId::Claude,
            confirm: false,
        },
    )
    .expect_err("unconfirmed conflict must stop");
    assert!(error.to_string().contains("requires confirmation"));
    assert_eq!(fs::read(home.join(".claude.json")).expect("after"), before);

    // 逐客户端：只写 Claude 与 Codex，其余客户端配置不动。
    let gemini_before = fs::read(home.join(".gemini/settings.json")).expect("gemini before");
    let result = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::WriteMcp {
            name: "shared".into(),
            client: ClientId::Claude,
            confirm: true,
        },
    )
    .expect("confirmed write");
    assert!(result.config.backed_up);
    assert_eq!(result.mcp_written.len(), 1);
    assert_eq!(result.mcp_written[0], "claude");
    assert_eq!(
        write_mcp_for(&data_root, &home, "shared", ClientId::Codex).len(),
        1
    );
    let codex_after_write = fs::read_to_string(home.join(".codex/config.toml")).expect("codex");
    assert!(codex_after_write.contains("[mcp_servers.existing]"));
    assert!(codex_after_write.contains("[mcp_servers.shared]"));

    // 断开只作用于被点的客户端：Claude 里移除，Codex 仍保留。
    let removed = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::RemoveMcp {
            name: "shared".into(),
            client: Some(ClientId::Claude),
            confirm: true,
        },
    )
    .expect("remove mcp for one client");
    assert_eq!(removed.mcp_written, vec!["claude".to_string()]);
    assert_eq!(
        fs::read(home.join(".gemini/settings.json")).expect("gemini after"),
        gemini_before,
        "未点选的客户端不应被改"
    );
    let claude: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".claude.json")).expect("claude"))
            .expect("parse claude");
    assert!(claude
        .get("mcp_servers")
        .is_some_and(|node| !node.as_object().expect("map").contains_key("shared")));
    assert!(claude
        .get("mcp_servers")
        .is_some_and(|node| node.as_object().expect("map").contains_key("existing")));
    let codex = fs::read_to_string(home.join(".codex/config.toml")).expect("codex");
    assert!(
        codex.contains("[mcp_servers.shared]"),
        "只断 Claude 时 Codex 的定义必须保留"
    );
}

#[test]
fn extension_write_unlinks_skill_with_confirm_and_rejects_escape_names() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let skill = data_root.join("manager-state/skills-store/escape");
    fs::create_dir_all(&skill).expect("skill source");
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, false)
        .expect("initialize projection");

    execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::LinkSkill {
            name: "escape".into(),
            client: ClientId::Codex,
        },
    )
    .expect("link");

    let error = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::UnlinkSkill {
            name: "../escape".into(),
            client: ClientId::Codex,
            confirm: true,
        },
    )
    .expect_err("path escape must be rejected");
    assert!(error.to_string().contains("escapes"));

    execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::UnlinkSkill {
            name: "escape".into(),
            client: ClientId::Codex,
            confirm: true,
        },
    )
    .expect("unlink");
    assert!(!home.join(".codex/skills/escape").exists());
    // 断链不动本源：管理器写入区里的本体仍在。
    assert!(skill.is_dir(), "断开链接不应删除源本体");
}

// 用户口径（2026-09-24）：源头 Skill 只读，客户端目录里只是链接；
// 图标开关 = 「连/断这一个客户端的链接」，既不该连累别的客户端，也绝不动本源。
#[test]
fn skill_toggle_links_only_the_clicked_client_and_never_touches_the_source() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let source = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&source).expect("source skill");
    fs::write(source.join("SKILL.md"), "demo").expect("manifest");
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");

    // 点 Claude：只有 Claude 亮。
    let linked = link_skill_for(&data_root, &home, "design-studio", ClientId::Claude);
    assert_eq!(linked, vec!["claude/design-studio".to_string()]);
    for relative in [
        ".codex/skills/design-studio",
        ".gemini/skills/design-studio",
        ".grok/skills/design-studio",
        ".config/opencode/skills/design-studio",
        ".hermes/skills/design-studio",
    ] {
        assert!(!home.join(relative).exists(), "不应被连带点亮：{relative}");
    }

    // 再点 Codex：只多 Codex。
    let linked = link_skill_for(&data_root, &home, "design-studio", ClientId::Codex);
    assert_eq!(linked, vec!["codex/design-studio".to_string()]);
    assert!(
        fs::symlink_metadata(home.join(".codex/skills/design-studio"))
            .expect("codex link")
            .is_symlink()
    );

    // 再点一次 Claude（关闭）：只断 Claude，Codex 与本源的链接关系不受影响。
    unlink_skill_for(&data_root, &home, "design-studio", ClientId::Claude);
    assert!(!home.join(".claude/skills/design-studio").exists());
    assert!(
        fs::symlink_metadata(home.join(".codex/skills/design-studio"))
            .expect("codex link")
            .is_symlink(),
        "断一个客户端不应影响其它客户端"
    );
    assert!(
        source.join("SKILL.md").exists(),
        "源目录只读：断开链接绝不能动本源 Skill"
    );
}

// 升级兼容：旧配置没有 `skill_targets` 时沿用「所有启用客户端」，
// 不能让既有落地在升级后被集体断开；用户第一次逐个开关时以实际落地为基线。
#[test]
fn legacy_config_without_skill_targets_keeps_existing_links_then_follows_explicit_toggles() {
    use opencodex_desktop_lib::modules::extensions::projection::save_with_config;
    use opencodex_desktop_lib::modules::extensions::ExtensionConfig;

    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let source = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&source).expect("source skill");
    fs::write(source.join("SKILL.md"), "demo").expect("manifest");
    let canonical_home = home.canonicalize().expect("canonical home");
    let targets =
        CLIENT_IDS.map(|client| ClientTarget::user_target(client, &canonical_home, true, true));

    let mut legacy = ExtensionConfig::default();
    legacy.skills.push("design-studio".to_string());
    legacy.enablement.insert(ClientId::Hermes, false);
    save_with_config(&data_root, &mut legacy, &targets).expect("save legacy projection");

    // 旧配置按「所有启用客户端」重放：Hermes 停用，其余五个都落链接。
    execute_extension_write_with_paths(&data_root, &home, ExtensionWriteCommand::ResyncSkills)
        .expect("resync legacy projection");
    for relative in [
        ".codex/skills/design-studio",
        ".claude/skills/design-studio",
        ".gemini/skills/design-studio",
        ".grok/skills/design-studio",
        ".config/opencode/skills/design-studio",
    ] {
        assert!(
            home.join(relative).exists(),
            "旧配置应保持全量链接：{relative}"
        );
    }
    assert!(!home.join(".hermes/skills/design-studio").exists());

    // 第一次逐个开关：以「当前实际落地」为基线，只改变被点的那一个客户端。
    unlink_skill_for(&data_root, &home, "design-studio", ClientId::Claude);
    assert!(!home.join(".claude/skills/design-studio").exists());
    for relative in [
        ".codex/skills/design-studio",
        ".gemini/skills/design-studio",
        ".grok/skills/design-studio",
        ".config/opencode/skills/design-studio",
    ] {
        assert!(
            home.join(relative).exists(),
            "只断 Claude，其它客户端不该被连带断开：{relative}"
        );
    }
    assert!(source.join("SKILL.md").exists());
}

fn base64_encode(payload: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    BASE64.encode(payload)
}

fn skill_zip(entries: &[(&str, &str)]) -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(cursor);
    let options: zip::write::SimpleFileOptions = zip::write::FileOptions::default();
    for (name, payload) in entries {
        archive
            .start_file(*name, options)
            .expect("start zip member");
        std::io::Write::write_all(&mut archive, payload.as_bytes()).expect("write zip member");
    }
    archive.finish().expect("finish zip").into_inner()
}

#[test]
fn skill_archive_import_persists_export_and_preferred_store() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("initialize projection");

    let bytes = skill_zip(&[
        (
            "sample/SKILL.md",
            "---\nname: sample\ndescription: ZIP skill\n---\nbody",
        ),
        ("sample/reference.md", "sample"),
    ]);
    let result = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::ImportSkillArchive {
            archive_name: "sample.zip".into(),
            archive_payload: base64_encode(&bytes),
        },
    )
    .expect("import archive");

    assert!(result.config.skills.contains(&"sample".to_string()));
    let export = data_root.join("exports/sample.zip");
    assert_eq!(
        fs::metadata(&export)
            .expect("export metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(
        fs::read_to_string(data_root.join("manager-state/skills-store/sample/SKILL.md"))
            .expect("store skill"),
        "---\nname: sample\ndescription: ZIP skill\n---\nbody"
    );

    let discovered = list_extensions_with_paths(&data_root, &home).expect("discover");
    assert!(discovered.skills.iter().any(|skill| skill.name == "sample"));
}

#[test]
fn skill_archive_import_rejects_traversal_without_store_writes() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("initialize projection");

    let bytes = skill_zip(&[("sample/../../escape.txt", "unsafe")]);
    let error = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::ImportSkillArchive {
            archive_name: "unsafe.zip".into(),
            archive_payload: base64_encode(&bytes),
        },
    )
    .expect_err("unsafe archive must fail");
    assert!(error.to_string().contains("invalid or unsafe"));
    assert!(!data_root
        .join("manager-state/skills-store/escape.txt")
        .exists());
    assert!(!data_root.join("manager-state/skills-store/sample").exists());
    assert!(!data_root
        .join("manager-state/skills-store/sample.zip")
        .exists());
}

#[test]
fn skill_uninstall_backups_directory_and_restore_recovers_latest() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let skill = data_root.join("manager-state/skills-store/sample");
    fs::create_dir_all(&skill).expect("skill fixture");
    fs::create_dir_all(skill.join("nested")).expect("nested fixture");
    fs::write(skill.join("SKILL.md"), "old content").expect("old manifest");
    fs::write(skill.join("nested/reference.md"), "reference").expect("nested file");
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("initialize projection");
    link_skill_for(&data_root, &home, "sample", ClientId::Codex);

    let unconfirmed =
        opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
            &data_root,
            &home,
            opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::UninstallSkill {
                name: "sample".into(),
                confirm: false,
            },
        )
        .expect_err("unconfirmed uninstall must fail");
    assert!(unconfirmed.to_string().contains("requires confirmation"));
    assert!(skill.join("SKILL.md").exists());

    opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::UninstallSkill {
            name: "sample".into(),
            confirm: true,
        },
    )
    .expect("uninstall skill");
    assert!(!skill.join("SKILL.md").exists());
    assert!(!home.join(".codex/skills/sample").exists());

    fs::create_dir_all(&skill).expect("recreate skill directory");
    fs::write(skill.join("SKILL.md"), "new unrelated content").expect("new content");
    opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::RestoreSkill {
            name: "sample".into(),
        },
    )
    .expect("restore skill");
    assert_eq!(
        fs::read_to_string(skill.join("SKILL.md")).expect("restored manifest"),
        "old content"
    );
    assert_eq!(
        fs::read_to_string(skill.join("nested/reference.md")).expect("restored reference"),
        "reference"
    );
}

#[test]
fn add_edit_and_delete_mcp_writes_enabled_clients_with_confirmation() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    fs::create_dir_all(home.join(".codex")).expect("codex context");
    fs::write(home.join(".codex/config.toml"), "").expect("empty codex config");
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("enable codex");
    for client in [
        ClientId::Claude,
        ClientId::Gemini,
        ClientId::Opencode,
        ClientId::Hermes,
    ] {
        set_extension_client_enabled_with_paths(&data_root, &home, client, false)
            .expect("disable other client");
    }

    let invalid = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::AddMcp {
            definition:
                opencodex_desktop_lib::modules::extensions::projection::RawServerDefinition {
                    name: "local_repl".into(),
                    value: serde_json::json!({ "command": "   " }),
                },
        },
    )
    .expect_err("invalid definition must fail");
    assert!(invalid.to_string().contains("not supported"));

    let added = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::AddMcp {
            definition:
                opencodex_desktop_lib::modules::extensions::projection::RawServerDefinition {
                    name: "local_repl".into(),
                    value: serde_json::json!({
                        "command": "node",
                        "args": ["--foo"],
                        "env": { "TOKEN": "secret-value" }
                    }),
                },
        },
    )
    .expect("add mcp");
    assert_eq!(
        added.mcp_written,
        vec!["codex".to_string(), "grok".to_string()]
    );
    let config = fs::read_to_string(home.join(".codex/config.toml")).expect("codex config");
    assert!(config.contains("[mcp_servers.local_repl]"));
    assert!(config.contains("command = \"node\""));
    assert!(config.contains("TOKEN = \"secret-value\""));

    let edited = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::EditMcp {
            name: "local_repl".into(),
            definition:
                opencodex_desktop_lib::modules::extensions::projection::RawServerDefinition {
                    name: "local_repl_v2".into(),
                    value: serde_json::json!({
                        "command": "node",
                        "args": ["--bar"],
                        "env": { "TOKEN": "secret-value" }
                    }),
                },
            confirm: true,
        },
    )
    .expect("edit mcp");
    assert_eq!(
        edited.mcp_written,
        vec!["codex".to_string(), "grok".to_string()]
    );
    let edited_config = fs::read_to_string(home.join(".codex/config.toml")).expect("edited config");
    assert!(edited_config.contains("local_repl_v2"));
    assert!(!edited_config.contains("[mcp_servers.local_repl]\n"));
    assert!(edited_config.contains("--bar"));

    let grok: serde_json::Value =
        serde_json::from_slice(&fs::read(home.join(".grok/user-settings.json")).expect("grok"))
            .expect("parse grok");
    let grok_servers = grok["mcp_servers"].as_array().expect("grok array");
    assert!(grok_servers
        .iter()
        .any(|item| item["name"] == "local_repl_v2"));
    assert!(grok_servers.iter().all(|item| item["name"] != "local_repl"));

    let deleted = execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::RemoveMcp {
            name: "local_repl_v2".into(),
            client: None,
            confirm: true,
        },
    )
    .expect("delete mcp");
    assert_eq!(
        deleted.mcp_written,
        vec!["codex".to_string(), "grok".to_string()]
    );
    assert!(!fs::read_to_string(home.join(".codex/config.toml"))
        .expect("deleted config")
        .contains("local_repl_v2"));
}

// ---------------------------------------------------------------------------
// FZ-23 / FZ-24：源目录配置化与分发方式接入写入路径（2026-09-19）
// ---------------------------------------------------------------------------

fn link_skill(data_root: &std::path::Path, home: &std::path::Path, name: &str) -> Vec<String> {
    link_skill_all(data_root, home, name)
}

#[test]
fn copy_sync_method_writes_real_copies_and_replaces_leftover_symlinks() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let source = home.join(".agents/skills/design-studio");
    fs::create_dir_all(source.join("nested")).expect("source tree");
    fs::write(source.join("SKILL.md"), "demo").expect("manifest");
    fs::write(source.join("nested/asset.txt"), "payload").expect("nested file");

    // 先建立权威投影（保持 Codex 启用），默认 symlink 方式。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");
    link_skill(&data_root, &home, "design-studio");
    assert!(
        fs::symlink_metadata(home.join(".codex/skills/design-studio"))
            .expect("codex link")
            .is_symlink(),
        "默认应为软链接"
    );

    // 切到「文件复制」，再同步一次：软链接残留必须被替换成独立副本。
    opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::SetSyncMethod {
            method: "copy".into(),
        },
    )
    .expect("set copy mode");
    link_skill(&data_root, &home, "design-studio");

    let copy = home.join(".codex/skills/design-studio");
    assert!(
        !fs::symlink_metadata(&copy).expect("copy").is_symlink(),
        "复制方式下不应再是软链接"
    );
    assert_eq!(
        fs::read_to_string(copy.join("SKILL.md")).expect("copied manifest"),
        "demo"
    );
    assert_eq!(
        fs::read_to_string(copy.join("nested/asset.txt")).expect("copied nested"),
        "payload"
    );
    // 复制不影响来源目录。
    assert!(source.join("SKILL.md").exists());
}

#[test]
fn custom_source_dir_is_used_by_discovery_and_link() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let custom = _root.path().join("custom-skills");
    let skill = custom.join("design-studio");
    fs::create_dir_all(&skill).expect("custom skill");
    fs::write(skill.join("SKILL.md"), "---\nname: design-studio\n---\n").expect("manifest");

    // 设置自定义源目录。
    opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::SetSourceDir {
            path: Some(custom.display().to_string()),
        },
    )
    .expect("set custom source dir");

    // 发现应读到自定义目录里的 Skill，并回报该目录。
    let discovered = list_extensions_with_paths(&data_root, &home).expect("discover custom");
    assert!(discovered.source_dir_custom);
    assert_eq!(discovered.source_dir_notice, None);
    assert_eq!(
        discovered.source_dir,
        custom
            .canonicalize()
            .expect("canonical")
            .display()
            .to_string()
    );
    assert_eq!(discovered.skills.len(), 1);
    assert_eq!(discovered.skills[0].name, "design-studio");

    // 同步应链到自定义目录。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");
    link_skill(&data_root, &home, "design-studio");
    assert_eq!(
        fs::read_link(home.join(".codex/skills/design-studio")).expect("symlink"),
        skill.canonicalize().expect("canonical skill")
    );

    // 自定义目录是只读来源：不把内容复制进管理器写入区。
    assert!(!data_root
        .join("manager-state/skills-store/design-studio")
        .exists());
}

#[test]
fn invalid_custom_source_dir_is_rejected_and_keeps_previous_value() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let store = data_root.join("manager-state/skills-store");
    fs::create_dir_all(&store).expect("store");
    // 先建立权威投影，确保配置已存在（拒绝写入时不该改动它）。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");
    let before =
        opencodex_desktop_lib::commands::extensions::extension_config_with_paths(&data_root, &home)
            .expect("read config")
            .revision;

    // 与受管域重叠（管理器写入区）必须被拒绝。
    let rejected = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::SetSourceDir {
            path: Some(store.display().to_string()),
        },
    );
    assert!(rejected.is_err(), "重叠目录必须被拒绝");

    // 失败不改变配置：仍是默认源目录，且修订号没有被推高。
    let config =
        opencodex_desktop_lib::commands::extensions::extension_config_with_paths(&data_root, &home)
            .expect("read config");
    assert_eq!(config.source_dir, None);
    assert!(!config.source_dir_custom);
    assert_eq!(config.revision, before);
}

#[test]
fn legacy_fingerprint_schema_is_migrated_without_conflict() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");

    // 把落盘文件降级为 schema 1 且指纹按旧结构写，模拟升级前的既有配置。
    let path = data_root.join("manager-state/extension-config.json");
    let raw = fs::read_to_string(&path).expect("read config");
    let mut payload: serde_json::Value = serde_json::from_str(&raw).expect("parse config");
    payload["fingerprint"]["schema_version"] = serde_json::json!(1);
    payload["fingerprint"]["fingerprint"] = serde_json::json!("0".repeat(64));
    fs::write(&path, serde_json::to_string_pretty(&payload).unwrap()).expect("downgrade config");

    // 读取不再报 external_modified（这是升级路径的关键）。
    let config =
        opencodex_desktop_lib::commands::extensions::extension_config_with_paths(&data_root, &home)
            .expect("legacy config must stay readable");
    assert_eq!(config.conflict, None);

    // 下一次写入落回当前 schema（3 = 逐客户端分发意图 `skill_targets`）。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("write after migration");
    let migrated: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
    assert_eq!(
        migrated["fingerprint"]["schema_version"],
        serde_json::json!(3)
    );
}

#[test]
fn resync_skills_realigns_targets_after_source_dir_change_without_bumping_revision() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    let standard = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&standard).expect("standard skill");
    fs::write(standard.join("SKILL.md"), "standard").expect("manifest");

    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Hermes, false)
        .expect("initialize projection");
    link_skill(&data_root, &home, "design-studio");
    assert_eq!(
        fs::read_link(home.join(".codex/skills/design-studio")).expect("symlink"),
        standard.canonicalize().expect("canonical")
    );

    // 换到自定义源目录（同名 Skill，不同内容）。
    let custom = _root.path().join("custom-skills");
    let custom_skill = custom.join("design-studio");
    fs::create_dir_all(&custom_skill).expect("custom skill");
    fs::write(custom_skill.join("SKILL.md"), "custom").expect("manifest");

    let before = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::SetSourceDir {
            path: Some(custom.display().to_string()),
        },
    )
    .expect("set custom source")
    .config
    .revision;

    // 设置源目录本身就完成了重新投影：链接已指向新来源。
    assert_eq!(
        fs::read_link(home.join(".codex/skills/design-studio")).expect("symlink"),
        custom_skill.canonicalize().expect("canonical")
    );

    // 模拟漂移：把落点弄丢（外部删除 / 手改），再用「重新同步」对齐。
    fs::remove_file(home.join(".codex/skills/design-studio")).expect("remove link");

    let resynced = opencodex_desktop_lib::commands::extensions::execute_extension_write_with_paths(
        &data_root,
        &home,
        opencodex_desktop_lib::commands::extensions::ExtensionWriteCommand::ResyncSkills,
    )
    .expect("resync");
    assert_eq!(resynced.config.revision, before, "重新同步不应涨修订号");
    assert!(
        resynced
            .skills_linked
            .iter()
            .any(|entry| entry.contains("codex") && entry.contains("design-studio")),
        "codex 应被重建：{:?}",
        resynced.skills_linked
    );
    assert_eq!(
        fs::read_link(home.join(".codex/skills/design-studio")).expect("symlink"),
        custom_skill.canonicalize().expect("canonical")
    );
}

// 回归：「MCP 敏感值脱敏」偏好此前没有任何消费方，args 里的令牌会原样进入界面。
#[test]
fn mcp_mask_preference_controls_argument_masking() {
    let (_root, data_root, home) = fixture();
    fs::create_dir_all(home.join(".codex")).expect("create codex dir");
    fs::write(
        home.join(".codex/config.toml"),
        r#"
[mcp_servers.srv]
type = "stdio"
command = "node"
args = ["--token=abc123", "--stdio"]
env = { SAFE = "x" }
"#,
    )
    .expect("write codex mcp");

    // 默认开启：值被掩码，说明文本也同步掩码。
    let masked = list_extensions_with_paths(&data_root, &home).expect("discover masked");
    assert_eq!(masked.servers[0].args, vec!["--token=••••", "--stdio"]);
    assert!(!masked.servers[0].description.contains("abc123"));

    opencodex_desktop_lib::modules::preferences::PreferencesStore::new(&data_root)
        .save(&opencodex_desktop_lib::modules::preferences::Preferences {
            mcp_mask: false,
            ..Default::default()
        })
        .expect("save preferences");
    let raw = list_extensions_with_paths(&data_root, &home).expect("discover raw");
    assert_eq!(raw.servers[0].args, vec!["--token=abc123", "--stdio"]);
}

#[test]
fn strips_yaml_frontmatter_but_keeps_the_rest_of_the_body() {
    let (_root, data_root, home) = fixture();
    let skill_dir = home.join(".agents/skills/bare-skill");
    fs::create_dir_all(&skill_dir).expect("create skill fixture");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: bare-skill\ndescription: 只有元数据的描述。\n---\n# 正文标题\n",
    )
    .expect("write manifest");

    let detail = read_skill_detail_with_paths("bare-skill", &data_root, &home).expect("detail");
    let body = detail.body.as_deref().unwrap_or_default();
    assert_eq!(body.trim(), "# 正文标题");
    assert!(!body.contains("description:"));

    // 没有 frontmatter 时不得改动正文。
    let plain_dir = home.join(".agents/skills/plain-skill");
    fs::create_dir_all(&plain_dir).expect("create skill fixture");
    fs::write(plain_dir.join("SKILL.md"), "# 直接正文\n\n- 一条\n").expect("write manifest");
    let plain = read_skill_detail_with_paths("plain-skill", &data_root, &home).expect("detail");
    assert_eq!(
        plain.body.as_deref().unwrap_or_default(),
        "# 直接正文\n\n- 一条\n"
    );

    // 只有 frontmatter 时正文为空：界面显示空态，而不是一段元数据。
    let only_dir = home.join(".agents/skills/only-meta");
    fs::create_dir_all(&only_dir).expect("create skill fixture");
    fs::write(only_dir.join("SKILL.md"), "---\nname: only-meta\n---\n").expect("write manifest");
    let only = read_skill_detail_with_paths("only-meta", &data_root, &home).expect("detail");
    assert_eq!(only.body.as_deref().unwrap_or_default(), "");
}

// 回归（2026-09-24 真机发现）：`remove_mcp` 曾用 JSON 序列化写回，
// 对 Codex 这类 TOML 目标会把 `config.toml` 写成 JSON —— 之后该客户端的
// 读/写/断开全部失败（界面上只看到「MCP 写入失败」）。断开必须按目标格式落盘。
#[test]
fn mcp_remove_keeps_toml_target_parseable() {
    let (_root, data_root, home) = fixture();
    write_all_clients(&home);
    populate_live_configs(&home);
    // 目标文件里带上要断开的服务器（真机场景就是从既有配置里断开）。
    fs::write(
        home.join(".codex/config.toml"),
        format!(
            "{}\n[mcp_servers.node_repl]\ntype = \"stdio\"\ncommand = \"node\"\nargs = [\"--stdio\"]\n",
            codex_toml()
        ),
    )
    .expect("codex config with node_repl");
    // 另一处也保留定义，供断开后验证「后续其它客户端仍可写入」。
    fs::write(
        home.join(".claude.json"),
        r#"{"mcp_servers":{"existing":{"type":"stdio","command":"keep"},"node_repl":{"type":"stdio","command":"node","args":["--stdio"]}}}"#,
    )
    .expect("claude config with node_repl");
    // 先建立统一配置：缺配置时写入门控会直接拒绝（首装缺陷，另行处理），
    // 本用例只针对「断开写回是否按目标格式落盘」这一条。
    set_extension_client_enabled_with_paths(&data_root, &home, ClientId::Codex, true)
        .expect("initialize projection");

    execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::WriteMcp {
            name: "node_repl".into(),
            client: ClientId::Codex,
            confirm: true,
        },
    )
    .expect("write mcp to codex");

    execute_extension_write_with_paths(
        &data_root,
        &home,
        ExtensionWriteCommand::RemoveMcp {
            name: "node_repl".into(),
            client: Some(ClientId::Codex),
            confirm: true,
        },
    )
    .expect("remove mcp from codex");

    let text = fs::read_to_string(home.join(".codex/config.toml")).expect("codex config");
    assert!(
        !text.trim_start().starts_with('{'),
        "TOML 目标被写成了 JSON：{}",
        &text[..text.len().min(80)]
    );

    let canonical = home.canonicalize().expect("canonical home");
    let codex = ClientTarget::user_target(ClientId::Codex, &canonical, true, true);
    assert!(
        read_server_definition(&codex, "node_repl")
            .expect("断开后 Codex 配置仍应可解析")
            .is_none(),
        "断开的服务器不应再读到"
    );
    assert!(
        read_server_definition(&codex, "existing")
            .expect("其余服务器仍应可解析")
            .is_some(),
        "同文件里的其它服务器必须保留"
    );

    // 断开一个客户端不应让后续其它客户端的写入连带失败。
    assert_eq!(
        write_mcp_for(&data_root, &home, "node_repl", ClientId::Gemini).len(),
        1
    );
}

// 用户决策 ②（2026-09-24）：首装没有统一配置时，按**默认配置**执行写入并在本次落盘建出配置。
// 此前这里返回 `NotConfigured`，全新安装第一次点图标同步 / 写 MCP 必然失败。
#[test]
fn first_write_without_projection_config_uses_defaults_and_creates_the_config() {
    let (_root, data_root, home) = fixture();
    let skill_dir = home.join(".agents/skills/design-studio");
    fs::create_dir_all(&skill_dir).expect("create skill fixture");
    fs::write(skill_dir.join("SKILL.md"), skill_manifest("设计。")).expect("write manifest");

    assert!(
        !projection::config_path(&data_root).exists(),
        "前置条件：首装不应已有统一配置"
    );

    // 第一次点一个客户端的图标：应成功，且只连被点的那一个。
    let linked = link_skill_for(&data_root, &home, "design-studio", ClientId::Claude);
    assert_eq!(linked, vec!["claude/design-studio".to_string()]);
    assert!(home.join(".claude/skills/design-studio").exists());
    assert!(
        !home.join(".codex/skills/design-studio").exists(),
        "首装第一次点选不得连带其它客户端"
    );
    assert!(
        projection::config_path(&data_root).exists(),
        "本次写入应把统一配置建出来"
    );

    // 建出的配置走默认值：默认源目录仍为空（= `<主目录>/.agents/skills`），用户之后可自行改成自定义目录。
    let stored: serde_json::Value = serde_json::from_slice(
        &fs::read(projection::config_path(&data_root)).expect("read created config"),
    )
    .expect("parse created config");
    assert_eq!(stored["config"]["source_store"], serde_json::Value::Null);
    assert_eq!(stored["config"]["sync_method"], "symlink");
    assert_eq!(stored["fingerprint"]["schema_version"], 3);
    assert_eq!(
        stored["config"]["skill_targets"]["design-studio"][0],
        "claude"
    );
}

// 同一条决策 ② 的 MCP 分支：**另一个**全新数据根（同样没有统一配置）上写 MCP。
#[test]
fn first_mcp_write_without_projection_config_creates_the_config() {
    let (_root, data_root, home) = fixture();
    fs::create_dir_all(home.join(".codex")).expect("codex dir");
    fs::write(home.join(".codex/config.toml"), codex_mcp_payload()).expect("codex config");
    assert!(
        !projection::config_path(&data_root).exists(),
        "前置条件：首装不应已有统一配置"
    );

    // 定义从 live 客户端读回，写进被点的那一个客户端。
    assert_eq!(
        write_mcp_for(&data_root, &home, "node_repl", ClientId::Gemini),
        vec!["gemini".to_string()]
    );
    let gemini = ClientTarget::user_target(ClientId::Gemini, &home, true, true);
    assert!(read_server_definition(&gemini, "node_repl")
        .expect("gemini config remains parseable")
        .is_some());
    assert!(
        projection::config_path(&data_root).exists(),
        "本次写入应把统一配置建出来"
    );
}

/// 回归：前端把 ZIP 载荷发成 camelCase（`archiveName` / `archivePayload`）。
///
/// `ImportSkillArchive` 是 `ExtensionWriteCommand` 中唯一带多词字段的变体；容器级
/// `rename_all = "snake_case"` 只重命名**变体名**，不重命名**变体字段**，因此后端实际
/// 期待 `archive_name`，真实导入会在反序列化阶段直接失败（`missing field archive_name`），
/// 界面只能显示「未归类的失败」，而 `exports/` 与 `skills-store/` 都不会落盘。
#[test]
fn extension_write_command_accepts_frontend_camel_case_import_payload() {
    let command: ExtensionWriteCommand = serde_json::from_value(serde_json::json!({
        "kind": "import_skill_archive",
        "archiveName": "sample.zip",
        "archivePayload": "UEsDBA=="
    }))
    .expect("前端 camelCase 载荷必须可反序列化");
    match command {
        ExtensionWriteCommand::ImportSkillArchive {
            archive_name,
            archive_payload,
        } => {
            assert_eq!(archive_name, "sample.zip");
            assert_eq!(archive_payload, "UEsDBA==");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

fn prepare_config_identity(
    root: &std::path::Path,
    event: &str,
    channel: opencodex_desktop_lib::modules::notifications::registry::Channel,
    input: &[u8],
) -> Option<opencodex_desktop_lib::modules::notifications::registry::EventIdentity> {
    use opencodex_desktop_lib::modules::notifications::registry;
    use sha2::{Digest, Sha256};
    let definition = registry::lookup(event).unwrap();
    Some(
        registry::candidate_identity(
            root,
            definition.object,
            definition.action,
            definition.phase,
            channel,
            Sha256::digest(input).into(),
        )
        .unwrap(),
    )
}

struct ConfigReceipt<'a> {
    root: &'a std::path::Path,
    identity: Option<opencodex_desktop_lib::modules::notifications::registry::EventIdentity>,
    store: std::sync::Arc<
        std::sync::Mutex<opencodex_desktop_lib::modules::notifications::NotificationStore>,
    >,
    terminals: Vec<bool>,
    resolved: Vec<usize>,
}
impl<'a> ConfigReceipt<'a> {
    fn new(root: &'a std::path::Path) -> Self {
        Self {
            root,
            identity: None,
            store: Default::default(),
            terminals: vec![],
            resolved: vec![],
        }
    }
}
impl projection::ConfigWriteObserver for ConfigReceipt<'_> {
    fn begin(&mut self, command: &projection::ProjectionCommand, home: &std::path::Path) {
        use opencodex_desktop_lib::commands::extensions::extension_config_candidate;
        self.identity = prepare_config_identity(
            self.root,
            "extension-config-save-failed",
            opencodex_desktop_lib::modules::notifications::registry::Channel::Local,
            &extension_config_candidate(command, home).unwrap(),
        );
        assert!(self.identity.is_some());
    }
    fn completed(&mut self, succeeded: bool) {
        use opencodex_desktop_lib::modules::notifications::registry::{
            terminal_delivery, Evidence, Trigger,
        };
        assert!(
            opencodex_desktop_lib::infrastructure::locking::TargetFileLock::try_lock_with_timeout(
                &projection::config_path(self.root),
                std::time::Duration::ZERO
            )
            .is_err()
        );
        let identity = self.identity.take().unwrap();
        let evidence = if succeeded {
            Evidence::Success {
                candidate: identity.candidate.clone(),
                verified: true,
            }
        } else {
            Evidence::Failure
        };
        let delivery = terminal_delivery(
            if succeeded {
                "extension-config-save-succeeded"
            } else {
                "extension-config-save-failed"
            },
            Trigger::User,
            identity,
            evidence,
            chrono::Utc::now(),
        )
        .unwrap();
        let outcome = opencodex_desktop_lib::commands::notifications::NotificationPublisher {
            store: &self.store,
            data_root: self.root,
        }
        .publish_event(&delivery)
        .unwrap();
        self.terminals.push(succeeded);
        self.resolved.push(outcome.resolved);
    }
}

#[test]
fn config_receipts_fail_closed_then_resolve_only_verified_persistent_retry() {
    use opencodex_desktop_lib::commands::extensions::execute_extension_write_observed;
    use opencodex_desktop_lib::modules::notifications::persistence::{
        load_notifications, notifications_path,
    };
    for variant in 0..3 {
        let (_temp, root, home) = fixture();
        write_all_clients(&home);
        let command = || match variant {
            0 => ExtensionWriteCommand::ToggleClient {
                client: ClientId::Codex,
                enabled: false,
            },
            1 => ExtensionWriteCommand::SetSourceDir { path: None },
            _ => ExtensionWriteCommand::SetSyncMethod {
                method: "copy".into(),
            },
        };
        let path = projection::config_path(&root);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"unreadable configuration").unwrap();
        let mut observer = ConfigReceipt::new(&root);
        assert!(execute_extension_write_observed(&root, &home, command(), &mut observer).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"unreadable configuration");
        // Repair the fixture explicitly; the application must not overwrite corruption.
        fs::remove_file(&path).unwrap();
        observer.store = std::sync::Arc::new(std::sync::Mutex::new(
            load_notifications(&notifications_path(&root)).unwrap(),
        ));
        let saved =
            execute_extension_write_observed(&root, &home, command(), &mut observer).unwrap();
        assert_eq!(observer.terminals, [false, true]);
        assert_eq!(observer.resolved, [0, 1]);
        let bytes = fs::read(&path).unwrap();
        let stored: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(stored["config"]["revision"], saved.config.revision);
        assert!(
            load_notifications(&notifications_path(&root))
                .unwrap()
                .all()[0]
                .resolved
        );
        for state in [
            notifications_path(&root),
            root.join("manager-state/event-scope.json"),
        ] {
            let raw = fs::read_to_string(state).unwrap();
            assert!(!raw.contains(home.to_str().unwrap()));
            assert!(!raw.contains(root.to_str().unwrap()));
            assert!(!raw.contains("unreadable configuration"));
        }
    }
}

#[test]
fn config_terminal_backup_refusal_preserves_authority_and_reports_failure() {
    use opencodex_desktop_lib::commands::extensions::execute_extension_write_observed;
    let (_temp, root, home) = fixture();
    execute_extension_write_with_paths(
        &root,
        &home,
        ExtensionWriteCommand::SetSyncMethod {
            method: "copy".into(),
        },
    )
    .unwrap();
    let path = projection::config_path(&root);
    let previous = fs::read(&path).unwrap();
    let backups = root.join("backups");
    if backups.exists() {
        fs::remove_dir_all(&backups).unwrap();
    }
    fs::write(&backups, b"blocked backup directory").unwrap();
    let mut observer = ConfigReceipt::new(&root);
    assert!(execute_extension_write_observed(
        &root,
        &home,
        ExtensionWriteCommand::SetSyncMethod {
            method: "symlink".into()
        },
        &mut observer
    )
    .is_err());
    assert_eq!(fs::read(path).unwrap(), previous);
    assert_eq!(observer.terminals, [false]);
}

#[test]
fn config_preflight_rejects_escaping_config_and_lock_without_terminal() {
    use opencodex_desktop_lib::commands::extensions::execute_extension_write_observed;
    for lock in [false, true] {
        let (temp, root, home) = fixture();
        let config = projection::config_path(&root);
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        let path = if lock {
            opencodex_desktop_lib::infrastructure::locking::lock_path_for(&config)
        } else {
            config.clone()
        };
        let outside = temp.path().join("outside.json");
        fs::write(&outside, b"keep").unwrap();
        std::os::unix::fs::symlink(&outside, path).unwrap();
        let mut observer = ConfigReceipt::new(&root);
        assert!(execute_extension_write_observed(
            &root,
            &home,
            ExtensionWriteCommand::SetSourceDir { path: None },
            &mut observer
        )
        .is_err());
        assert_eq!(fs::read(outside).unwrap(), b"keep");
        assert!(observer.terminals.is_empty());
        assert!(observer.identity.is_none());
    }
}

#[test]
fn live_asset_command_does_not_publish_configuration_only_receipt() {
    use opencodex_desktop_lib::commands::extensions::execute_extension_write_observed;
    let (_temp, root, home) = fixture();
    let mut observer = ConfigReceipt::new(&root);
    // This remains an asset operation even though it can also create configuration.
    assert!(execute_extension_write_observed(
        &root,
        &home,
        ExtensionWriteCommand::UninstallSkill {
            name: "missing".into(),
            confirm: false
        },
        &mut observer
    )
    .is_err());
    assert!(observer.identity.is_none());
    assert!(observer.terminals.is_empty());
}

#[test]
fn configuration_candidate_isolates_commands_home_channels_and_preferences() {
    use opencodex_desktop_lib::commands::extensions::extension_config_candidate;
    use opencodex_desktop_lib::modules::notifications::registry::{
        terminal_delivery, Channel, Evidence, Trigger,
    };
    use prepare_config_identity as prepare;
    use projection::ProjectionCommand;
    let (_temp, root, home) = fixture();
    let input =
        extension_config_candidate(&ProjectionCommand::SetSourceDir { path: None }, &home).unwrap();
    let initial = prepare(
        &root,
        "extension-config-save-failed",
        Channel::Local,
        &input,
    )
    .unwrap();
    assert_eq!(
        initial,
        prepare(
            &root,
            "extension-config-save-failed",
            Channel::Local,
            &input
        )
        .unwrap()
    );
    // Preferences uses a different object scope, so it must not rotate or resolve extension config.
    let prefs = prepare(
        &root,
        "preferences-save-failed",
        Channel::Local,
        b"different preferences",
    )
    .unwrap();
    assert_ne!(prefs.object, initial.object);
    assert_eq!(
        initial,
        prepare(
            &root,
            "extension-config-save-failed",
            Channel::Local,
            &input
        )
        .unwrap()
    );
    let mut wrong_channel = initial.clone();
    wrong_channel.channel = Channel::Stable;
    assert!(terminal_delivery(
        "extension-config-save-succeeded",
        Trigger::User,
        wrong_channel.clone(),
        Evidence::Success {
            candidate: wrong_channel.candidate,
            verified: true
        },
        chrono::Utc::now()
    )
    .is_err());
    let changed = extension_config_candidate(
        &ProjectionCommand::SetSyncMethod {
            method: "copy".into(),
        },
        &home,
    )
    .unwrap();
    let other = prepare(
        &root,
        "extension-config-save-failed",
        Channel::Local,
        &changed,
    )
    .unwrap();
    assert_ne!(other.candidate, initial.candidate);
    let other_home = extension_config_candidate(
        &ProjectionCommand::SetSyncMethod {
            method: "copy".into(),
        },
        &home.join("other"),
    )
    .unwrap();
    let moved = prepare(
        &root,
        "extension-config-save-failed",
        Channel::Local,
        &other_home,
    )
    .unwrap();
    assert_ne!(moved.candidate, other.candidate);
    let rotated_back = prepare(
        &root,
        "extension-config-save-failed",
        Channel::Local,
        &input,
    )
    .unwrap();
    assert_ne!(rotated_back.candidate, initial.candidate);
    let mut observer = ConfigReceipt::new(&root);
    observer.identity = Some(initial);
    // Publish a genuine failure; a later candidate's verified success cannot clear it.
    use opencodex_desktop_lib::commands::notifications::NotificationPublisher;
    let failure = terminal_delivery(
        "extension-config-save-failed",
        Trigger::User,
        observer.identity.take().unwrap(),
        Evidence::Failure,
        chrono::Utc::now(),
    )
    .unwrap();
    let publisher = NotificationPublisher {
        store: &observer.store,
        data_root: &root,
    };
    publisher.publish_event(&failure).unwrap();
    let success = terminal_delivery(
        "extension-config-save-succeeded",
        Trigger::User,
        rotated_back.clone(),
        Evidence::Success {
            candidate: rotated_back.candidate,
            verified: true,
        },
        chrono::Utc::now(),
    )
    .unwrap();
    assert_eq!(publisher.publish_event(&success).unwrap().resolved, 0);
    assert!(!observer.store.lock().unwrap().all()[0].resolved);
}
