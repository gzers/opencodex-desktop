"""Follow-up: literal conditions are complete name segments, not suffix-only.
Offline naming experiment; reuses frozen model projection. No new source reads.
"""
from pathlib import Path
import importlib.util,json,re
BASE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('match_candidate',BASE/'evaluate.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

def condition(provider,raw,family,version,segment,position='any'):
    f,view,ns,reason=m.family(provider,raw)
    if f!=family:return False
    parsed=m.parse_version(view)
    if not parsed or parsed['ambiguous_numeric_suffix']:return False
    if parsed['components']!=version.split('.'):return False
    # Candidate literal boundary deliberately limited to '-' (not regex substring).
    occurrences=list(re.finditer(r'(?:^|-)(?P<piece>'+re.escape(segment.lower())+r')(?=-|$)',view))
    start=len(parsed['prefix']);end=len(view)-len(parsed['tail'])
    return any(position=='any' or position=='before_version' and x.end('piece')<=start
               or position=='after_version' and x.start('piece')>=end for x in occurrences)

cases=[
 ('claude_literal_before','other','claude-opus-5-5','Claude','5.5','opus','any',True),
 ('claude_dotted','other','claude-opus-5.5','Claude','5.5','opus','any',True),
 ('claude_position_before','other','claude-opus-5-5','Claude','5.5','opus','before_version',True),
 ('claude_not_suffix','other','claude-opus-5-5','Claude','5.5','opus','after_version',False),
 ('gpt_literal_after','other','gpt-6-sol','GPT','6','sol','any',True),
 ('gpt_position_after','other','gpt-6-sol','GPT','6','sol','after_version',True),
 ('gpt_not_prefix','other','gpt-6-sol','GPT','6','sol','before_version',False),
 ('version_dot_dash','other','gpt-6-1-sol','GPT','6.1','sol','any',True),
 ('inside_name_not_fragment','other','gpt-6-solar','GPT','6','sol','any',False),
 ('series_boundary','other','claude-opusplus-5-5','Claude','5.5','opus','any',False),
 ('version_boundary','other','claude-opus-5-50','Claude','5.5','opus','any',False),
 ('wrong_family','other','gpt-5.5-opus','Claude','5.5','opus','any',False),
 ('no_automatic_spelling_fix','other','claude-opus-5-5','Claude','5.5','oups','any',False),
 ('date_kept','other','claude-opus-5-5-20261106','Claude','5.5','opus','before_version',True),
 ('series_and_version_are_and','other','claude-sonnet-5-5','Claude','5.5','opus','any',False),
 ('no_implicit_reorder','other','claude-5-5-opus','Claude','5.5','opus','any',False)
]
results=[]
for name,p,raw,f,v,l,pos,expected in cases:
    actual=condition(p,raw,f,v,l,pos)
    results.append({'case':name,'raw_model_id':raw,'family':f,'numeric_version':v,'literal_segment':l,'position':pos,'expected':expected,'actual':actual})
assert all(r['expected']==r['actual'] for r in results)
snapshot=json.loads((BASE/'model-id-snapshot.json').read_text())
queries=[('Claude','5.5','opus','before_version'),('GPT','6','sol','after_version'),('GPT','6.1','sol','after_version')]
actual_samples=[]
for f,v,l,pos in queries:
    rows=[{'provider':p['provider'],'raw_model_id':model} for p in snapshot['providers'] for model in p['model_ids'] if condition(p['provider'],model,f,v,l,pos)]
    actual_samples.append({'rule':{'family':f,'version':v,'literal_segment':l,'position':pos},'rows':rows,'unique_ids':sorted({r['raw_model_id'] for r in rows})})
assert all(r['rows'] for r in actual_samples)
report={'status':'passed-limited-naming-experiment','synthetic_cases':len(results),'synthetic_passed':sum(r['expected']==r['actual'] for r in results),'cases':results,'real_saved_examples':actual_samples,'limits':['Literal conditions support any/before-version/after-version within declared family syntax; NOT global suffix-only.','Any-position predicate does not reorder names or declare all token permutations identical.','Current illustrative grammar retains known family layout; new layouts require explicit syntax declarations.','Literal token boundary here is explicitly hyphen; dot/hyphen equivalence remains limited to numeric version slots.','No automatic spelling correction or supplier/capability identity assertions.','Production and prototype unchanged; previous analysis files preserved.']}
(BASE/'literal-segments-review.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':report['status'],'synthetic_passed':report['synthetic_passed'],'real_examples':[{'rule':r['rule'],'rows':len(r['rows']),'unique_ids':r['unique_ids']} for r in actual_samples]},ensure_ascii=False,indent=2))
