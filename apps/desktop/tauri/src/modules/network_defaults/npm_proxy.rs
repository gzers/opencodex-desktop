//! Immutable command-boundary npm proxy snapshot. No writes to OS settings,
//! shell environment or npmrc. Unsupported native routing is an error, never
//! silently direct. No proxy credentials from the OS enter argv or diagnostics.
use crate::{
    errors::{AppError, AppResult},
    modules::preferences::ProxyPolicy,
};
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    http: Option<String>,
    https: Option<String>,
    bypass: String,
}
impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NpmProxySnapshot")
            .field("http", &self.http.is_some())
            .field("https", &self.https.is_some())
            .finish_non_exhaustive()
    }
}
fn fail(reason: &str) -> AppError {
    AppError::RuntimeManaged {
        code: "npm_proxy_unavailable".into(),
        detail: format!(
            "npm 代理未能按当前设置解析（{reason}），本次请求未执行；请调整应用网络设置后重试。"
        ),
    }
}
fn endpoint(value: &str) -> AppResult<String> {
    let u = reqwest::Url::parse(value).map_err(|_| fail("invalid_address"))?;
    if !matches!(u.scheme(), "http" | "socks5h")
        || u.host().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
        || !matches!(u.path(), "" | "/")
        || u.port() == Some(0)
    {
        return Err(fail("unsupported_address"));
    }
    Ok(u.as_str().trim_end_matches('/').to_owned())
}
fn native_bypass(values: Vec<String>, simple: bool) -> AppResult<String> {
    if simple {
        return Err(fail("simple_hostname_bypass"));
    }
    let mut out = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let value = value.strip_prefix("*.").unwrap_or(value);
        if value.len() > 253
            || value.contains(['*', '/', ';', ',', ':', '<', '>'])
            || value.chars().any(char::is_whitespace)
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
        {
            return Err(fail("unsupported_native_bypass"));
        }
        out.push(value.to_owned());
    }
    Ok(out.join(","))
}
impl Snapshot {
    pub fn configured_proxy(&self) -> bool {
        self.http.is_some() || self.https.is_some()
    }
    #[cfg(test)]
    pub(crate) fn system_fixture(
        http: Option<&str>,
        https: Option<&str>,
        bypass: &str,
    ) -> AppResult<Self> {
        Self::resolve(ProxyPolicy::System, String::new(), || {
            Ok(Self {
                http: http.map(endpoint).transpose()?,
                https: https.map(endpoint).transpose()?,
                bypass: bypass.into(),
            })
        })
    }
    pub fn capture(policy: ProxyPolicy, bypass: String) -> AppResult<Self> {
        Self::resolve(policy, bypass, native)
    }
    fn resolve(
        policy: ProxyPolicy,
        bypass: String,
        read: impl FnOnce() -> AppResult<Self>,
    ) -> AppResult<Self> {
        let mut result = match policy {
            ProxyPolicy::None => Self::default(),
            ProxyPolicy::Manual(value) => {
                let value = endpoint(&value)?;
                Self {
                    http: Some(value.clone()),
                    https: Some(value),
                    bypass: String::new(),
                }
            }
            ProxyPolicy::System => read()?,
        };
        // npm-registry-fetch passes httpsProxy || proxy for every registry URL.
        // A split native route cannot be represented by these command knobs.
        if result.http != result.https {
            return Err(fail("split_protocol_proxy"));
        }
        if bypass.len() > 8192 || bypass.contains(['\r', '\n', '\0']) {
            return Err(fail("invalid_bypass"));
        }
        if !bypass.trim().is_empty() {
            if !result.bypass.is_empty() {
                result.bypass.push(',');
            }
            result.bypass.push_str(bypass.trim());
        }
        Ok(result)
    }
    pub fn with_bypass(mut self, bypass: String) -> Self {
        self.bypass = bypass;
        self
    }
    pub fn apply(&self, command: &mut std::process::Command) {
        // Explicit npm config overrides proxy values in the user's existing
        // npmrc. npm ignores EMPTY config env values: explicit false/null are
        // required for direct, while transport env stays empty.
        command.env("npm_config_proxy", self.http.as_deref().unwrap_or("false"));
        command.env(
            "npm_config_https_proxy",
            self.https.as_deref().unwrap_or("null"),
        );
        for (key, value) in [
            ("HTTP_PROXY", self.http.as_deref()),
            ("HTTPS_PROXY", self.https.as_deref()),
        ] {
            command.env(key, value.unwrap_or(""));
        }
        command
            .env("NO_PROXY", &self.bypass)
            // A whitespace String parses to empty but is not discarded by
            // npm's env loader, so an old npmrc bypass cannot shadow this one.
            .env(
                "npm_config_noproxy",
                if self.bypass.is_empty() {
                    " "
                } else {
                    &self.bypass
                },
            );
    }
    pub fn apply_environment(&self, environment: &mut crate::modules::process::EnvironmentPolicy) {
        environment.http_proxy = Some(self.http.as_deref().unwrap_or("").into());
        environment.https_proxy = Some(self.https.as_deref().unwrap_or("").into());
        environment.no_proxy = Some(self.bypass.clone().into());
    }
}
#[cfg(target_os = "macos")]
use system_configuration::core_foundation::{
    base::CFType, dictionary::CFDictionary, string::CFString,
};
#[cfg(target_os = "macos")]
fn native() -> AppResult<Snapshot> {
    use system_configuration::dynamic_store::SCDynamicStoreBuilder;
    let store = SCDynamicStoreBuilder::new("OpenCodeX Desktop npm proxy")
        .build()
        .ok_or_else(|| fail("system_store"))?;
    let map = store
        .get_proxies()
        .ok_or_else(|| fail("system_proxy_read"))?;
    macos_map(&map)
}
#[cfg(target_os = "macos")]
fn macos_map(map: &CFDictionary<CFString, CFType>) -> AppResult<Snapshot> {
    use system_configuration::core_foundation::{array::CFArray, base::TCFType, number::CFNumber};
    fn flag(map: &CFDictionary<CFString, CFType>, key: &str) -> AppResult<bool> {
        match map.find(CFString::new(key)) {
            None => Ok(false),
            Some(v) => match v.downcast::<CFNumber>().and_then(|n| n.to_i32()) {
                Some(0) => Ok(false),
                Some(1) => Ok(true),
                _ => Err(fail("native_flag")),
            },
        }
    }
    if flag(map, "ProxyAutoConfigEnable")? || flag(map, "ProxyAutoDiscoveryEnable")? {
        return Err(fail("automatic_proxy_script"));
    }
    let route = |prefix: &str, scheme: &str| -> AppResult<Option<String>> {
        if !flag(map, &format!("{prefix}Enable"))? {
            return Ok(None);
        }
        let host = map
            .find(CFString::new(&format!("{prefix}Proxy")))
            .and_then(|v| v.downcast::<CFString>())
            .map(|s| s.to_string())
            .ok_or_else(|| fail("native_host"))?;
        let port = map
            .find(CFString::new(&format!("{prefix}Port")))
            .and_then(|v| v.downcast::<CFNumber>())
            .and_then(|n| n.to_i32())
            .filter(|n| (1..=65535).contains(n))
            .ok_or_else(|| fail("native_port"))?;
        let host = if host.parse::<std::net::Ipv6Addr>().is_ok() {
            format!("[{host}]")
        } else {
            host
        };
        endpoint(&format!("{scheme}://{host}:{port}")).map(Some)
    };
    let socks = route("SOCKS", "socks5h")?;
    let http = route("HTTP", "http")?.or_else(|| socks.clone());
    let https = route("HTTPS", "http")?.or(socks);
    if http.is_none() && https.is_none() {
        return Ok(Snapshot::default());
    }
    let values = match map.find(CFString::new("ExceptionsList")) {
        None => Vec::new(),
        Some(value) => {
            let array = value
                .downcast::<CFArray>()
                .ok_or_else(|| fail("native_bypass_type"))?;
            array
                .iter()
                .map(|v| {
                    unsafe { CFType::wrap_under_get_rule(*v) }
                        .downcast::<CFString>()
                        .map(|s| s.to_string())
                        .ok_or_else(|| fail("native_bypass_type"))
                })
                .collect::<AppResult<Vec<_>>>()?
        }
    };
    Ok(Snapshot {
        http,
        https,
        bypass: native_bypass(values, flag(map, "ExcludeSimpleHostnames")?)?,
    })
}
#[cfg(any(windows, test))]
fn windows_routes(server: &str, bypass: &str, automatic: bool) -> AppResult<Snapshot> {
    if automatic {
        return Err(fail("automatic_proxy_script"));
    }
    if server.trim().is_empty() {
        return Ok(Snapshot::default());
    }
    let mut http = None;
    let mut https = None;
    let mut socks = None;
    for item in server.split(';').filter(|s| !s.trim().is_empty()) {
        let (kind, host) = item.split_once('=').unwrap_or(("all", item));
        let kind = kind.trim();
        let value = endpoint(&format!(
            "{}://{}",
            if kind == "socks" { "socks5h" } else { "http" },
            host.trim()
        ))?;
        match kind {
            "http" => {
                if http.replace(value).is_some() {
                    return Err(fail("duplicate_route"));
                }
            }
            "https" => {
                if https.replace(value).is_some() {
                    return Err(fail("duplicate_route"));
                }
            }
            "socks" => {
                if socks.replace(value).is_some() {
                    return Err(fail("duplicate_route"));
                }
            }
            "all" => {
                if http.is_some() || https.is_some() {
                    return Err(fail("duplicate_route"));
                }
                http = Some(value.clone());
                https = Some(value);
            }
            _ => return Err(fail("unknown_native_route")),
        }
    }
    Ok(Snapshot {
        http: http.or_else(|| socks.clone()),
        https: https.or(socks),
        bypass: native_bypass(bypass.split(';').map(str::to_owned).collect(), false)?,
    })
}
#[cfg(windows)]
fn native() -> AppResult<Snapshot> {
    use windows_sys::Win32::{
        Foundation::GlobalFree,
        Networking::WinHttp::{
            WinHttpGetIEProxyConfigForCurrentUser, WINHTTP_CURRENT_USER_IE_PROXY_CONFIG,
        },
    };
    struct Config(WINHTTP_CURRENT_USER_IE_PROXY_CONFIG);
    impl Drop for Config {
        fn drop(&mut self) {
            for p in [
                self.0.lpszAutoConfigUrl,
                self.0.lpszProxy,
                self.0.lpszProxyBypass,
            ] {
                if !p.is_null() {
                    unsafe {
                        GlobalFree(p.cast());
                    }
                }
            }
        }
    }
    fn text(p: *const u16) -> AppResult<String> {
        if p.is_null() {
            return Ok(String::new());
        }
        let mut count = 0usize;
        while count < 8192 && unsafe { *p.add(count) } != 0 {
            count += 1;
        }
        if count == 8192 {
            return Err(fail("native_value_limit"));
        }
        String::from_utf16(unsafe { std::slice::from_raw_parts(p, count) })
            .map_err(|_| fail("native_value_encoding"))
    }
    let mut config = Config(WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default());
    if unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut config.0) } == 0 {
        return Err(fail("system_proxy_read"));
    }
    let automatic =
        config.0.fAutoDetect != 0 || !text(config.0.lpszAutoConfigUrl)?.trim().is_empty();
    windows_routes(
        &text(config.0.lpszProxy)?,
        &text(config.0.lpszProxyBypass)?,
        automatic,
    )
}
#[cfg(not(any(windows, target_os = "macos")))]
fn native() -> AppResult<Snapshot> {
    Err(fail("unsupported_platform"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    mod macos_dictionary {
        use super::*;
        use system_configuration::core_foundation::{
            array::CFArray, base::TCFType, number::CFNumber,
        };
        fn number(n: i32) -> CFType {
            CFNumber::from(n).as_CFType()
        }
        fn text(s: &str) -> CFType {
            CFString::new(s).as_CFType()
        }
        fn map(values: Vec<(&str, CFType)>) -> CFDictionary<CFString, CFType> {
            CFDictionary::from_CFType_pairs(
                &values
                    .into_iter()
                    .map(|(k, v)| (CFString::new(k), v))
                    .collect::<Vec<_>>(),
            )
        }
        fn common() -> Vec<(&'static str, CFType)> {
            vec![
                ("HTTPEnable", number(1)),
                ("HTTPProxy", text("127.0.0.1")),
                ("HTTPPort", number(8080)),
                ("HTTPSEnable", number(1)),
                ("HTTPSProxy", text("127.0.0.1")),
                ("HTTPSPort", number(8080)),
            ]
        }
        #[test]
        fn real_cf_dictionary_values_admit_common_proxy_and_socks_with_typed_bypass() {
            let mut values = common();
            values.push((
                "ExceptionsList",
                CFArray::from_CFTypes(&[
                    CFString::new("*.example.test"),
                    CFString::new("127.0.0.1"),
                ])
                .as_CFType(),
            ));
            let input = map(values);
            let s = Snapshot::resolve(ProxyPolicy::System, "saved.test".into(), || {
                macos_map(&input)
            })
            .unwrap();
            assert_eq!(s.http.as_deref(), Some("http://127.0.0.1:8080"));
            assert_eq!(s.https, s.http);
            assert_eq!(s.bypass, "example.test,127.0.0.1,saved.test");
            let input = map(vec![
                ("SOCKSEnable", number(1)),
                ("SOCKSProxy", text("::1")),
                ("SOCKSPort", number(1080)),
            ]);
            let s = Snapshot::resolve(ProxyPolicy::System, String::new(), || macos_map(&input))
                .unwrap();
            assert_eq!(s.http.as_deref(), Some("socks5h://[::1]:1080"));
            assert_eq!(s.https, s.http);
        }
        #[test]
        fn real_cf_dictionary_rejects_wrong_types_invalid_flags_pac_and_nonrepresentable_routes() {
            for (key, value) in [
                ("HTTPEnable", number(2)),
                ("HTTPProxy", number(1)),
                ("HTTPPort", text("8080")),
                ("HTTPPort", number(0)),
                ("ProxyAutoConfigEnable", number(1)),
                ("ProxyAutoDiscoveryEnable", number(1)),
                ("ExcludeSimpleHostnames", number(1)),
                ("ExceptionsList", text("wrong type")),
                (
                    "ExceptionsList",
                    CFArray::from_CFTypes(&[CFNumber::from(1)]).as_CFType(),
                ),
                (
                    "ExceptionsList",
                    CFArray::from_CFTypes(&[CFString::new("169.254/16")]).as_CFType(),
                ),
                ("HTTPSProxy", text("other.test")),
            ] {
                let mut values = common();
                values.retain(|(k, _)| *k != key);
                values.push((key, value));
                let input = map(values);
                assert!(
                    Snapshot::resolve(ProxyPolicy::System, String::new(), || macos_map(&input))
                        .is_err(),
                    "{key}"
                );
            }
            let s = macos_map(&map(Vec::new())).unwrap();
            assert!(!s.configured_proxy());
        }
    }
    fn command(s: &Snapshot) -> std::process::Command {
        let mut c = std::process::Command::new("/not-executed");
        s.apply(&mut c);
        c
    }
    fn value(c: &std::process::Command, key: &str) -> String {
        c.get_envs()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
    #[test]
    fn explicit_none_and_manual_never_read_native_and_system_keeps_static_route() {
        for p in [
            ProxyPolicy::None,
            ProxyPolicy::Manual("http://127.0.0.1:8080".into()),
        ] {
            let s = Snapshot::resolve(p, "saved.example".into(), || panic!("native must not run"))
                .unwrap();
            let c = command(&s);
            assert_eq!(value(&c, "NO_PROXY"), "saved.example");
        }
        let s = Snapshot::resolve(ProxyPolicy::System, "saved.example".into(), || {
            Ok(Snapshot {
                http: Some("http://127.0.0.1:8080".into()),
                https: Some("http://127.0.0.1:8080".into()),
                bypass: "system.example".into(),
            })
        })
        .unwrap();
        let c = command(&s);
        assert_eq!(value(&c, "npm_config_proxy"), "http://127.0.0.1:8080");
        assert_eq!(value(&c, "npm_config_https_proxy"), "http://127.0.0.1:8080");
        assert_eq!(
            value(&c, "npm_config_noproxy"),
            "system.example,saved.example"
        );
        let direct = command(&Snapshot::default());
        assert_eq!(value(&direct, "npm_config_proxy"), "false");
        assert_eq!(value(&direct, "npm_config_https_proxy"), "null");
    }
    #[test]
    fn unavailable_or_unsupported_native_policy_never_falls_back_to_direct() {
        assert!(
            Snapshot::system_fixture(Some("http://one:80"), Some("http://two:81"), "").is_err()
        );
        assert!(Snapshot::system_fixture(Some("http://one:80"), None, "").is_err());
        assert!(
            Snapshot::resolve(ProxyPolicy::System, String::new(), || Err(fail(
                "system_proxy_read"
            )))
            .is_err()
        );
        for s in [
            "http://u:p@host:8080",
            "http://host:8080/path",
            "file:///proxy",
            "http://host:0",
            "http://",
        ] {
            assert!(endpoint(s).is_err());
        }
        for s in ["169.254/16", "<local>", "*", "10.*", "foo:80"] {
            assert!(native_bypass(vec![s.into()], false).is_err());
        }
        assert!(native_bypass(Vec::new(), true).is_err());
        assert_eq!(
            native_bypass(vec!["*.example.test".into(), "localhost".into()], false).unwrap(),
            "example.test,localhost"
        );
    }
    #[test]
    fn windows_native_strings_preserve_split_routes_and_refuse_automatic_or_unknown_rules() {
        let s = windows_routes(
            "http=one.test:8080;https=two.test:8081",
            "*.local.test;localhost",
            false,
        )
        .unwrap();
        assert_eq!(s.http.as_deref(), Some("http://one.test:8080"));
        assert_eq!(s.https.as_deref(), Some("http://two.test:8081"));
        assert_eq!(s.bypass, "local.test,localhost");
        let s = windows_routes("socks=[::1]:1080", "", false).unwrap();
        assert_eq!(s.http, s.https);
        assert_eq!(s.http.as_deref(), Some("socks5h://[::1]:1080"));
        for (server, bypass, auto) in [
            ("one:80", "", true),
            ("ftp=host:80", "", false),
            ("http=a:80;http=b:81", "", false),
            ("one:80", "<local>", false),
            ("user:secret@host:80", "", false),
        ] {
            assert!(windows_routes(server, bypass, auto).is_err());
        }
    }
    #[test]
    #[ignore = "explicit read-only native proxy inspection; never changes OS settings"]
    fn actual_native_proxy_snapshot_reports_supported_or_refused_without_exposing_values() {
        match Snapshot::capture(ProxyPolicy::System, String::new()) {
            Ok(s) => println!(
                "NATIVE_PROXY_READ supported=true http={} https={} bypassItems={}",
                s.http.is_some(),
                s.https.is_some(),
                s.bypass.split(',').filter(|s| !s.is_empty()).count()
            ),
            Err(AppError::RuntimeManaged { code, detail }) => {
                println!("NATIVE_PROXY_READ supported=false code={code} detail={detail}")
            }
            Err(_) => panic!("unexpected error classification"),
        }
    }
}
