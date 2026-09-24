# Репозитории: полная матрица решений

Snapshot: 2026-09-12. [Метод и ограничения](README.md), [план](../../roadmap/neuron-cell-convergence.md),
[конкретные файлы](change-map.md). Отдельная строка для каждого обнаруженного Git root/маркера
и каждой группы non-Git исходников. HEAD/dirty отражают скан, не текущее состояние.

`C/N` — число файлов с широкими лексическими совпадениями cell/neuron, **не число
необходимых правок**. Включают std types, biology, cancelled и прочие омонимы.
`F` — прочитанные текстовые файлы; для sparse checkout включены доступные index blobs.
`*` после HEAD означает dirty; `—` означает недоступные Git metadata, а не чистый Git.
Все пути относительно `~/cyber`. Подробные SHA, counts, errors — в [scan.json](scan.json).

## Git repositories и worktrees (110)

| Repository / source tree | Kind · HEAD | F · C/N | Решение · пакеты | Что делать |
|---|---|---|---|---|
| [.github](../../../.github) | repository · `e0c5c62964` | 4 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [aos](../../../aos) | repository · `9ea98613ba` | 18 · 15/12 | documentation · P11 | Building/runnable cells в README/apps и всех service pages → service/prog и явная governance; не создавать ключ каждой площадке. |
| [arc](../../../arc) | repository · `7f10855296` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [bbg](../../../bbg) | repository · `bbf24f14a1*` | 156 · 24/42 | implementation · P03 P04 P07 | Реэкспорт общего ID без runtime dependency; shared Database transactions и migration staging; DAS/unimem cells сохранить. |
| [bel](../../../bel) | repository · `9668806709` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [bita](../../../bita) | repository · `cff2b0af4d` | 223 · 9/0 | keep · — | Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически. |
| [bootloader](../../../bootloader) | repository · `d7e3ac029b` | 18 · 0/5 | legacy-compatibility · P10 | Исторические chain identities, account keys, protobuf/JSON и snapshot data сохранить; миграция только через explicit bridge/adapters. |
| [bostrom](../../../bostrom) | repository · `8301165ac0*` | 88 · 1/28 | legacy-compatibility · P10 | Исторические chain identities, account keys, protobuf/JSON и snapshot data сохранить; миграция только через explicit bridge/adapters. |
| [bt](../../../bt) | repository · `7d52d029be` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [cell](../../../cell) | repository · `34bbff9dbf` | 55 · 50/9 | implementation · P02 P03 P05 P06 P07 P12 | Эволюция в neuron: ID/model/progs, engine concurrency, authority, legacy migration; сохранить Git и immutable suite. |
| [cloud-forest](../../../cloud-forest) | repository · `5e2c0256a5` | 54 · 3/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [conformance](../../../conformance) | repository · `ff726be545` | 2 · 0/0 | integration · P00 P13 | Самостоятельный scaffold: согласовать owner harness с soft3/conformance; новые migration/identity scenarios без фиктивного PASS. |
| [crystal](../../../crystal) | repository · `a23781cce1` | 553 · 24/157 | documentation · P01 P11 | Canonical neuron/prog/soul/avatar/node vocabulary и ссылки; протокольные neuron statements сохранить, исправить конфликт поведения/soul. |
| [ctx](../../../ctx) | repository · `688d859ffb` | 4 · 1/2 | generated-source · P11 P12 | ctx.md — generated context. Обновить canonical inputs/pinned paths в build.nu, затем rebuild с provenance; не править bundle вручную. |
| [cve](../../../cve) | repository · `727eec0074` | 356 · 10/3 | keep · — | Нет runtime CellId/identity API кандидатов; UI/dependency/общие слова сохранить. Legacy checkout не синхронизировать автоматически. |
| `cve-legacy-local` (путь отсутствует после snapshot) | repository · `4eb90e5394` | 318 · 9/2 | keep · — | Нет runtime CellId/identity API кандидатов; UI/dependency/общие слова сохранить. Legacy checkout не синхронизировать автоматически. |
| `cve-legacy-local/docs` (путь отсутствует после snapshot) | repository · `6e963d6fd5` | 5 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [cyb](../../../cyb) | repository · `1b62676ce0*` | 218 · 62/65 | implementation · P01 P04 P06 P08 P09 | Одна subject model; Cell/SharedCell → graph session; identity registry, explicit author, typed navigation; удалить дублирование core. |
| [cyber](../../../cyber) | repository · `cb603db4ba` | 231 · 42/117 | documentation · P01 P11 P12 | Cell ladder, hierarchy, 3c/oikos, node modes/product/ownership/CLI, roadmaps; shard/partial/service роли развести, generated copies перестроить. |
| [cyber-valley](../../../cyber-valley) | repository · `79b5849d8f` | 102 · 11/15 | documentation · P11 | Satoshi soul/agent docs: обновить software ontology links; физические/образовательные и biological cells сохранить. |
| [cybergraph](../../../cybergraph) | repository · `0213210e46` | 52 · 9/34 | implementation · P03 P04 P07 | Общий NeuronId; graph/session durability и imports; namespace остаётся адресом данных; согласовать claims/CAS. |
| [cyberia](../../../cyberia) | repository · `bacb7a378b` | 178 · 18/14 | documentation · P11 | Foundation архитектура и protocol system/marketplace: service/shard governance; участник-neuron сохраняется; таблицы и биология KEEP. |
| [cyberia/research/cyberia-my](../../../cyberia/research/cyberia-my) | repository · `b33a7e75c2` | 40 · 6/11 | compatibility · P03 P10 | Оба checkout: src/signal.rs domain key и ADR-036-style body, local storage/sequence; подтвердить один canonical source и сохранить wire. |
| [cyberia/research/events](../../../cyberia/research/events) | repository · `7e394be928` | 680 · 29/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [cyberia/research/mimi](../../../cyberia/research/mimi) | repository · `1a3eec42b5` | 77 · 1/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [cyberia-blog](../../../cyberia-blog) | repository · `49f044a349` | 105 · 3/11 | publication · P11 P12 | Исторические blog posts не переписывать; обновить current subgraph links и regenerate build из canonical sources. |
| [cyberia-capital](../../../cyberia-capital) | repository · `ba7a4f7789` | 3 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [cyberia-my](../../../cyberia-my) | repository · `04361c0d7c` | 40 · 6/11 | compatibility · P03 P10 | Оба checkout: src/signal.rs domain key и ADR-036-style body, local storage/sequence; подтвердить один canonical source и сохранить wire. |
| [cyberia-to](../../../cyberia-to) | repository · `d587b7d016` | 3 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [cybernet](../../../cybernet) | repository · `e7ba0351f9` | 255 · 11/86 | legacy-compatibility · P10 | CosmWasm neuron_uid/hotkey/coldkey/netuid — native protocol model; не заменять локальный UID на cyber NeuronId и не мигрировать contracts этим merge. |
| [cybernode](../../../cybernode) | repository · `d689247885` | 165 · 5/3 | documentation · P10 P12 | Deployment/lytics storage docs и команды сверить с release; chain endpoints сохранить; сервера этой работой не менять. |
| [cyberstates](../../../cyberstates) | repository · `fa4ec20d21` | 342 · 7/0 | keep · — | cell_id в pages/map.rs — UI карта/DOM; совпадение не относится к runtime identity. |
| [cybervalley-io](../../../cybervalley-io) | repository · `61592d341f` | 3 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [cybics](../../../cybics) | repository · `e39086850f` | 558 · 90/82 | documentation · P11 | Только software ontology references к архитектуре; biological/memory/data-structure cells и математические analogies сохраняются. |
| [cybland](../../../cybland) | repository · `b1a212fe53` | 1 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [dif](../../../dif) | repository · `75673ab9a3` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [eidos](../../../eidos) | repository · `42d2211d94` | 69 · 2/2 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [episode-1](../../../episode-1) | repository · `db69b1bdc2` | 34 · 2/1 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [erga](../../../erga) | repository · `848e17e252` | 74 · 5/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [evy](../../../evy) | repository · `c29fed9e29` | 899 · 90/4 | keep · — | Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически. |
| [foculus](../../../foculus) | repository · `b96f4d9bbe*` | 70 · 9/40 | compatibility · P03 P04 P06 | Согласовать ID imports и writer/finality contract; сохранить signal bytes, per-neuron chain и отдельный network. |
| [fs](../../../fs) | repository · `370182b8ae` | 5 · 0/3 | documentation · P01 P10 | Patch author/neuron namespace и scoped program access; per-neuron patch chain не смешивать с runtime commit index. |
| [glia](../../../glia) | repository · `ef2e4c007c*` | 157 · 2/3 | integration · P06 P09 | Inference adapter получает bounded jobs/context; нейросетевые neurons и tensor layout не переименовывать. |
| [go-cyber](../../../go-cyber) | repository · `cdf00d88bf` | 1329 · 19/109 | legacy-compatibility · P10 | Исторические chain identities, account keys, protobuf/JSON и snapshot data сохранить; миграция только через explicit bridge/adapters. |
| [hemera](../../../hemera) | repository · `ef90d20361*` | 131 · 8/6 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [honeycrisp](../../../honeycrisp) | repository · `9a32c93bdd*` | 307 · 16/0 | keep · — | Unimem cell_id_distinct — память/allocator, не субъект; самостоятельные worktrees учтены, не массово переписывать. |
| [honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a](../../../honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a) | worktree · `52feff7855*` | 248 · 14/0 | keep · — | Unimem cell_id_distinct — память/allocator, не субъект; самостоятельные worktrees учтены, не массово переписывать. |
| [honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291](../../../honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291) | worktree · `41766033fa*` | 266 · 14/0 | keep · — | Unimem cell_id_distinct — память/allocator, не субъект; самостоятельные worktrees учтены, не массово переписывать. |
| [hub](../../../hub) | repository · `88b8e7cd22` | 109 · 0/8 | legacy-compatibility · P10 | hub-skills хранит neuron/network/protocol/endpoint: сохранить contract storage/wire; bridge должен квалифицировать, а не переименовать поля. |
| [inf](../../../inf) | repository · `b0e4506eb7` | 64 · 5/25 | compatibility · P03 P04 P10 | Neuron value/query scoping и BbgSource: subject не подменять произвольным Particle; добавить prog/task projections у владельцев schemas. |
| [inf/rs/cozo](../../../inf/rs/cozo) | repository · `481af058ab` | 228 · 7/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [joy](../../../joy) | repository · `d065814258*` | 41 · 8/0 | compatibility · P02 P05 | Cells в formula/state — substrate pairs и private-state positions; сохранить bytes/proofs и согласовать с terms-map, не переименовать в neuron. |
| [kadek](../../../kadek) | repository · `162ad62f18` | 33 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [lens](../../../lens) | repository · `9b134c4cc9*` | 53 · 2/0 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [li](../../../li) | repository · `93eef22a7c` | 123 · 1/0 | keep · — | Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически. |
| [lytics](../../../lytics) | repository · `02f6cc9bec` | 38 · 12/24 | compatibility · P03 P10 | Реальный mudra domain consumer: golden derivation/signature/ingest vectors, маленький build graph; std::cell и deployed identities оставить. |
| [maker](../../../maker) | repository · `6e28681419` | 23 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [midao](../../../midao) | repository · `85ffdd56ad` | 94 · 5/6 | compatibility · P10 P11 | DAO/collective authority ссылки согласовать; legal/contract/physical terminology и развёрнутые storage layouts сохранить. |
| [midao/lib/forge-std](../../../midao/lib/forge-std) | worktree · `8e40513d67` | 68 · 1/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [mir](../../../mir) | repository · `b9c832d501` | 40 · 5/11 | keep · — | Spatial rendering/сетка и чтение graph neurons; самостоятельного cell subject API не найдено, переименования не требуется. |
| [mona](../../../mona) | repository · `ddb62bae26` | 27 · 2/0 | keep · — | Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически. |
| [mudra](../../../mudra) | repository · `d6eab0d251` | 38 · 1/23 | compatibility · P01 P03 P06 P10 | Зафиксировать derivation/identity profiles и proof boundary; claim/domain signatures сохраняются; не переносить crypto в engine. |
| [neural](../../../neural) | repository · `6623a8e9cd` | 24 · 9/15 | adapter · P04 P08 | CLI импортирует cyb_core::Cell: перейти на graph session и явный subject; поправить Cargo/docs/examples. |
| [nika](../../../nika) | repository · `3dfb67415a` | 32 · 3/0 | keep · — | Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически. |
| [nika/.vendor/nockchain](../../../nika/.vendor/nockchain) | repository · `3dee0d5d3d` | 839 · 169/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [nox](../../../nox) | repository · `2612673cae*` | 148 · 19/15 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [omi/refs/omi-local](../../../omi/refs/omi-local) | repository · `ca588d97cf` | 3614 · 276/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [omi/refs/omi-upstream](../../../omi/refs/omi-upstream) | repository · `06e031a77c` | 952 · 24/2 | reference-partial · — | Sparse/partial reference: доступные файлы и 55 index blobs прочитаны; 12 397 blobs отсутствуют локально. Нет вывода о содержимом непрочитанной части. |
| [omi/refs/parakeet.cpp](../../../omi/refs/parakeet.cpp) | repository · `e75de9b6b9*` | 382 · 7/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [omi/refs/parakeet.cpp/third_party/ggml](../../../omi/refs/parakeet.cpp/third_party/ggml) | worktree · `e705c5fed4*` | 1201 · 13/0 | reference · — | Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости. |
| [opt](../../../opt) | repository · `b5dcfe8464` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [optica](../../../optica) | repository · `9f0c352cfe` | 69 · 5/5 | compatibility · P11 P12 | Проверить wiki alias resolution/root neuron и rebuild links; layout/table/spatial grid cells остаются. |
| [organiq](../../../organiq) | repository · `955fbd86bd` | 510 · 17/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [oxytocin](../../../oxytocin) | repository · `44e2843a65` | 1 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [perla](../../../perla) | repository · `d06b980d3f` | 23 · 1/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [portal](../../../portal) | repository · `51b8498350` | 99 · 5/11 | legacy-compatibility · P10 | Cyber/Cosmos NeuronBandwidth и contract queries сохранить; new identity boundary только в adapters, не в deployed schema. |
| [prysm](../../../prysm) | repository · `652e83e036*` | 96 · 28/49 | adapter · P08 | cyb-core dependency; active/pinned/launcher cell_id → typed destination; NeuronId в UI не равен bech32 format string. |
| [pure](../../../pure) | repository · `f219cb5a3b` | 1 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [qu](../../../qu) | repository · `a3820d6cc1` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [quanta](../../../quanta) | repository · `220712c8d7*` | 104 · 13/0 | keep · — | Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено. |
| [radio](../../../radio) | repository · `f2b1298daa` | 842 · 81/9 | compatibility · P06 P10 | Transport endpoint ≠ neuron; проверить discovery/auth adapters; iroh/Cid, draw.io mxCell и vendor storage vocabulary сохранить. |
| [ren](../../../ren) | repository · `b791f6b540` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [rockets](../../../rockets) | repository · `3f9b8df7e8` | 1 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [rs](../../../rs) | repository · `ae11518dca` | 153 · 33/0 | adapter · P02 P05 P11 | Собственный cell! macro, Cell/CellMetadata и cyb-cell tutorial: bounded module, не subject. Перейти к module!/Module naming, сохранить codegen/lifecycle, состыковать с prog/worker contract. |
| [rune](../../../rune) | repository · `cc186f38f6` | 62 · 23/20 | adapter · P02 P05 | run_cell → prog/event driver; checkpoint ABI и authority tests; Noun::Cell → Pair только отдельной terms миграцией. |
| [sec](../../../sec) | repository · `99e076daa9` | 2 · 0/1 | keep · — | Один концептуальный neuron privacy reference; crypto semantics менять не требуется. |
| [seq](../../../seq) | repository · `575898a50d` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [soft3](../../../soft3) | repository · `9d1b75b8ab*` | 53 · 15/18 | composition · P00 P01 P10 P12 P13 | Shared contracts, roadmap precedence, adapters/release manifest/conformance, сайт и SDK entry points; не копировать engine. |
| [soft3/js](../../../soft3/js) | repository · `f796d97e84` | 56 · 1/13 | compatibility · P10 P12 | Отдельный nested repository. Сохранить Bostrom/CosmJS addresses и wire; адаптировать лишь потребляемый новый API, без смены dependency versions. |
| [soma](../../../soma) | repository · `312177028a` | 12 · 1/7 | adapter · P01 P09 | Снять root Soul Neuron и neuron=cognitive worker; task/prog lineage, recovery и context. Комментарии kernel об identity/log cell исправить. |
| [space-pussy](../../../space-pussy) | repository · `fbe53dd93f` | 1591 · 27/133 | legacy-compatibility · P10 | Исторические chain identities, account keys, protobuf/JSON и snapshot data сохранить; миграция только через explicit bridge/adapters. |
| [strata](../../../strata) | repository · `7f6864abd2` | 244 · 1/0 | keep · — | Математический substrate; найденные cells не runtime субъект. Нет основания менять алгебру или package ради merge. |
| [superhuman](../../../superhuman) | repository · `386c311c90` | 214 · 31/5 | keep · — | Биология/медицина и общие graph ссылки; архитектурного CellId API нет, клетки и нейроны организма сохранить. |
| [sym](../../../sym) | repository · `446865716d` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [tape](../../../tape) | repository · `a9e26cb5bb` | 20 · 4/7 | compatibility · P04 P08 | Neuron molecule — reference/display, не runtime record; новый dialect для typed references по необходимости; framing/signatures неизменны. |
| [ten](../../../ten) | repository · `b4418cf182` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [tok](../../../tok) | repository · `118058fee4` | 13 · 0/5 | compatibility · P03 P06 P13 | Протокольные NeuronId, stake/focus/karma/balances и accounting остаются; execution budgets не создают второй экономический ledger. |
| [trident](../../../trident) | repository · `180b8f155a*` | 558 · 33/23 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [trisha](../../../trisha) | repository · `e97b5439bf*` | 548 · 19/8 | compatibility · P03 P10 | Существующий cli/neuron.rs и Neptune wallet: native key/account lifecycle сохранить; при интеграции квалифицировать identity domain. |
| [tru](../../../tru) | repository · `146cec7463*` | 116 · 5/69 | compatibility · P03 P06 P13 | Протокольные NeuronId, stake/focus/karma/balances и accounting остаются; execution budgets не создают второй экономический ledger. |
| [true-cyber](../../../true-cyber) | repository · `6cd1d8e6ff` | 7 · 3/2 | adapter · P08 P10 P12 | Проверить product facade CLI/sync/link, help и документацию против soft3; не превращать node process в neuron. |
| [uhash](../../../uhash) | repository · `ff8b3bf043` | 70 · 1/1 | keep · — | Viewing-economy references и UI/general cells; merge не меняет settlement/reward contract. |
| [valkyra](../../../valkyra) | repository · `fda590497d` | 1 · 1/0 | keep · — | README: lexical cell match; самостоятельных subject/API/Cargo consumers не обнаружено. |
| [warriors](../../../warriors) | repository · `27872352a6` | 5 · 1/0 | keep · — | Execution/mining guidance сохраняется; worker — исполнитель, не neuron. Cell match — cancellation substring. |
| [wav](../../../wav) | repository · `3c47ffea54` | 2 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [wysm](../../../wysm) | repository · `bde62d57e4` | 319 · 30/1 | keep · — | WASM Cell(u64)/stack slots и std types остаются; worker integration следует execution-model, отдельного subject runtime здесь нет. |
| [xena](../../../xena) | repository · `6ea943a2ea` | 48 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |
| [zheng](../../../zheng) | repository · `0cc40aa10b*` | 109 · 10/7 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [zheng-pin](../../../zheng-pin) | worktree · `0452894c5c` | 82 · 4/7 | compatibility · P02 P03 P05 P13 | Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename. |
| [zoya](../../../zoya) | repository · `ef8f582f25` | 21 · 0/0 | no-hits · — | В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации. |

## Повреждённый checkout и generated Git markers (7)

| Repository / source tree | Kind · HEAD | F · C/N | Решение · пакеты | Что делать |
|---|---|---|---|---|
| [cyber/cyber/build/cyb/honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a](../../../cyber/cyber/build/cyb/honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a) | generated-git-marker · `—` | 293 · 29/13 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [cyber/cyber/build/cyb/honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291](../../../cyber/cyber/build/cyb/honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291) | generated-git-marker · `—` | 314 · 32/13 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [cyberia-blog/build/cyb/honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a](../../../cyberia-blog/build/cyb/honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a) | generated-git-marker · `—` | 293 · 29/13 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [cyberia-blog/build/cyb/honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291](../../../cyberia-blog/build/cyb/honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291) | generated-git-marker · `—` | 314 · 32/13 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [cyberia-blog/build/cyberia/midao/lib/forge-std](../../../cyberia-blog/build/cyberia/midao/lib/forge-std) | generated-git-marker · `—` | 76 · 1/0 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [cyberia-blog/build/neural/nu](../../../cyberia-blog/build/neural/nu) | generated-git-marker · `—` | 2730 · 469/17 | generated-copy · P12 | Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог. |
| [nu](../../../nu) | broken-git-checkout · `—` | 2305 · 248/0 | broken-reference · — | Git link сломан; исходники просканированы через filesystem fallback. Terminal/grid/std cells KEEP; repair Git metadata — отдельная работа. |

## Исходники вне Git (6 групп)

| Repository / source tree | Kind · HEAD | F · C/N | Решение · пакеты | Что делать |
|---|---|---|---|---|
| [.](../../../.) | non-git-source-tree · `—` | 3 · 0/0 | workspace · P00 | Root doctrine и non-Git files учтены; runtime subject references не обнаружены. |
| [.claude](../../../.claude) | non-git-source-tree · `—` | 2 · 0/0 | workspace · — | Локальная конфигурация вне Git учтена; целевой модели не касается. |
| [.grok](../../../.grok) | non-git-source-tree · `—` | 1 · 0/0 | workspace · — | Локальная конфигурация вне Git учтена; целевой модели не касается. |
| [omi](../../../omi) | non-git-source-tree · `—` | 40 · 1/0 | keep · — | Non-Git Rust source tree отдельно от refs; cell совпадения только Cargo.lock, прямого merge API нет. |
| [omi-scan](../../../omi-scan) | non-git-source-tree · `—` | 2 · 0/0 | workspace · — | Служебные source files вне Git; совпадений целевой модели нет. |
| [zheng-022](../../../zheng-022) | non-git-source-tree · `—` | 81 · 3/7 | reference · — | Копия proof implementation вне Git; сохранить как version reference, canonical proof changes только в выбранном zheng revision. |

## Alias

- `honeycrisp/.claude/worktrees/strata` → `/Users/master/cyber/strata`; источник уже учтён, повторно не сканировался.

Новый `neuron` пока отсутствует: это целевой successor `cell`, а не пропущенный repository.
