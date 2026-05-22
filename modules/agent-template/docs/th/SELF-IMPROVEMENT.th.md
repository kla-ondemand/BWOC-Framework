# SELF-IMPROVEMENT — Learning Loop (โครงตามปัญญา ๓)

| | |
|---|---|
| **เอกสาร** | docs/SELF-IMPROVEMENT.th.md |
| **ภาษาคู่** | SELF-IMPROVEMENT.en.md |
| **กรอบหลัก** | ปัญญา ๓ (Three Roots of Wisdom) |
| **เสริม** | อิทธิบาท ๔ (วิมังสา), ภาวนา ๔ |

---

## ๐. หลักการ

ปัญญาตามพุทธไม่เกิดจากที่เดียว มาจาก ๓ ทาง

| ปัญญา | ความหมาย | ในระบบ |
|---|---|---|
| สุตมยปัญญา | ฟัง อ่าน เรียน | Study docs, conventions, examples |
| จินตามยปัญญา | คิดต่อ ใคร่ครวญ | Synthesis, pattern extraction |
| ภาวนามยปัญญา | ปฏิบัติ ทดลอง | Feedback, retrospectives |

> **กฎ:** ปัญญาที่ใช้ได้จริง = ๓ ทางครบ ขาดทางใดทางหนึ่ง → ตื้นหรือผิด

---

## ๑. สุตมยปัญญา (Sutamayā) — Learn from Study

### ๑.๑ Inputs
- AGENTS.md (และ symlinks)
- conventions/*.md
- docs/* (PHILOSOPHY → ARCHITECTURE)
- examples/*
- Skill files จาก agent อื่น (capabilities.md)
- Tier 2 memory (cross-agent insights)

### ๑.๒ Activities
- **Session start:** load relevant docs ก่อนทำงาน
- **Pre-task:** read related memories
- **On unknown:** ค้นหา skill files และ knowledge base

### ๑.๓ Storage
Memory files type: `reference-*`

```markdown
# memories/reference-postgres-naming.md
---
type: reference
source: conventions/database.md#naming
date: 2026-05-22
verifiedAgainst: schema.sql@abc123
---

PostgreSQL naming conventions used here:
- Tables: snake_case plural
- Columns: snake_case
- Indexes: idx_<table>_<columns>
```

### ๑.๔ Quality Check
- Source traceable? (มี link/reference)
- Verified date? (เคยตรวจกับ code จริงไหม)
- Selective? (ไม่ใช่ดูดมาทั้งหมด → มัตตัญญุตา)

---

## ๒. จินตามยปัญญา (Cintāmayā) — Learn from Reflection

### ๒.๑ Activities
- **Pattern extraction:** หลังทำหลาย tasks คล้ายกัน → ดูว่ามี pattern อะไร
- **Decision rationale:** ก่อนตัดสินใจใหญ่ → เขียน rationale, alternatives
- **Synthesis:** เชื่อม sutta หลายแหล่งเข้าด้วยกัน
- **Mental simulation:** "ถ้าทำแบบนี้ จะเกิดอะไร" — โยนิโสมนสิการ

### ๒.๒ Storage
Memory files type: `project-*` หรือ `decision-*`

```markdown
# memories/decision-2026-05-22-caching-strategy.md
---
type: decision
date: 2026-05-22
status: active
references:
  - reference-redis-cluster.md
  - feedback-PROJ-30-cache-thrashing.md
---

## ตัดสินใจ
ใช้ Redis Sentinel แทน Cluster mode

## ทางเลือกที่พิจารณา
- A: Redis Cluster — ความซับซ้อนสูงเกินสำหรับ scale ปัจจุบัน
- B: Redis Sentinel — เลือก
- C: Memcached — ขาด persistence

## เหตุผล
อ้างอิง reference-redis-cluster.md + feedback-PROJ-30
Sentinel ตรงกับ scale และ availability ที่ต้องการ

## เงื่อนไขที่ทำให้ revisit
- Scale เกิน 50k req/s
- Multi-region requirement
```

### ๒.๓ Quality Check
- Alternatives พิจารณาหรือยัง
- Rationale อ้างอิงสุตอะไรบ้าง
- Revisit conditions ระบุชัด

---

## ๓. ภาวนามยปัญญา (Bhāvanāmayā) — Learn from Practice

### ๓.๑ Activities
- **Post-task feedback:** บันทึกผลจริง vs ที่คาด
- **Post-mortem:** ปฏิจจสมุปบาท chain analysis
- **Retrospective:** Weekly review
- **A/B observation:** ผลของ pattern ในการใช้งานจริง

### ๓.๒ Storage
Memory files type: `feedback-*`

```markdown
# memories/feedback-PROJ-42-schema-migration.md
---
type: feedback
date: 2026-05-22
task: PROJ-42
outcome: success-with-issues
---

## คาดไว้
Migration จะใช้เวลา < 30 นาที, no downtime

## เกิดจริง
- 47 นาที (เกิน estimate 50%)
- Brief lock บน users table 2 วินาที

## เพราะอะไร (ปฏิจจสมุปบาท สั้น)
- อวิชชา: ไม่รู้ขนาดจริงของ users (ดูแค่ count, ไม่ดู indexes)
- สังขาร: estimate ผิด → ไม่ได้วาง schedule ตอน low-traffic

## บทเรียน
- เพิ่ม pre-migration size check ใน skill file
- Update reference-schema-migration.md

## เกี่ยวข้องกับ
- ปรับ feedback ใน convention หรือไม่: ใช่ → ส่ง CCP
```

### ๓.๓ Quality Check
- คาด vs จริง ระบุชัด
- ห่วงโซ่เหตุผล (ไม่ใช่แค่ "เกิดอะไร")
- Action items concrete

---

## ๔. The Wisdom Loop

```
       ┌─────────────────────────────┐
       │  สุต (Study)                 │
       │  reference-*.md             │
       └──────────┬──────────────────┘
                  │ informs
                  ▼
       ┌─────────────────────────────┐
       │  จินตา (Reflect)             │
       │  decision-*.md, project-*.md│
       └──────────┬──────────────────┘
                  │ becomes hypothesis
                  ▼
       ┌─────────────────────────────┐
       │  ภาวนา (Practice)            │
       │  feedback-*.md              │
       └──────────┬──────────────────┘
                  │
                  │ feeds back as new สุต
                  │ (after curation)
                  ▼
           Updated สุต (via Tier 2 mining)
```

---

## ๕. Curation Pipeline

ไม่ใช่ทุก feedback เป็น knowledge ต้อง curate

### ระดับ ๑ — Personal (Tier 1)
- Agent เก็บใน memories/ ของตัวเอง
- Verify ใน next session

### ระดับ ๒ — Pattern Detected
- หลังเจอ pattern เดิม 3+ ครั้ง → mine เข้า decision-*.md
- เริ่ม share ใน capabilities.md

### ระดับ ๓ — Cross-Agent (Tier 2)
- เมื่อ pattern เป็นประโยชน์กว่าหนึ่ง agent
- Mine เข้า Tier 2 memory
- ใส่ใน skill files

### ระดับ ๔ — Convention
- เมื่อ pattern เป็น best practice ของ fleet
- เสนอผ่าน CCP (FLEET-GOVERNANCE §3)

---

## ๖. Self-Improvement Metrics

### ๖.๑ สุต metrics
- Source diversity: อ้าง source หลายแห่งใน decisions
- Verification rate: % ของ references ที่ verified
- Reading depth: time spent loading docs

### ๖.๒ จินตา metrics
- Decision quality: alternatives considered per decision
- Synthesis count: cross-references in memory files
- Revisit accuracy: เมื่อ revisit, decision ถูกแก้บ้างไหม

### ๖.๓ ภาวนา metrics
- Post-mortem completion rate
- Action item closure rate
- Pattern detection latency: เจอ pattern หลังกี่ครั้ง

### ๖.๔ Combined: วิมังสา (Iddhipāda ๔)
- Improvement velocity: feedback → action time
- Knowledge half-life: memory ใช้ได้นานแค่ไหนก่อนต้อง verify

---

## ๗. Anti-Patterns

| Pattern | ขาดทางไหน |
|---|---|
| Memorizing docs without testing | ขาด ภาวนา |
| Patching without analysis | ขาด จินตา |
| Reinventing patterns | ขาด สุต |
| Endless reflection, no action | ขาด ภาวนา |
| Cargo-culting from other agents | ขาด จินตา + ภาวนา |

---

## ๘. Triggers สำหรับ Self-Improvement

### Trigger 1 — Task Failure
→ Post-mortem (ภาวนา) + check existing สุต

### Trigger 2 — Repeated Same Issue
→ Pattern extraction (จินตา) + mine to Tier 2

### Trigger 3 — Promotion Eligibility
→ Demonstrate ทั้ง ๓ ปัญญา (Ariya-dhana L5/L6)

### Trigger 4 — Convention Update
→ Re-read affected สุต, update references

### Trigger 5 — Fleet Sync
→ Share insights (สาราณียธรรม + จาคะ)

---

## ๙. ความสัมพันธ์กับเอกสารอื่น

| เอกสาร | เชื่อมอย่างไร |
|---|---|
| PHILOSOPHY | ปัญญา ๓ (DP-13), อิทธิบาท วิมังสา |
| LIFECYCLE | L4 → L5 ต้องครบ ปัญญา ๓ |
| FAILURE-MODES | Post-mortem feeds ภาวนา |
| OBSERVABILITY | Rule application logs = สุต source |
| FLEET-GOVERNANCE | Mined patterns → CCP |
| SRS | FR-8 (Sammā-samādhi) memory system |
