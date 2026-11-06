"""Prepare reviewable 0.2.23 template configuration data from pinned source rules.
Writes project candidate artifact only. Never reads/writes live runtime configuration.
"""
from pathlib import Path
import importlib.util,json,hashlib,uuid
BASE=Path(__file__).resolve().parent
REPO=next(p for p in BASE.parents if (p/'.adg/project.json').exists())
TARGET=REPO/'docs/04-项目资料/01-项目支撑资料/06-问题记录/模型模版通用匹配配置/model-templates.0.2.23.candidate.json'
spec=importlib.util.spec_from_file_location('analysis',BASE/'evaluate.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
seed=json.loads((BASE/'release-seed-rules.json').read_text())
identity=json.loads((BASE/'source-identity.json').read_text())
labels={m.LABEL_MAP.get(f['label'],f['label']):f for f in seed['families']}
base_layouts={
 'GPT':['gpt-'],'Claude':['claude-'+x+'-' for x in ['opus','sonnet','haiku','fable','mythos']],
 'Gemini':['gemini-','gemini-omni-'],'GLM':['glm-'],'HY':['hy'],
 'Qwen':['qwen','qwen-'],'DeepSeek':['deepseek-v'],'Kimi':['kimi-k'],
 'MiniMax':['minimax-m','minimax-h'],'MiMo':['mimo-v'],'Grok':['grok-'],'Step':['step-']}
def tag_prefix(prefix):return 'layout-'+hashlib.sha256(prefix.encode()).hexdigest()[:12]
def generalized_patterns(family):
 r=m.FAMILY_RULES[family];out=[]
 for kind in ['exact','prefix','number_after']:
  for value in r.get(kind,[]):out.append({'kind':'ascii_numeric_prefix' if kind=='number_after' else kind,'value':value})
 # Preserve baseline identities not covered by the generic family-name grammar.
 for p in labels[family]['content']['matching']['include']:
  if m.family('other',p['value'],False)[0]!=family and p not in out:out.append(p)
 return out

templates=[]
for f in seed['families']:
 family=m.LABEL_MAP.get(f['label'],f['label']);c=f['content']
 layouts=[{'id':tag_prefix(x),'literalPrefix':x,'versionCapture':'complete_numeric_components','numericJoiners':['.','-'],'tailCapture':'literal_remainder','trailingDatePolicy':'retain_valid_YYYYMMDD_after_hyphen','ambiguousNumericTail':'no_equivalence'} for x in base_layouts.get(family,[])]
 overrides=[]
 for v in c.get('variants',[]):
  parsed=m.parse_version(v['suffix']);conditions=[]
  if parsed and not parsed['ambiguous_numeric_suffix']:
   prefix=parsed['prefix'];lid=tag_prefix(prefix)
   if lid not in {x['id'] for x in layouts}:
    layouts.append({'id':lid,'literalPrefix':prefix,'versionCapture':'complete_numeric_components','numericJoiners':['.','-'],'tailCapture':'literal_remainder','trailingDatePolicy':'retain_valid_YYYYMMDD_after_hyphen','ambiguousNumericTail':'no_equivalence'})
   conditions.append({'kind':'numeric_version','components':parsed['components']})
   # Series/spec conditions may occur before or after version, never suffix-only.
   prefix_words=prefix.rstrip('-').split('-')[1:]
   if not prefix.endswith('-'):prefix_words=prefix_words[:-1]
   words=[w for w in (prefix_words+parsed['tail'].lstrip('-').split('-')) if w and any('a'<=ch.lower()<='z' for ch in w)]
   for word in dict.fromkeys(words):conditions.append({'kind':'literal_segment','value':word,'position':'any','boundary':'hyphen_or_string_edge'})
   selector={'all':conditions,'layoutId':lid,'literalTail':parsed['tail']}
  else:selector={'all':[{'kind':'exact_id','value':v['suffix']}]}
  overrides.append({'id':v['id'],'name':v['name'],'selector':selector,'rules':v['rules'],'lockedFields':v.get('lockedFields',[]),'sourceExactId':v['suffix'],'scope':'known_identity_only'})
 matching={'channelMode':c['matching']['channelMode'],'channels':c['matching']['channels'],'caseSensitive':False,'priority':c['matching']['priority'],'include':generalized_patterns(family),'exclude':c['matching'].get('exclude',[])}
 content={'description':c['description'],'matching':matching,'nameLayouts':layouts,'commonRules':c['rules'],'conditionOverrides':overrides}
 digest=hashlib.sha256(json.dumps(content,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
 templates.append({'templateId':f['id'],'familyKey':family,'label':f['label'],'candidateRevisionId':str(uuid.uuid5(uuid.NAMESPACE_URL,'ocxd/0.2.23/general-matching/'+f['id']+'/'+digest)),'revisionState':'candidate','sourceContentSha256':hashlib.sha256(json.dumps(c,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'content':content})

config={'format':'ocxd-model-template-candidate-v1','targetVersion':'0.2.23','stage':'prepared_not_activated','productionReaderSupport':'pending_0.2.23_implementation','source':{'codeCommit':identity['repository_head'],'seedSha256':seed['source_sha256'],'modelSnapshotSha256':hashlib.sha256((BASE/'model-id-snapshot.json').read_bytes()).hexdigest()},
 'semantics':{'rawModelId':'preserve_for_requests_and_routing','matchingCase':'case_insensitive_in_new_candidate_revision_only','channelProtocolProperty':False,'modelProtocolResolution':['adopted_template_common','matched_condition_override','explicit_single_model'],'unresolvedModelProtocol':'needs_explicit_configuration_before_using_affected_route','listDiscoveryAdapter':'independent_of_model_request_protocol','runtimeProviderAdapter':'derive_from_final_model_protocol_not_channel_default','groupOperator':'all','excludePriority':'before_include','literalPositionDefault':'any','versionEquivalence':'only_declared_numeric_slot','versionComponents':'strings_preserve_leading_zeroes_and_full_arity','unknownFamily':'keep_existing_config','unknownVariant':'inherit_common_only','samePriorityOverlap':'error','equivalentDuplicateSelector':'error','autoAdoptTemplates':True,'adoptionTrigger':'successful_read_existing_channels','adoptionTarget':'unsaved_draft','automaticPersistentWrite':False,'readVersionSelection':'explicit_reference_else_designated_saved_default','manualSingleModelOverrides':'preserve_explicit_only','unmatchedReadModel':'preserve_existing','autoRewriteSavedRevisions':False,'namespaceMapping':'explicit_channel_only_matching_view','unlistedProtocolsOrCapabilities':'do_not_infer_from_name'},
 'channelNamespaceCandidates':[{'providerId':provider,'channelUuidBinding':'resolve_current_binding_before_adoption','removeLeadingPrefixes':prefixes,'requestModelId':'unchanged','approvalState':'candidate_for_explicit_adoption'} for provider,prefixes in m.PREFIXES.items()],
 'unmatchedPolicy':{'action':'leave_existing','coverageCompletionRequired':False,'frozenSampleUnassignedIds':18,'frozenSampleRepeatedPrefixIds':1,'repeatedPrefixHandling':'do_not_collapse_or_map','networkDiscoveryRequired':False},'templates':templates}
TARGET.parent.mkdir(parents=True,exist_ok=True)
TARGET.write_text(json.dumps(config,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'candidate_file':str(TARGET.relative_to(REPO)),'templates':len(templates),'include_patterns':sum(len(t['content']['matching']['include']) for t in templates),'known_identity_overrides':sum(len(t['content']['conditionOverrides']) for t in templates),'structured_overrides':sum('layoutId' in o['selector'] for t in templates for o in t['content']['conditionOverrides']),'current_library_mutation':False},ensure_ascii=False,indent=2))
