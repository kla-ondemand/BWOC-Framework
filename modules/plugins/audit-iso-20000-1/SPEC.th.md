---
title: การตรวจสอบ ISO/IEC 20000-1 ระบบบริหารบริการ IT (stub)
aliases:
  - audit-iso-20000-1
tags:
  - group/framework-plugins
  - type/plugin
  - kind/audit
  - domain/compliance
  - standard/iso-iec-20000-1
  - status/stub
maturity: L0
---

# การตรวจสอบ ISO/IEC 20000-1 ระบบบริหารบริการ IT (stub)

> [!abstract] **ปลั๊กอิน stub** ประกาศ criteria ของ ITSM ที่ runtime ในอนาคตจะตรวจสอบ; ส่ง `status = "not_implemented"` สำหรับทุก criterion พร้อม remedy เดียวกันคือ `"Runtime deferred to BWOC-EPIC-3."` สอดคล้องกับสคีมาเท่านั้น — ยังไม่มี logic การ audit จริง runtime เต็มรูปแบบจะลงใน [[../../notes/2026-05-26_iso-compliance-plugins|BWOC-EPIC-3]]

## ทำไมเป็น Stub

[[../../notes/2026-05-26_iso-compliance-plugins|บันทึกกรอบความคิด EPIC-2]] อธิบายความลึกแบบลำดับชั้น (full / stub / stub / stub) สำหรับ ISO ปลั๊กอินทั้ง 4 ตัว สรุปสั้น:

- ISO/IEC 20000-1 อธิบาย **แนวปฏิบัติบริหารบริการ** — การดูแล service catalogue, สัญญา SLA, การจัดการ incident และ problem, บันทึก change — ที่อยู่ในเครื่องมือบริหารบริการ (ITSM platform, ระบบ ticket) ไม่ใช่ในไฟล์ workspace การอนุมาน "องค์กรนี้ดำเนินการจัดการ incident" จาก "repo นี้มีไฟล์ `INCIDENTS.md`" จะทำให้ audit เป็นเท็จ (Musāvāda — ดู [PHILOSOPHY.en.md](../../docs/en/PHILOSOPHY.en.md) §Sila 5)
- Runtime สำหรับปลั๊กอินคลาส 20000-1 ต้องการ evidence แบบ adapter (query ระบบ ITSM, sample ticket, การ parse รายงาน SLA) ซึ่งยังไม่มีใน v1 ของ [PLUGINS.en.md §Audit Findings Schema](../../docs/en/PLUGINS.en.md#audit-findings-schema) การสร้างสิ่งเหล่านั้นเป็นงานของ EPIC-3
- Operator ที่รัน `bwoc plugin list --kind audit` หลัง Sprint 3 จะเห็น ISO framework ครบทั้ง 4 ตัว การที่ไม่มี 20000-1 จะสื่อว่า "BWOC ไม่มีความเห็นเรื่อง ITSM"; รูปแบบ stub สื่อว่า "BWOC มี placeholder, runtime อยู่ใน roadmap" นี่คือสัญญาณที่ซื่อสัตย์

## Criteria (v0.1.0)

ITSM criteria หลัก 8 รายการ ดึงจาก clauses หลักของ ISO/IEC 20000-1:2018[^iso-20000-1-2018] ลำดับการประกาศใน [[criteria]] คือลำดับรายงาน (PLUGINS.en.md line 84) ค่า `criterion_id` คงที่ข้ามรุ่น (PLUGINS.en.md §Stability); การเปลี่ยนชื่อเป็น major-version bump

| `criterion_id` | Clause | หัวเรื่อง | Severity |
|---|---|---|---|
| `20000-1-service-management-system-scope` | 4.3 | ขอบเขตของระบบบริหารบริการ | high |
| `20000-1-service-policy-and-objectives` | 5.2 | นโยบายและวัตถุประสงค์การบริหารบริการ | high |
| `20000-1-service-catalogue` | 8.3.1 | Service catalogue | medium |
| `20000-1-service-level-management` | 8.3.3 | การบริหารระดับการบริการ | high |
| `20000-1-change-management` | 8.5.1 | การบริหารการเปลี่ยนแปลง | high |
| `20000-1-incident-management` | 8.6.1 | การบริหาร incident | high |
| `20000-1-problem-management` | 8.6.3 | การบริหาร problem | medium |
| `20000-1-continual-improvement` | 10.2 | การปรับปรุงอย่างต่อเนื่อง | medium |

Criteria ทั้ง 8 ครอบคลุมแนวปฏิบัติหลักที่องค์กรเชิงสอดคล้อง 20000-1 คาดว่าจะดำเนินการ สมดุลระหว่าง clauses **plan-the-SMS** (4, 5) และ **operate-the-SMS** (8 — service portfolio, service delivery, resolution) Sub-clauses 8.2 (service portfolio), 8.4 (supply & demand — capacity, demand, budgeting), และ 8.7 (service assurance — availability, continuity, info security) เลื่อนไป v0.2.0

## วิธีรันงานวันนี้

```bash
bwoc audit run --plugin audit-iso-20000-1 --json
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
    "criterion_id": "20000-1-service-management-system-scope",
    "severity": "high",
    "status": "not_implemented",
    "evidence": { "kind": "none", "value": "" },
    "remedy": "Runtime deferred to BWOC-EPIC-3."
  },
  {
    "criterion_id": "20000-1-service-policy-and-objectives",
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
[plugins.audit-iso-20000-1]
enabled = true
```

ปลั๊กอินไม่ประกาศ `[config.schema]` ใน manifest — interface ระดับ workspace มีแค่ key `enabled` แบบสากล อาจเพิ่ม key adapter ของเครื่องมือ ITSM (`itsm_endpoint`, `ticket_query`, ฯลฯ) เมื่อ runtime ของ EPIC-3 มี

## สิ่งที่ EPIC-3 จะเพิ่ม

Runtime ของ EPIC-3 ต้องการอย่างน้อย:

- **Adapter evidence** Read-only adapter ต่อ ITSM platform ที่นิยม (Jira Service Management, ServiceNow, Zendesk, plain CSV exports) ที่ตอบได้ว่า "incident จำนวนกี่รายการในไตรมาสที่ผ่านมา, อัตรา breach SLA เท่าไร"
- **Sampling และ rolling window** SLA performance เป็นอัตราตามช่วงเวลา ไม่ใช่ artifact ตัวเดียว Runtime ต้องการวิธีประกาศ "sampled N incidents from the last 90 days" โดยไม่ทำให้จำนวน finding พองตัว
- **Operator attestation** Clause บางตัว (นโยบายการบริหารบริการ, การมุ่งมั่นของผู้บริหาร) ลดทอนเป็น "ใช่ เรามี, sign off โดย X เมื่อ Y" สคีมาต้องการ evidence kind แบบ attestation

จนกว่าสิ่งเหล่านั้นจะลง ทุก criterion ในไฟล์นี้ส่ง `not_implemented` ค่า `criterion_id` คงที่ — เมื่อ EPIC-3 ลง runtime ID ไม่เปลี่ยน เปลี่ยนแค่ค่า `status` / `evidence` / `remedy`

## Maturity

ประกาศ **L0** — stub, ไม่มี runtime, สอดคล้องกับสคีมาเท่านั้น เลื่อนเป็น **L1** เมื่อ EPIC-3 ลง runtime และอย่างน้อยหนึ่ง criterion ส่ง finding ที่ไม่ใช่ `not_implemented` กับ workspace จริง

## ความเป็นกลาง

ค่าใน manifest ไม่ระบุ LLM backend, vendor หรือ model `kind = "audit"` เป็น enum ของ framework เอง (`BWOC-10`) ปลั๊กอินอ้างถึง "ISO/IEC 20000-1" เฉพาะใน `description`, ในเนื้อหา SPEC นี้, และใน namespace มาตรฐาน `20000-1-*` ของ criterion-id — ไม่ใช้ใน key ของ `criteria.toml` หรือในค่า finding นอกเหนือจาก namespace นั้น ผ่านกฎ **Samānattatā**

## แหล่งอ้างอิง

ITSM criteria ดึงจาก clauses หลักของ ISO/IEC 20000-1:2018 (4 ถึง 10) ที่เผยแพร่:

- ISO/IEC 20000-1:2018 — *Information technology — Service management — Part 1: Service management system requirements.* International Organization for Standardization. ISO catalogue entry: <https://www.iso.org/standard/70636.html> [^iso-20000-1-2018]
- ISO/IEC JTC 1/SC 40 — *IT service management and IT governance.* คณะกรรมการเทคนิคร่วมที่รับผิดชอบ ISO/IEC 20000 family: <https://www.iso.org/committee/5013818.html>
- ISO/IEC 20000-10:2018 — *Concepts and vocabulary* (excerpted ใน online browsing platform ของ ISO สำหรับ terminology เท่านั้น): มีประโยชน์ในการแยกแยะคำว่า "service", "SMS", และ "SLA"

[^iso-20000-1-2018]: ISO/IEC 20000-1:2018 มีโครงสร้างตาม Annex SL high-level structure (clauses 4 บริบท, 5 ภาวะผู้นำ, 6 การวางแผน, 7 การสนับสนุน, 8 การปฏิบัติการ, 9 การประเมินผลการทำงาน, 10 การปรับปรุง) Clause 8 (Operation of the SMS) เป็น clause แนวปฏิบัติ ITSM หลัก และมี 7 sub-clauses (8.1 operational planning, 8.2 service portfolio, 8.3 relationship & agreement, 8.4 supply & demand, 8.5 service design build & transition, 8.6 resolution & fulfilment, 8.7 service assurance) Criteria ทั้ง 8 ที่นี่ครอบคลุมหนึ่งแนวปฏิบัติหลักจาก clauses 4, 5, 8.3, 8.5, 8.6, และ 10; sub-clauses ที่เหลือ (8.2, 8.4, 8.7, รวมทั้ง 8.3 และ 8.6 child หลายตัว) เลื่อนไปขยายใน v0.2.0 เมื่อ runtime ของ EPIC-3 มี

## ดูเพิ่ม

- [[../../docs/en/PLUGINS.en|PLUGINS.en.md]] — สเปคของ plugin; แถวของ `audit` kind (BWOC-10), Audit Findings Schema (BWOC-11), ตัวอย่างสถานะ stub
- [[../../notes/2026-05-26_iso-compliance-plugins|2026-05-26_iso-compliance-plugins.md]] — บันทึกกรอบความคิด EPIC-2 (ทำไม stub, ทำไมเลื่อนไป EPIC-3)
- [[../audit-iso-29110/SPEC|audit-iso-29110]] — ปลั๊กอิน audit อ้างอิงที่รันได้จาก BWOC-13
- [[../audit-iso-9001/SPEC|audit-iso-9001]], [[../audit-iso-27001/SPEC|audit-iso-27001]] — ปลั๊กอิน stub พี่น้องที่ส่งใน sprint เดียวกัน
