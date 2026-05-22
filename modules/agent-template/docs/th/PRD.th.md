# PRD — Product Requirements Document

## Agent Base Profile (โครงตามอริยสัจ ๔)

| | |
|---|---|
| **เอกสาร** | PRD.th.md |
| **เวอร์ชัน** | 2.0 |
| **วันที่** | 2026-05-22 |
| **ภาษาคู่** | PRD.en.md |
| **อ้างอิงปรัชญา** | PHILOSOPHY.th.md |
| **อ้างอิงต้นแบบ** | `kla-ondemand/atlas-agent-oracle-template` |

> **โครงเอกสาร** จัดตามอริยสัจ ๔ — ทุกข์ สมุทัย นิโรธ มรรค
> โดยมีส่วนเสริมตาม สัปปุริสธรรม ๗ (รู้บริบท), สังคหวัตถุ ๔ (UX), อิทธิบาท ๔ (success metrics), ไตรลักษณ์ (constraints)

---

## ภาค ๑ : ทุกข์ (Dukkha) — ปัญหาที่ระบบนี้แก้

### ๑.๑ บทสรุปผู้บริหาร

Agent Base Profile คือ **template เปล่าสำหรับสร้าง AI coding agent** ที่เน้นการ "จำได้และเชื่อมโยงได้" หนึ่งรีโป หนึ่ง agent โดยแต่ละ agent โคลนมาจาก template นี้แล้วเติม persona, skills, mindset ของตนเอง

ระบบออกแบบตามหลักพุทธทั้งหมด — ดูรายละเอียดที่ PHILOSOPHY.th.md

### ๑.๒ ทุกข์ ๔ ประการของวงการ AI Coding Agent ปัจจุบัน

#### ทุกข์ ๑ — ความหลงลืม (Amnesia)
แต่ละ session เริ่มจากศูนย์ บทเรียน การตัดสินใจ context ที่สั่งสมไว้หายเมื่อปิดแชต ผู้ใช้ต้องเล่าซ้ำ ๆ

#### ทุกข์ ๒ — การติดยี่ห้อ (Vendor Lock-in)
Agent ผูกกับ tooling ของ LLM provider เจ้าเดียว เปลี่ยน provider = เขียนใหม่หมด

#### ทุกข์ ๓ — การชนกัน (Multi-Agent Collision)
หลาย agent ทำงานในรีโปเดียวกันชน branch กัน ใช้ `git stash` ส่งต่อ state กันมั่ว ปะปนกัน

#### ทุกข์ ๔ — ความกระจัดกระจาย (Structural Chaos)
แต่ละ agent มี layout ของตัวเอง ไม่มีมาตรฐาน ทำให้ประสานงานระหว่าง agent ไม่ได้

---

## ภาค ๒ : สมุทัย (Samudaya) — สาเหตุของปัญหา

### ๒.๑ การวิเคราะห์ root cause

| ทุกข์ | สมุทัยตื้น | สมุทัยลึก |
|---|---|---|
| หลงลืม | ไม่มี persistent storage | ขาด "สัมมาสติ" — ไม่มีกลไกระลึกข้ามเซสชัน |
| ติดยี่ห้อ | Tooling ผูก backend เฉพาะ | ขาด "สมานัตตตา" — ไม่ปฏิบัติทุก backend เสมอกัน |
| ชนกัน | ใช้ working directory ร่วม | ขาด "อนัตตา" — ยึดติด branch ของตน |
| กระจัดกระจาย | ไม่มี convention กลาง | ขาด "สีลสามัญญตา" — ไม่มีกติกาเดียวกัน |

### ๒.๒ ตัณหา (Craving) ที่ก่อทุกข์
- ตัณหาในความสะดวกระยะสั้น → ใช้ `git stash` แทนการแยก worktree
- ตัณหาในความเป็นเจ้าของ → ยึด branch, ไม่ cleanup
- ตัณหาในของเก่า → เชื่อ memory โดยไม่ verify
- ตัณหาในขอบเขตที่กว้าง → ทำงานนอก scope (มัตตัญญุตา)

---

## ภาค ๓ : นิโรธ (Nirodha) — ภาพความสำเร็จ

### ๓.๑ Vision Statement
AI coding agent ทุกตัวในองค์กรใช้โครงเดียวกัน ความรู้สั่งสมได้ agent หลายตัวร่วมงานได้แบบฝูง ไม่ใช่เครื่องมือเดี่ยว ๆ

### ๓.๒ ภาพความสำเร็จที่วัดได้

| เมื่อสำเร็จแล้วจะเป็นแบบนี้ | วัดอย่างไร |
|---|---|
| Agent จำการตัดสินใจจากเซสชันก่อนได้ | ผ่าน prior-decision test ≥ 95% |
| สลับ LLM backend ได้โดยไม่แก้ไฟล์ | Same task → equivalent output บน 4 backends |
| Agent 3 ตัวทำงานพร้อมกัน 0 collisions | CI smoke test ผ่าน 100 task-pairs |
| Agent ใหม่ commit แรกได้ใน 30 นาที | จับเวลา onboarding |
| Agent ประสานงานกันได้ผ่าน protocol | 2 agents consensus exchange สำเร็จ |

### ๓.๓ ฉันทะของระบบ (System Aspiration)
- **ฉันทะในการจำ** — เก็บเฉพาะที่จำเป็น คุณภาพมากกว่าปริมาณ
- **ฉันทะในการเชื่อมโยง** — หาความสัมพันธ์ระหว่าง context
- **ฉันทะในการปล่อยวาง** — ทำเสร็จแล้วไม่ยึด

---

## ภาค ๔ : มรรค (Magga) — ทางแห่งความสำเร็จ

### ๔.๑ มรรค ๘ ทาง (รายละเอียดเต็มใน SRS)

| มรรค | Pillar ในระบบ |
|---|---|
| สัมมาทิฏฐิ | Persona & Identity ที่ชัดเจน |
| สัมมาสังกัปปะ | Task planning ด้วยอริยสัจ |
| สัมมาวาจา | Inter-agent communication protocol |
| สัมมากัมมันตะ | Worktree isolation + scoped commits |
| สัมมาอาชีวะ | Trust model + neutrality |
| สัมมาวายามะ | Verification gates (ปธาน ๔) |
| สัมมาสติ | Memory system Tier 1 + Tier 2 |
| สัมมาสมาธิ | Session lifecycle ที่ตั้งมั่น |

### ๔.๒ Phased Roadmap ตาม ภาวนา ๔

| Phase | ปธาน | งาน |
|---|---|---|
| Phase 1 | สังวรปธาน (กันชั่วใหม่) | MVP: AGENTS.md + symlinks + Tier 1 memory + worktrees + scripts |
| Phase 2 | ปหานปธาน (ละชั่วเก่า) | ลบ legacy patterns: ห้าม stash, ห้าม shared dir, enforce conventions |
| Phase 3 | ภาวนาปธาน (สร้างดีใหม่) | Tier 2 memory, interconnect, self-improvement loop |
| Phase 4 | อนุรักขนาปธาน (รักษาดี) | Reference agent gallery, fleet dashboards, signed templates |

---

## ภาค ๕ : สัปปุริสธรรม ๗ — รู้บริบท ๗ มิติ

### ๕.๑ Stakeholder & Context Analysis

#### ธัมมัญญุตา — รู้เหตุ
หลักการเบื้องหลังคือ "remember and connect" ไม่ใช่ "do tasks faster" agent มีหน้าที่สั่งสมและเชื่อมโยงความรู้

#### อัตถัญญุตา — รู้ผล
ผลปลายทาง: ลด onboarding cost ของ agent ใหม่, ลด context-rebuild cost ของผู้ใช้, เปิดทางสู่ multi-agent collaboration

#### อัตตัญญุตา — รู้ตน (Personas)

| Persona | บทบาท | ความต้องการหลัก |
|---|---|---|
| **Agent Author** | ผู้สร้าง agent ใหม่ | Clone-and-customize เร็ว |
| **Agent Operator** | ผู้ใช้งาน agent จริง | Predictable, no collision, persistent memory |
| **Platform Maintainer** | ผู้ดูแล template | Validation tools, trust model |
| **LLM CLI** | ผู้บริโภค template ที่ runtime | Backend-neutral instructions |

#### มัตตัญญุตา — รู้ประมาณ (Non-Goals)
- **ไม่** เป็น runtime หรือ orchestrator
- **ไม่** มี memory database ในตัว
- **ไม่** เป็น agent marketplace
- **ไม่** ผูกกับภาษาโปรแกรมใดภาษาเดียว
- **ไม่** ทำ chat UI

#### กาลัญญุตา — รู้กาล
- ใช้เมื่อจะสร้าง agent ใหม่ที่ต้องจำข้ามเซสชัน
- ไม่ใช้สำหรับ one-shot script หรือ ad-hoc query
- ไม่ใช้กับ agent ที่ไม่มี repo

#### ปริสัญญุตา — รู้ชุมชน
ชุมชนเป้าหมายคือทีมที่ใช้ AI coding assistant หลายตัว ในรีโปร่วม ภายในองค์กรเดียว มีวินัย git workflow และต้องการ governance

#### ปุคคลัญญุตา — รู้บุคคล
ผู้ใช้คือ developer/AI engineer ที่คุ้นกับ git, markdown, shell, JSON ไม่ใช่ผู้ใช้ทั่วไป

---

## ภาค ๖ : สังคหวัตถุ ๔ — หลัก UX

### ๖.๑ ทาน (Generosity)
- Default config ครบ ใช้ได้เลย
- Skills, mindsets, persona scaffolds ให้พร้อม
- ตัวอย่างชัดเจน

### ๖.๒ ปิยวาจา (Pleasant Communication)
- Error messages ระบุ placeholder ที่ขาดด้วยชื่อ
- README ขั้นตอนชัด
- เอกสาร bilingual

### ๖.๓ อัตถจริยา (Beneficial Action)
- Script ไม่ใช่แค่ทำงาน แต่ตรวจสอบให้ด้วย
- Validation บอกว่าผิดอะไร แก้ยังไง

### ๖.๔ สมานัตตตา (Equanimity / Equality)
- ทุก backend เท่าเทียม ผ่าน symlink ชี้ AGENTS.md เดียวกัน
- ไม่มี "primary backend"

---

## ภาค ๗ : อิทธิบาท ๔ — Success Metrics

| ธรรม | KPI |
|---|---|
| **ฉันทะ** | Agent author satisfaction score ≥ 4/5 |
| **วิริยะ** | Task completion rate ≥ 90%; retry-on-fail enabled |
| **จิตตะ** | Verification gate compliance 100% on merged PRs |
| **วิมังสา** | Self-improvement metrics tracked; monthly skill progression |

### KPI เสริม

| Metric | Target |
|---|---|
| Time-to-first-commit (agent ใหม่) | ≤ 30 นาที |
| Branch collision rate | 0 ใน 100 task-pairs |
| Backend portability | 4 backends, identical behavior |
| Memory recall accuracy | ≥ 95% |
| Neutrality check pass rate | 100% on official templates |
| Task-log completeness | 100% |

---

## ภาค ๘ : ไตรลักษณ์ — ข้อจำกัดและการปล่อยวาง

### ๘.๑ อนิจจัง — ทุกอย่างเปลี่ยน
- Memory มี timestamp และ pruning policy
- Convention เปลี่ยนได้ผ่าน versioning ของ template
- Backend ที่รองรับวันนี้อาจไม่มีในอนาคต — ต้องออกแบบให้เพิ่ม/ลดได้

### ๘.๒ ทุกขัง — สภาพเดิมทนไม่ได้
- Stale branch = ทุกข์ → cleanup ตามจังหวะ
- Stale memory = ทุกข์ → mine เข้า Tier 2 หรือลบ
- Stale convention = ทุกข์ → review ตามรอบ

### ๘.๓ อนัตตา — ไม่ใช่ตัวตน
- Agent ไม่เป็นเจ้าของ branch หรือ worktree
- Template ไม่ใช่ "ของใคร" — ทุก fork เท่ากัน
- Memory ไม่ใช่ "ความจริง" — code ปัจจุบันคือความจริง

---

## ภาค ๙ : อจินไตย — สิ่งที่ไม่ทำในเอกสารนี้

| อจินไตย | สิ่งที่ไม่ทำ |
|---|---|
| พุทธวิสัย | ไม่ระบุว่า LLM provider ไหนดีกว่ากัน |
| ฌานวิสัย | ไม่อธิบาย internals ของ LLM |
| กรรมวิบาก | ไม่คาดการณ์ผลทางธุรกิจระยะยาว |
| โลกจินตา | ไม่กำหนด AI ethics, governance ระดับโลก |

---

## ภาค ๑๐ : ความเสี่ยง (Risks) และการรับมือ

| ความเสี่ยง | ลักษณะพุทธ | การรับมือ |
|---|---|---|
| Symlink พังบน Windows | อนิจจัง — env เปลี่ยน | Document WSL workaround |
| Agent ข้าม worktree isolation | ตัณหาในความเร็ว | Hook in `.claude/settings.json` ขัดขวาง |
| Memory โตไม่หยุด | ตัณหาในการเก็บ | 200-line cap (มัตตัญญุตา) + Tier 2 |
| Forked template เลือนออกจาก neutrality | สังขาร = เปลี่ยนแปลง | `check-agent-neutrality.sh` ใน CI |
| Placeholder ค้างไม่ถูกแทนที่ | ความประมาท | Manifest-driven validation |

---

## ภาค ๑๑ : คำถามเปิด (Open Questions)

1. Agent ควรสามารถ fork ตัวเองกลางทาง task ได้ไหม
2. Schema มาตรฐานของ inter-agent message คืออะไร
3. Template ควรใช้ semver ในการ version หรือไม่
4. ควรมี signed template (sigstore-style) หรือไม่

---

## ภาคผนวก A : สรุปการ Mapping กับ PHILOSOPHY

| Section นี้ | หลักธรรม |
|---|---|
| ภาค ๑ ทุกข์ | อริยสัจ ๑ |
| ภาค ๒ สมุทัย | อริยสัจ ๒ |
| ภาค ๓ นิโรธ | อริยสัจ ๓ |
| ภาค ๔ มรรค | อริยสัจ ๔ + มรรค ๘ |
| ภาค ๕ บริบท | สัปปุริสธรรม ๗ |
| ภาค ๖ UX | สังคหวัตถุ ๔ |
| ภาค ๗ KPI | อิทธิบาท ๔ |
| ภาค ๘ Constraints | ไตรลักษณ์ |
| ภาค ๙ Out of scope | อจินไตย ๔ |
| ภาค ๑๐ Risks | ตัณหา + การรับมือ |
| ภาค ๑๑ Open Q | (เปิด ไม่ผูกหลักใดเฉพาะ) |

---

## ภาคผนวก — บันทึกการเปลี่ยนแปลง (Changelog)

### v2.0 (2026-05-22)
- **แก้ไข forced metaphors:** เปลี่ยน `อจินไตย` → `มัตตัญญุตา` ในจุดที่หมายถึง "รู้ประมาณของขอบเขตงาน" (อจินไตย คงไว้เฉพาะ ๔ กรณีต้นฉบับ — Buddha-visaya, Jhāna-visaya, Kamma-vipāka, Loka-cintā)
- **เพิ่มเอกสารคู่ขนาน:**
  - `FAILURE-MODES.md` (ปฏิจจสมุปบาท) — failure analysis
  - `LIFECYCLE.md` (ภาวนา ๔ + อริยทรัพย์ ๗) — agent lifecycle
  - `OBSERVABILITY.md` (สติปัฏฐาน ๔ + กรรม ๓) — monitoring + audit
  - `COORDINATION-PROTOCOL.md` (กัลยาณมิตร ๗ + สาราณียธรรม ๖) — inter-agent
  - `FLEET-GOVERNANCE.md` (อปริหานิยธรรม ๗) — org-level governance
  - `SELF-IMPROVEMENT.md` (ปัญญา ๓) — learning loop
  - `THREAT-MODEL.md` (ตัณหา ๓ + สีล ๕) — security
  - `ANTIPATTERNS.md` (มิจฉาตามมรรค ๘) — wrong-path catalog
  - `GLOSSARY.md` — Pali + technical terms reference
  - `OVERVIEW.md` — entry-point document
- **ขยาย PHILOSOPHY.md** ให้ครอบคลุม ๒๒ หลักธรรม (เดิม ๑๓) ใน ๖ หมวด

### v1.0 (2026-05-22)
- เอกสารเริ่มต้น ๔ ฉบับ (PHILOSOPHY, PRD, SRS, ARCHITECTURE) แบบ bilingual
