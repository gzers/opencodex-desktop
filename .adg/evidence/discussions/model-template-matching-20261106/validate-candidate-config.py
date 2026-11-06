"""Strict candidate-field checks + data-driven offline matcher, NOT a product reader.
stdlib only; no network or daily-config access. Reports actual candidate JSON replay.
"""
from pathlib import Path
import importlib.util,json,re,datetime,hashlib,uuid,copy
BASE=Path(__file__).resolve().parent
REPO=next(p for p in BASE.parents if (p/'.adg/project.json').exists())
CANDIDATE=REPO/'docs/04-项目资料/01-项目支撑资料/06-问题记录/模型模版通用匹配配置/model-templates.0.2.23.candidate.json'
spec=importlib.util.spec_from_file_location('frozen_analysis',BASE/'evaluate.py')
baseline=importlib.util.module_from_spec(spec);spec.loader.exec_module(baseline)

def require(value,why):
    if not value:raise ValueError(why)

def validate(cfg,seed):
    require(set(cfg)=={'format','targetVersion','stage','productionReaderSupport','source','semantics','channelNamespaceCandidates','unmatchedPolicy','templates'},'top-level fields')
    require(cfg['format']=='ocxd-model-template-candidate-v1' and cfg['targetVersion']=='0.2.23','candidate version')
    require(cfg['stage']=='prepared_not_activated','activation must remain disabled')
    require(cfg['semantics']['groupOperator']=='all' and cfg['semantics']['autoAdoptTemplates'] is True and cfg['semantics']['adoptionTarget']=='unsaved_draft' and cfg['semantics']['adoptionTrigger']=='successful_read_existing_channels' and cfg['semantics']['automaticPersistentWrite'] is False and not cfg['semantics']['autoRewriteSavedRevisions'],'explicit adoption/AND')
    require(cfg['semantics']['versionEquivalence']=='only_declared_numeric_slot','numeric-only equivalence')
    require(cfg['semantics']['channelProtocolProperty'] is False and cfg['semantics']['modelProtocolResolution']==['adopted_template_common','matched_condition_override','explicit_single_model'],'no channel protocol fallback')
    require(cfg['semantics']['listDiscoveryAdapter']=='independent_of_model_request_protocol','discovery protocol independence')
    require(cfg['semantics']['samePriorityOverlap']=='error' and cfg['semantics']['equivalentDuplicateSelector']=='error','conflict policy')
    require(cfg['unmatchedPolicy']['action']=='leave_existing' and not cfg['unmatchedPolicy']['coverageCompletionRequired'],'unmatched policy')
    source={x['id']:x for x in seed['families']};ids=set();revision_ids=set()
    for ns in cfg['channelNamespaceCandidates']:
        require(set(ns)=={'providerId','channelUuidBinding','removeLeadingPrefixes','requestModelId','approvalState'},'namespace fields')
        require(ns['providerId'] and ns['channelUuidBinding']=='resolve_current_binding_before_adoption','namespace explicit current binding')
        require(ns['requestModelId']=='unchanged' and ns['approvalState']=='candidate_for_explicit_adoption','namespace no raw rewrite')
        ps=ns['removeLeadingPrefixes'];require(ps and len(ps)==len(set(ps)) and all(p and len(p)<=256 for p in ps),'namespace prefixes')
    for t in cfg['templates']:
        require(set(t)=={'templateId','familyKey','label','candidateRevisionId','revisionState','sourceContentSha256','content'},'template fields')
        require(t['templateId'] in source and t['templateId'] not in ids,'template identity');ids.add(t['templateId'])
        uuid.UUID(t['candidateRevisionId']);require(t['candidateRevisionId'] not in revision_ids and t['candidateRevisionId']!=t['templateId'],'candidate revision identity');revision_ids.add(t['candidateRevisionId'])
        require(t['revisionState']=='candidate','candidate revision state')
        c=t['content'];require(set(c)=={'description','matching','nameLayouts','commonRules','conditionOverrides'},'content fields')
        original=source[t['templateId']]['content']
        require(t['sourceContentSha256']==hashlib.sha256(json.dumps(original,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'source content hash')
        require(c['commonRules']==original['rules'],'no guessed common capabilities')
        r=c['matching'];require(set(r)=={'channelMode','channels','caseSensitive','priority','include','exclude'},'match fields')
        require(r['channelMode'] in ('all','whitelist','blacklist') and r['caseSensitive'] is False and type(r['priority']) is int and r['include'],'match metadata and new candidate case policy')
        for p in r['include']+r['exclude']:
            require(set(p)=={'kind','value'} and p['kind'] in ('exact','prefix','glob','ascii_numeric_prefix') and isinstance(p['value'],str) and 0<len(p['value'])<=256,'pattern')
        layouts={}
        for layout in c['nameLayouts']:
            require(set(layout)=={'id','literalPrefix','versionCapture','numericJoiners','tailCapture','trailingDatePolicy','ambiguousNumericTail'},'layout fields')
            require(layout['id'] not in layouts and layout['literalPrefix'],'layout identity')
            require(layout['numericJoiners']==['.','-'] and layout['versionCapture']=='complete_numeric_components' and layout['tailCapture']=='literal_remainder','layout semantics')
            require(layout['trailingDatePolicy']=='retain_valid_YYYYMMDD_after_hyphen' and layout['ambiguousNumericTail']=='no_equivalence','build handling');layouts[layout['id']]=layout
        old={v['id']:v for v in original.get('variants',[])};seen=set();keys=set()
        for o in c['conditionOverrides']:
            require(set(o)=={'id','name','selector','rules','lockedFields','sourceExactId','scope'},'override fields')
            require(o['id'] in old and o['id'] not in seen,'override identity');seen.add(o['id']);v=old[o['id']]
            require(o['rules']==v['rules'] and o['lockedFields']==v.get('lockedFields',[]) and o['sourceExactId']==v['suffix'],'preserve known override parameters and provenance')
            require(o['scope']=='known_identity_only','unknown capability guard')
            sel=o['selector'];require(set(sel) in ({'all'},{'all','layoutId','literalTail'}),'selector fields');require(sel['all'],'nonempty AND')
            if 'layoutId' in sel:
                require(sel['layoutId'] in layouts and isinstance(sel['literalTail'],str),'selector layout/tail')
            for cond in sel['all']:
                require(cond.get('kind') in ('numeric_version','literal_segment','exact_id'),'condition kind')
                if cond['kind']=='numeric_version':
                    require(set(cond)=={'kind','components'} and cond['components'] and all(isinstance(x,str) and re.fullmatch('[0-9]+',x) for x in cond['components']),'numeric condition')
                    require('layoutId' in sel,'numeric condition needs declared slot')
                elif cond['kind']=='literal_segment':
                    require(set(cond)=={'kind','value','position','boundary'} and cond['value'] and cond['position'] in ('any','before_version','after_version') and cond['boundary']=='hyphen_or_string_edge','literal condition')
                else:require(set(cond)=={'kind','value'} and cond['value'],'exact condition')
            versions=[x['components'] for x in sel['all'] if x['kind']=='numeric_version']
            if 'layoutId' in sel:
                require(len(versions)==1,'single complete numeric slot')
                key=(layouts[sel['layoutId']]['literalPrefix'].lower(),tuple(versions[0]),sel['literalTail'].lower())
            else:key=('exact',tuple(x['value'].lower() for x in sel['all']))
            require(key not in keys,'equivalent duplicate selector');keys.add(key)
        require(seen==set(old),'all known overrides retained')
    require(ids==set(source),'17 baseline template identities retained')
    return True

def parse(layout,view,case=False):
    prefix=layout['literalPrefix'];p=prefix if case else prefix.lower();s=view if case else view.lower()
    if not s.startswith(p):return None
    body=s[len(p):];pieces=re.split(r'([.-])',body)
    if not pieces or not re.fullmatch('[0-9]+',pieces[0]):return None
    nums=[pieces[0]];seps=[];end=len(pieces[0]);i=1
    while i+1<len(pieces) and re.fullmatch('[0-9]+',pieces[i+1]):
        seps.append(pieces[i]);nums.append(pieces[i+1]);end+=1+len(pieces[i+1]);i+=2
    tail=body[end:]
    if len(nums)>=3 and seps[-1]=='-' and len(nums[-1])==8:
        try:datetime.datetime.strptime(nums[-1],'%Y%m%d')
        except ValueError:pass
        else:tail='-'+nums[-1]+tail;nums=nums[:-1];seps=seps[:-1]
    if tail.startswith('.'):return None
    if len(nums)>=3 and len(nums[-1]) in (4,6) and '-' in seps:return None
    return {'prefix':p,'components':nums,'tail':tail,'versionStart':len(p),'versionEnd':len(s)-len(tail)}

def pattern(p,model,case):
    if p['kind']=='ascii_numeric_prefix':
        v=p['value'] if case else p['value'].lower();m=model if case else model.lower()
        return m.startswith(v) and len(m)>len(v) and m[len(v)] in '0123456789'
    return baseline.literal(p,model,case)

def family_matches(cfg,provider,raw):
    view=raw;namespace=None
    ns=next((x for x in cfg['channelNamespaceCandidates'] if x['providerId']==provider),None)
    if ns:
        for p in sorted(ns['removeLeadingPrefixes'],key=len,reverse=True):
            if raw.lower().startswith(p.lower()):view=raw[len(p):];namespace=p;break
    hits=[]
    for t in cfg['templates']:
        r=t['content']['matching'];case=r['caseSensitive'];v=view if case else view.lower()
        if r['channelMode']=='whitelist' and provider not in r['channels']:continue
        if r['channelMode']=='blacklist' and provider in r['channels']:continue
        prefixes=[p['value'] if case else p['value'].lower() for p in r['include'] if p['kind']=='prefix']
        if any(v.startswith(p+p) for p in prefixes):continue
        if any(pattern(p,view,case) for p in r['exclude']):continue
        if any(pattern(p,view,case) for p in r['include']):hits.append(t)
    if hits:
        priority=max(t['content']['matching']['priority'] for t in hits)
        hits=[t for t in hits if t['content']['matching']['priority']==priority]
    require(len(hits)<=1,'highest priority conflict')
    return (hits[0] if hits else None),view,namespace

def selector_matches(t,o,view):
    c=t['content'];case=c['matching']['caseSensitive'];s=view if case else view.lower();sel=o['selector'];parsed=None
    if 'layoutId' in sel:
        layout=next(x for x in c['nameLayouts'] if x['id']==sel['layoutId']);parsed=parse(layout,view,case)
        expected_tail=sel['literalTail'] if case else sel['literalTail'].lower()
        if not parsed or parsed['tail']!=expected_tail:return False
    for cond in sel['all']:
        if cond['kind']=='numeric_version':
            if not parsed or parsed['components']!=cond['components']:return False
        elif cond['kind']=='exact_id':
            if s!=(cond['value'] if case else cond['value'].lower()):return False
        else:
            val=cond['value'] if case else cond['value'].lower()
            matches=list(re.finditer(r'(?:^|-)(?P<piece>'+re.escape(val)+r')(?=-|$)',s))
            if not matches:return False
            pos=cond['position']
            if pos!='any' and not parsed:return False
            if pos=='before_version' and not any(x.end('piece')<=parsed['versionStart'] for x in matches):return False
            if pos=='after_version' and not any(x.start('piece')>=parsed['versionEnd'] for x in matches):return False
    return True

def canonical(cfg,raw):
    t,view,_=family_matches(cfg,'other',raw)
    if not t:return None
    candidates=[p for layout in t['content']['nameLayouts'] if (p:=parse(layout,view,t['content']['matching']['caseSensitive']))]
    if len(candidates)!=1:return None
    p=candidates[0];return (t['templateId'],p['prefix'],tuple(p['components']),p['tail'])

def equivalent(cfg,a,b):
    x,y=canonical(cfg,a),canonical(cfg,b)
    return x is not None and x==y

def main():
    cfg=json.loads(CANDIDATE.read_text());seed=json.loads((BASE/'release-seed-rules.json').read_text());validate(cfg,seed)
    snapshot=json.loads((BASE/'model-id-snapshot.json').read_text());old=json.loads((BASE/'model-results.json').read_text());oldrows={(r['provider'],r['raw_model_id']):r for r in old}
    # Generic family rewrite also keeps every previously seeded family identity.
    source_include_checks=0
    for f in seed['families']:
        for pattern_item in f['content']['matching']['include']:
            t,_,_=family_matches(cfg,'other',pattern_item['value'])
            require(t is not None and t['templateId']==f['id'],'seed include identity lost')
            source_include_checks+=1
    replay=[]
    for p in snapshot['providers']:
        for model in p['model_ids']:
            t,view,ns=family_matches(cfg,p['provider'],model);family=t['familyKey'] if t else None
            require(family==oldrows[(p['provider'],model)]['candidate_naming_family'],'frozen family coverage drift')
            os=[o for o in t['content']['conditionOverrides'] if selector_matches(t,o,view)] if t else []
            require(len(os)<=1,'overlapping known capability overrides')
            replay.append({'provider':p['provider'],'rawModelId':model,'family':family,'matchingView':view,'namespace':ns,'knownOverrideId':os[0]['id'] if os else None,'unknownVariantAction':'inherit_common_only' if t and not os else None,'parametersSource':'pinned_release_only' if os else 'no_new_capability_inference'})
    preserved=[]
    for t in cfg['templates']:
        for o in t['content']['conditionOverrides']:
            hits=[x for x in t['content']['conditionOverrides'] if selector_matches(t,x,o['sourceExactId'])]
            require(len(hits)==1 and hits[0]['id']==o['id'],'source override no longer selectable')
            preserved.append({'templateId':t['templateId'],'overrideId':o['id'],'sourceExactId':o['sourceExactId'],'parametersPreserved':True})
    synthetics=[]
    for name,a,b,wanted in baseline.CASES:
        actual=equivalent(cfg,a,b);require(actual==wanted,'version boundary: '+name);synthetics.append({'case':name,'expected':wanted,'actual':actual})
    for p,model,wanted in baseline.FAMILY_NEGATIVES:
        actual=family_matches(cfg,p,model)[0];actual=actual['familyKey'] if actual else None
        require(actual==wanted,'family boundary: '+model);synthetics.append({'case':'family:'+p+':'+model,'expected':wanted,'actual':actual})
    # Known 6-sol parameters cannot spill into future versions/specs or dated aliases.
    gpt=next(t for t in cfg['templates'] if t['familyKey']=='GPT')
    for model in ['gpt-5.5-mini','gpt-99-sol','gpt-6-sol-mini','gpt-6-sol-20261106']:
        require(not any(selector_matches(gpt,o,model) for o in gpt['content']['conditionOverrides']),'known capability spillover')
    # Exercise literal positions against THIS JSON, rather than the prior sketch.
    literal_cases=[('Claude','claude-opus-5-5',['5','5'],'opus','any',True),
      ('Claude','claude-opus-5-5',['5','5'],'opus','before_version',True),
      ('Claude','claude-opus-5-5',['5','5'],'opus','after_version',False),
      ('GPT','gpt-6-sol',['6'],'sol','any',True),
      ('GPT','gpt-6-sol',['6'],'sol','after_version',True),
      ('GPT','gpt-6-sol',['6'],'sol','before_version',False),
      ('GPT','gpt-6-solar',['6'],'sol','any',False),
      ('Claude','claude-opus-5-5',['5','5'],'oups','any',False)]
    for family,raw,version,word,position,wanted in literal_cases:
        t=next(t for t in cfg['templates'] if t['familyKey']==family)
        parsed=[(l,p) for l in t['content']['nameLayouts'] if (p:=parse(l,raw))]
        require(len(parsed)==1,'literal fixture layout')
        layout,p=parsed[0]
        o={'selector':{'layoutId':layout['id'],'literalTail':p['tail'],'all':[{'kind':'numeric_version','components':version},{'kind':'literal_segment','value':word,'position':position,'boundary':'hyphen_or_string_edge'}]}}
        require(selector_matches(t,o,raw)==wanted,'literal position/segment fixture')
    malformed=[('candidate_activated',lambda c:c.update(stage='active')),
      ('missing_template',lambda c:c['templates'].pop()),
      ('guessed_common_parameter',lambda c:c['templates'][0]['content']['commonRules'].update(contextWindow={'mode':'custom','value':999999})),
      ('unknown_numeric_separator',lambda c:c['templates'][0]['content']['nameLayouts'][0].update(numericJoiners=['.','-','_'])),
      ('equivalent_duplicate',lambda c:c['templates'][0]['content']['conditionOverrides'][1].update(selector=copy.deepcopy(c['templates'][0]['content']['conditionOverrides'][0]['selector']))),
      ('missing_source_override',lambda c:c['templates'][0]['content']['conditionOverrides'].pop())]
    rejected=[]
    for name,mutate in malformed:
        c=copy.deepcopy(cfg);mutate(c)
        try:validate(c,seed)
        except ValueError as e:rejected.append({'case':name,'rejected':True,'reason':str(e)})
        else:raise ValueError('invalid candidate accepted: '+name)
    summary={'status':'passed_candidate_config_only','candidateSha256':hashlib.sha256(CANDIDATE.read_bytes()).hexdigest(),'format':cfg['format'],'targetVersion':cfg['targetVersion'],'productionReaderSupport':cfg['productionReaderSupport'],'templates':len(cfg['templates']),'generalAndRetainedExactIncludeRules':sum(len(t['content']['matching']['include']) for t in cfg['templates']),'sourceExactIncludeRules':sum(len(f['content']['matching']['include']) for f in seed['families']),'allOriginalSeedIdsRetained':source_include_checks,'knownParameterOverridesPreserved':len(preserved),'structuredOverrides':sum('layoutId' in o['selector'] for t in cfg['templates'] for o in t['content']['conditionOverrides']),'frozenLocalRows':len(replay),'uniqueModelsMatched':len({r['rawModelId'] for r in replay if r['family']}),'localRowAssignmentsSameAsEarlierAnalysis':True,'knownOverridesMatchedOnFrozenRows':sum(bool(r['knownOverrideId']) for r in replay),'syntheticVersionAndFamilyBoundaries':len(synthetics),'literalPositionAndBoundaryCases':len(literal_cases),'invalidConfigsRejected':len(rejected),'unknownCapabilitySpilloverNegatives':4,'validationMethod':'stdlib strict fields + pinned source equality + data-driven candidate interpreter; not an installed JSON-Schema/production reader','dailyConfigReadsOrWrites':0,'networkRequests':0,'scope':'Prepared file and offline checks only; prototype/Schema adoption/product integration still pending.'}
    (BASE/'candidate-config-validation.json').write_text(json.dumps({'summary':summary,'preservedParameters':preserved,'syntheticBoundaries':synthetics,'rejectedMalformedCandidates':rejected},ensure_ascii=False,indent=2)+'\n')
    (BASE/'candidate-config-model-results.json').write_text(json.dumps(replay,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(summary,ensure_ascii=False,indent=2))

if __name__=='__main__':main()
