# Neuron/cell: аудит локального workspace

Итог реализации и миграции 2026-09-13: [release audit](release.md),
[финальные dispositions](final-repositories.md) и [журнал выполнения](implementation.md).
Ниже сохранён исходный аудит до реализации с его датой и границами.

Дата: 2026-09-12. Это факты сканирования и карта воздействия.
Требования и порядок реализации находятся в [roadmap](../../roadmap/neuron-cell-convergence.md).
Миграция кода, ключей, repository и данных этим аудитом не выполнялась.

- [Каждый repository и назначенное решение](repositories.md).
- [Файлы, находки и работа по компонентам](change-map.md).
- [Полный machine-readable snapshot](scan.json), [все кандидаты](matches.jsonl).
- [Сканер](scan.py), [reviewed dispositions и генератор inventory](report.py), [dispositions.json](dispositions.json).

## Покрытие

Скан выполнялся с **08:29:40 по 08:31:32 UTC**. Это последовательность локальных
снимков файлов/репозиториев, не атомарный snapshot всей машины. Совпавшие файлы
снабжены SHA-256; Git HEAD/dirty записаны для доступных checkout. Последующие
изменения workspace и документы этого аудита не входят в исходный baseline.

| Объект | Количество | Что проверено |
|---|---:|---|
| Git-маркеры | 117 | Discovery не зависит от Git ignore; проверен настоящий show-toplevel |
| Действительные Git roots | 110 | Nested repos, vendor/reference checkouts и linked worktrees учтены отдельно |
| Повреждённый Git checkout | 1 | `nu`: исходники прочитаны через filesystem fallback |
| Generated ложные Git markers | 6 | Не выданы за самостоятельные repositories; source copies просканированы отдельно |
| Non-Git source groups | 6 | Root, .claude, .grok, omi, omi-scan, zheng-022 |
| Symlink directory aliases | 1 | Honeycrisp worktree alias → strata; target уже учтён |
| Прочитанные текстовые файлы | 29 193 | 22 739 в valid Git roots; 4 020 в generated marker trees; 2 434 в nu/non-Git groups |
| Файлы с `cell` substring | 2 433 | Включая cancelled/excellent, std cells, биологию и generated copies |
| Файлы с `neuron` substring | 1 483 | Включая протокольные и биологические значения |
| Domain-regex кандидаты | 102 | Узкий дополнительный фильтр, **не** число реальных subject APIs |
| Identity-regex кандидаты | 398 | Подписи, ключи, chains и ID references |
| Dependency-regex кандидаты | 23 | Включают docs/locks/ложные совпадения; Cargo границы проверены отдельно |

В scan поле `kind: worktree` технически означает `.git` **файл**, поэтому
включает также submodule-style checkout. Число 110 — число действительных roots,
а не число уникальных remote projects. Две копии cyberia-my и разные worktrees
могут содержать один проект, но для миграции это разные локальные деревья.

## Метод

1. Filesystem walk всего `~/cyber`, включая hidden directories и nested roots.
   `.git` contents и build/cache directories исключены из рекурсивного чтения.
   Root directory сам не является Git repository.
2. Проверка каждого Git-маркера через `git rev-parse --show-toplevel`.
   Для корректных roots: tracked + nonignored untracked files, HEAD, branch,
   dirty status. Nested source принадлежит самому глубокому найденному root.
3. Широкое regex-чтение всех доступных текстовых файлов перечисленного scope,
   затем дополнительные domain/identity/dependency filters. Вывод содержит
   paths, counts, hashes и номера строк, без исходного текста и секретов.
4. Sparse index: доступные локальные blobs прочитаны через `git cat-file`,
   без checkout и автоматического fetch отсутствующих объектов.
5. Семантическая проверка ключевых contracts/implementation, manifests и
   репозиториев с широкими совпадениями. Отдельный `rg` проход проверил direct
   Cargo consumers; так обнаружены graph callers prysm/neural и Rs macros.
6. По каждому repository назначено решение. Это **migration triage**, а не
   утверждение о полном ручном code review всех 29 тысяч файлов. Нулевой regex
   результат не доказывает отсутствие семантики с совершенно другим названием.

Rs — показательный случай: `cell!`/`CellMetadata` не покрываются узким фильтром
CellId, но попали в широкий scan и получили отдельный implementation пакет.
Напротив, `std::cell::Cell` попадает в domain regex и остаётся KEEP.

## Исключения и недоступные исходники

Все 117 markers и шесть non-Git групп получили строку в inventory. Однако
**не все indexed файлы Omi reference доступны локально**:

- `omi/refs/omi-upstream`: sparse/partial checkout. Прочитаны рабочие файлы и
  55 доступных index blobs; **12 397** absent blobs отсутствуют в локальном object
  store. Их пути перечислены в `scan.json → repositories → errors` как
  `sparse_index_blob_unavailable`. Содержимое этой части не проверено. Reference
  repository не назначен целью rename; перед изменением его adapter потребуется
  targeted source fetch/review соответствующей ревизии.
- `nu/.git` ссылается на отсутствующий `.git/modules/vendor/nushell`. Прочитаны
  2 305 текстовых файлов из самого дерева; revision из Git подтвердить нельзя.
- Шесть `.git` markers под `cyber/cyber/build` и `cyberia-blog/build` разрешаются
  в Git root родителя. Это generated copies, не дополнительные проекты.
- Не читались credentials по маскам (`.env*`, private-key/seed names, key files),
  binaries, symlink files и build/cache metadata. Счётчики: 38 credential files,
  1 527 binary files, 16 symlink files; 7 339 build/metadata exclusions.
  Это категории событий сканера, не обязательное разбиение уникальных файлов.
- 12 486 `missing_or_gitlink` — отсутствующие рабочие paths/embedded entries,
  в основном sparse Omi; это не число дополнительных repositories. Доступные
  sparse blobs учитываются отдельно. Нет filesystem discovery/read errors.
- Файлы более 32 MiB ограничены сканером; в этом прогоне текстовых исключений
  по размеру не было. Binary prefix проверялся до ограничения размера.
- Полные Git histories, другие ветки/remotes, ignored untracked files и
  external symlink targets не являются scope этого source snapshot.

Ограничение Omi не скрыто под фразой «совпадений нет». Для собственных целевых
runtime/identity repositories таких source gaps не обнаружено.

## Выводы, определяющие roadmap

1. Один subject neuron согласуется с уже существующим понятием prog в soft3.
2. Runtime-cell ID сейчас — birth particle; миграция требует явного provenance
   mapping, supported authority и сохранения legacy schema bytes.
3. Старый cyb Cell — multi-neuron graph aggregate. Его функциональность переходит
   в graph/session boundary, а не целиком в экземпляр neuron.
4. Одно live execution в engine, singleton maps и локальная placeholder authority
   требуют функциональных изменений до полноценного агента.
5. Scope шире Rust runtime: Rs macros, UI destinations, soma ontology, node modes,
   graph shards, service/ledger roles, context builder и настоящие consumers ключей.
6. Число совпадений сильно завышает rename scope. Upstream/native protocol types,
   storage cells, биология, signed history и generated files требуют сохранения.

Проверяемые source anchors и следствия перечислены в [F01–F16](change-map.md).

## Повторная проверка перед завершением

Workspace изменялся параллельно. Повторное filesystem discovery не обнаружило
новых Git roots, но `cve-legacy-local` и его nested `docs` уже отсутствовали.
Они остаются в baseline inventory с явной пометкой, без неработающих file links.
Таким образом, 110 valid roots — число исходного scan, не обещание неизменного
текущего числа проектов.

Изменились отдельные files bbg/cybergraph/foculus; появились native-storage и
native-node specs. Их контракты дополнительно прочитаны, и F16/P04 прямо
предписывают reuse coordinator вместо конкурирующей реализации. Эти параллельные
изменения, `soft3/specs/README.md` и исходный dirty lockfile не редактировались
этой работой. [revalidation.json](revalidation.json) фиксирует повторную проверку.

## Воспроизведение

Из `~/cyber/soft3`:

```sh
python3 audit/neuron-cell/scan.py --root ~/cyber --out /tmp/neuron-cell-rescan
```

Другой output directory сохраняет исходный baseline. Для нового снимка сканер
исключает собственную output directory; сохранённый старый audit будет виден как
обычные source files, поэтому его нужно учитывать при сравнении counts.
Более точное сравнение — repository HEAD и file SHA, а не общий grep count.

```sh
python3 audit/neuron-cell/report.py
```

Report строится из сохранённого scan.json и проверяет, что каждому discovered
path назначено решение ровно один раз. При добавлении repository assertion
потребует явной классификации; неизвестные деревья не превращаются в KEEP автоматически.

Скан не запускает repository code, тесты или deployments и не меняет checkout.
Этот отчёт не свидетельствует о прохождении Rust tests или готовности merge.

## Проверки артефактов

После записи roadmap и inventory проверены:

- 123 dispositions точно покрывают все baseline roots/source groups; неизвестных
  и повторно назначенных путей нет.
- 3 676 candidate file records в matches.jsonl совпадают с суммой записей scan.json.
- SHA-256 сохранённого scan.py совпадает с scanner fingerprint baseline.
- 166 локальных Markdown links разрешаются; исчезнувшие baseline directories
  обозначены текстом, а не неработающими ссылками.
- P00–P13, G01–G14 и S01–S08 имеют полный набор строк в roadmap.
- Python scripts синтаксически корректны; новые filenames допустимы на NTFS;
  проверка whitespace новых файлов и `git diff --check` прошли.

Rust builds/tests не запускались: текущие изменения — план, аудит и ссылки.
Поведенческие tests перечислены как будущие release gates, без отметок PASS.
