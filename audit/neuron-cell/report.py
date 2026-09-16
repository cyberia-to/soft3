#!/usr/bin/env python3
"""Render reviewed dispositions into this audit; leave surveyed source trees alone."""
from pathlib import Path
from collections import Counter
import json
from urllib.parse import quote

HERE = Path(__file__).resolve().parent
scan = json.loads((HERE / 'scan.json').read_text())
dispositions = {}


def assign(paths, kind, packages, action):
    for path in paths.split():
        assert path not in dispositions, path
        dispositions[path] = dict(kind=kind, packages=packages, action=action)


assign('cell', 'implementation', 'P02 P03 P05 P06 P07 P12',
       'Эволюция в neuron: ID/model/progs, engine concurrency, authority, legacy migration; сохранить Git и immutable suite.')
assign('cyb', 'implementation', 'P01 P04 P06 P08 P09',
       'Одна subject model; Cell/SharedCell → graph session; identity registry, explicit author, typed navigation; удалить дублирование core.')
assign('cybergraph', 'implementation', 'P03 P04 P07',
       'Общий NeuronId; graph/session durability и imports; namespace остаётся адресом данных; согласовать claims/CAS.')
assign('bbg', 'implementation', 'P03 P04 P07',
       'Реэкспорт общего ID без runtime dependency; shared Database transactions и migration staging; DAS/unimem cells сохранить.')
assign('foculus', 'compatibility', 'P03 P04 P06',
       'Согласовать ID imports и writer/finality contract; сохранить signal bytes, per-neuron chain и отдельный network.')
assign('mudra', 'compatibility', 'P01 P03 P06 P10',
       'Зафиксировать derivation/identity profiles и proof boundary; claim/domain signatures сохраняются; не переносить crypto в engine.')
assign('rune', 'adapter', 'P02 P05',
       'run_cell → prog/event driver; checkpoint ABI и authority tests; Noun::Cell → Pair только отдельной terms миграцией.')
assign('neural', 'adapter', 'P04 P08',
       'CLI импортирует cyb_core::Cell: перейти на graph session и явный subject; поправить Cargo/docs/examples.')
assign('prysm', 'adapter', 'P08',
       'cyb-core dependency; active/pinned/launcher cell_id → typed destination; NeuronId в UI не равен bech32 format string.')
assign('soma', 'adapter', 'P01 P09',
       'Снять root Soul Neuron и neuron=cognitive worker; task/prog lineage, recovery и context. Комментарии kernel об identity/log cell исправить.')
assign('rs', 'adapter', 'P02 P05 P11',
       'Собственный cell! macro, Cell/CellMetadata и cyb-cell tutorial: bounded module, не subject. Перейти к module!/Module naming, сохранить codegen/lifecycle, состыковать с prog/worker contract.')
assign('soft3', 'composition', 'P00 P01 P10 P12 P13',
       'Shared contracts, roadmap precedence, adapters/release manifest/conformance, сайт и SDK entry points; не копировать engine.')
assign('soft3/js', 'compatibility', 'P10 P12',
       'Отдельный nested repository. Сохранить Bostrom/CosmJS addresses и wire; адаптировать лишь потребляемый новый API, без смены dependency versions.')
assign('true-cyber', 'adapter', 'P08 P10 P12',
       'Проверить product facade CLI/sync/link, help и документацию против soft3; не превращать node process в neuron.')
assign('lytics', 'compatibility', 'P03 P10',
       'Реальный mudra domain consumer: golden derivation/signature/ingest vectors, маленький build graph; std::cell и deployed identities оставить.')
assign('cyberia-my cyberia/research/cyberia-my', 'compatibility', 'P03 P10',
       'Оба checkout: src/signal.rs domain key и ADR-036-style body, local storage/sequence; подтвердить один canonical source и сохранить wire.')
assign('trisha', 'compatibility', 'P03 P10',
       'Существующий cli/neuron.rs и Neptune wallet: native key/account lifecycle сохранить; при интеграции квалифицировать identity domain.')
assign('inf', 'compatibility', 'P03 P04 P10',
       'Neuron value/query scoping и BbgSource: subject не подменять произвольным Particle; добавить prog/task projections у владельцев schemas.')
assign('radio', 'compatibility', 'P06 P10',
       'Transport endpoint ≠ neuron; проверить discovery/auth adapters; iroh/Cid, draw.io mxCell и vendor storage vocabulary сохранить.')
assign('tape', 'compatibility', 'P04 P08',
       'Neuron molecule — reference/display, не runtime record; новый dialect для typed references по необходимости; framing/signatures неизменны.')
assign('fs', 'documentation', 'P01 P10',
       'Patch author/neuron namespace и scoped program access; per-neuron patch chain не смешивать с runtime commit index.')
assign('glia', 'integration', 'P06 P09',
       'Inference adapter получает bounded jobs/context; нейросетевые neurons и tensor layout не переименовывать.')
assign('conformance', 'integration', 'P00 P13',
       'Самостоятельный scaffold: согласовать owner harness с soft3/conformance; новые migration/identity scenarios без фиктивного PASS.')
assign('cyber', 'documentation', 'P01 P11 P12',
       'Cell ladder, hierarchy, 3c/oikos, node modes/product/ownership/CLI, roadmaps; shard/partial/service роли развести, generated copies перестроить.')
assign('aos', 'documentation', 'P11',
       'Building/runnable cells в README/apps и всех service pages → service/prog и явная governance; не создавать ключ каждой площадке.')
assign('crystal', 'documentation', 'P01 P11',
       'Canonical neuron/prog/soul/avatar/node vocabulary и ссылки; протокольные neuron statements сохранить, исправить конфликт поведения/soul.')
assign('cyberia', 'documentation', 'P11',
       'Foundation архитектура и protocol system/marketplace: service/shard governance; участник-neuron сохраняется; таблицы и биология KEEP.')
assign('cybics', 'documentation', 'P11',
       'Только software ontology references к архитектуре; biological/memory/data-structure cells и математические analogies сохраняются.')
assign('ctx', 'generated-source', 'P11 P12',
       'ctx.md — generated context. Обновить canonical inputs/pinned paths в build.nu, затем rebuild с provenance; не править bundle вручную.')
assign('cybernode', 'documentation', 'P10 P12',
       'Deployment/lytics storage docs и команды сверить с release; chain endpoints сохранить; сервера этой работой не менять.')
assign('optica', 'compatibility', 'P11 P12',
       'Проверить wiki alias resolution/root neuron и rebuild links; layout/table/spatial grid cells остаются.')
assign('cyberia-blog', 'publication', 'P11 P12',
       'Исторические blog posts не переписывать; обновить current subgraph links и regenerate build из canonical sources.')
assign('cyber-valley', 'documentation', 'P11',
       'Satoshi soul/agent docs: обновить software ontology links; физические/образовательные и biological cells сохранить.')
assign('bootloader bostrom go-cyber space-pussy', 'legacy-compatibility', 'P10',
       'Исторические chain identities, account keys, protobuf/JSON и snapshot data сохранить; миграция только через explicit bridge/adapters.')
assign('cybernet', 'legacy-compatibility', 'P10',
       'CosmWasm neuron_uid/hotkey/coldkey/netuid — native protocol model; не заменять локальный UID на cyber NeuronId и не мигрировать contracts этим merge.')
assign('hub', 'legacy-compatibility', 'P10',
       'hub-skills хранит neuron/network/protocol/endpoint: сохранить contract storage/wire; bridge должен квалифицировать, а не переименовать поля.')
assign('portal', 'legacy-compatibility', 'P10',
       'Cyber/Cosmos NeuronBandwidth и contract queries сохранить; new identity boundary только в adapters, не в deployed schema.')
assign('midao', 'compatibility', 'P10 P11',
       'DAO/collective authority ссылки согласовать; legal/contract/physical terminology и развёрнутые storage layouts сохранить.')
assign('nox hemera lens eidos trident zheng zheng-pin', 'compatibility', 'P02 P03 P05 P13',
       'Pin data/hash/checkpoint/proof boundary и проверить затронутые consumers. Substrate pair/DAS/VM/crypto terms — отдельные контракты, не neuron rename.')
assign('tok tru', 'compatibility', 'P03 P06 P13',
       'Протокольные NeuronId, stake/focus/karma/balances и accounting остаются; execution budgets не создают второй экономический ledger.')
assign('honeycrisp honeycrisp/.claude/worktrees/agent-aa1259cb10112b22a honeycrisp/.claude/worktrees/agent-ad6c77c38e86bc291',
       'keep', '—', 'Unimem cell_id_distinct — память/allocator, не субъект; самостоятельные worktrees учтены, не массово переписывать.')
assign('wysm', 'keep', '—', 'WASM Cell(u64)/stack slots и std types остаются; worker integration следует execution-model, отдельного subject runtime здесь нет.')
assign('mir', 'keep', '—', 'Spatial rendering/сетка и чтение graph neurons; самостоятельного cell subject API не найдено, переименования не требуется.')
assign('strata', 'keep', '—', 'Математический substrate; найденные cells не runtime субъект. Нет основания менять алгебру или package ради merge.')
assign('bita evy li mona nika', 'keep', '—',
       'Кандидаты domain regex оказались std::cell::Cell/fork APIs; их сохранять. Native Bitcoin/chain/runtime типы не становятся neuron автоматически.')
assign('cyberstates', 'keep', '—', 'cell_id в pages/map.rs — UI карта/DOM; совпадение не относится к runtime identity.')
assign('superhuman', 'keep', '—', 'Биология/медицина и общие graph ссылки; архитектурного CellId API нет, клетки и нейроны организма сохранить.')
assign('uhash', 'keep', '—', 'Viewing-economy references и UI/general cells; merge не меняет settlement/reward contract.')
assign('warriors', 'keep', '—', 'Execution/mining guidance сохраняется; worker — исполнитель, не neuron. Cell match — cancellation substring.')
assign('sec', 'keep', '—', 'Один концептуальный neuron privacy reference; crypto semantics менять не требуется.')
assign('valkyra', 'keep', '—', 'README: lexical cell match; самостоятельных subject/API/Cargo consumers не обнаружено.')
assign('cve cve-legacy-local', 'keep', '—', 'Нет runtime CellId/identity API кандидатов; UI/dependency/общие слова сохранить. Legacy checkout не синхронизировать автоматически.')
assign('cloud-forest cyberia/research/events cyberia/research/mimi episode-1 erga organiq perla quanta',
       'keep', '—', 'Текстовые/UI/dependency/предметные совпадения; direct runtime subject или identity dependency не выявлены. Изменений по этому merge не назначено.')
assign('joy', 'compatibility', 'P02 P05',
       'Cells в formula/state — substrate pairs и private-state positions; сохранить bytes/proofs и согласовать с terms-map, не переименовать в neuron.')
assign('inf/rs/cozo nika/.vendor/nockchain midao/lib/forge-std omi/refs/omi-local omi/refs/parakeet.cpp omi/refs/parakeet.cpp/third_party/ggml',
       'reference', '—', 'Отдельный embedded/reference Git root: upstream API/vocabulary сохранять; менять собственный adapter только при доказанной зависимости.')
assign('omi/refs/omi-upstream', 'reference-partial', '—',
       'Sparse/partial reference: доступные файлы и 55 index blobs прочитаны; 12 397 blobs отсутствуют локально. Нет вывода о содержимом непрочитанной части.')
assign('nu', 'broken-reference', '—',
       'Git link сломан; исходники просканированы через filesystem fallback. Terminal/grid/std cells KEEP; repair Git metadata — отдельная работа.')
assign('.github arc bel bt cyberia-capital cyberia-to cybervalley-io cybland dif kadek maker opt oxytocin pure qu ren rockets seq sym ten wav xena zoya cve-legacy-local/docs',
       'no-hits', '—', 'В доступных текстовых исходниках cell/neuron совпадений нет; прямой задачи миграции не выявлено, scaffold не требует новой реализации.')

for row in scan['repositories']:
    if row['kind'] == 'generated-git-marker':
        dispositions[row['path']] = dict(kind='generated-copy', packages='P12', action=
            'Ложный Git root в build copy; исходники учтены отдельно, править canonical repository и regenerate, не этот каталог.')

assign('.', 'workspace', 'P00', 'Root doctrine и non-Git files учтены; runtime subject references не обнаружены.')
assign('.claude .grok', 'workspace', '—', 'Локальная конфигурация вне Git учтена; целевой модели не касается.')
assign('omi', 'keep', '—', 'Non-Git Rust source tree отдельно от refs; cell совпадения только Cargo.lock, прямого merge API нет.')
assign('omi-scan', 'workspace', '—', 'Служебные source files вне Git; совпадений целевой модели нет.')
assign('zheng-022', 'reference', '—', 'Копия proof implementation вне Git; сохранить как version reference, canonical proof changes только в выбранном zheng revision.')

rows = scan['repositories'] + scan['non_git_sources']
assert set(dispositions) == {r['path'] for r in rows}, (
    set(dispositions) ^ {r['path'] for r in rows})
(HERE / 'dispositions.json').write_text(json.dumps(dispositions, ensure_ascii=False, indent=2) + '\n')

header = '''# Репозитории: полная матрица решений

Snapshot: 2026-09-12. [Метод и ограничения](README.md), [план](../../roadmap/neuron-cell-convergence.md),
[конкретные файлы](change-map.md). Отдельная строка для каждого обнаруженного Git root/маркера
и каждой группы non-Git исходников. HEAD/dirty отражают скан, не текущее состояние.

`C/N` — число файлов с широкими лексическими совпадениями cell/neuron, **не число
необходимых правок**. Включают std types, biology, cancelled и прочие омонимы.
`F` — прочитанные текстовые файлы; для sparse checkout включены доступные index blobs.
`*` после HEAD означает dirty; `—` означает недоступные Git metadata, а не чистый Git.
Все пути относительно `~/cyber`. Подробные SHA, counts, errors — в [scan.json](scan.json).

'''


def table(items):
    lines = ['| Repository / source tree | Kind · HEAD | F · C/N | Решение · пакеты | Что делать |',
             '|---|---|---|---|---|']
    for row in items:
        path = row['path']
        d, counts = dispositions[path], row['counts']
        head = row.get('head', '—')[:10] + ('*' if row.get('dirty') else '')
        link = '../../../' + quote(path, safe='/')
        label = (f'[{path}]({link})' if (Path(scan['root']) / path).exists()
                 else f'`{path}` (путь отсутствует после snapshot)')
        lines.append(f"| {label} | {row['kind']} · `{head}` | "
                     f"{counts.get('text_files_scanned', 0)} · {counts.get('cell_files', 0)}/{counts.get('neuron_files', 0)} | "
                     f"{d['kind']} · {d['packages']} | {d['action']} |")
    return '\n'.join(lines) + '\n'


normal = [r for r in scan['repositories'] if r['kind'] in ('repository', 'worktree')]
special = [r for r in scan['repositories'] if r['kind'] not in ('repository', 'worktree')]
output = header + f'## Git repositories и worktrees ({len(normal)})\n\n' + table(normal)
output += f'\n## Повреждённый checkout и generated Git markers ({len(special)})\n\n' + table(special)
output += '\n## Исходники вне Git (6 групп)\n\n' + table(scan['non_git_sources'])
output += '\n## Alias\n\n'
for alias in scan['aliases']:
    output += f"- `{alias['path']}` → `{alias['target']}`; источник уже учтён, повторно не сканировался.\n"
output += '\nНовый `neuron` пока отсутствует: это целевой successor `cell`, а не пропущенный repository.\n'
(HERE / 'repositories.md').write_text(output)
print(json.dumps({'rows': len(rows), 'valid_git_roots': len(normal),
                  'dispositions': dict(Counter(v['kind'] for v in dispositions.values()))}, ensure_ascii=False))
