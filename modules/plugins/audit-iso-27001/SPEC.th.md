---
title: การตรวจสอบ ISO/IEC 27001 ระบบบริหารความมั่นคงสารสนเทศ (stub)
aliases:
  - audit-iso-27001
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-iec-27001
  - status/stub
maturity: L0
---

# การตรวจสอบ ISO/IEC 27001 ระบบบริหารความมั่นคงสารสนเทศ (stub)

> [!abstract] **ปลั๊กอิน stub** ประกาศ criteria ของ ISMS ที่ runtime ในอนาคตจะตรวจสอบ; ส่ง `status = "not_implemented"` สำหรับทุก criterion พร้อม remedy เดียวกันคือ `"Runtime deferred to BWOC-EPIC-3."` สอดคล้องกับสคีมาเท่านั้น — ยังไม่มี logic การ audit จริง runtime เต็มรูปแบบจะลงใน [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]]

## ทำไมเป็น Stub

[[../../notes/2026-05-26_iso-compliance-plugins|บันทึกกรอบความคิด EPIC-2]] อธิบายความลึกแบบลำดับชั้น (full / stub / stub / stub) สำหรับ ISO ปลั๊กอินทั้ง 4 ตัว สรุปสั้น:

- ISO/IEC 27001 อธิบาย **แนวปฏิบัติบริหารความมั่นคงสารสนเทศ** — กระบวนการประเมินความเสี่ยง, การเลือก control (Annex A), นโยบายการควบคุมการเข้าถึง, ความพร้อมตอบสนอง incident, การฝึกซ้อมความต่อเนื่องของธุรกิจ — ที่ลดทอนเป็น evidence เชิงองค์กร (เอกสารนโยบาย, SoA ที่ sign off, บันทึกการซ้อม incident, รายงานการฝึก) มากกว่า artifact ใน repository การอนุมาน "องค์กรนี้ได้ทำการประเมินความเสี่ยงด้านความมั่นคงสารสนเทศ" จาก "repo นี้มีไฟล์ `RISKS.md`" จะทำให้ audit เป็นเท็จ (Musāvāda — ดู [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5)
- Runtime สำหรับปลั๊กอินคลาส 27001 ต้องการ attestation + sampling + control-mapping evidence (ข้อความที่ operator sign off พร้อม provenance, sampled control จาก 93 รายการของ Annex A, traceability ไปยัง Statement of Applicability) ซึ่งยังไม่มีใน v1 ของ [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema) การสร้างสิ่งเหล่านั้นเป็นงานของ EPIC-3
- Operator ที่รัน `bwoc plugin list --kind audit` หลัง Sprint 3 จะเห็น ISO framework ครบทั้ง 4 ตัว การที่ไม่มี 27001 จะสื่อว่า "BWOC ไม่มีความเห็นเรื่อง InfoSec management"; รูปแบบ stub สื่อว่า "BWOC มี placeholder, runtime อยู่ใน roadmap" นี่คือสัญญาณที่ซื่อสัตย์

## Criteria (v0.1.0)

ISMS criteria หลัก 8 รายการ ดึงจาก clauses หลักและ Annex A controls ของ ISO/IEC 27001:2022[^iso-27001-2022] ลำดับการประกาศใน [[criteria]] คือลำดับรายงาน (PLUGINS.en.md line 84) ค่า `criterion_id` คงที่ข้ามรุ่น (PLUGINS.en.md §Stability); การเปลี่ยนชื่อเป็น major-version bump

| `criterion_id` | อ้างอิง | หัวเรื่อง | Severity |
|---|---|---|---|
| `27001-isms-scope` | 4.3 | ขอบเขตของ ISMS | high |
| `27001-information-security-policy` | 5.2 | นโยบายความมั่นคงสารสนเทศ | high |
| `27001-risk-assessment` | 6.1.2 | การประเมินความเสี่ยงด้านความมั่นคงสารสนเทศ | critical |
| `27001-statement-of-applicability` | 6.1.3 | Statement of Applicability | critical |
| `27001-access-control` | A.5.15 | การควบคุมการเข้าถึง | high |
| `27001-incident-management` | A.5.24 | การวางแผนและเตรียมการบริหาร incident | high |
| `27001-business-continuity` | A.5.29 | ความมั่นคงสารสนเทศระหว่างเหตุขัดข้อง | medium |
| `27001-internal-audit` | 9.2 | การ audit ภายใน | high |

Criteria ทั้ง 8 ผสม **clauses main-body** สี่ตัว (4.3, 5.2, 6.1.2, 6.1.3, 9.2 — ข้อกำหนดการตั้ง ISMS ที่ทุกองค์กรที่ได้รับการรับรองต้องดำเนินการ) และ **Annex A controls** สามตัวจาก theme *Organizational* (5.15 access control, 5.24 incident preparation, 5.29 continuity) Annex A controls ถูกจัด theme ใหม่และตัดซ้ำจาก 114 ตัว (ใน :2013) เหลือ 93 ตัว (ใน :2022) ข้าม 4 themes — *Organizational* (37), *People* (8), *Physical* (14), *Technological* (34) Annex A controls ที่เหลืออีก 90 ตัวเลื่อนไป v0.2.0 เมื่อ runtime ของ EPIC-3 sample ได้สอดคล้อง

## วิธีรันงานวันนี้

```bash
bwoc audit run --plugin audit-iso-27001 --json
```

Dispatcher spawns `audit.sh` (ตาม `BWOC-12`) ด้วยสัญญา env มาตรฐาน: `BWOC_WORKSPACE`, `BWOC_PLUGIN_DIR`, `BWOC_AUDIT_OPERATION` Stub จะ:

1. อ่าน `criteria.toml` จาก `BWOC_PLUGIN_DIR`
2. สำหรับ criterion แต่ละตัวตามลำดับการประกาศ ส่ง finding ที่มี `status = "not_implemented"`, `evidence = { kind: "none", value: "" }`, และ `remedy = "Runtime deferred to BWOC-EPIC-3."`
3. ออกด้วยรหัส `0` เมื่อสำเร็จ

`BWOC_WORKSPACE` ถูก **ไม่อ่าน** อย่างเจตนา Stub ไม่ตรวจสอบ workspace และการแกล้งทำว่าตรวจจะทำให้ audit เป็นเท็จ นี่คือการป้องกัน Musāvāda ที่ระดับปลั๊กอิน

## ตัวอย่าง Output

```json
[
  {
    "criterion_id": "27001-isms-scope",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "27001-information-security-policy",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  }
]
```

`bwoc audit run` ห่อ output นี้ไว้ใน envelope หลัก `{ workspace, runs: [{ plugin, version, started_at, finished_at, findings: [...] }, ...], summary }` Stub สอดคล้องกับ envelope เดียวกับปลั๊กอินรันได้ `audit-iso-29110` — operator ที่เรียนรู้ตัวหนึ่งก็เรียนรู้อีกตัว

## การตั้งค่า

```toml
# workspace.toml
[plugins.audit-iso-27001]
enabled = true
```

ปลั๊กอินไม่ประกาศ `[config.schema]` ใน manifest — interface ระดับ workspace มีแค่ key `enabled` แบบสากล อาจเพิ่ม key การเลือก control (`soa_path`, `theme_filter`, ฯลฯ) เมื่อ runtime ของ EPIC-3 มี

## สิ่งที่ EPIC-3 จะเพิ่ม

Runtime ของ EPIC-3 ต้องการอย่างน้อย:

- **Attestation evidence** ข้อความที่ operator ให้ ("การประเมินความเสี่ยงครั้งล่าสุดของเราคือ 2026-03-10, sign off โดย CISO, ขอบเขต: customer-data flows") พร้อม provenance และ timestamp
- **SoA-driven control sampling** Statement of Applicability ประกาศว่า 93 Annex A controls ใดอยู่ใน scope Runtime ต้อง sample เฉพาะตัวที่อยู่ใน scope ประกาศ `not_applicable` สำหรับที่เหลือ และ surface gap ของ SoA
- **Time-bounded evidence** การประเมินความเสี่ยงเมื่อ 3 ปีก่อนไม่ตอบโจทย์ 6.1.2 เหมือนการประเมินครั้งล่าสุดเมื่อไตรมาสที่แล้ว สคีมาต้องการมิติ "as of" หรือ "valid through"
- **Control-to-clause traceability** Control บางตัวสนับสนุนหลาย clause (เช่น A.5.24 incident-management feeds ทั้ง clause 9 และ clause 10) Runtime ต้องการวิธีแสดงโดยไม่ซ้ำ finding

จนกว่าสิ่งเหล่านั้นจะลง ทุก criterion ในไฟล์นี้ส่ง `not_implemented` ค่า `criterion_id` คงที่ — เมื่อ EPIC-3 ลง runtime ID ไม่เปลี่ยน เปลี่ยนแค่ค่า `status` / `evidence` / `remedy`

## Maturity

ประกาศ **L0** — stub, ไม่มี runtime, สอดคล้องกับสคีมาเท่านั้น เลื่อนเป็น **L1** เมื่อ EPIC-3 ลง runtime และอย่างน้อยหนึ่ง criterion ส่ง finding ที่ไม่ใช่ `not_implemented` กับ workspace จริง

## ความเป็นกลาง

ค่าใน manifest ไม่ระบุ LLM backend, vendor หรือ model `kind = "audit"` เป็น enum ของ framework เอง (`BWOC-10`) ปลั๊กอินอ้างถึง "ISO/IEC 27001" เฉพาะใน `description`, ในเนื้อหา SPEC นี้, และใน namespace มาตรฐาน `27001-*` ของ criterion-id — ไม่ใช้ใน key ของ `criteria.toml` หรือในค่า finding นอกเหนือจาก namespace นั้น ผ่านกฎ **Samānattatā**

## แหล่งอ้างอิง

ISMS criteria ดึงจาก clauses หลักของ ISO/IEC 27001:2022 (4 ถึง 10) และชุด Annex A controls ที่เผยแพร่:

- ISO/IEC 27001:2022 — *Information security, cybersecurity and privacy protection — Information security management systems — Requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/27001> [^iso-27001-2022]
- ISO/IEC 27002:2022 — *Information security, cybersecurity and privacy protection — Information security controls.* ISO catalogue entry: <https://www.iso.org/standard/75652.html> มาตรฐานเสริมที่ให้แนวทาง implementation สำหรับ Annex A controls
- ISO/IEC JTC 1/SC 27 — *Information security, cybersecurity and privacy protection.* หน้า landing สาธารณะ: <https://www.iso.org/committee/45306.html> คณะกรรมการเทคนิคร่วมที่รับผิดชอบ ISO/IEC 27000 family

[^iso-27001-2022]: ISO/IEC 27001:2022 มีโครงสร้างตาม Annex SL high-level structure (clauses 4 บริบท, 5 ภาวะผู้นำ, 6 การวางแผน, 7 การสนับสนุน, 8 การปฏิบัติการ, 9 การประเมินผลการทำงาน, 10 การปรับปรุง) Annex A มี 93 ควบคุมความมั่นคงสารสนเทศที่จัดเป็น 4 themes — *Organizational* (37 controls, prefix A.5.x), *People* (8 controls, prefix A.6.x), *Physical* (14 controls, prefix A.7.x), และ *Technological* (34 controls, prefix A.8.x) — ปรับโครงสร้างใหม่จากการจัด 114-control / 14-clause ใน ISO/IEC 27001:2013 Criteria ทั้ง 8 ที่นี่ครอบคลุม clauses main-body ห้าตัวบวก Annex A controls สามตัวจาก theme *Organizational* ที่เป็นหัวเรื่องการดำเนินการ ISMS; Annex A controls ที่เหลืออีก 90 ตัวเลื่อนไปขยายใน v0.2.0 เมื่อ runtime ของ EPIC-3 sample ได้สอดคล้อง

## ดูเพิ่ม

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — สเปคของ plugin; แถวของ `audit` kind (BWOC-10), Audit Findings Schema (BWOC-11), ตัวอย่างสถานะ stub
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — บันทึกกรอบความคิด EPIC-2 (ทำไม stub, ทำไมเลื่อนไป EPIC-3)
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — ปลั๊กอิน audit อ้างอิงที่รันได้จาก BWOC-13
- [[../audit-iso-9001/SPEC|audit-iso-9001]], [[../audit-iso-20000-1/SPEC|audit-iso-20000-1]] — ปลั๊กอิน stub พี่น้องที่ส่งใน sprint เดียวกัน
