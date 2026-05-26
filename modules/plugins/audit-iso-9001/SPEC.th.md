---
title: การตรวจสอบ ISO 9001 ระบบบริหารคุณภาพ (stub)
aliases:
  - audit-iso-9001
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-9001
  - status/stub
maturity: L0
---

# การตรวจสอบ ISO 9001 ระบบบริหารคุณภาพ (stub)

> [!abstract] **ปลั๊กอิน stub** ประกาศ criteria ของ QMS ที่ runtime ในอนาคตจะตรวจสอบ; ส่ง `status = "not_implemented"` สำหรับทุก criterion พร้อม remedy เดียวกันคือ `"Runtime deferred to BWOC-EPIC-3."` สอดคล้องกับสคีมาเท่านั้น — ยังไม่มี logic การ audit จริง runtime เต็มรูปแบบจะลงใน [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]]

## ทำไมเป็น Stub

[[../../notes/2026-05-26_iso-compliance-plugins|บันทึกกรอบความคิด EPIC-2]] อธิบายความลึกแบบลำดับชั้น (full / stub / stub / stub) สำหรับ ISO ปลั๊กอินทั้ง 4 ตัว สรุปสั้น:

- ISO 9001 อธิบาย **แนวปฏิบัติเชิงองค์กร** — นโยบายคุณภาพ, การทบทวนของผู้บริหาร, การ audit ภายใน, การดำเนินการแก้ไข — ที่ไม่สามารถลดทอนเป็นการเช็คการมีอยู่ของไฟล์ใน workspace ได้ การอนุมาน "องค์กรนี้มีนโยบายคุณภาพที่เป็นเอกสาร" จาก "repo นี้มีไฟล์ `POLICY.md`" จะทำให้ audit เป็นเท็จ (Musāvāda — ดู [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5)
- Runtime สำหรับปลั๊กอินคลาส 9001 ต้องการ evidence model ที่รวยกว่า (operator attestation, evidence ที่ผูกกับเวลา, sampling) ซึ่งยังไม่มีใน v1 ของ [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema) การสร้างสิ่งเหล่านั้นเป็นงานของ EPIC-3
- Operator ที่รัน `bwoc plugin list --kind audit` หลัง Sprint 3 จะเห็น ISO framework ครบทั้ง 4 ตัว การที่ไม่มี 9001 จะสื่อว่า "BWOC ไม่มีความเห็นเรื่อง QMS"; รูปแบบ stub สื่อว่า "BWOC มี placeholder, runtime อยู่ใน roadmap" นี่คือสัญญาณที่ซื่อสัตย์

## Criteria (v0.1.0)

QMS criteria หลัก 8 รายการ ดึงจาก clauses หลักของ ISO 9001:2015[^iso-9001-2015] ลำดับการประกาศใน [[criteria]] คือลำดับรายงาน (PLUGINS.en.md line 84) ค่า `criterion_id` คงที่ข้ามรุ่น (PLUGINS.en.md §Stability); การเปลี่ยนชื่อเป็น major-version bump

| `criterion_id` | Clause | หัวเรื่อง | Severity |
|---|---|---|---|
| `9001-context-of-organization` | 4 | บริบทขององค์กร | high |
| `9001-leadership-and-policy` | 5.2 | ภาวะผู้นำและนโยบายคุณภาพ | high |
| `9001-risks-and-opportunities` | 6.1 | การดำเนินการกับความเสี่ยงและโอกาส | high |
| `9001-competence-and-awareness` | 7.2 | สมรรถนะและความตระหนัก | medium |
| `9001-documented-information` | 7.5 | สารสนเทศที่เป็นเอกสาร | medium |
| `9001-internal-audit` | 9.2 | การ audit ภายใน | high |
| `9001-management-review` | 9.3 | การทบทวนของผู้บริหาร | high |
| `9001-corrective-action` | 10.2 | ความไม่สอดคล้องและการดำเนินการแก้ไข | medium |

Criteria 8 รายการครอบคลุมหนึ่งแนวปฏิบัติต่อ clause หลัก (4 ถึง 10) ที่องค์กรเชิงสอดคล้อง QMS ทุกที่คาดว่าจะดำเนินการ Sub-clause ที่เหลือ (ทรัพยากร, โครงสร้างพื้นฐาน, การติดตามและวัดผลของกระบวนการปฏิบัติการ, แบบสำรวจความพึงพอใจของลูกค้า ฯลฯ) เลื่อนไป v0.2.0 เมื่อ runtime ของ EPIC-3 มีและรองรับได้

## วิธีรันงานวันนี้

```bash
bwoc audit run --plugin audit-iso-9001 --json
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
    "criterion_id": "9001-context-of-organization",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "9001-leadership-and-policy",
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
[plugins.audit-iso-9001]
enabled = true
```

ปลั๊กอินไม่ประกาศ `[config.schema]` ใน manifest — interface ระดับ workspace มีแค่ key `enabled` แบบสากล อาจเพิ่ม key `profile` หรือ `scope` เมื่อ runtime ของ EPIC-3 มี

## สิ่งที่ EPIC-3 จะเพิ่ม

Runtime ของ EPIC-3 ต้องการอย่างน้อย:

- **Attestation evidence** ข้อความที่ operator ให้ ("การทบทวนของผู้บริหารครั้งล่าสุดของเราคือ 2026-04-15, sign off โดย X") ที่ framework บันทึกได้พร้อมตราประทับ provenance
- **Time-bounded evidence** การทบทวนของผู้บริหารเมื่อ 3 ปีก่อนไม่ตอบโจทย์ clause เดียวกันกับการทบทวนครั้งล่าสุดเมื่อไตรมาสที่แล้ว สคีมาต้องการมิติ "valid through" หรือ "as of"
- **Sampling** การสอดคล้องของ audit ภายในเรื่อง coverage ข้าม QMS ไม่ใช่ artifact ตัวเดียว Runtime ต้องการวิธีประกาศ "sampled N of M processes" โดยไม่ทำให้จำนวน finding พองตัว

จนกว่าสิ่งเหล่านั้นจะลง ทุก criterion ในไฟล์นี้ส่ง `not_implemented` ค่า `criterion_id` คงที่ — เมื่อ EPIC-3 ลง runtime ID ไม่เปลี่ยน เปลี่ยนแค่ค่า `status` / `evidence` / `remedy`

## Maturity

ประกาศ **L0** — stub, ไม่มี runtime, สอดคล้องกับสคีมาเท่านั้น เลื่อนเป็น **L1** เมื่อ EPIC-3 ลง runtime และอย่างน้อยหนึ่ง criterion ส่ง finding ที่ไม่ใช่ `not_implemented` กับ workspace จริง

## ความเป็นกลาง

ค่าใน manifest ไม่ระบุ LLM backend, vendor หรือ model `kind = "audit"` เป็น enum ของ framework เอง (`BWOC-10`) ปลั๊กอินอ้างถึง "ISO 9001" เฉพาะใน `description`, ในเนื้อหา SPEC นี้, และใน namespace มาตรฐาน `9001-*` ของ criterion-id — ไม่ใช้ใน key ของ `criteria.toml` หรือในค่า finding นอกเหนือจาก namespace นั้น ผ่านกฎ **Samānattatā**

## แหล่งอ้างอิง

QMS criteria ดึงจาก clauses หลักของ ISO 9001:2015 (4 ถึง 10) ที่เผยแพร่:

- ISO 9001:2015 — *Quality management systems — Requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/62085.html> [^iso-9001-2015]
- ISO/TC 176/SC 2 — *Quality management and quality assurance — Quality systems.* หน้า landing สาธารณะ: <https://www.iso.org/committee/53896.html> คณะกรรมการเทคนิคที่รับผิดชอบ ISO 9000 family
- ISO — *Quality management principles* (โบรชัวร์เปิดเผยที่สรุปหลักการ QMS ทั้ง 7 ที่รองรับการแก้ไขปี 2015): <https://www.iso.org/publication/PUB100080.html>

[^iso-9001-2015]: ISO 9001:2015 มีโครงสร้างตาม Annex SL high-level structure (clauses 4 บริบท, 5 ภาวะผู้นำ, 6 การวางแผน, 7 การสนับสนุน, 8 การปฏิบัติการ, 9 การประเมินผลการทำงาน, 10 การปรับปรุง) Criteria ทั้ง 8 ที่นี่ครอบคลุมหนึ่งแนวปฏิบัติหลักต่อ clause หลักที่องค์กรเชิงสอดคล้อง QMS ทุกที่คาดว่าจะดำเนินการ; sub-clauses ที่เหลือ (7.1 ทรัพยากร, 8.5 การจัดหาผลิตภัณฑ์และบริการ, 9.1.2 ความพึงพอใจของลูกค้า ฯลฯ) เลื่อนไปขยายใน v0.2.0 เมื่อ runtime ของ EPIC-3 มี

## ดูเพิ่ม

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — สเปคของ plugin; แถวของ `audit` kind (BWOC-10), Audit Findings Schema (BWOC-11), ตัวอย่างสถานะ stub
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — บันทึกกรอบความคิด EPIC-2 (ทำไม stub, ทำไมเลื่อนไป EPIC-3)
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — ปลั๊กอิน audit อ้างอิงที่รันได้จาก BWOC-13
- [[../audit-iso-20000-1/SPEC|audit-iso-20000-1]], [[../audit-iso-27001/SPEC|audit-iso-27001]] — ปลั๊กอิน stub พี่น้องที่ส่งใน sprint เดียวกัน
