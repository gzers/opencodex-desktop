//! File-only configuration synchronization transport. No ETag, redirect or retry.
//! Connection handles bind immutable endpoint revisions; URLs and credentials never
//! enter transport errors or the synchronization journal.
use crate::modules::model_config::sync_v2::types::*;
use async_trait::async_trait;
use percent_encoding::percent_decode_str;
use reqwest::{Client, Method, Response, Url};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
    time::Duration,
};

struct BoundConnection {
    handle: ConnectionHandle,
    root: Url,
    username: String,
    password: String,
}
impl Drop for BoundConnection {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.password.zeroize();
    }
}
pub struct FileWebDavTransport {
    client: Client,
    bindings: RwLock<BTreeMap<String, BoundConnection>>,
}
fn err(kind: TransportKind, status: Option<u16>, delivery: Delivery) -> SyncTransportError {
    SyncTransportError {
        kind,
        http_status: status,
        delivery,
    }
}
fn invalid(kind: TransportKind) -> SyncTransportError {
    err(kind, None, Delivery::NotSent)
}
fn status_error(status: u16) -> SyncTransportError {
    use TransportKind::*;
    let kind = match status {
        401 => Unauthorized,
        403 => Forbidden,
        404 => NotFound,
        206 => PartialResponse,
        300..=399 => Redirect,
        507 => InsufficientStorage,
        500..=599 => Server,
        _ => Protocol,
    };
    err(kind, Some(status), Delivery::ResponseReceived)
}
fn request_error(e: reqwest::Error, writing: bool) -> SyncTransportError {
    use TransportKind::*;
    let delivery = if e.is_builder() || e.is_connect() {
        Delivery::NotSent
    } else if writing {
        Delivery::MayHaveBeenSent
    } else {
        Delivery::NotSent
    };
    let kind = if e.is_timeout() {
        Timeout
    } else if e.is_builder() {
        Protocol
    } else if e.is_connect() && certificate_error(&e) {
        Tls
    } else {
        Disconnected
    };
    err(kind, None, delivery)
}
fn certificate_error(e: &reqwest::Error) -> bool {
    let mut current = std::error::Error::source(e);
    while let Some(e) = current {
        let s = e.to_string().to_ascii_lowercase();
        if s.contains("certificate") || s.contains("unknownissuer") {
            return true;
        }
        current = e.source();
    }
    false
}
impl FileWebDavTransport {
    #[cfg(feature = "integration-test")]
    pub fn with_test_root_certificate(pem: &[u8]) -> Result<Self, SyncTransportError> {
        let certificate =
            reqwest::Certificate::from_pem(pem).map_err(|_| invalid(TransportKind::Tls))?;
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .add_root_certificate(certificate)
            .build()
            .map_err(|e| request_error(e, false))?;
        Ok(Self {
            client,
            bindings: RwLock::new(BTreeMap::new()),
        })
    }
    pub fn new() -> Result<Self, SyncTransportError> {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| request_error(e, false))?;
        Ok(Self {
            client,
            bindings: RwLock::new(BTreeMap::new()),
        })
    }
    /// Called by the credential owner, never by raw UI DTOs. A handle cannot be
    /// rebound to another endpoint; new settings need a new immutable handle.
    pub fn bind(
        &self,
        handle: ConnectionHandle,
        base: &str,
        directory: &str,
        username: String,
        password: String,
    ) -> Result<(), SyncTransportError> {
        require_digest(&handle.binding_id).map_err(|_| invalid(TransportKind::Protocol))?;
        require_digest(&handle.revision).map_err(|_| invalid(TransportKind::Protocol))?;
        let mut root = Url::parse(base).map_err(|_| invalid(TransportKind::Protocol))?;
        if root.scheme() != "https"
            || root.host_str().is_none()
            || !root.username().is_empty()
            || root.password().is_some()
            || root.query().is_some()
            || root.fragment().is_some()
            || username.is_empty()
            || password.is_empty()
            || handle.id.is_empty()
        {
            return Err(invalid(TransportKind::Protocol));
        }
        // Reject before URL normalization can erase traversal evidence.
        let raw = base.split_once("://").unwrap().1;
        let raw_path = raw.find('/').map(|i| &raw[i..]).unwrap_or("");
        checked_components(raw_path)?;
        let parts = checked_components(directory)?;
        if parts.is_empty() {
            return Err(invalid(TransportKind::Protocol));
        }
        {
            let mut path = root
                .path_segments_mut()
                .map_err(|_| invalid(TransportKind::Protocol))?;
            path.pop_if_empty();
            for p in parts {
                path.push(&p);
            }
            path.push("");
        }
        let mut bindings = self
            .bindings
            .write()
            .map_err(|_| invalid(TransportKind::Protocol))?;
        if let Some(old) = bindings.get(&handle.id) {
            if old.handle.binding_id != handle.binding_id
                || old.handle.revision != handle.revision
                || old.root != root
                || old.username != username
                || old.password != password
            {
                return Err(invalid(TransportKind::Protocol));
            }
            return Ok(());
        }
        bindings.insert(
            handle.id.clone(),
            BoundConnection {
                handle,
                root,
                username,
                password,
            },
        );
        Ok(())
    }
    fn request(
        &self,
        h: &ConnectionHandle,
        method: Method,
        path: Option<&SyncRelativePath>,
        timeout: u32,
    ) -> Result<(reqwest::RequestBuilder, Url), SyncTransportError> {
        if timeout == 0 || timeout > 30_000 {
            return Err(invalid(TransportKind::Protocol));
        }
        let bindings = self
            .bindings
            .read()
            .map_err(|_| invalid(TransportKind::Protocol))?;
        let c = bindings
            .get(&h.id)
            .filter(|v| v.handle.binding_id == h.binding_id && v.handle.revision == h.revision)
            .ok_or_else(|| invalid(TransportKind::Protocol))?;
        let url = match path {
            Some(p) => c
                .root
                .join(p.as_str())
                .map_err(|_| invalid(TransportKind::Protocol))?,
            None => c.root.clone(),
        };
        let req = self
            .client
            .request(method, url.clone())
            .basic_auth(&c.username, Some(&c.password))
            .timeout(Duration::from_millis(timeout.into()))
            .header("Cache-Control", "no-cache");
        Ok((req, url))
    }
}
fn checked_components(raw: &str) -> Result<Vec<String>, SyncTransportError> {
    if raw.contains('\\') || raw.contains('?') || raw.contains('#') {
        return Err(invalid(TransportKind::InvalidHref));
    }
    let raw = raw.strip_prefix('/').unwrap_or(raw);
    let raw = raw.strip_suffix('/').unwrap_or(raw);
    if raw.is_empty() {
        return Ok(vec![]);
    }
    raw.split('/')
        .map(|s| {
            let decoded = percent_decode_str(s)
                .decode_utf8()
                .map_err(|_| invalid(TransportKind::InvalidHref))?;
            if decoded.is_empty()
                || decoded == "."
                || decoded == ".."
                || decoded.contains('/')
                || decoded.contains('\\')
                || decoded.chars().any(|c| c.is_control())
                || decoded.contains('%')
            {
                return Err(invalid(TransportKind::InvalidHref));
            }
            Ok(decoded.into_owned())
        })
        .collect()
}
fn child_identity(root: &Url, href: &str) -> Result<Option<String>, SyncTransportError> {
    checked_components(
        href.split_once("://")
            .map(|(_, s)| s.find('/').map(|i| &s[i..]).unwrap_or(""))
            .unwrap_or(href),
    )?;
    let target = root
        .join(href)
        .map_err(|_| invalid(TransportKind::InvalidHref))?;
    if target.scheme() != root.scheme()
        || target.host_str() != root.host_str()
        || target.port_or_known_default() != root.port_or_known_default()
        || target.query().is_some()
        || target.fragment().is_some()
        || !target.username().is_empty()
        || target.password().is_some()
    {
        return Err(invalid(TransportKind::InvalidHref));
    }
    let parent = checked_components(root.path())?;
    let child = checked_components(target.path())?;
    if child == parent {
        return Ok(None);
    }
    if child.len() != parent.len() + 1 || !child.starts_with(&parent) {
        return Err(invalid(TransportKind::InvalidHref));
    }
    Ok(child.last().cloned())
}
async fn body(mut response: Response, max: u64) -> Result<Vec<u8>, SyncTransportError> {
    let status = response.status().as_u16();
    if response.content_length().is_some_and(|n| n > max) {
        return Err(err(
            TransportKind::TooLarge,
            Some(status),
            Delivery::ResponseReceived,
        ));
    }
    let mut data = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| {
        err(
            TransportKind::PartialResponse,
            Some(status),
            Delivery::ResponseReceived,
        )
    })? {
        if data.len() as u64 + chunk.len() as u64 > max {
            return Err(err(
                TransportKind::TooLarge,
                Some(status),
                Delivery::ResponseReceived,
            ));
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}
#[async_trait]
impl ConfigSyncTransport for FileWebDavTransport {
    async fn list_children(
        &self,
        h: &ConnectionHandle,
        l: ListLimits,
    ) -> Result<ChildListing, SyncTransportError> {
        if l.max_xml_bytes == 0
            || l.max_xml_bytes > 16 * 1024 * 1024
            || l.max_entries == 0
            || l.max_entries > 10000
        {
            return Err(invalid(TransportKind::Protocol));
        }
        let (request, root) = self.request(
            h,
            Method::from_bytes(b"PROPFIND").unwrap(),
            None,
            l.timeout_ms,
        )?;
        let response=request.header("Depth","1").header("Content-Type","application/xml; charset=utf-8")
            .body(r#"<?xml version="1.0" encoding="utf-8"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/><d:getcontentlength/></d:prop></d:propfind>"#)
            .send().await.map_err(|e|request_error(e,false))?;
        if response.status().as_u16() != 207 {
            return Err(status_error(response.status().as_u16()));
        }
        let bytes = body(response, l.max_xml_bytes).await?;
        parse_listing(&root, &bytes, l.max_entries)
    }
    async fn get_file(
        &self,
        h: &ConnectionHandle,
        p: &SyncRelativePath,
        l: ReadLimits,
    ) -> Result<FileRead, SyncTransportError> {
        if l.max_bytes == 0 || l.max_bytes > 64 * 1024 * 1024 {
            return Err(invalid(TransportKind::Protocol));
        }
        let (request, _) = self.request(h, Method::GET, Some(p), l.timeout_ms)?;
        let response = request.send().await.map_err(|e| request_error(e, false))?;
        let status = response.status().as_u16();
        if status != 200 {
            return Err(status_error(status));
        }
        Ok(FileRead {
            body: body(response, l.max_bytes).await?,
            http_status: status,
        })
    }
    async fn put_file(
        &self,
        h: &ConnectionHandle,
        p: &SyncRelativePath,
        frozen: &[u8],
        l: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError> {
        if l.max_bytes == 0 || l.max_bytes > 64 * 1024 * 1024 || frozen.len() as u64 > l.max_bytes {
            return Err(invalid(TransportKind::TooLarge));
        }
        let (request, _) = self.request(h, Method::PUT, Some(p), l.timeout_ms)?;
        let response = request
            .header("Content-Type", "application/octet-stream")
            .body(frozen.to_vec())
            .send()
            .await
            .map_err(|e| request_error(e, true))?;
        let status = response.status().as_u16();
        if !matches!(status, 200 | 201 | 204) {
            return Err(status_error(status));
        }
        Ok(PutResponse {
            http_status: status,
        })
    }
}

#[derive(Default)]
struct Node {
    dav: bool,
    name: String,
    text: String,
    children: Vec<Node>,
}
impl Node {
    fn named(&self, name: &str) -> bool {
        self.dav && self.name == name
    }
    fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.children.iter().filter(move |n| n.named(name))
    }
    fn one(&self, name: &str) -> Result<&Node, SyncTransportError> {
        let mut values = self.children.iter().filter(|n| n.named(name));
        let first = values
            .next()
            .ok_or_else(|| invalid(TransportKind::InvalidListing))?;
        if values.next().is_some() {
            return Err(invalid(TransportKind::InvalidListing));
        }
        Ok(first)
    }
}
fn parse_xml(bytes: &[u8]) -> Result<Node, SyncTransportError> {
    use quick_xml::{events::Event, name::ResolveResult, NsReader};
    let text = std::str::from_utf8(bytes).map_err(|_| invalid(TransportKind::InvalidListing))?;
    if text.chars().any(|c| {
        !(matches!(c, '\t' | '\r' | '\n') || c >= ' ' && c != '\u{fffe}' && c != '\u{ffff}')
    }) {
        return Err(invalid(TransportKind::InvalidListing));
    }
    let mut reader = NsReader::from_reader(bytes);
    let mut stack: Vec<Node> = vec![];
    let mut root = None;
    let mut count = 0usize;
    loop {
        let (ns, event) = reader
            .read_resolved_event()
            .map_err(|_| invalid(TransportKind::InvalidListing))?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                for attribute in e.attributes() {
                    attribute.map_err(|_| invalid(TransportKind::InvalidListing))?;
                }
                // Expand empty elements so the same stack parser handles both.
                count += 1;
                if count > 300_000 || stack.len() >= 32 {
                    return Err(invalid(TransportKind::TooLarge));
                }
                let n = Node {
                    dav: matches!(ns,ResolveResult::Bound(v) if v.as_ref()=="DAV:"),
                    name: e.local_name().as_ref().to_string(),
                    ..Node::default()
                };
                if empty {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(n)
                    } else if root.replace(n).is_some() {
                        return Err(invalid(TransportKind::InvalidListing));
                    }
                } else {
                    stack.push(n);
                }
            }
            Event::End(_) => {
                let n = stack
                    .pop()
                    .ok_or_else(|| invalid(TransportKind::InvalidListing))?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(n)
                } else if root.replace(n).is_some() {
                    return Err(invalid(TransportKind::InvalidListing));
                }
            }
            Event::Text(e) => {
                let text = e.as_ref();
                if let Some(parent) = stack.last_mut() {
                    if parent.text.len() + text.len() > 65536 {
                        return Err(invalid(TransportKind::TooLarge));
                    }
                    parent.text.push_str(text);
                } else if !text.trim().is_empty() {
                    return Err(invalid(TransportKind::InvalidListing));
                }
            }
            Event::GeneralRef(e) => {
                let name = e.as_ref();
                let escaped = format!("&{name};");
                let value = quick_xml::escape::unescape(&escaped)
                    .map_err(|_| invalid(TransportKind::InvalidListing))?;
                stack
                    .last_mut()
                    .ok_or_else(|| invalid(TransportKind::InvalidListing))?
                    .text
                    .push_str(&value);
            }
            Event::DocType(_) | Event::PI(_) | Event::CData(_) => {
                return Err(invalid(TransportKind::InvalidListing))
            }
            Event::Decl(_) | Event::Comment(_) => {}
            Event::Eof => break,
        }
    }
    if !stack.is_empty() {
        return Err(invalid(TransportKind::InvalidListing));
    }
    root.filter(|n| n.named("multistatus"))
        .ok_or_else(|| invalid(TransportKind::InvalidListing))
}
fn successful_status(n: &Node) -> Result<bool, SyncTransportError> {
    let parts: Vec<_> = n.text.split_whitespace().collect();
    if !n.children.is_empty()
        || parts.len() < 2
        || !matches!(parts[0], "HTTP/1.0" | "HTTP/1.1" | "HTTP/2" | "HTTP/2.0")
    {
        return Err(invalid(TransportKind::InvalidListing));
    }
    let code = parts[1]
        .parse::<u16>()
        .map_err(|_| invalid(TransportKind::InvalidListing))?;
    Ok((200..300).contains(&code))
}
fn parse_listing(root: &Url, bytes: &[u8], max: u32) -> Result<ChildListing, SyncTransportError> {
    let xml = parse_xml(bytes)?;
    let mut identities = BTreeSet::new();
    let mut entries = vec![];
    let mut count = 0;
    for r in &xml.children {
        if !r.named("response") {
            return Err(invalid(TransportKind::InvalidListing));
        }
        count += 1;
        if count > max {
            return Err(invalid(TransportKind::TooLarge));
        }
        let href = r.one("href")?;
        if !href.children.is_empty() {
            return Err(invalid(TransportKind::InvalidHref));
        }
        let identity = child_identity(root, href.text.trim())?;
        if !identities.insert(identity.clone()) {
            return Err(invalid(TransportKind::InvalidHref));
        }
        for status in r.all("status") {
            if !successful_status(status)? {
                return Err(invalid(TransportKind::PartialResponse));
            }
        }
        let mut resource = None;
        let mut length = None;
        for stat in r.all("propstat") {
            let success = successful_status(stat.one("status")?)?;
            let props = stat.one("prop")?;
            for prop in &props.children {
                if prop.named("resourcetype") {
                    if !success || resource.is_some() {
                        return Err(invalid(TransportKind::PartialResponse));
                    }
                    resource = Some(prop.children.iter().any(|n| n.named("collection")));
                }
                if prop.named("getcontentlength") && success && !prop.text.trim().is_empty() {
                    if length.is_some() {
                        return Err(invalid(TransportKind::InvalidListing));
                    }
                    length = Some(
                        prop.text
                            .trim()
                            .parse::<u64>()
                            .map_err(|_| invalid(TransportKind::InvalidListing))?,
                    );
                }
            }
        }
        let collection = resource.ok_or_else(|| invalid(TransportKind::PartialResponse))?;
        match identity {
            Some(name) => entries.push(ChildEntry {
                name,
                is_collection: collection,
                content_length: length,
            }),
            None if !collection => return Err(invalid(TransportKind::InvalidListing)),
            None => {}
        }
    }
    if count == 0 {
        return Err(invalid(TransportKind::InvalidListing));
    }
    Ok(ChildListing {
        entries,
        http_status: 207,
        response_bytes: bytes.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> Url {
        Url::parse("https://dav.example.test/app/user/").unwrap()
    }
    fn response(href: &str, properties: &str) -> String {
        format!(r#"<d:response><d:href>{href}</d:href>{properties}</d:response>"#)
    }
    fn props(collection: bool) -> String {
        format!(
            r#"<d:propstat><d:prop><d:resourcetype>{}</d:resourcetype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat>"#,
            if collection { "<d:collection/>" } else { "" }
        )
    }
    fn listing(rows: &[String]) -> Vec<u8> {
        format!(
            r#"<?xml version="1.0"?><d:multistatus xmlns:d="DAV:">{}</d:multistatus>"#,
            rows.join("")
        )
        .into_bytes()
    }
    #[test]
    fn depth_one_excludes_self_and_allows_missing_optional_length() {
        let optional = r#"<d:propstat><d:prop><d:getcontentlength/></d:prop><d:status>HTTP/1.1 404 Not Found</d:status></d:propstat>"#;
        let bytes = listing(&[
            response("/app/user/", &props(true)),
            response("file.json", &(props(false) + optional)),
        ]);
        let result = parse_listing(&root(), &bytes, 10).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].name, "file.json");
        assert!(result.entries[0].content_length.is_none());
        assert!(!result.entries[0].is_collection);
    }
    #[test]
    fn href_rejects_cross_origin_traversal_encoded_separators_and_duplicate_identity() {
        for h in [
            "https://evil.test/app/user/file",
            "//evil.test/app/user/file",
            "../file",
            "%2e%2e/file",
            "x%2ffile",
            "x%5cfile",
            "%252e%252e",
            "file?q=1",
            "file#f",
            "/app/user/sub/file",
            "file//",
        ] {
            assert!(child_identity(&root(), h).is_err(), "{h}");
        }
        assert_eq!(
            child_identity(&root(), "https://dav.example.test/app/user/file").unwrap(),
            Some("file".into())
        );
        let bytes = listing(&[
            response("file", &props(false)),
            response("/app/user/file", &props(false)),
        ]);
        assert!(matches!(
            parse_listing(&root(), &bytes, 10).unwrap_err().kind,
            TransportKind::InvalidHref
        ));
    }
    #[test]
    fn partial_truncated_failed_required_properties_and_dtd_never_look_empty() {
        let good = listing(&[response("/app/user/", &props(true))]);
        assert!(parse_listing(&root(), &good[..good.len() - 10], 10).is_err());
        assert!(parse_listing(&root(), &listing(&[]), 10).is_err());
        let failed = props(false).replace("200 OK", "404 Not Found");
        assert!(parse_listing(&root(), &listing(&[response("file", &failed)]), 10).is_err());
        let dtd = String::from_utf8(good).unwrap().replace(
            "?><d:",
            "?><!DOCTYPE x [<!ENTITY x SYSTEM 'file:///secret'>]><d:",
        );
        assert!(parse_listing(&root(), dtd.as_bytes(), 10).is_err());
        assert!(parse_listing(&root(), b"<multistatus><response/></multistatus>", 10).is_err());
    }
    #[test]
    fn self_counts_toward_limit_and_entities_cannot_hide_href_query() {
        let bytes = listing(&[
            response("/app/user/", &props(true)),
            response("file", &props(false)),
        ]);
        assert!(matches!(
            parse_listing(&root(), &bytes, 1).unwrap_err().kind,
            TransportKind::TooLarge
        ));
        let bytes = listing(&[response("file&#63;key=secret", &props(false))]);
        assert!(parse_listing(&root(), &bytes, 10).is_err());
    }
    #[test]
    fn immutable_binding_never_sends_credentials_to_rebound_endpoint() {
        let transport = FileWebDavTransport::new().unwrap();
        let handle = ConnectionHandle {
            id: "connection".into(),
            binding_id: format!("sha256:{}", "a".repeat(64)),
            revision: format!("sha256:{}", "b".repeat(64)),
        };
        transport
            .bind(
                handle.clone(),
                "https://dav.example.test/app",
                "user",
                "u".into(),
                "p".into(),
            )
            .unwrap();
        assert!(transport
            .bind(
                handle.clone(),
                "https://evil.test/app",
                "user",
                "u".into(),
                "p".into()
            )
            .is_err());
        let (request, url) = transport.request(&handle, Method::GET, None, 1000).unwrap();
        assert_eq!(url.as_str(), "https://dav.example.test/app/user/");
        let request = request.build().unwrap();
        assert!(request.headers().get("Authorization").is_some());
        assert!(request.headers().get("If-Match").is_none());
        assert!(request.headers().get("If-None-Match").is_none());
        let mut changed = handle;
        changed.revision = format!("sha256:{}", "c".repeat(64));
        assert!(transport
            .request(&changed, Method::PUT, None, 1000)
            .is_err());
    }
}
