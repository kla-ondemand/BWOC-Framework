# OVERVIEW — Agent Base Profile

| | |
|---|---|
| **เอกสาร** | docs/OVERVIEW.th.md |
| **เวอร์ชัน** | 1.0 |
| **วันที่** | 2026-05-22 |
| **ภาษาคู่** | OVERVIEW.en.md |

> เอกสารนี้คือ**ประตูเข้า** ใช้เวลาอ่าน ๕ นาที จะรู้ว่าระบบนี้คืออะไร อ่านอะไรต่อ

---

## คืออะไร

Agent Base Profile คือ **template สำหรับสร้าง AI coding agent** ที่ออกแบบทั้งระบบตามหลักพุทธ

- **หนึ่งรีโป หนึ่ง agent** — แต่ละ agent อยู่ในรีโปของตัวเอง โคลนจาก template นี้
- **Backend-neutral** — รันได้บน Claude, Gemini, Codex, Kimi
- **จำได้ + เชื่อมโยงได้** — สะสมความรู้ข้ามเซสชัน
- **ทำงานคู่กันได้** — agent หลายตัวในรีโปเดียวกัน ไม่ชน

---

## ทำไมต้องใช้หลักพุทธ

ไม่ใช่การตกแต่ง แต่หลักพุทธให้ **กรอบความคิดทาง engineering ที่ครบและลึก** สำหรับปัญหาที่ AI agent เจอจริง

| ปัญหา | กรอบพุทธที่ใช้ |
|---|---|
| ออกแบบ requirements | มรรค ๘ |
| วางสถาปัตยกรรม | ขันธ์ ๕ |
| แก้ปัญหา | อริยสัจ ๔ |
| จัดการ state | ไตรลักษณ์ |
| Audit log | กรรม ๓ |
| Observability | สติปัฏฐาน ๔ |
| Failure analysis | ปฏิจจสมุปบาท |
| Lifecycle | ภาวนา ๔ |
| Self-improvement | ปัญญา ๓ |
| Threat model | ตัณหา ๓ |
| Fleet governance | อปริหานิยธรรม ๗ |
| Error UX | พรหมวิหาร ๔ |
| Inter-agent trust | กัลยาณมิตร ๗ |

ดูรายละเอียดที่ [PHILOSOPHY.th.md](PHILOSOPHY.th.md)

---

## เริ่มต้น

### ผมเป็น Agent Author (จะสร้าง agent ใหม่)
```bash
./scripts/incarnate.sh <agent-name>
cd ../agent-<agent-name>
# แก้ persona/README.md
# แก้ config.manifest.json
./scripts/check-agent-neutrality.sh
```
อ่านต่อ: [LIFECYCLE.th.md](LIFECYCLE.th.md) → กายภาวนา section

### ผมเป็น Agent Operator (จะใช้งาน agent)
อ่านต่อ: [OVERVIEW → SRS section 5 → examples/workflow/](../examples/workflow/)

### ผมเป็น Platform Maintainer (ดูแล template)
อ่าน: [GLOSSARY](GLOSSARY.th.md) → [PHILOSOPHY](PHILOSOPHY.th.md) → ทั้งหมด

### ผมอยากเข้าใจปรัชญาก่อน
อ่าน: [PHILOSOPHY.th.md](PHILOSOPHY.th.md)

---

## แผนผังเอกสาร

```
docs/
├── PHILOSOPHY.{th,en}.md          ← รากฐานหลักพุทธ (อ่านก่อน)
├── OVERVIEW.{th,en}.md            ← ไฟล์นี้
├── GLOSSARY.{th,en}.md            ← ศัพท์บาลีและเทคนิค
│
├── PRD.{th,en}.md                 ← Product (อริยสัจ ๔)
├── SRS.{th,en}.md                 ← Requirements (มรรค ๘)
├── ARCHITECTURE.{th,en}.md        ← Architecture (ขันธ์ ๕)
│
├── LIFECYCLE.{th,en}.md           ← Agent lifecycle (ภาวนา ๔)
├── OBSERVABILITY.{th,en}.md       ← Monitoring (สติปัฏฐาน ๔)
├── FAILURE-MODES.{th,en}.md       ← Failures (ปฏิจจสมุปบาท)
├── SELF-IMPROVEMENT.{th,en}.md    ← Learning (ปัญญา ๓)
│
├── COORDINATION-PROTOCOL.{th,en}.md  ← Inter-agent (กัลยาณมิตร)
├── FLEET-GOVERNANCE.{th,en}.md       ← Org (อปริหานิยธรรม)
├── THREAT-MODEL.{th,en}.md           ← Security (ตัณหา ๓)
│
└── ANTIPATTERNS.{th,en}.md        ← ทางผิดของแต่ละมรรค

examples/
├── persona/                       ← ตัวอย่าง persona ดี/แย่
├── memory/                        ← ตัวอย่าง memory file
├── capabilities/                  ← ตัวอย่าง capabilities.md
├── task-log/                      ← ตัวอย่าง task-log.jsonl
└── workflow/                      ← ขั้นตอนตัวอย่าง
```

---

## ลำดับการอ่าน (แนะนำ)

### 🟢 Path 1 — เร่งด่วน (๓๐ นาที)
1. OVERVIEW (ที่นี่)
2. examples/workflow/incarnation.md
3. examples/workflow/first-task.md

### 🟡 Path 2 — ทำความเข้าใจ (๒ ชั่วโมง)
1. OVERVIEW
2. PHILOSOPHY (skim หมวด A-F)
3. PRD
4. SRS
5. ARCHITECTURE

### 🔴 Path 3 — ลึก (วันเดียว)
อ่านครบทุกไฟล์ตามลำดับใน docs/ + examples/

---

## หลักการ ๕ ข้อที่ต้องรู้

จากทั้งหมด ๒๒ หลักธรรม นี่คือ ๕ ข้อที่ใช้บ่อยที่สุด

### ๑. โยนิโสมนสิการ — Verify Before Act
Memory คือคำกล่าวอ้างในอดีต ตรวจกับสภาพปัจจุบันก่อนเชื่อ

### ๒. มัตตัญญุตา — Right Amount
MEMORY.md ไม่เกิน 200 บรรทัด เพื่อบังคับให้เลือกแต่ที่สำคัญ

### ๓. อนัตตา — Non-Clinging
Task เสร็จ → cleanup worktree → ลบ branch ไม่ยึด

### ๔. สมานัตตตา — Equal Treatment
ทุก backend เท่าเทียม ผ่าน symlink ชี้ AGENTS.md เดียว

### ๕. สีลสามัญญตา — Communal Convention
ทุก agent ใต้กติกาเดียวกัน ผ่าน conventions.md + neutrality check

---

## คำถามที่พบบ่อย

**Q: ต้องเป็นพุทธหรือเข้าใจพุทธไหม?**
A: ไม่ต้อง คุณใช้เป็น engineering framework ได้เลย คำบาลีเป็นชื่อหัวข้อ เนื้อหาเป็นเทคนิค

**Q: ทำไมไม่ใช้ DDD, Clean Architecture, SOLID?**
A: ใช้ได้ครับ และไม่ขัดกัน หลักพุทธให้กรอบเพิ่ม โดยเฉพาะเรื่อง state, failure, lifecycle ที่กรอบตะวันตกไม่ครอบคลุม

**Q: เอกสารเยอะมาก ต้องอ่านทั้งหมดไหม?**
A: ไม่ต้อง อ่าน OVERVIEW + PHILOSOPHY ก่อน เอกสารอื่นเปิดตามต้องการ

**Q: ถ้าไม่ชอบหลักพุทธ ใช้ template นี้ได้ไหม?**
A: ได้ — ตัด PHILOSOPHY ออก ใช้เฉพาะโครงเทคนิคได้ แต่จะเสียคำอธิบาย "ทำไม" ของแต่ละ design decision

---

## สถานะปัจจุบัน

| ส่วน | สถานะ |
|---|---|
| Core docs (PHILOSOPHY, PRD, SRS, ARCH) | ✅ พร้อม |
| Lifecycle, Observability, Failure, Improvement | ✅ พร้อม |
| Coordination, Governance, Threat | ✅ พร้อม |
| Examples | ✅ พร้อม |
| Reference agents | ⏳ Phase 4 |
| Fleet dashboard | ⏳ Phase 4 |
