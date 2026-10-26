//! 只读解析「官方 config.json 里这个渠道已配置的 Key」，供节点检测复用。
//!
//! 为什么需要：读取现有渠道时，草稿侧的 `credentialRef` 是 null（只保留公开读取
//! 回执），前端也没有让用户为每次检测重填 Key。于是检测请求不带认证头发出，
//! 服务返回 401，界面显示「认证失败」——用户看到的是「测速无效」。
//!
//! 边界（不可放宽）：
//! 1. **只读**，不写回 config.json，不复制 Key 到管理器钥匙串。
//! 2. 只有在**未提供任何 Key** 时才读，用户本次输入优先。
//! 3. 必须**同源**（scheme + host + port）**且协议一致**才携带；节点地址被改过、
//!    或协议对不上，一律不携带——否则等于把 Key 发给任意地址。
//! 4. 失败原因要分得开：「服务拒绝」与「本机没有可确认的凭据」是两件事。
use crate::modules::backup::safety;
use serde_json::Value;
use std::path::Path;
use zeroize::Zeroizing;

/// 同源判定：scheme + host + port 必须完全一致（端口按协议补默认值）。
pub fn same_origin(left: &str, right: &str) -> bool {
    let (Ok(a), Ok(b)) = (reqwest::Url::parse(left), reqwest::Url::parse(right)) else {
        return false;
    };
    a.scheme() == b.scheme()
        && a.host_str() == b.host_str()
        && a.port_or_known_default() == b.port_or_known_default()
}

fn provider<'a>(config: &'a Value, provider_id: &str) -> Option<&'a Value> {
    config.get("providers")?.get(provider_id)
}

/// 该渠道在官方配置里声明的接口协议。
pub fn declared_protocol(config: &Value, provider_id: &str) -> Option<String> {
    let row = provider(config, provider_id)?;
    row.get("adapter")?.as_str().map(str::to_owned)
}

/// 该渠道是否声明了「需要 Key」的认证方式。
pub fn declared_auth_mode(config: &Value, provider_id: &str) -> Option<String> {
    let row = provider(config, provider_id)?;
    row.get("authMode")?.as_str().map(str::to_owned)
}

/// 该渠道在官方配置里的服务基地址。
pub fn declared_base_url(config: &Value, provider_id: &str) -> Option<String> {
    let row = provider(config, provider_id)?;
    row.get("baseUrl")?.as_str().map(str::to_owned)
}

pub struct Resolved {
    pub secret: Option<Zeroizing<String>>,
    /// 渠道声明了需要 Key，但本机没有可用于这个节点的凭据。
    pub missing: bool,
}

/// 纯函数版本：给定官方配置内容判定能否把 Key 用于这个节点。
pub fn resolve_from_config(
    config: &Value,
    provider_id: &str,
    node_url: &str,
    protocol: &str,
) -> Resolved {
    let needs_key = declared_auth_mode(config, provider_id).is_none_or(|mode| mode == "key");
    let Some(expected) = declared_base_url(config, provider_id) else {
        return Resolved {
            secret: None,
            missing: needs_key,
        };
    };
    // 同源 + 协议一致，两者都要：任一不满足都不携带。
    if !same_origin(&expected, node_url)
        || declared_protocol(config, provider_id).as_deref() != Some(protocol)
    {
        return Resolved {
            secret: None,
            missing: needs_key,
        };
    }
    let secret = provider(config, provider_id)
        .and_then(|row| row.get("apiKey"))
        .and_then(Value::as_str)
        .filter(|key| !key.trim().is_empty() && key.len() <= 8192 && !key.contains(['\r', '\n']))
        .map(|key| Zeroizing::new(key.to_owned()));
    Resolved {
        missing: needs_key && secret.is_none(),
        secret,
    }
}

/// 读盘版本。文件缺失 / 不可读 / 结构异常一律当作「没有可确认的凭据」，
/// 不把读取失败伪装成认证失败，也不阻断检测（无认证的服务仍然可以被检测）。
pub fn resolve(home: &Path, provider_id: &str, node_url: &str, protocol: &str) -> Resolved {
    let empty = Resolved {
        secret: None,
        missing: false,
    };
    let path = home.join("config.json");
    if safety::check_path(home, &path, false).is_err() || !path.exists() {
        return empty;
    }
    let Ok(bytes) = safety::read_file(&path, 16 * 1024 * 1024) else {
        return empty;
    };
    let Ok(config) = serde_json::from_slice::<Value>(&bytes) else {
        return empty;
    };
    resolve_from_config(&config, provider_id, node_url, protocol)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Value {
        serde_json::json!({"providers":{"demo":{"adapter":"openai-responses","baseUrl":"https://api.example.test","authMode":"key","apiKey":"sk-fixture-SECRET"}}})
    }
    #[test]
    fn uses_the_key_only_for_the_same_origin_and_protocol() {
        let resolved = resolve_from_config(
            &config(),
            "demo",
            "https://api.example.test",
            "openai-responses",
        );
        assert_eq!(
            resolved.secret.as_deref().map(String::as_str),
            Some("sk-fixture-SECRET")
        );
        assert!(!resolved.missing);
    }
    #[test]
    fn refuses_a_different_host_scheme_or_port() {
        for url in [
            "https://evil.example.test",
            "http://api.example.test",       // scheme 不同
            "https://api.example.test:8443", // port 不同
        ] {
            let resolved = resolve_from_config(&config(), "demo", url, "openai-responses");
            assert!(resolved.secret.is_none(), "{url} 不应携带 Key");
            assert!(resolved.missing, "{url} 应报「无可确认凭据」");
        }
    }
    #[test]
    fn refuses_a_mismatched_protocol_even_on_the_same_origin() {
        let resolved =
            resolve_from_config(&config(), "demo", "https://api.example.test", "anthropic");
        assert!(resolved.secret.is_none());
    }
    #[test]
    fn reports_missing_only_when_the_channel_declares_a_key() {
        let open = serde_json::json!({"providers":{"demo":{"adapter":"openai-chat","baseUrl":"https://api.example.test","authMode":"none"}}});
        let resolved =
            resolve_from_config(&open, "demo", "https://api.example.test", "openai-chat");
        assert!(resolved.secret.is_none());
        // 声明不需要 Key 的渠道，没有 Key 不是错误。
        assert!(!resolved.missing);
    }
    #[test]
    fn unknown_provider_or_missing_file_yields_no_secret() {
        let resolved = resolve_from_config(
            &config(),
            "absent",
            "https://api.example.test",
            "openai-responses",
        );
        assert!(resolved.secret.is_none());
        let temp = tempfile::tempdir().unwrap();
        let missing = resolve(
            temp.path(),
            "demo",
            "https://api.example.test",
            "openai-responses",
        );
        assert!(missing.secret.is_none());
    }
}
