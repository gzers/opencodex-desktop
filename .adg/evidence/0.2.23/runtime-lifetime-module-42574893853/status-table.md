# 候选生命周期轮完整结果（未验收）

整项失败，accepted:false。六case、原子归档及清理通过；严格Clippy失败。

|检查|结果|毫秒/首行错误|
|---|---|---|
|leasePrototype|pass|153.79|
|detachedLeasePrototype|pass|200.857|
|selfJobLeasePrototype|pass|152.555|
|detachedSelfJobLeasePrototype|pass|192.807|
|candidateLifetimePrototype|pass|153.465|
|detachedCandidateLifetimePrototype|pass|194.789|
|candidate archive/actual atomic entry/failure preservation|pass||
|strict Clippy|fail|unreachable pattern: redundant exhaustive-match fallback|
|fixture and staging cleanup|pass||
