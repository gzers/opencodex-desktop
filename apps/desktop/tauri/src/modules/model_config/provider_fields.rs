//! Frozen official 2.84.0 provider editor field contract. Unknown source fields
//! stop export instead of being silently dropped. Secrets stay in the protected
//! authorization object; runtime observations are never portable write authority.
use super::schema::*;
use serde_json::Value;
/// 官方容忍但不属于 `OcxProviderConfig` 类型的历史 / 身份回显字段。
///
/// 实测依据（2026-10-24）：
/// - 官方 2.84.0 的 `OcxProviderConfig`（144 个字段）**没有** `name`；
/// - 但官方 `ocx config validate` 对含 `name` 的配置返回 ok；
/// - 且官方 `POST /api/providers` 会把请求体里的 `name` 回写进 provider 对象。
///
/// 因此它既不是需要保留的写入权威，也不该让导出整体失败。
/// 这里显式列出「已知可忽略」的字段：校验后从公开载荷剔除，
/// 未知字段仍然拒绝——安全原则不变。
pub const ECHO: &[&str] = &["name"];
pub const PUBLIC: &[&str] = &[
    "alias",
    "modelAliases",
    "modelDisplayNames",
    "defaultAliases",
    "adapter",
    "codexToolMode",
    "projectContext",
    "requestPacing",
    "mcpMaxTools",
    "mcpMaxSchemaBytes",
    "mcpMaxResultBytes",
    "modelAdapters",
    "fastWire",
    "responseTierAuthoritative",
    "fastEnabled",
    "baseUrl",
    "responsesPath",
    "chatCompletionsPath",
    "commandCodeVersion",
    "statelessResponses",
    "requiresAdjacentResponsesToolResults",
    "requiresPairedResponsesToolResults",
    "annotateEmptyToolOutputs",
    "supportsServiceTier",
    "modelSupportsServiceTier",
    "preserveResponsesReasoningContent",
    "preserveResponsesInputItemIds",
    "preserveResponsesMessageMetadata",
    "dropResponsesReasoningItems",
    "modelReasoningEffortsAuthoritative",
    "decodesNativeCompactionBlobs",
    "allowEncryptedV2AgentTasks",
    "allowPrivateNetwork",
    "noProxy",
    "upstreamHttpVersion",
    "upstreamWebsocket",
    "directGeminiWireRenames",
    "googleToolSchemaPolicy",
    "disabled",
    "codexAccountMode",
    "apiKeyTransport",
    "apiKeyPoolStrategy",
    "defaultModel",
    "models",
    "liveModels",
    "selectedModels",
    "retainModels",
    "newModelPolicy",
    "modelPreset",
    "contextWindow",
    "modelContextWindows",
    "modelContextTiers",
    "modelInputModalities",
    "modelCapabilities",
    "modelAutoCompactTokenLimits",
    "defaultMaxOutputTokens",
    "modelMaxOutputTokens",
    "modelCosts",
    "forwardClientHeaders",
    "openRouterRouting",
    "modelOpenRouterRouting",
    "vercelGatewayRouting",
    "modelVercelGatewayRouting",
    "authMode",
    "oauthAccountFailover",
    "anthropicAccountPool",
    "anthropicOAuthInstance",
    "keyOptional",
    "freeTier",
    "note",
    "modelSuffixBracketStrip",
    "refreshPolicy",
    "reasoningEfforts",
    "modelReasoningEfforts",
    "modelSuppressSyntheticMax",
    "modelDefaultReasoningEfforts",
    "pinnedReasoningEffort",
    "modelPinnedReasoningEfforts",
    "autoReviewModel",
    "autoReviewModelOverrides",
    "modelSupportsReasoningSummaries",
    "modelSupportsVerbosity",
    "supportsVerbosity",
    "modelReasoningSummaryDelivery",
    "modelPreferHostedTools",
    "supportsOpenAiWebSearchToolFields",
    "xaiResponsesXSearch",
    "supportsResponsesCustomTools",
    "unsupportedHostedTools",
    "responsesSnapshotRepair",
    "webSearchBridge",
    "reasoningEffortMap",
    "modelReasoningEffortMap",
    "reasoningWireFormat",
    "noReasoningModels",
    "noTemperatureModels",
    "noTopPModels",
    "noStopModels",
    "noPenaltyModels",
    "noStructuredOutputModels",
    "noJsonSchemaModels",
    "omitReasoningEffortWithToolsModels",
    "parallelToolCalls",
    "pinParallelToolCallsFalse",
    "terminalContinuationGuard",
    "openaiChatEofTolerance",
    "foldDeveloperRoleToSystem",
    "promptCacheKey",
    "chatServiceTier",
    "responsesItemIdRepair",
    "autoToolChoiceOnlyModels",
    "preserveReasoningContentModels",
    "requiresReasoningPlaceholderModels",
    "showThinkingSummary",
    "hideRawReasoning",
    "retryOn429",
    "transientRetryOn5xx",
    "retryOnReset",
    "reasoningSplitModels",
    "inlineThinkTagModels",
    "reasoningDetailsModels",
    "thinkingToggleModels",
    "thinkingBudgetModels",
    "escapeBuiltinToolNames",
    "anthropicEofTolerance",
    "noVisionModels",
    "googleMode",
    "project",
    "location",
    "unsafeAllowNativeLocalExec",
    "nativeLocalExec",
    "tlsProfile",
];
pub const PRIVATE: &[&str] = &[
    "proxy",
    "apiKey",
    "apiKeyPool",
    "headers",
    "mcpServers",
    "desktopExecutor",
];
pub const RUNTIME: &[&str] = &[
    "apiKeySelectionRevision",
    "_apiKeyAttempt",
    "initialModelSelection",
    "modelMaxInputTokens",
    "xaiResponsesDefaultVersion",
    "zaiResponsesDefaultVersion",
];
pub fn validate_public(value: &Value) -> ModelResult<()> {
    let fields = value
        .as_object()
        .ok_or_else(|| invalid("provider_fields", "渠道原字段不是对象"))?;
    if fields.keys().any(|key| !PUBLIC.contains(&key.as_str())) {
        return Err(invalid(
            "provider_fields",
            "渠道包含未支持或不可公开的原字段，未截断采用",
        ));
    }
    if serde_json::to_vec(value)?.len() > 2 * 1024 * 1024 {
        return Err(invalid("provider_fields", "渠道原字段超过上限"));
    }
    // Same recursive redaction boundary as the official snapshot sent to the UI.
    super::upstream::reject_secrets(value)
}
pub fn public_from_raw(raw: &Value) -> ModelResult<Value> {
    let fields = raw
        .as_object()
        .ok_or_else(|| invalid("provider_fields", "渠道源配置不是对象"))?;
    if fields.keys().any(|key| {
        !PUBLIC.contains(&key.as_str())
            && !PRIVATE.contains(&key.as_str())
            && !RUNTIME.contains(&key.as_str())
            && !ECHO.contains(&key.as_str())
    }) {
        return Err(invalid(
            "provider_fields",
            "实际渠道含当前适配器未知字段，请升级适配后再导出",
        ));
    }
    let selected = Value::Object(
        fields
            .iter()
            .filter(|(key, _)| PUBLIC.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    );
    validate_public(&selected)?;
    Ok(selected)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn original_public_overlays_survive_but_secrets_runtime_and_unknowns_cannot_become_write_authority(
    ) {
        let raw = json!({"adapter":"openai-chat","baseUrl":"https://example.test/v1","alias":"team","modelAliases":{"a":"route-a"},"apiKeyPoolStrategy":"fill-first","reasoningEffortMap":{"high":"medium"},"apiKey":"fixture-secret","headers":{"Authorization":"fixture-secret"},"apiKeySelectionRevision":"runtime"});
        let public = public_from_raw(&raw).unwrap();
        assert_eq!(public["modelAliases"], raw["modelAliases"]);
        assert_eq!(public["apiKeyPoolStrategy"], "fill-first");
        assert!(
            public.get("apiKey").is_none()
                && public.get("headers").is_none()
                && public.get("apiKeySelectionRevision").is_none()
        );
        assert!(validate_public(&raw).is_err());
        let mut unknown = raw;
        unknown["futureCredentialBlock"] = json!({"secret":"x"});
        assert!(public_from_raw(&unknown).is_err());
    }
    #[test]
    fn tolerated_echo_field_does_not_block_export_and_never_becomes_write_authority() {
        // 真实场景（2026-10-24）：官方 POST /api/providers 会把请求体里的 name 回写进
        // provider 对象，于是真实配置里出现 name。它不在 OcxProviderConfig 里，
        // 但官方 config validate 接受。此前我们把它判成「未知字段」并阻断整个导出，
        // 导致含该字段的渠道永远导不出去。
        let raw = json!({
            "adapter": "openai-responses",
            "baseUrl": "https://example.test/v1",
            "name": "FeiMao-CNM",
            "models": ["a"],
            "selectedModels": ["a"]
        });
        let public = public_from_raw(&raw).expect("echo field must not block export");
        assert!(
            public.get("name").is_none(),
            "echo field must not become write authority"
        );
        assert_eq!(public["adapter"], "openai-responses");
        assert_eq!(public["models"][0], "a");

        // 未知字段仍然拒绝：放宽的只有白名单里点名的 name。
        let mut unknown = raw;
        unknown["futureSecretBlock"] = json!({"secret": "x"});
        assert!(public_from_raw(&unknown).is_err());
    }
}
