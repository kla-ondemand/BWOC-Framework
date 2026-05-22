# Roadmap

แผนทีละ phase ของ BWOC **Phase** อธิบาย milestone ของการ implement; แต่ละ phase อาจครอบคลุม SemVer release หลายครั้ง ดู [`VERSION.md`](../../VERSION.md) สำหรับการแยก version กับ phase ดู [`VISION.th.md`](../../VISION.th.md) สำหรับ success criteria ที่ ๑ ปีและ ๓ ปี

---

## สถานะปัจจุบัน

**Phase ที่ active:** Phase 1 v2.0 — *รากฐาน อุปฺปาท* — กำลังดำเนินการ
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

### กำลังทำ

- การ implement คำสั่ง `bwoc-cli`
- Runtime `bwoc-agent` (Phase 1 DoD: "I am alive" อ่าน `config.manifest.json`)
- โครงสร้าง workspace บน disk (`.bwoc/workspace.toml`, `.bwoc/agents.toml`)
- Central memory directory `~/.bwoc/`

### ส่งมอบใน Phase 1 v2.0 (เสร็จแล้ว)

รายการทั้งหมดด้านล่าง implement แล้ว Definition of Done ของ phase นี้ (uppāda end-to-end สำหรับ backend หนึ่ง) บรรลุ เหลือเฉพาะ HELD policy items (CODEOWNERS · ISSUE_TEMPLATE/config.yml) และการตัดสินใจของผู้ใช้เรื่อง release-tag

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

- Control socket ของ `bwoc-agent` — expose `status`, `log`, `send` ให้ CLI
- คำสั่ง `bwoc status` · `log` · `send`
- Process supervision จริง: จัดการ signal, restart-on-crash, health check
- Memory ระดับ workspace (`<workspace>/.bwoc/memory/`)
- Validation ข้าม backend: uppāda + ṭhiti เต็มกับ Claude, Gemini, Codex, และ Kimi CLI (สมานัตตตา ในทางปฏิบัติ)
- Release pipeline GitHub Actions: matrix build สำหรับ macOS · Linux · Windows; binary ที่ sign; checksum; GitHub Release
- เครื่องมือ memory mining และ interface Tier 2 backend ที่ pluggable

---

## Phase 3 — วยะ + Interconnect

**นิยามของเสร็จ:** ชีวิตของ agent จบลงอย่างสะอาด; agent ประสานงานโดยไม่มีศูนย์กลาง

- `bwoc stop <name>` — หยุดอย่างนุ่มนวลพร้อม signal escalation
- `bwoc retire <name>` — vaya เต็มรูปแบบ: ล้าง worktree, ปล่อย branch, ตัด memory, ลบออกจาก registry
- `bwoc workspace prune` — เก็บกวาด entry ของ agent ที่ลอย
- Inter-agent messaging — channel สัมมาวาจา; กฎ Sāraṇīyadhamma 6 ของความนุ่มนวล
- Trust scoring — Kalyāṇamitta 7 ใช้กับการประกาศ capability และที่มาของข้อความ
- Config routing ระดับ workspace `.bwoc/interconnect/`
- Reference implementation ของ Tier 2 memory backend

---

## Phase 4 — Reference Agent + Fleet

**นิยามของเสร็จ:** ความเป็นไปได้ของ ecosystem พิสูจน์แล้ว; governance ของ fleet ระดับ production ข้าม vendor ทำได้

- Agent อ้างอิงสามตัวหรือมากกว่าในธรรมชาติ สร้างโดยผู้ดูแลนอกทีมผู้เขียนต้นฉบับ (ตาม [`VISION.th.md`](../../VISION.th.md) success ที่ ๑ ปี)
- Fleet dashboard — Aparihāniya-dhamma 7 governance ใช้กับการติดตั้ง multi-agent จริง
- ศัพท์ BWOC (Yoniso manasikāra checks, Mattaññutā caps, Sīla baselines, Kalyāṇamitta trust scores) ปรากฏใน codebase ที่ไม่มีความสัมพันธ์กับ project นี้ (success ที่ ๓ ปี)
- รูปแบบ fleet ระดับ production ข้าม vendor ใช้ในองค์กรมากกว่าหนึ่งแห่ง

---

## ข้ามทุก Phase

- **Bilingual parity** — เอกสารสเปกทุกฉบับมี EN canonical + TH (และภาษาอื่น ๆ ในอนาคต); hook bilingual-reminder gate สิ่งนี้
- **Backend neutrality** — feature CLI ทุกตัวทำงานกับ backend ๔ ตัวที่ประกาศ; `/check-neutrality` gate สิ่งนี้สำหรับ `AGENTS.md`
- **Doc-version + software-version คงสอดคล้อง** — ทั้งคู่ stamped อัตโนมัติทุก edit ของ Claude Code
- **Open-source readiness** — artifact ทุกตัวที่ contributor สาธารณะต้องการ (CONTRIBUTING, SECURITY, CoC, LICENSE, VERSION, CHANGELOG, VISION, ROADMAP) up to date และถูกต้อง

---

## สิ่งที่ไม่ใช่เป้าหมาย

ดู [`VISION.th.md` §สิ่งที่ไม่ใช่เป้าหมาย](../../VISION.th.md#สิ่งที่ไม่ใช่เป้าหมาย) สรุป: BWOC ไม่ใช่ศาสนา, ไม่ใช่ runtime/SDK/LLM, ไม่ใช่ตัวแทนของ DDD / Clean Architecture / SOLID, ไม่เอนเอียง vendor, และไม่ใช่กรอบเพิ่มผลผลิต

---

## ดูเพิ่ม

- [`VERSION.md`](../../VERSION.md) — version ปัจจุบันและ SemVer policy
- [`VISION.th.md`](../../VISION.th.md) — success criteria ที่ ๑ ปีและ ๓ ปี
- [`CHANGELOG.md`](../../CHANGELOG.md) — อะไร ship แล้ว เมื่อไหร่
- [`ARCHITECTURE.th.md`](ARCHITECTURE.th.md) — ส่วนประกอบทำงานร่วมกันอย่างไร
