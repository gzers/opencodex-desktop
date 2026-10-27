//! Protocol model-list paths. Every candidate keeps the supplied origin and proxy prefix.
use reqwest::Url;

/// At most two paths, in order. Compatibility probing is only allowed after a path error.
pub(super) fn candidates(base: &Url, protocol: &str) -> Vec<Url> {
    let path = base.path().trim_end_matches('/');
    let paths = if protocol == "ollama-native" {
        vec![if path.ends_with("/api/tags") {
            path.to_owned()
        } else if path.ends_with("/api") {
            format!("{path}/tags")
        } else {
            format!("{path}/api/tags")
        }]
    } else {
        let complete = path.ends_with("/models");
        let root = path.strip_suffix("/models").unwrap_or(path);
        let default_version = if protocol == "google" { "v1beta" } else { "v1" };
        let versioned = version_prefix(root);
        let direct = format!("{root}/models");
        let standard = format!("{root}/{default_version}/models");
        if let Some(prefix) = versioned {
            vec![direct, format!("{prefix}/models")]
        } else if complete || !root.is_empty() && protocol != "google" {
            vec![direct, standard]
        } else {
            vec![standard, direct]
        }
    };
    paths
        .into_iter()
        .map(|path| {
            let mut target = base.clone();
            target.set_path(&path);
            target
        })
        .collect()
}

// Remove only a final API version segment; never discard a proxy prefix.
fn version_prefix(path: &str) -> Option<&str> {
    let (prefix, segment) = path.rsplit_once('/')?;
    let version = segment.strip_prefix('v')?;
    let suffix = version.trim_start_matches(|c: char| c.is_ascii_digit());
    if suffix.len() == version.len() {
        return None;
    }
    let recognized = suffix.is_empty()
        || ["alpha", "beta"].iter().any(|stage| {
            suffix
                .strip_prefix(stage)
                .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
        });
    recognized.then_some(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_protocol_paths_keep_versions_complete_endpoints_and_proxy_prefixes() {
        for protocol in ["openai-chat", "openai-responses", "anthropic"] {
            for (input, expected) in [
                ("", ["/v1/models", "/models"]),
                ("/", ["/v1/models", "/models"]),
                ("/v1/", ["/v1/models", "/models"]),
                ("/models", ["/models", "/v1/models"]),
                ("/v1/models/", ["/v1/models", "/models"]),
                ("/proxy/v1/", ["/proxy/v1/models", "/proxy/models"]),
                ("/proxy/v1/models", ["/proxy/v1/models", "/proxy/models"]),
                ("/proxy/", ["/proxy/models", "/proxy/v1/models"]),
                ("/proxy/models", ["/proxy/models", "/proxy/v1/models"]),
                ("/proxy/v2", ["/proxy/v2/models", "/proxy/models"]),
                ("/proxy/v1beta2", ["/proxy/v1beta2/models", "/proxy/models"]),
                (
                    "/proxy/vendor",
                    ["/proxy/vendor/models", "/proxy/vendor/v1/models"],
                ),
            ] {
                let base = Url::parse(&format!("https://relay.example.test:8443{input}")).unwrap();
                let before = base.to_string();
                let targets = candidates(&base, protocol);
                assert_eq!(
                    targets.iter().map(Url::path).collect::<Vec<_>>(),
                    expected,
                    "{protocol} {input}"
                );
                assert!(targets
                    .iter()
                    .all(|target| target.origin() == base.origin()));
                assert_eq!(base.as_str(), before);
            }
        }
    }

    #[test]
    fn google_and_ollama_accept_complete_endpoints_and_existing_protocol_prefixes() {
        for (protocol, input, expected) in [
            ("google", "", vec!["/v1beta/models", "/models"]),
            ("google", "/v1beta/", vec!["/v1beta/models", "/models"]),
            ("google", "/v1", vec!["/v1/models", "/models"]),
            (
                "google",
                "/proxy",
                vec!["/proxy/v1beta/models", "/proxy/models"],
            ),
            (
                "google",
                "/proxy/v1beta/models/",
                vec!["/proxy/v1beta/models", "/proxy/models"],
            ),
            (
                "google",
                "/proxy/models",
                vec!["/proxy/models", "/proxy/v1beta/models"],
            ),
            ("ollama-native", "", vec!["/api/tags"]),
            ("ollama-native", "/api/", vec!["/api/tags"]),
            ("ollama-native", "/api/tags/", vec!["/api/tags"]),
            ("ollama-native", "/proxy/api", vec!["/proxy/api/tags"]),
            ("ollama-native", "/proxy/api/tags", vec!["/proxy/api/tags"]),
            ("ollama-native", "/proxy", vec!["/proxy/api/tags"]),
        ] {
            let base = Url::parse(&format!("http://localhost:8080{input}")).unwrap();
            let targets = candidates(&base, protocol);
            assert_eq!(
                targets.iter().map(Url::path).collect::<Vec<_>>(),
                expected,
                "{protocol} {input}"
            );
            assert!(targets
                .iter()
                .all(|target| target.origin() == base.origin()));
        }
    }

    #[test]
    fn encoded_proxy_segments_are_preserved() {
        let base = Url::parse("https://example.test/proxy%20name/a%2Fb/v1/").unwrap();
        let targets = candidates(&base, "anthropic");
        assert_eq!(targets[0].path(), "/proxy%20name/a%2Fb/v1/models");
        assert_eq!(targets[1].path(), "/proxy%20name/a%2Fb/models");
    }
}
