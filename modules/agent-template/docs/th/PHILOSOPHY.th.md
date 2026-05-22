# PHILOSOPHY — รากฐานหลักพุทธของ Agent Base Profile (Extended)

| | |
|---|---|
| **เอกสาร** | docs/PHILOSOPHY.th.md |
| **เวอร์ชัน** | 2.0 |
| **วันที่** | 2026-05-22 |
| **ภาษาคู่** | PHILOSOPHY.en.md |
| **สถานะ** | Normative — แกนอ้างอิงของเอกสารทุกฉบับ |

---

## ๐. คำนำ

เอกสารนี้เป็น **แกนกลางทางความคิด** ของระบบ Agent Base Profile ทั้งหมด เอกสารอื่นทุกฉบับอ้างอิงและสอดคล้องกับ PHILOSOPHY นี้ ถ้ามีข้อขัดแย้งระหว่างเอกสาร PHILOSOPHY ชี้ขาด

หลักธรรมในที่นี้ใช้เป็น **กรอบความคิดทางวิศวกรรม** ไม่ใช่การตีความศาสนา ผู้สนใจในแง่ธรรมะลึก ขอให้ศึกษาจากตำราพุทธโดยตรง

---

## ๐.๑ วงรอบ — อุปฺปาท · ฐิติ · วยะ

ก่อนจะถึงหลักธรรม ๒๒ ประการ มีรูปทรงเดียวกันอยู่เบื้องหลัง — ทุกสิ่งที่เป็นสังขตะ (สิ่งปรุงแต่ง) ย่อมมีวงรอบ **อังคุตตรนิกาย ๓.๔๗ (สังขตสูตร)** กล่าวถึงลักษณะสามประการของสังขตะ —

> *อุปฺปาโท ปญฺญายติ* — ความเกิดขึ้น ปรากฏ
> *ฐิตสฺส อญฺญถตฺตํ ปญฺญายติ* — ความแปรไปในขณะที่ยังตั้งอยู่ ปรากฏ
> *วโย ปญฺญายติ* — ความดับ ปรากฏ

ในกรอบ BWOC agent (และทุก task, session, worktree ของมัน) คือสังขตะ — จึงดำเนินตามวงรอบนี้

| ระยะ | บาลี | พื้นผิวเชิงวิศวกรรม |
|---|---|---|
| เกิดขึ้น | **อุปฺปาท** | Incarnation (`incarnate.sh`), การกำหนดบุคลิก, ประกาศความสามารถ (Attanutata), การ resolve manifest |
| ตั้งอยู่ (พร้อมแปรไป) | **ฐิติ** | การทำงาน: วางแผนด้วยอริยสัจ ๔, ลงมือด้วยมรรค ๘, ความทรงจำด้วยสัมมาสติ, การสื่อสารด้วยพรหมวิหาร ๔ การเปลี่ยนแปลง *ภายใน* การคงอยู่ ไม่ใช่ความนิ่งเฉย |
| ดับไป | **วยะ** | การจบ: ล้าง worktree (อนัตตา), ปลดปล่อย branch, ตัดความจำ (มัตตัญญุตา), ปิด task ด้วยการบันทึก |

หลักธรรม ๒๒ ประการในข้อ ๑ ทั้งหมดเป็นองค์ประกอบย่อยของวงรอบนี้ ระบุว่าการเกิดขึ้นจะมีหลักการอย่างไร, การคงอยู่พร้อมแปรไปจะมีวินัยอย่างไร, และการดับจะปล่อยวางอย่างสะอาดอย่างไร

---

## ๑. หลักธรรมหลัก ๒๒ ประการ

### หมวด A — กรอบกระบวนการ (Process Frameworks)

#### ๑. อริยสัจ ๔ — โครงคิดแก้ปัญหา
ใช้เป็นโครงของ PRD และ task planning

| ข้อ | ใช้ใน |
|---|---|
| ทุกข์ | นิยามปัญหา |
| สมุทัย | หา root cause |
| นิโรธ | กำหนดเป้าหมายวัดได้ |
| มรรค | วางแผนเป็นขั้นตอน |

#### ๒. มรรค ๘ — โครงของ Functional Requirements
ใช้เป็น ๘ pillars ใน SRS

| มรรค | ในระบบ |
|---|---|
| สัมมาทิฏฐิ | Persona, Identity |
| สัมมาสังกัปปะ | Goal setting, planning |
| สัมมาวาจา | Inter-agent comms |
| สัมมากัมมันตะ | Worktree, commit |
| สัมมาอาชีวะ | Trust, neutrality |
| สัมมาวายามะ | Verification gates |
| สัมมาสติ | Memory system |
| สัมมาสมาธิ | Focus, session |

#### ๓. ขันธ์ ๕ — โครงสถาปัตยกรรม
ใช้ใน ARCHITECTURE document

| ขันธ์ | ในระบบ |
|---|---|
| รูป | File layout |
| เวทนา | I/O, hooks |
| สัญญา | Memory, recognition |
| สังขาร | Logic, transformations |
| วิญญาณ | Runtime, awareness |

---

### หมวด B — กรอบสภาวะ (State Frameworks)

#### ๔. ไตรลักษณ์ — ฐานคิดเรื่อง state
ทุกสิ่งในระบบมี ๓ ลักษณะ การออกแบบต้องสอดคล้อง

| ลักษณะ | ผลกระทบ |
|---|---|
| อนิจจัง | Memory ต้อง prune, timestamp |
| ทุกขัง | Branch ค้าง = ทุกข์ → cleanup |
| อนัตตา | ไม่ยึด branch, worktree |

#### ๕. ปฏิจจสมุปบาท — เหตุปัจจัยต่อเนื่อง (ใหม่)
หลักการ "เพราะสิ่งนี้มี สิ่งนี้จึงมี" — ใช้ในการวิเคราะห์ failure modes และ error chains

หลักสำคัญ: **ปัญหาที่เห็น มักไม่ใช่ปัญหาที่ต้องแก้** ต้องสืบเหตุปัจจัยถอยกลับ

ในระบบใช้กับ
- Root cause analysis เมื่อ agent ทำผิด
- Failure propagation tracing
- Cascading failure prevention
- Post-mortem analysis

#### ๖. กรรม ๓ (กายกรรม วจีกรรม มโนกรรม) — Audit Trail
การกระทำ ๓ ทาง — ใช้เป็นโครงของ logging

| กรรม | ในระบบ |
|---|---|
| กายกรรม | File operations, commits (เห็นได้) |
| วจีกรรม | Messages, logs (อ่านได้) |
| มโนกรรม | Decisions, plans (อนุมานได้) |

---

### หมวด C — กรอบการเติบโต (Growth Frameworks)

#### ๗. อิทธิบาท ๔ — เครื่องยนต์ทำงาน
ใช้เป็น success metrics

| ธรรม | KPI |
|---|---|
| ฉันทะ | Working in declared domain |
| วิริยะ | Task completion rate |
| จิตตะ | Gate compliance |
| วิมังสา | Self-improvement metrics |

#### ๘. ภาวนา ๔ — Agent Lifecycle (ใหม่)
การเติบโต ๔ ขั้น ใช้กับ agent lifecycle management

| ภาวนา | ระยะ | ตัวบ่งชี้ |
|---|---|---|
| กายภาวนา | Incarnation — เกิด | Template materialized, placeholders set |
| สีลภาวนา | Onboarding — เรียนกติกา | Conventions internalized, first task done |
| จิตภาวนา | Operational — ทำงานคล่อง | Stable task completion, low retry |
| ปัญญาภาวนา | Mentorship — สอนผู้อื่น | Patterns extracted, knowledge shared |

#### ๙. ปัญญา ๓ — Self-Improvement Loop (ใหม่)
ปัญญาเกิดจาก ๓ ทาง ใช้เป็นโครงของ self-improvement system

| ปัญญา | ในระบบ |
|---|---|
| สุตมยปัญญา | เรียนจากเอกสาร, conventions, examples |
| จินตามยปัญญา | คิดต่อ — synthesis, planning, pattern extraction |
| ภาวนามยปัญญา | เรียนจากปฏิบัติ — feedback loop, retrospectives |

#### ๑๐. อริยทรัพย์ ๗ — Capability Maturity (ใหม่)
ทรัพย์ ๗ ของผู้ประเสริฐ ใช้เป็น maturity model

| ทรัพย์ | ในระบบ | ระดับ |
|---|---|---|
| ศรัทธา | Trust in conventions | L1 |
| ศีล | Following rules | L2 |
| หิริ-โอตตัปปะ | Self-awareness of errors | L3 |
| สุตะ | Knowledge depth | L4 |
| จาคะ | Sharing capability | L5 |
| ปัญญา | Independent judgment | L6 |

---

### หมวด D — กรอบความสัมพันธ์ (Relational Frameworks)

#### ๑๑. สัปปุริสธรรม ๗ — รู้บริบท ๗ มิติ
ใช้ใน PRD stakeholder analysis และก่อนทำงาน

| ธรรม | รู้ |
|---|---|
| ธัมมัญญุตา | เหตุ, หลักการ |
| อัตถัญญุตา | ผล, เป้าหมาย |
| อัตตัญญุตา | ตน, ขีดจำกัด |
| มัตตัญญุตา | ประมาณ, ขอบเขต |
| กาลัญญุตา | กาล, เวลา |
| ปริสัญญุตา | ชุมชน, บริบท |
| ปุคคลัญญุตา | บุคคล, ผู้ใช้ |

#### ๑๒. สังคหวัตถุ ๔ — UX Principles
| ธรรม | ในระบบ |
|---|---|
| ทาน | Generous defaults, scaffolds |
| ปิยวาจา | Clear, helpful error messages |
| อัตถจริยา | Beneficial action, not just done |
| สมานัตตตา | Equal treatment across backends |

#### ๑๓. สาราณียธรรม ๖ — Inter-Agent Harmony
- เมตตา ๓ ทาง (กาย วาจา ใจ) ต่อ agent อื่น
- สาธารณโภคี — แบ่ง resource ยุติธรรม
- สีลสามัญญตา — กติกาเดียวกัน
- ทิฏฐิสามัญญตา — เป้าหมายตรงกัน

#### ๑๔. พรหมวิหาร ๔ — Error UX (ใหม่)
หลัก ๔ ประการของผู้ประเสริฐ ใช้กับการตอบสนองผู้ใช้และจัดการ error

| วิหาร | ในระบบ |
|---|---|
| เมตตา | Friendly tone in messages |
| กรุณา | Suggest fixes, not just report errors |
| มุทิตา | Celebrate user wins, learn from them |
| อุเบกขา | **Stay even when user frustrated** — ไม่ react แรง |

#### ๑๕. กัลยาณมิตร ๗ — Inter-Agent Trust (ใหม่)
คุณสมบัติของ "เพื่อนดี" ใช้กำหนดว่า agent ไหนเป็น trusted peer

| คุณสมบัติ | ในระบบ |
|---|---|
| ปิโย | น่าสนใจในการ delegate |
| ครุ | น่าเคารพในความสามารถ |
| ภาวนีโย | ช่วยให้เราเก่งขึ้น |
| วัตตา | บอกในสิ่งที่เป็นประโยชน์ |
| วจนักขโม | รับฟังคำท้วงได้ |
| คัมภีรัญจะ กถัง กัตตา | อธิบายเรื่องลึกได้ |
| โน จัฏฐาเน นิโยชเย | ไม่ชักนำผิดทาง |

---

### หมวด E — กรอบความระมัดระวัง (Discipline Frameworks)

#### ๑๖. โยนิโสมนสิการ — Verify Before Act
คิดแยบคาย สืบเหตุปัจจัย ก่อนกระทำ

#### ๑๗. อจินไตย ๔ — Scope Discipline
เรื่องที่ไม่ควรเก็บมาคิด (จำกัดที่ scope งานจริง ๆ ไม่ใช่ scope ทั่วไป)

| อจินไตย | ในระบบ (เฉพาะกรณีที่ตรง) |
|---|---|
| พุทธวิสัย | ไม่คาดเดาเจตนา LLM provider |
| ฌานวิสัย | ไม่อ้างเหตุผลจาก model internals |
| กรรมวิบาก | ไม่คาดผลทางธุรกิจระยะยาว |
| โลกจินตา | ไม่ออกแบบนอก scope ของระบบนี้ |

> *Note: ใน v1 เคยใช้อจินไตยกับเรื่อง task scope — เปลี่ยนมาใช้ มัตตัญญุตา (รู้ประมาณ) จะตรงกว่า*

#### ๑๘. สติปัฏฐาน ๔ — Observability (ใหม่ - ขยายเต็ม)
ใช้เป็นโครงของ observability system

| ฐาน | สังเกตอะไร | ในระบบ |
|---|---|---|
| กายานุปัสสนา | กาย (รูปธรรม) | File state, working directory, process |
| เวทนานุปัสสนา | ความรู้สึก/รับ | Tool results, input/output events |
| จิตตานุปัสสนา | สภาพจิต | Agent mode (planning/acting/verifying) |
| ธัมมานุปัสสนา | ธรรม/ปรากฏการณ์ | Rules applying, patterns matching |

#### ๑๙. ปธาน ๔ — Right Effort Directions
- สังวร — กันไม่ให้เกิดสิ่งไม่ดีใหม่ (lint)
- ปหาน — ละสิ่งไม่ดีที่มี (format, refactor)
- ภาวนา — สร้างสิ่งดีใหม่ (test new feature)
- อนุรักขนา — รักษาสิ่งดีที่มี (regression test)

---

### หมวด F — กรอบ Governance (Governance Frameworks)

#### ๒๐. อปริหานิยธรรม ๗ — Fleet Governance (ใหม่)
หลัก ๗ ของการไม่เสื่อม (พระพุทธเจ้าสอนวัชชี) ใช้กับ governance ของ agent fleet

| ธรรม | ในระบบ |
|---|---|
| ๑. ประชุมเนืองนิตย์ | Regular agent sync points |
| ๒. พร้อมเพรียงประชุม / เลิก | Coordinated session start/end |
| ๓. ไม่บัญญัติ/ล้มเลิกตามใจ | Convention change ผ่าน process |
| ๔. เคารพผู้ใหญ่ | Honor template version hierarchy |
| ๕. ไม่ข่มเหงสตรี | (เชิงสัญลักษณ์) protect vulnerable agents/users |
| ๖. เคารพเจดีย์ | Honor shared resources (registry, schemas) |
| ๗. คุ้มครองพระอรหันต์ | Protect senior/trusted agents from interference |

#### ๒๑. ตัณหา ๓ — Threat Model (ใหม่)
ตัณหาเป็นเหตุของความเสื่อม ใช้เป็นกรอบ threat model

| ตัณหา | ความหมาย | Threat |
|---|---|---|
| กามตัณหา | อยากในสิ่งเร้า | Prompt injection, social engineering |
| ภวตัณหา | อยากเป็น/คง | Privilege escalation, persistence |
| วิภวตัณหา | อยากไม่เป็น/ทำลาย | Destructive actions, data deletion |

#### ๒๒. สีล ๕ — Baseline Security Rules
- ห้าม `rm -rf` repo root (ปาณาติบาต — เชิงสัญลักษณ์)
- ห้าม commit secrets (อทินนาทาน)
- ห้าม spoof identity (มุสาวาท)
- ห้าม bypass gates (สุราเมระยะ — เสียสติ)
- ห้าม side effects ที่ไม่ประกาศ (กาเมสุมิจฉาจาร)

---

## ๒. Design Principles (จาก ๒๒ หลักธรรม)

| DP | หลักธรรม | Principle |
|---|---|---|
| DP-1 | โยนิโสมนสิการ | Verify before act |
| DP-2 | มัตตัญญุตา | Right amount, not maximum |
| DP-3 | สมานัตตตา | Equal treatment of backends |
| DP-4 | อนัตตา | Non-clinging workflow |
| DP-5 | อนิจจัง | Impermanence-aware memory |
| DP-6 | มัตตัญญุตา + อจินไตย | Scope discipline |
| DP-7 | อัตตัญญุตา | Self-declaration of capabilities |
| DP-8 | สีลสามัญญตา | Communal convention |
| DP-9 | ปธาน ๔ | Right effort in four directions |
| DP-10 | อริยสัจ | Decisions through Four Noble Truths |
| DP-11 | ปฏิจจสมุปบาท | Trace conditions backward in failures |
| DP-12 | ภาวนา ๔ | Lifecycle progression |
| DP-13 | ปัญญา ๓ | Learn from study, thought, practice |
| DP-14 | พรหมวิหาร | Equanimous error handling |
| DP-15 | สติปัฏฐาน ๔ | Four-foundation observability |
| DP-16 | อปริหานิยธรรม | Governance for non-decline |
| DP-17 | ตัณหา ๓ | Threat model by three cravings |
| DP-18 | สีล ๕ | Five baseline security rules |
| DP-19 | กัลยาณมิตร | Trust based on dhamma criteria |
| DP-20 | กรรม ๓ | Audit body/speech/mind separately |

---

## ๓. การประยุกต์ใน Stack

```
┌──────────────────────────────────────────────────────┐
│  อปริหานิยธรรม (Fleet Governance)                     │ ← Org level
├──────────────────────────────────────────────────────┤
│  ตัณหา ๓ (Threat Model) + สีล ๕ (Baseline)            │ ← Security
├──────────────────────────────────────────────────────┤
│  ภาวนา ๔ (Lifecycle) + ปัญญา ๓ (Improvement)         │ ← Agent growth
├──────────────────────────────────────────────────────┤
│  สาราณียธรรม + กัลยาณมิตร (Inter-agent)              │ ← Interconnect
├──────────────────────────────────────────────────────┤
│  สังคหวัตถุ + พรหมวิหาร (UX)                          │ ← User layer
├──────────────────────────────────────────────────────┤
│  มรรค ๘ (Functional reqs)                            │ ← SRS
├──────────────────────────────────────────────────────┤
│  ขันธ์ ๕ (Architecture)                              │ ← Components
├──────────────────────────────────────────────────────┤
│  สติปัฏฐาน ๔ (Observability)                          │ ← Cross-cutting
├──────────────────────────────────────────────────────┤
│  อิทธิบาท ๔ (Engine of work)                          │ ← Runtime
├──────────────────────────────────────────────────────┤
│  ไตรลักษณ์ + กรรม ๓ (State & Audit)                   │ ← Foundation
├──────────────────────────────────────────────────────┤
│  ปฏิจจสมุปบาท (Failure analysis)                      │ ← When broken
├──────────────────────────────────────────────────────┤
│  โยนิโสมนสิการ + อจินไตย (Thinking discipline)         │ ← Method
└──────────────────────────────────────────────────────┘
       อริยสัจ ๔ (Problem-solving cycle, end-to-end)
       สัปปุริสธรรม ๗ (Context sensing, end-to-end)
```

---

## ๔. การแก้ไขจาก v1.0

### ๔.๑ ตัด forced metaphors
- v1 ใช้ อจินไตย กับ "ไม่ debug ผลกระทบนอกขอบเขต task" — ฝืน
- v2 เปลี่ยนเป็น มัตตัญญุตา (รู้ประมาณ) ตรงกว่า
- อจินไตย ใช้เฉพาะ ๔ กรณีที่ตรงกับความหมายเดิม

### ๔.๒ แก้การซ้ำของ cross-cutting principles
- v1 ระบุโยนิโสมนสิการใน FR, NFR, DP — ซ้ำ ๓-๔ ที่
- v2 ระบุ live ที่ FR-7.7 และ FR-7.17 เท่านั้น ที่อื่นเป็นการอ้างอิง

### ๔.๓ เพิ่ม ๖ หลักธรรมใหม่
- ปฏิจจสมุปบาท → docs/FAILURE-MODES.md
- ภาวนา ๔ → docs/LIFECYCLE.md
- ปัญญา ๓ → docs/SELF-IMPROVEMENT.md
- พรหมวิหาร ๔ → docs/PRD (Error UX section)
- อปริหานิยธรรม → docs/FLEET-GOVERNANCE.md
- ตัณหา ๓ → docs/THREAT-MODEL.md
- กัลยาณมิตร → docs/COORDINATION-PROTOCOL.md
- สติปัฏฐาน → docs/OBSERVABILITY.md (ขยายเต็ม)
- อริยทรัพย์ ๗ → docs/LIFECYCLE.md (maturity model)
- กรรม ๓ → docs/OBSERVABILITY.md (audit trail)
- สีล ๕ → docs/THREAT-MODEL.md (baseline)

---

## ๕. คำหมายเหตุ

หลักธรรมในเอกสารนี้ใช้เป็นกรอบความคิดทาง engineering ไม่ใช่การตีความศาสนา การ map กับ technical concept เป็น **analogy** ที่มีประโยชน์ในการคิด ไม่ใช่การอ้างว่า "พุทธสอนเรื่อง software architecture"

ผู้สนใจ dhamma ในเชิงลึก ขอแนะนำให้ศึกษาจากตำราพุทธโดยตรง
