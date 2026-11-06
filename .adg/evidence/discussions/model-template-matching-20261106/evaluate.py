"""Offline rule feasibility replay. Reads ONLY frozen whitelisted model-ID projection.
No network, production config writes, template adoption or model inference.
Family expectations are manually annotated naming groups, not capability truth.
"""
from pathlib import Path
import json,re,collections,hashlib,datetime,time
BASE=Path(__file__).resolve().parent

def glob(p,s):
    # Mirrors literal */? matching.rs contract; brackets are literals.
    pi=si=0;star=None;retry=0
    while si<len(s):
        if pi<len(p) and (p[pi]=='?' or p[pi]==s[si]):pi+=1;si+=1
        elif pi<len(p) and p[pi]=='*':star=pi;pi+=1;retry=si
        elif star is not None:retry+=1;si=retry;pi=star+1
        else:return False
    while pi<len(p) and p[pi]=='*':pi+=1
    return pi==len(p)

def literal(pattern,model,case=False):
    p=pattern['value'];m=model
    if not case:p=p.lower();m=m.lower()
    return {'exact':lambda:m==p,'prefix':lambda:m.startswith(p),'glob':lambda:glob(p,m)}[pattern['kind']]()

FAMILY_RULES={
 'GPT':{'prefix':['gpt-']},'Claude':{'prefix':['claude-']},'Gemini':{'prefix':['gemini-']},
 'GLM':{'prefix':['glm-'],'exact':['glm']},'Qwen':{'number_after':['qwen'],'prefix':['qwen-']},
 'HY':{'number_after':['hy']},'DeepSeek':{'prefix':['deepseek-']},'Kimi':{'prefix':['kimi-','moonshot-']},
 'MiniMax':{'prefix':['minimax-']},'Step':{'prefix':['step-']},'MiMo':{'prefix':['mimo-']},
 'Grok':{'prefix':['grok-']},'DoubaoSeed':{'prefix':['doubao-seed-']},
 'ERNIE':{'prefix':['ernie-']},'Mistral':{'prefix':['mistral-','devstral-']},
 'Gemma':{'prefix':['gemma-'],'number_after':['gemma']},'Llama':{'prefix':['llama-'],'number_after':['llama']}}
# Candidate channel-specific namespace declarations, NOT confirmed supplier equivalences.
PREFIXES={'GZGG-NewApi':['aixj-','btka-','ciii-','input-','siliconflow-',
 'gxlq-bailian-','gxlq-local-','gzgg-local-ezio-itx-','gzgg-local-ezio-pc-','gzgg-local-strata-']}
LABEL_MAP={'智谱 GLM':'GLM','腾讯混元 HY':'HY','千问 Qwen':'Qwen','Kimi / Moonshot':'Kimi','阶跃 Step':'Step','小米 MiMo':'MiMo','豆包 Seed':'DoubaoSeed','百度 ERNIE':'ERNIE','Mistral / Devstral':'Mistral'}

# Explicit anchored family/version slots. No arbitrary number search or punctuation rewrite.
VERSION_GRAMMARS=[
 ('GPT',r'^(gpt-)(.*)$'),
 ('Claude',r'^(claude-(?:opus|sonnet|haiku|fable|mythos)-)(.*)$'),
 ('Gemini',r'^(gemini-(?:omni-)?)(.*)$'),
 ('GLM',r'^(glm-)(.*)$'),('HY',r'^(hy)(.*)$'),
 ('Qwen',r'^(qwen-?)(.*)$'),('DeepSeek',r'^(deepseek-v)(.*)$'),
 ('Kimi',r'^(kimi-k)(.*)$'),('MiniMax',r'^(minimax-[mh])(.*)$'),
 ('MiMo',r'^(mimo-v)(.*)$'),('Grok',r'^(grok-)(.*)$'),('Step',r'^(step-)(.*)$')]

def view(provider,model,namespaces=True):
    m=model.lower();namespace=None
    if namespaces:
        for p in sorted(PREFIXES.get(provider,[]),key=len,reverse=True):
            if m.startswith(p):namespace=p;m=m[len(p):];break
    return m,namespace

def family(provider,model,namespaces=True):
    m,namespace=view(provider,model,namespaces)
    if m.startswith(('gpt-gpt-','claude-claude-','gemini-gemini-')):
        return None,m,namespace,'ambiguous_repeated_family_marker'
    hits=[]
    for f,rules in FAMILY_RULES.items():
        if (m in rules.get('exact',[]) or any(m.startswith(p) for p in rules.get('prefix',[]))
            or any(m.startswith(p) and len(m)>len(p) and m[len(p)] in '0123456789' for p in rules.get('number_after',[]))):hits.append(f)
    return hits[0] if len(hits)==1 else None,m,namespace,'matched' if len(hits)==1 else ('conflict' if hits else 'unassigned')

def parse_version(model):
    m=model.lower()
    for f,pattern in VERSION_GRAMMARS:
        hit=re.fullmatch(pattern,m)
        if not hit:continue
        prefix,body=hit.groups()
        # Each component must be an entire ASCII digit token. Keep leading zeros.
        pieces=re.split(r'([.-])',body)
        if not pieces or not re.fullmatch(r'[0-9]+',pieces[0]):return None
        nums=[pieces[0]];seps=[];end=len(pieces[0]);i=1
        while i+1<len(pieces) and re.fullmatch(r'[0-9]+',pieces[i+1]):
            seps.append(pieces[i]);nums.append(pieces[i+1]);end+=1+len(pieces[i+1]);i+=2
        tail=body[end:];date=None
        # Explicit rule: a trailing -YYYYMMDD is a retained build token, never version.
        if len(nums)>=3 and seps[-1]=='-' and len(nums[-1])==8:
            try:
                date=datetime.datetime.strptime(nums[-1],'%Y%m%d').date().isoformat()
            except ValueError:pass
            else:
                tail='-'+nums[-1]+tail;nums=nums[:-1];seps=seps[:-1]
        # A dot tail with a nonnumeric token is not a version grammar we understand.
        if tail.startswith('.'):return None
        return {'family':f,'prefix':prefix,'components':nums,'tail':tail,'retained_date':date,
                'ambiguous_numeric_suffix':len(nums)>=3 and len(nums[-1]) in (4,6) and '-' in seps}
    return None

def identity(model):
    p=parse_version(model)
    return (p['family'],p['prefix'],tuple(p['components']),p['tail']) if p and not p['ambiguous_numeric_suffix'] else None

def equivalent(a,b):
    # Equivalence mode is explicitly enabled in this experiment; no raw ID mutation.
    ka,kb=identity(a),identity(b)
    return ka is not None and ka==kb

CASES=[
 # Positive equalities, with fully retained series/spec/build identity.
 ('dot_dash_gpt','gpt-5.5','gpt-5-5',True),
 ('dot_dash_claude','claude-fable-5.1','claude-fable-5-1',True),
 ('same_spec','gpt-5.5-mini','gpt-5-5-mini',True),
 ('three_components','gpt-5.5.1','gpt-5-5-1',True),
 ('same_dated_build','claude-haiku-4.5-20251001','claude-haiku-4-5-20251001',True),
 ('gpt_minor','gpt-6.1-sol','gpt-6-1-sol',True),
 ('qwen_version_before_size','qwen3.8-27b','qwen3-8-27b',True),
 ('deepseek_version','deepseek-v4.1-flash','deepseek-v4-1-flash',True),
 ('gemini_version','gemini-3.5-flash','gemini-3-5-flash',True),
 ('glm_version','glm-5.3-flash','glm-5-3-flash',True),
 ('kimi_version','kimi-k2.7-code','kimi-k2-7-code',True),
 ('minimax_version','MiniMax-M3.1-Flash','minimax-m3-1-flash',True),
 # Negative controls protect every non-version portion and component boundary.
 ('different_major','gpt-5.5','gpt-15.5',False),
 ('different_minor','gpt-5.5','gpt-5.50',False),
 ('patch_not_minor','gpt-5.5','gpt-5.5.1',False),
 ('dashed_patch_not_minor','gpt-5.5','gpt-5-5-1',False),
 ('missing_separator','gpt-5.5','gpt-55',False),
 ('underscore_not_declared','gpt-5.5','gpt-5_5',False),
 ('space_not_declared','gpt-5.5','gpt-5 5',False),
 ('any_char_not_separator','gpt-5.5','gpt-5x5',False),
 ('different_spec','gpt-5.5-mini','gpt-5-5-pro',False),
 ('suffix_is_not_version','gpt-5.5-mini','gpt-5.5-mini.1',False),
 ('family_not_substring','gpt-5.5','notgpt-5-5',False),
 ('channel_prefix_not_implicit','gpt-5.5','vendor-gpt-5-5',False),
 ('series_not_same','claude-opus-5.5','claude-sonnet-5-5',False),
 ('family_not_same','gpt-5.5','glm-5-5',False),
 ('build_not_ignored','claude-haiku-4.5','claude-haiku-4-5-20251001',False),
 ('date_not_equated','claude-haiku-4.5-20251001','claude-haiku-4-5-20251002',False),
 ('build_date_delimiters_not_global','claude-haiku-4.5-20251001','claude-haiku-4-5-2025-10-01',False),
 ('invalid_date_not_build','claude-haiku-4.5','claude-haiku-4-5-20251301',False),
 ('leading_zero_not_folded','gpt-5.5','gpt-05-5',False),
 ('version_zero_not_folded','gpt-5.5','gpt-5-05',False),
 ('date_suffix_ambiguous','grok-4.20','grok-4.20-0309',False),
 ('ambiguous_date_slots_not_equated','grok-4.20-0309','grok-4-20-0309',False),
 ('ambiguous_six_digit_build','gpt-5.5-260915','gpt-5-5-260915',False),
 ('rolling_alias_not_proven','gpt-5.5','gpt-5.5-latest',False),
 ('tag_not_punctuation_normalized','gpt-5.5-code-mini','gpt-5-5-code.mini',False),
 ('non_ascii_separator','gpt-5.5','gpt-5\u20135',False),
 ('qualified_number','gpt-5.5','gpt-5.5b',False),
 ('date_in_other_name','gpt-5.5','model-2026-05-05',False),
 ('review_task_not_model','gpt-5.5','codex-auto-review',False),
 ('retain_quantization','qwen3.8-27b-q6','qwen3-8-27b-q8',False),
 ('retain_model_size','qwen3.8-27b','qwen3-8-72b',False),
 ('unparsed_symbolic_id','gpt-reserve','gpt-reserve',False)
]

FAMILY_NEGATIVES=[('other','notgpt-5.5',None),('other','backup-gpt-5.5',None),
 ('other','qwenish-3.8',None),('other','hypothesis-3',None),('other','doubao-seedream-5',None),
 ('other','text-embedding-3-large',None),('other','claudette-5.5',None),
 ('other','aixj-gpt-5.5',None),('other','qwen５.５',None),('GZGG-NewApi','aixj-gpt-5.5','GPT'),
 ('GZGG-NewApi','ciii-gpt-gpt-5.6-terra',None),('other','mygemini-3.8',None),
 ('other','gemini-embedding-2-preview','Gemini')]


def main():
    snapshot=json.loads((BASE/'model-id-snapshot.json').read_text())
    seed=json.loads((BASE/'release-seed-rules.json').read_text())
    expected_doc=json.loads((BASE/'expected-families.json').read_text())
    expected={m:(f if f!='unassigned' else None) for f,ms in expected_doc['groups'].items() for m in ms.split()}
    selected_union={m for p in snapshot['providers'] for m in (p['explicit_selected_ids'] or [])}
    ids=sorted({m for p in snapshot['providers'] for m in p['model_ids']})
    assert set(expected)==set(ids)
    rows=[]
    for p in snapshot['providers']:
        for m in p['model_ids']:
            old=[];old_variants=[]
            for f in seed['families']:
                c=f['content'];r=c['matching'];case=r.get('caseSensitive',False)
                if any(literal(x,m,case) for x in r['include']) and not any(literal(x,m,case) for x in r.get('exclude',[])):
                    old.append(LABEL_MAP.get(f['label'],f['label']))
                    mm=m if case else m.lower()
                    vs=[v for v in c.get('variants',[]) if mm.endswith(v['suffix'] if case else v['suffix'].lower())]
                    if vs:old_variants.append(max(vs,key=lambda x:len(x['suffix']))['id'])
            direct,_,_,_=family(p['provider'],m,False)
            candidate,view_id,ns,reason=family(p['provider'],m,True)
            parsed=parse_version(view_id)
            rows.append({'provider':p['provider'],'raw_model_id':m,'explicitly_selected':m in p['explicit_selected_ids'] if p['explicit_selected_ids'] is not None else None,
                'expected_naming_family':expected[m],'baseline_potential_seed_families':old,'baseline_potential_variant_ids':old_variants,
                'direct_general_family':direct,'candidate_naming_family':candidate,'matching_view':view_id,'channel_namespace':ns,
                'namespace_status':'candidate_unconfirmed_do_not_adopt' if ns else None,'reason':reason,'parsed_version':parsed,
                'capability_parameters_changed':False})
    annotated=[r for r in rows if r['expected_naming_family']]
    incorrect=[r for r in rows if r['candidate_naming_family'] and r['candidate_naming_family']!=r['expected_naming_family']]
    uncovered=[r for r in rows if r['expected_naming_family'] and not r['candidate_naming_family']]
    matched_unique={r['raw_model_id'] for r in rows if r['candidate_naming_family']}
    baseline_unique={r['raw_model_id'] for r in rows if r['baseline_potential_seed_families']}
    direct_unique={r['raw_model_id'] for r in rows if r['direct_general_family']}
    real_equiv=[]
    for i,a in enumerate(ids):
        for b in ids[i+1:]:
            if a.lower()!=b.lower() and equivalent(a,b):real_equiv.append({'a':a,'b':b,'meaning':'same naming structure under explicit version rule; supplier/capability identity NOT proven'})
    separator_tests=[{'case':name,'a':a,'b':b,'expected':exp,'actual':equivalent(a,b)} for name,a,b,exp in CASES]
    family_tests=[{'provider':p,'model':m,'expected':exp,'actual':family(p,m)[0]} for p,m,exp in FAMILY_NEGATIVES]
    assert all(t['expected']==t['actual'] for t in separator_tests),[t for t in separator_tests if t['expected']!=t['actual']]
    assert all(t['expected']==t['actual'] for t in family_tests)
    # Explicit overlapping equivalent declarations must fail, not pick by byte length/order.
    dup_keys=[identity('gpt-5.5-mini'),identity('gpt-5-5-mini')]
    duplicate_detected=len(set(dup_keys))<len(dup_keys)
    # Unknown future matches only inherited common fields, not a known variant's capabilities.
    common_all_inherit=all(v.get('mode')=='inherit' for f in seed['families'] for v in f['content']['rules'].values())
    parsed_rows=[r for r in rows if r['parsed_version']]
    by_channel=[]
    for p in snapshot['providers']:
        rs=[r for r in rows if r['provider']==p['provider']]
        by_channel.append({'provider':p['provider'],'local_model_count':len(rs),'baseline_potential_family_matches':sum(bool(r['baseline_potential_seed_families']) for r in rs),'general_direct_matches':sum(bool(r['direct_general_family']) for r in rs),'general_with_declared_namespace_matches':sum(bool(r['candidate_naming_family']) for r in rs),'namespace_dependent_count':sum(bool(r['channel_namespace']) for r in rs),'review_needed':sum(r['reason']=='ambiguous_repeated_family_marker' for r in rs)})
    summary={'status':'naming-rule-feasibility-passed-with-explicit-limits','providers':len(snapshot['providers']),'providers_with_local_models':sum(bool(p['model_ids']) for p in snapshot['providers']),'providers_without_local_models':sum(not p['model_ids'] for p in snapshot['providers']),'channel_model_rows':len(rows),'unique_ids':len(ids),'explicitly_selected_unique_ids':len(selected_union),'baseline_seed_family_count':len(seed['families']),'baseline_include_rule_kinds':dict(collections.Counter(x['kind'] for f in seed['families'] for x in f['content']['matching']['include'])),
      'coverage_unique':{'baseline_potential_exact_seed':len(baseline_unique),'general_direct':len(direct_unique),'with_explicit_channel_namespace_candidates':len(matched_unique),'manually_annotated_supported_naming_families':sum(v is not None for v in expected.values()),'unassigned_other_families_or_special_tasks':sum(v is None for v in expected.values()),'requires_manual_review':len({r['raw_model_id'] for r in uncovered})},
      'coverage_rows':{'baseline':sum(bool(r['baseline_potential_seed_families']) for r in rows),'direct':sum(bool(r['direct_general_family']) for r in rows),'with_declared_namespace_candidates':len([r for r in rows if r['candidate_naming_family']])},
      'incorrect_family_predictions_against_manual_naming_labels':len(incorrect),'active_template_adoption':snapshot['adoption'],'current_common_rules_all_inherit':common_all_inherit,'parsed_version_rows':len(parsed_rows),'real_dot_dash_pairs':real_equiv,'synthetic_separator_cases':len(separator_tests),'synthetic_separator_passed':sum(t['actual']==t['expected'] for t in separator_tests),'synthetic_family_boundary_cases':len(family_tests),'synthetic_family_boundary_passed':sum(t['actual']==t['expected'] for t in family_tests),'duplicate_equivalent_variant_detected':duplicate_detected,
      'scope_limits':['Current saved local list only; 11 providers have no local models; no discovery network calls.','152-ID inventory informed rule design: this is an exhaustive replay, NOT a blind held-out validation of future providers.','Manual expectations label visible model-name families, NOT vendor identity, protocol, context window, reasoning or other capabilities.','Namespace declarations are explicit candidates requiring user confirmation before production adoption.','Zero active templateRefs/modelRefs: baseline counts are potential rule coverage, NOT currently broken runtime configurations.','Version equivalence applies ONLY inside an explicit anchored numeric slot and preserves series/spec/build/tail and numeric component strings.','4/6-digit date-like suffixes remain ambiguous and are not silently stripped.','No production code, template library, adopted revision or model config changes.'],
      'by_channel':by_channel,'uncovered':uncovered,'rule_design':{'family_rules':FAMILY_RULES,'channel_scoped_namespace_candidates':PREFIXES,'version_grammar_candidates':VERSION_GRAMMARS,'allowed_numeric_separators':['.','-'],'case_policy':'current default case insensitive; explicit case-sensitive rules remain available','unknown_capability_policy':'inherit/unknown, no inferred values'}}
    assert not incorrect and duplicate_detected
    (BASE/'model-results.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
    (BASE/'synthetic-results.json').write_text(json.dumps({'separator':separator_tests,'family_boundaries':family_tests},ensure_ascii=False,indent=2)+'\n')
    (BASE/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:v for k,v in summary.items() if k not in ('by_channel','uncovered','rule_design','scope_limits')},ensure_ascii=False,indent=2))

if __name__=='__main__':main()
