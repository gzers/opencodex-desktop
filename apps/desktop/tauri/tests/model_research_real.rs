//! Explicit opt-in public HTTPS read/search. No accounts, secrets or model calls.
use opencodex_desktop_lib::modules::model_config::agent;

#[tokio::test]
#[ignore = "explicit public-network read/search admission; no model generation"]
async fn public_text_and_search_return_recorded_sources_without_model_requests() {
    assert_eq!(std::env::var("OCX_TEST_RESEARCH_REAL").as_deref(), Ok("1"));
    let root = tempfile::tempdir().unwrap();
    let direct = agent::research(vec!["https://www.rfc-editor.org/rfc/rfc8259.txt".into()])
        .await
        .expect("public RFC text read must complete with the actual research service");
    assert_eq!(direct.len(), 1);
    assert!(direct[0].text.contains("JavaScript Object Notation"));
    let recorded = agent::record_sources(root.path(), direct).unwrap();
    for source in &recorded {
        assert!(source.receipt_id.is_some());
        assert!(source
            .content_digest
            .as_deref()
            .unwrap()
            .starts_with("sha256:"));
        println!(
            "direct_source url={} retrieved={} receipt={} digest={}",
            source.url,
            source.retrieved_at,
            source.receipt_id.as_deref().unwrap(),
            source.content_digest.as_deref().unwrap()
        );
    }
    let found = agent::search("site:rfc-editor.org RFC 8259 JSON".into())
        .await
        .expect("actual search must yield at least one readable public source");
    assert!(!found.is_empty());
    let sources = agent::record_sources(root.path(), found).unwrap();
    for source in &sources {
        assert!(!source.text.trim().is_empty());
        assert!(source.url.starts_with("https://"));
        println!(
            "search_source url={} retrieved={} receipt={} digest={}",
            source.url,
            source.retrieved_at,
            source.receipt_id.as_deref().unwrap(),
            source.content_digest.as_deref().unwrap()
        );
    }
    assert!(!root
        .path()
        .join("manager-state/model-family-templates.json")
        .exists());
    assert!(!root.path().join("manager-state/model-policy.json").exists());
}
