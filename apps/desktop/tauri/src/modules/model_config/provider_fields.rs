//! Frozen official 2.84.0 provider editor field contract. Unknown source fields
//! stop export instead of being silently dropped. Secrets stay in the protected
//! authorization object; runtime observations are never portable write authority.
use super::schema::*;
use serde_json::Value;
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
}
