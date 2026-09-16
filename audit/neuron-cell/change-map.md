# Карта изменений по исходным файлам

Наблюдения за локальным checkout на 2026-09-12. Колонка «план» — назначенная
работа, а не сообщение о её выполнении. Пакеты P00–P13 описаны в
[roadmap](../../roadmap/neuron-cell-convergence.md); полный список всех repositories,
включая KEEP и reference, — в [inventory](repositories.md).

Пути ниже относительно `~/cyber`. Группы `*.rs`/`specs/*` означают область
изменения, а не утверждение, что каждый файл в ней содержит CellId.
Полный список лексических кандидатов с hashes и первыми номерами строк находится
в [matches.jsonl](matches.jsonl). Список кандидатов не является скриптом замен.

## Основные находки

| ID | Проверенное наблюдение | Следствие для плана |
|---|---|---|
| F01 | [soft3 terms](../../specs/terms.md) уже определяет neuron как actor, запускающий progs, и отдельно prog | Не нужен дополнительный cell subject; P01 закрепляет существующую идею |
| F02 | В том же словаре два identity rules: H(secret) и H(pubkey); [mudra claim](../../../mudra/src/claim.rs) содержит neuron_of(pubkey) | P01/P03: profiles и vectors до новых writers; не менять существующие ключи |
| F03 | [engine create](../../../cell/engine/src/engine.rs) строит birth particle и использует его как namespace/head; Rust API часто принимает Particle, а не отдельный тип CellId | Это семантическая миграция identity, не только поиск токена CellId |
| F04 | [records](../../../cell/engine/src/records.rs): local-private-database-owner, карты max 1 и replacement при write_live; engine возвращает Busy | P05 меняет representation и admission; P06 добавляет supported authority binding |
| F05 | [schema suite](../../../cell/model/schema-suite-v1.txt) — immutable hash input | P07 оставляет оригинальные bytes и reader; новый текст suite означает новую identity |
| F06 | [cyb core Cell](../../../cyb/core/src/cell.rs) содержит graph с chains нескольких neurons; commit игнорирует результат log.write/flush | P04: graph/session role и корректная durability. Не переименовывать aggregate в Neuron |
| F07 | [копия cyb Cell](../../../cyb/crates/cyb/src/cell.rs) сосуществует с core | Сверить consumers, выбрать единую реализацию; удалить duplicate только после migration callers |
| F08 | [cyb identity](../../../cyb/shell/src/worlds/identity.rs) — shared identity resource; [SharedCell](../../../cyb/shell/src/worlds/mod.rs) проходит через worlds | P08: registry и явный subject у action; смена UI не меняет pending effect |
| F09 | [cybergraph application](../../../cybergraph/src/application.rs) уже использует generic namespace/Proposal; [bbg application storage](../../../bbg/rs/src/storage/application.rs) предоставляет общие транзакции | Не добавлять neuron-specific database; migrate records поверх общих ports |
| F10 | [cell node](../../../cell/node/src/lib.rs) уже использует общий Database; redb доступен отдельной migration feature | Сохранить новую BBG/Fjall интеграцию, не вернуть устаревший storage дизайн |
| F11 | [cyber cell](../../../cyber/cell.md), hierarchy и node-modes смешивают runtime, service, ledger, shard и local-node roles | P11 разбирает роли, сохраняя capabilities; область графа не становится ключом |
| F12 | [soma-spec](../../../soma/soma-spec.md) описывает Soul как root Neuron и neuron как cognitive worker | P09 адаптирует к cyb meanings, task/prog decomposition и named robot |
| F13 | [prysm launcher](../../../prysm/molecules/specs/launcher.md) и chroma передают cell_id в UI | P08 вводит typed destination; вкладка или app page не обязана быть neuron |
| F14 | [Rs Cell declarations](../../../rs/reference/cells.md), [core traits](../../../rs/core/src/core_types.rs), [macro codegen](../../../rs/macros/src/cell/codegen.rs) реализуют bounded OS modules | P05/P11: module naming и adapter contract. Это не std::cell и не второй neuron |
| F15 | [ctx build](../../../ctx/build.nu) собирает ctx.md из cyber graph и pinned paths | P12: inputs и rebuild; ctx не authority для переопределения свежего cyb |
| F16 | После исходного scan появились [native storage contract](../../../cybergraph/specs/native-storage.md) и [node adapter contract](../../specs/native-node.md); в cybergraph добавлена module declaration | P04 должен переиспользовать coordinator/importer по мере готовности, а не создавать дубль. Реализация этим аудитом не проверена |

## Runtime и identity foundations

| Repository / точные точки входа | План | Проверка |
|---|---|---|
| `cell/Cargo.toml`, `model/`, `engine/`, `rune/`, `node/`, `cli/`, lockfile | P03: neuron packages и minimal ID crate, путь repository — P12 | Cargo graph, no runtime для identity-only client; не форматировать sibling repos через fmt --all |
| `cell/model/src/{data,records,lib}.rs`, `schema-suite-v1.txt` | P02/P03/P07: new subject/prog/invocation records и bounded legacy reader | Старые schema/code/checkpoint hashes неизменны; new suite rejects malformed records |
| `cell/engine/src/{lib,engine,records,execution}.rs` | P05/P06: maps вместо одного Live; prog state conflicts; subject-bound effects | Fan-out, cancel, restart, unknown outcome, reservation settlement, grant revocation |
| `cell/rune/src/lib.rs`, `rune/rs/interp/event.rs`, `rune/specs/checkpoints.md`, `rune/rs/interp/tests/authority.rs` | P05: runtime adapter и run_prog, pinned ABI | Сохранённое продолжение возобновляется тем же codec; bounds enforced; authority не из VM data |
| `cell/node/src/lib.rs`, `node/tests/{runtime,recovery}.rs` | P06/P07: Graph/Ward ports, supported writer profile, importer | CommitUnknown разрешается проверкой записи; duplicate admission/result не создаёт новый effect |
| `cell/cli/src/main.rs`, `cli/tests/{process,storage}.rs` | P07/P08: explicit subject/network, inspect/migrate, new binary/help | Старый store inspect/read; no dispatch в migrate; restartable staged activation |
| `cell/specs/*`, `docs/{cell-convergence,implementation}.md`, README/CLAUDE | P02: полный контракт; P12: путь, install и contributor instructions | Таблица старое→новое и guarantees/local-profile status; historical audit evidence сохранить |
| `bbg/rs/src/types.rs`, `rs/src/storage/{application,mod}.rs`, Cargo.toml | P03/P04: общий NeuronId и shared transactional adapter | Нет bbg→engine зависимости; byte layout, committed heads/claims/history сохраняются |
| `cybergraph/src/{application,content,lib}.rs`, Cargo.toml и storage specs/tests | P04: graph/session service, durable import старого signal log | Ошибки хранения не маскируются; multi-neuron state и protocol replay согласованы |
| `cybergraph/specs/native-storage.md`, будущая/текущая реализация `src/native`, `soft3/specs/native-node.md` | P04: сверить параллельно появившийся native coordinator с application runtime и reuse ports | Общая durable boundary; local unsigned acceptance не становится production neuron authorization |
| `foculus/src/` signal/chain/wire, Cargo.toml, specs | P03/P06: ID import и writer coordination boundary | Per-neuron chain, Signal.network и signature bytes не становятся runtime commit format |
| `mudra/src/{claim,domain,seed,lib}.rs`, `tests/vectors.rs`, specs identity/bridge | P01/P03/P06: identity profile matrix, common ID re-export при необходимости | Existing bridge/domain keys и claims побайтно прежние; rotation только по профилю |
| `nox`, `hemera`, `lens`, `eidos`, `trident`, `zheng`, `zheng-pin` APIs/specs | P02/P05/P13: pin необходимого codec/runtime/proof profile | Без hash fork; mathematical/VM cell → pair работа остаётся отдельной |
| `joy/rs/formula.rs`, `joy/specs/cli.md` | P02/P05: согласовать data/checkpoint boundary при интеграции | Substrate cells/private positions не subject; proofs и CLI semantics прежние |
| `rs/core/src/{core_types,lib,runtime}.rs`, `rs/macros/src/{lib.rs,cell/}`, `rs/rsc/src/lints/` | P05: cell! / CellMetadata → module! / ModuleMetadata, generated interface и diagnostics | Compile/codegen/migration fixtures и сохранение metadata; реальное deadline enforcement проверять у executor |
| `rs/reference/{cells,async,restrictions}.md`, `rs/docs/tutorials/cyb-cell.md`, README и tests | P11: описать module/prog/worker boundary, paths и examples | Не создавать identity для OS module; temporary alias имеет границу удаления |

## Robot, UI и агент

| Repository / точки входа | План | Проверка |
|---|---|---|
| `cyb/specs/{architecture,neuron,README}.md`, anatomy, product/robot, parts/cells | P01/P08: supersede двухуровневую proposal, согласовать 22 функции | Никакого отдельного actor CellId; prog/page/data IDs сохранены по смыслу |
| `cyb/core/src/{cell,lib,money,signal,sense,wire}.rs`, соответствующие `crates/cyb/src/` | P04: общая graph session и устранение duplicate | Multi-neuron logs/imports, money caller, protocol order; legacy reader без dual write |
| `cyb/shell/src/worlds/mod.rs`, `identity.rs`, `sigma/`, `vault/` | P06/P08: subject registry, binding, scoped signing | Watch-only, foreign identity, key/network/device selection, no ambient author |
| Worlds `com/`, `body/{mod,chainsync,relay}.rs`, `soma_bridge.rs`, `attention.rs`, `graph.rs`, `snapshot.rs`, `viewer.rs`, shell lib | P08: SharedCell callers → graph handle + explicit context | Pending requests сохраняют автора, сеть, now и grant после UI changes |
| `cyb/shell/src/worlds/robot/mod.rs`, `cyb/cells/landing.rune`, `.claude/plans/live-cell-runtime.md` | P08/P12: typed navigation и prog/page assets; mark old plan superseded | Landing/view не заводит neuron; redirects старых cell:// не исполняют непроверенный код |
| `cyb/parts/{ward,vault,body,soul,sigma,log,state,plan,time,sense,now,com,memory,brain,soma,radio,fs,vision,voice,name,avatar}.md` | P01/P08/P09: responsibilities по матрице roadmap | У каждой функции owner и auth/context/history contract; не переносить все в neuron |
| `cyb/cli/{Cargo.toml,src/main.rs}`, `neural/rs/cli/{Cargo.toml,src/main.rs}`, neural README/specs | P08: terminal actions через общий API с явным subject | CLI и GUI используют один graph/identity contract |
| `prysm/Cargo.toml`, `rs/`, atoms/specs/neuron, molecules/specs/neuron-card | P08: adapter к graph API и identity display | Bech32 presentation отдельно от native ID, без secret/runtime dependency в UI atom |
| `prysm/molecules/specs/{launcher,stars}.md`, `chroma/specs/{com,space}.md` | P08: CellList/ActiveCell/ContextSubject → typed references | Pins/context/navigation различают neuron, prog, view, particle |
| `soma/{CLAUDE,README,soma-spec}.md`, `kernel/src/lib.rs`, research/agent-research, hermes-learning-loop | P09: root-neuron ontology убрать; task strategy/learning оставить | Parent/child lineage, budget joins, steering/cancel и recovery; historical research помечен по статусу |
| `glia` inference adapters, `cyb/shell/src/worlds/models/` и brain/body specs | P06/P09: pinned model/provider context и bounded worker requests | Model failover/streaming/result не сбрасывают identity, grant или budget |
| `cell/specs/{agent,evaluation,conformance}.md` → neuron profile | P09: сохранить полный agent contract и назначить owners | Requirements→scenario→evidence, отдельно от обещания Hermes parity |

## Действующие потребители и интеграции

| Repository / точки входа | План | Что сохраняется |
|---|---|---|
| `lytics/rs/event/Cargo.toml`, event/domain signing и ingestion tests | P03/P10: narrow ID/auth interface, golden vectors | Domain derivation, signed payloads, маленькая dependency tree; без deployment этой работой |
| `cyberia-my/{Cargo.toml,src/signal.rs}`, `src/{world,portal,erp,economy,elements}.rs`; те же пути в `cyberia/research/cyberia-my` | P10: проверить оба checkout, canonical source и interface | DOMAIN/HRP, signing body, storage sequence, verification; не переписывать local history |
| `trisha/cli/{neuron,neptune,mine}.rs`, `cli/tests/neptune_wallet.rs` | P10: identity-domain adapter только где подключается новый model | Existing wallet/network/key lifecycle; no synthetic native ID for Neptune address |
| `inf/rs/value/src/lib.rs`, BbgSource, specs relations/language, docs queries | P03/P10: типы и projections neuron/prog/task | Нативные Neuron values и query scope; никакого runtime engine внутри query language |
| `radio` own adapters/specs, `tape/spec/`, `tape/impl/rust/src/molecule.rs`, `fs/{patch,sync}.md`, patch/spec | P06/P10: typed subject/context в own dialect/adapters | Transport identity, framing, foreign Cid, patch chain, signatures |
| `tok`, `tru` neuron-accounted state/contracts/tests | P03/P06/P13: regression по затронутым ID imports и grants | Экономические законы и протокольная бухгалтерия |
| `soft3/crate/src/{lib,node}.rs`, CLI/schema/MCP/py scaffolds, `true-cyber` facade | P10/P12: новый facade API и честный capability/status manifest | Node execution-model, verifier boundary; scaffold не объявлять готовым SDK |
| `soft3/js` sdk source/tests, `go-cyber`, `space-pussy`, `bostrom`, `bootloader` | P10: bridge compatibility fixtures | Cosmos account types, protobuf/JSON, deployed identities, snapshot provenance |
| `cybernet/cw/src/{uids,neuron_info,weights,root}.rs`, contracts tests | P10: foreign identity binding при integration | hotkey/coldkey/netuid/UID semantics; contract schema не мигрируется от слова neuron |
| `hub/contracts/hub-skills/src/{msg,state,execute,query}.rs`, `portal/packages/cyber-std`, `midao` | P10/P11: registry/collective authority contract references | Existing service endpoints, contract addresses/storage и governance |

### Проверенные прямые Cargo consumers

Дополнительный `rg` проход по manifests, включая hidden sources и исключая
target/build, нашёл следующие непосредственные границы:

| API / package | Consumers в текущем checkout |
|---|---|
| `cell-model`, `cell-engine`, `cell-node`, `cell-rune` | Только crates внутри `cell` workspace |
| `cyb-core` | `cyb/cli`, `cyb/shell`, `prysm`, `neural/rs/cli` |
| `cyber-mudra` | `cyb/cli`, `cyb/shell`, `lytics/rs/event`, оба checkout cyberia-my |

Это direct manifest edges, не полный транзитивный Cargo graph и не доказательство
отсутствия будущих consumers. Поэтому большая часть риска сейчас в схемах,
архитектуре и старом cyb graph API; новый cell engine ещё можно объединить до
широкого распространения независимого публичного API.

## Онтология, node profiles и производные документы

| Repository / точки входа | План |
|---|---|
| `soft3/specs/{terms,execution-model,README}.md`; roadmap terms-map/migration/stack-completeness; README/site | P01: убрать противоречивый KEEP domain cell, добавить neuron/prog contract рядом с execution; P12: release/navigation |
| `cyber/{cell,hierarchy,3c,network,whitepaper}.md`, research/oikos и spectral cell division | P11: runtimes→neurons/progs, ledgers→books/issuer, knowledge→shards; сохранить governance/proofs/split semantics |
| `cyber/specs/{component-ownership,node-modes,node-product,cyb-node,worker,README}.md`, CLI specs, roadmap a-local-node/b-computation/c-network | P11: full/partial/light, graph session vs subject; актуализировать capability tables и verification duties |
| `aos/{README,apps,map}.md`, portal/oracle/hub/temple/cyberver/senate и остальные building pages | P11: service/prog, own actor только при нужной authority; playground/building не обязаны иметь signing ID |
| `crystal/{neuron,prog,soul,avatar,node,component}.md` где файл существует, и ссылки в knowledge graph | P11: canonical vocabulary; отсутствующий canonical prog page добавить у владельца словаря, существующие определения согласовать с cyb |
| `cyberia/foundation/{architecture,vision,whitepaper}.md`, protocol/{README,system,marketplace,marketplace-spec,space-accounting}.md | P11: роль neuron в governance и роль shard/service в пространстве; таблицы/Cancelled не трогать |
| `cybics/comp/`, info/file, crypto/data-structures и software ontology references | P11: только определения software subjects; biology/chemistry/memory cells KEEP |
| `cyber-valley/buildings/satoshi/{soul,mind,link}.md` | P11: software ontology links без переименования биологических нейронов |
| `ctx/build.nu`, `ctx/ctx.md` | P12: пересобрать по актуальным sources, проверить устаревшие pinned paths, сохранить generator/source manifest |
| `optica/src/graph/mod.rs` и output/link resolution; `cyberia-blog/subgraphs/`, source build configuration | P12: aliases neuron/cell/prog, old URLs и generated graph, без переписывания old blog facts |
| `cybernode/servers/cyberproxy/lytics.md`, current deployment docs | P12: storage/CLI names и supported revisions; deployment вне этого плана |
| `soft3/conformance/` и standalone `conformance` | P13: общий manifest и реальные owner scenarios, не два конкурирующих harness |

Для файлов без целевых изменений решение зафиксировано поимённо в inventory.
Это включает все nested upstream repositories, parallel worktrees, non-Git copies
и шесть generated `.git` markers. Они не должны превращаться в массовый rewrite.
