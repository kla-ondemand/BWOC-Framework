# Roadmap

แผนทีละ phase ของ BWOC **Phase** อธิบาย milestone ของการ implement; แต่ละ phase อาจครอบคลุม SemVer release หลายครั้ง ดู [`VERSION.md`](../../VERSION.md) สำหรับการแยก version กับ phase ดู [`VISION.th.md`](../../VISION.th.md) สำหรับ success criteria ที่ 1 ปีและ 3 ปี

---

## สถานะปัจจุบัน

**Phase ที่ active:** Phase 2 — *การปฏิบัติ ฐิติ* — กำลังดำเนินการ DoD ของ Phase 1 v2.0 บรรลุแล้ว
**Software-Version:** ดู [`VERSION.md`](../../VERSION.md)
**Document-Version:** ดู [`VERSION.md`](../../VERSION.md)

---

## Phase 1 v2.0 — รากฐาน อุปฺปาท

**นิยามของเสร็จ:** end-to-end **อุปฺปาท** สำหรับ backend หนึ่งตัว — incarnate · check · spawn agent ที่รันได้

### เสร็จแล้ว

- Cargo workspace (`bwoc-core`, `bwoc-cli`, `bwoc-agent`) scaffold; edition 2024; MSRV 1.85
- `VERSION.md` มี `Software-Version`, `Document-Version`, และ `Last-Updated`; auto-managed โดย `.claude/hooks/auto-version.sh`
- Open-source hygiene: `VISION.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`; root `README.md` พร้อม badge, TOC, footer
- เอกสารสเปก (bilingual EN/TH ทุกตัว): `PHILOSOPHY` §0.1 *วงรอบ*, `GLOSSARY`, `ARCHITECTURE`, `INCARNATION`, `WORKSPACE`, `NAMING`
- Crate README (`bwoc-core`, `bwoc-cli`, `bwoc-agent`)
- เครื่องมือ Claude Code: 4 project skills (`/incarnate`, `/check-neutrality`, `/check-bilingual`, `/task-log`); 2 PostToolUse hooks (`bilingual-reminder`, `auto-version`)
- shell script `incarnate.sh` และ `check-agent-neutrality.sh` ใน template (ใช้ได้วันนี้; จะถูก port เป็น Rust)

### ส่งมอบใน Phase 1 v2.0 (เสร็จแล้ว)

รายการทั้งหมดด้านล่าง implement แล้ว Definition of Done ของ phase นี้ (uppāda end-to-end สำหรับ backend หนึ่ง) **บรรลุ** เหลือเฉพาะ HELD policy items (`CODEOWNERS` · `ISSUE_TEMPLATE/config.yml`) ที่รอ user direction; release pipeline พร้อมใช้แล้ว (ดู Phase 2)

| รายการ | สเปก | สถานะ |
|---|---|---|
| `bwoc init [path]` | [`WORKSPACE.th.md`](WORKSPACE.th.md#cli-surface) | ✓ |
| `bwoc workspace info` · `validate` | [`WORKSPACE.th.md`](WORKSPACE.th.md#cli-surface) | ✓ |
| `bwoc new <name>` (port ของ `incarnate.sh`) | [`INCARNATION.th.md`](INCARNATION.th.md) | ✓ |
| `bwoc check [path]` (port ของ `check-agent-neutrality.sh`) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ |
| `bwoc spawn <name>` (minimal `exec`) | [`ARCHITECTURE.th.md`](ARCHITECTURE.th.md#การไหลของข้อมูล--bwoc-spawn-agent-foo) | ✓ |
| `bwoc list` (อ่าน `.bwoc/agents.toml`) | [`WORKSPACE.th.md`](WORKSPACE.th.md) | ✓ |
| flag `--lang` wired เข้ากับ Project Fluent (locale TH + EN) | [`crates/bwoc-cli/README.md`](../../crates/bwoc-cli/README.md) | ✓ ครบ 8 surface (init/list/spawn/workspace info/workspace validate/check/new/bwoc-agent) |
| Skill `/check-naming` (audit `*.md` กับ `NAMING.th.md`) | [`NAMING.th.md`](NAMING.th.md#audit) | ✓ + wired เข้า `.github/workflows/docs.yml` |
| Runtime ทำงานจาก directory ใดก็ได้ | template ของ agent embedded ผ่าน `include_dir!` + env `BWOC_TEMPLATE` + cache `~/.bwoc/template/` | ✓ |
| Bump version major/minor ด้วยมือ | `scripts/bump-version.sh <level> [--software\|--document\|--both]` | ✓ (patch ยัง auto-bump ผ่าน hook) |

---

## Phase 2 — การปฏิบัติ ฐิติ

**นิยามของเสร็จ:** agent ดำเนินงานพร้อม control surface จริง; backend หลายตัวถูกใช้งาน; release ทำซ้ำได้

### ส่งมอบใน Phase 2 (เสร็จแล้ว)

| รายการ | หมายเหตุ |
|---|---|
| Daemon `bwoc-agent --serve` | Unix-only (`.bwoc/agent.pid` + `.bwoc/agent.sock`; stub cfg-gated บน Windows) |
| IPC control socket — protocol แบบ line-text | `PING`/`STATUS`/`STOP` ผ่าน Unix domain socket; debug ได้ด้วย `nc -U` |
| `bwoc status [name]` | health + runtime indicator (●/○) + uptime ผ่าน socket query |
| `bwoc list` | registry view + runtime indicator + INBOX count + filter `--running` / `--status` / `--backend` |
| `bwoc send <to> <msg>` + `bwoc inbox <agent>` | JSONL inbox ที่ `<agent>/.bwoc/inbox.jsonl`; `--watch` / `--clear` / `--limit` / `--json` |
| `bwoc doctor` | env + workspace diagnostic; `--auto` กวาด `agent.pid` / `agent.sock` / `inbox.cursor` ที่ stale |
| `bwoc start <name>` (idempotent) | flip registry + spawn `bwoc-agent --serve` ถ้ายังไม่ทำงาน; `--no-daemon` ข้าม spawn |
| `bwoc ping <name>` | CLI client สำหรับคำสั่ง PING ของ daemon |
| `bwoc chat <name>` (+ `--tmux`) | resolve backend จาก registry; exec `bwoc spawn` |
| `bwoc dashboard` (TUI) | ratatui-based; agents pane + detail pane + auto-refresh 2s + hotkey `t` เปิด tmux |
| Daemon-side inbox watch + cursor | ประกาศ envelope ใหม่ไปยัง stderr; `.bwoc/inbox.cursor` รอด restart |
| `--json` ครอบคลุม read-only commands | `list`, `status`, `workspace info`, `workspace validate`, `check` |
| CI matrix | `ubuntu-latest` · `macos-latest` · `windows-latest` เขียวทุก push |
| Release pipeline (CalVer) | `release.yml` trigger เมื่อ push tag `v<YYYY>.<M>.<D>-<patch>`; 4 binary cross-platform + `.sha256` → GitHub Release ที่สร้างอัตโนมัติ |
| Help system (ใน binary) | 9 topic: `getting-started`, `backends`, `workspace`, `manifest`, `arc`, `lifecycle`, `daemon`, `messaging`, `persona` |
| Shell completion | `bwoc completion <bash\|zsh\|fish\|powershell\|elvish>` ผ่าน clap_complete |
| `bwoc init` เขียน `.gitignore` | exclude daemon ephemerals (PID/socket/cursor) สำหรับ user workspace |
| `bwoc new --scope / --out-of-scope / --mindsets / --skills` | persona substitution + mindset/skill stub seeding ตอน incarnate |
| Module `livecheck` ที่ใช้ร่วม | รวม 5 copy ของ `signal_zero_alive` / `running_pid` / `query_uptime` / `format_uptime` / `inbox_count` |
| Stub `bwoc-agent --serve` สำหรับ Windows | build + run default mode ได้; `--serve` exit 2 พร้อมข้อความ "Unix-only" |
| `bwoc log <agent>` | Tail daemon stderr จาก `<agent>/.bwoc/agent.log`; `-f`/`--follow` สำหรับ live stream; `-n N` สำหรับ N บรรทัดล่าสุด; `--clear` truncate ในที่ |
| Per-workspace memory scaffold | `bwoc init` สร้าง `.bwoc/memory/` พร้อม README อธิบาย 4-tier scope hierarchy (per-agent / per-workspace / per-user / Tier 2) |
| `bwoc memory list \| show \| put \| search` | CLI read/write/search ครบสำหรับ `.bwoc/memory/`: `list` (table + `--json`), `show <name>`, `put <name>` (จาก stdin หรือ `--file`, atomic + `--force`), `search <query>` (substring แบบ case-insensitive); ทุก subcommand บังคับ flat-name + ห้าม traversal |

### ที่เหลือก่อน ship

- **Supervision restart-on-crash** — daemon ปัจจุบัน exit เมื่อมี signal; auto-respawn / health-check loop ยังไม่ทำ
- **Cross-backend validation** — uppāda + ṭhiti เต็มกับ 4 backend CLI ใน CI (พิสูจน์ Samānattatā)
- **Code signing** — Apple notarization + Windows Authenticode สำหรับ release artifact (ต้องการ user-cert authorization)
- **Build Linux musl** — `x86_64-unknown-linux-gnu` + `aarch64-unknown-linux-gnu` ship แล้ว; musl (Alpine / distroless) เพิ่มได้เมื่อมีความต้องการ
- **เครื่องมือ memory mining และ interface Tier 2 backend ที่ pluggable**
- **Daemon path สำหรับ Windows ผ่าน named-pipe** — แทน stub cfg-gated ด้วย implementation Windows จริง

---

## Phase 3 — วยะ + Interconnect

**นิยามของเสร็จ:** ชีวิตของ agent จบลงอย่างสะอาด; agent ประสานงานโดยไม่มีศูนย์กลาง

### ส่งมอบใน Phase 3 (เสร็จแล้ว)

| รายการ | หมายเหตุ |
|---|---|
| `bwoc stop <name>` | escalation ladder 3 ขั้น: socket `STOP` → SIGTERM → SIGKILL (รอ ~3s ระหว่างขั้น); idempotent; รายงานว่าขั้นไหนทำให้ daemon จบ |
| `bwoc retire <name>` | ลบจาก registry; `--keep-files` เก็บ agent dir ไว้ |
| `bwoc workspace prune` | ปรับ phantom registry entries vs orphan agent dirs; `--apply` ลบ drift ที่ปลอดภัย |
| User → agent inbox (สัมมาวาจา Phase 0) | `bwoc send` + `bwoc inbox` ship เป็น JSONL envelope; รากฐานสำหรับ agent → agent messaging |

### ที่เหลือสำหรับ Phase 3

- **vaya เต็มรูปแบบ** สำหรับ `bwoc retire` — ปัจจุบัน registry-only พร้อม optional file delete; ต้องเพิ่ม worktree cleanup + branch release + memory prune + interconnect deregistration
- **Agent → agent messaging** — channel สัมมาวาจาจริง; กฎ Sāraṇīyadhamma 6 ของความนุ่มนวล
- **Trust scoring** — Kalyāṇamitta 7 ใช้กับการประกาศ capability และที่มาของข้อความ
- **`.bwoc/interconnect/`** config routing ระดับ workspace
- **Reference implementation ของ Tier 2 memory backend**

---

## Phase 4 — Reference Agent + Fleet

**นิยามของเสร็จ:** ความเป็นไปได้ของ ecosystem พิสูจน์แล้ว; governance ของ fleet ระดับ production ข้าม vendor ทำได้

- Agent อ้างอิงสามตัวหรือมากกว่าในธรรมชาติ สร้างโดยผู้ดูแลนอกทีมผู้เขียนต้นฉบับ (ตาม [`VISION.th.md`](../../VISION.th.md) success ที่ 1 ปี)
- Fleet dashboard — Aparihāniya-dhamma 7 governance ใช้กับการติดตั้ง multi-agent จริง
- ศัพท์ BWOC (Yoniso manasikāra checks, Mattaññutā caps, Sīla baselines, Kalyāṇamitta trust scores) ปรากฏใน codebase ที่ไม่มีความสัมพันธ์กับ project นี้ (success ที่ 3 ปี)
- รูปแบบ fleet ระดับ production ข้าม vendor ใช้ในองค์กรมากกว่าหนึ่งแห่ง

---

## ข้ามทุก Phase

- **Bilingual parity** — เอกสารสเปกทุกฉบับมี EN canonical + TH (และภาษาอื่น ๆ ในอนาคต); hook bilingual-reminder gate สิ่งนี้
- **Backend neutrality** — feature CLI ทุกตัวทำงานกับ backend 4 ตัวที่ประกาศ; `/check-neutrality` gate สิ่งนี้สำหรับ `AGENTS.md`
- **Doc-version + software-version คงสอดคล้อง** — ทั้งคู่ stamped อัตโนมัติทุก edit ของ Claude Code
- **Open-source readiness** — artifact ทุกตัวที่ contributor สาธารณะต้องการ (CONTRIBUTING, SECURITY, CoC, LICENSE, VERSION, CHANGELOG, VISION, ROADMAP) up to date และถูกต้อง

---

## สิ่งที่ไม่ใช่เป้าหมาย

ดู [`VISION.th.md` §สิ่งที่ไม่ใช่เป้าหมาย](../../VISION.th.md#สิ่งที่ไม่ใช่เป้าหมาย) สรุป: BWOC ไม่ใช่ศาสนา, ไม่ใช่ runtime/SDK/LLM, ไม่ใช่ตัวแทนของ DDD / Clean Architecture / SOLID, ไม่เอนเอียง vendor, และไม่ใช่กรอบเพิ่มผลผลิต

---

## ดูเพิ่ม

- [`VERSION.md`](../../VERSION.md) — version ปัจจุบันและ SemVer policy
- [`VISION.th.md`](../../VISION.th.md) — success criteria ที่ 1 ปีและ 3 ปี
- [`CHANGELOG.md`](../../CHANGELOG.md) — อะไร ship แล้ว เมื่อไหร่
- [`ARCHITECTURE.th.md`](ARCHITECTURE.th.md) — ส่วนประกอบทำงานร่วมกันอย่างไร
