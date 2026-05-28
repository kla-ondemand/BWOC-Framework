---
title: gcloud-project — บริบท Project ของ Google Cloud
aliases:
  - gcloud-project
tags:
  - group/framework-plugins
  - type/plugin
  - kind/workflow
  - domain/integration
  - integration/gcloud
maturity: L1
---

# gcloud-project — บริบท Project ของ Google Cloud

> [!abstract] หนึ่งในสองปลั๊กอินอ้างอิงสาย `workflow` สำหรับ foundation ของ GCP (`BWOC-EPIC-8`) รับผิดชอบ **บริบท project** — รายการ project ที่เข้าถึงได้, การ describe หนึ่ง project และการตั้งค่า project เริ่มต้นของ `gcloud` ในเครื่อง คำสั่ง: `list`, `show`, `set-default` (verb เขียนเพียงตัวเดียวใน foundation; **มีด่านยืนยันจากผู้ดูแลใน CLI** และแตะเฉพาะ config DB ของ `gcloud` ในเครื่อง — ไม่มีการเปลี่ยนสถานะใน API ระยะไกล) source ฟังก์ชันช่วยจาก [[../gcloud-auth/SPEC.th|`gcloud-auth`]] ที่เป็นพี่น้อง เหตุผลฉบับเต็ม: [[../../../notes/2026-05-28_gcloud-workflow-plugin-architecture|บันทึกออกแบบ BWOC-51]]

## ทำไมต้องสองปลั๊กอิน ทำไม kind `workflow`

`gcloud-project` ส่งมาแยกจาก `gcloud-auth` เพื่อให้ slice GCP ในอนาคต (`gcloud-compute`, `gcloud-storage`, …) สามารถพึ่งพา foundation auth โดยไม่ต้องสืบทอด verb ฝั่งจัดการ project ฟังก์ชันช่วยที่ใช้ร่วมกันอยู่ใน `gcloud-auth/gcloud.sh` และ source ที่นี่ตอน startup — การ resolve credential ถูกนิยามเพียงครั้งเดียว ส่วนการใช้ kind `workflow` (ไม่ใช่ kind `gcp` ใหม่) เป็นรูปร่างที่ถูกต้องเพราะเฟรมเวิร์กไม่ได้ถือ lifecycle: เอเจนต์เรียกออก, ผลลัพธ์ของ gcloud ถูกนำเสนอ เหตุผลฉบับเต็ม: บันทึกออกแบบ §1 + 2

## คำสั่ง (Verbs)

| คำสั่ง | ทิศทาง | Auth | HTTP / ผลข้างเคียง | ด่าน |
|---|---|---|---|---|
| `list` | อ่าน | ต้องมี | `gcloud projects list` (มี paging ภายใน คืน envelope เดียว) | ไม่มี |
| `show` | อ่าน | ต้องมี | `gcloud projects describe <id>` (ใช้ `gcloud config get-value project` ถ้าไม่ส่ง `.project`) | ไม่มี |
| `set-default` | **เขียน (ในเครื่อง)** | ต้องมี | `gcloud config set project <id>` — แก้เฉพาะ `~/.config/gcloud/configurations/...`; ไม่มี API ระยะไกล | **ยืนยันโดยผู้ดูแล** (ใน CLI `bwoc gcloud` — BWOC-52) |

`set-default` คือ verb เขียนเพียงตัวเดียวใน foundation ของ EPIC-8 ความเสี่ยงจำกัดอยู่ในเครื่องของผู้ดูแล การย้อนกลับทำได้ง่าย (`gcloud config set project <previous>`) ด่านยืนยันของ CLI ยังเปิดไว้เพราะการตั้งค่าผิดจะ route verb ของ agent ในอนาคตไปที่ project ที่ไม่ถูกต้องอย่างเงียบ ๆ — นั่นคือ footgun ที่ slice นี้พยายามหลีกเลี่ยง (บันทึก §Decision 4)

## การทำงาน

CLI `bwoc gcloud` (`BWOC-52`) เรียก `gcloud.sh` จากไดเรกทอรีนี้:

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GCLOUD_OPERATION` (env) | `list` \| `show` \| `set-default` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace; path ของ SA JSON ในพี่น้อง resolve ใต้นี้ |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้; ใช้หา helpers พี่น้องที่ `../gcloud-auth/gcloud.sh` |
| stdin | JSON บรรทัดเดียว เช่น `{"operation":"list"}`, `{"operation":"show","project":"my-proj"}`, `{"operation":"set-default","project":"my-proj"}` |

เมื่อสำเร็จ: ออกด้วยรหัส `0`, ปล่อย JSON หนึ่งอ็อบเจกต์ทาง stdout เมื่อผิดพลาด: ข้อความวินิจฉัยทาง stderr + ออกด้วยรหัสไม่ใช่ศูนย์

## การยืนยันตัวตน

ปลั๊กอินนี้ **ไม่เคยอ่านค่า credential ใด ๆ** มัน source helpers พี่น้องของ [[../gcloud-auth/SPEC.th#การยืนยันตัวตน|`gcloud-auth`]] (`gcloud_assert_cli`, `gcloud_assert_authenticated`) ตอน startup และถาม `gcloud` ถึงสถานะของ project [[auth|auth.toml]] ประกาศสัญญา auth **เดียวกัน** (รูปร่างเท่านั้น) กับพี่น้อง เพื่อให้:

- `bwoc check` ตรวจสัญญาแต่ละปลั๊กอินได้อิสระ
- ผู้ดูแลที่ดูปลั๊กอินนี้เพียงตัวเดียวเห็นโมเดล auth ครบโดยไม่ต้องไล่ดูพี่น้อง

ลำดับความสำคัญของ credential เหมือนกับ [[../gcloud-auth/SPEC.th#การยืนยันตัวตน|`gcloud-auth §การยืนยันตัวตน`]]: ADC → `.bwoc/secrets/gcloud-sa.json` → `BWOC_GCLOUD_*` env ปลั๊กอินล้มเหลวทันทีพร้อมข้อความที่ชัดเจนหากไม่มี credential ที่ active

> [!danger] **ศีล — อทินนาทาน** การรับประกันเดียวกับพี่น้อง: ไม่มี token ใดเข้ามาใน address space ของปลั๊กอินนี้ เราเรียก `gcloud` และนำเสนอผลลัพธ์เท่านั้น `set-default` ไม่ส่ง, ไม่ log, ไม่เก็บ credential ในรูปแบบใด — มันเขียนเฉพาะ project ID ไปยัง `~/.config/gcloud/`

## รูปร่างผลลัพธ์

### `list`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "list",
  "total": 3,
  "projects": [
    { "project_id": "my-proj-1", "project_number": "111111111111", "name": "My Project 1", "lifecycle_state": "ACTIVE" },
    { "project_id": "my-proj-2", "project_number": "222222222222", "name": "My Project 2", "lifecycle_state": "ACTIVE" },
    { "project_id": "archived",  "project_number": "333333333333", "name": "Archived",    "lifecycle_state": "DELETE_REQUESTED" }
  ]
}
```

`total` คือความยาว array ที่ `gcloud` ในเครื่องคืนมา ถ้าผู้ดูแลมี project จำนวนมาก `gcloud` เองจัดการ paging และคืนชุดเต็ม ปลั๊กอินไม่ paginate ซ้ำ

### `show`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "show",
  "project_id": "my-proj-1",
  "project_number": "111111111111",
  "name": "My Project 1",
  "lifecycle_state": "ACTIVE",
  "create_time": "2024-01-15T08:00:00Z",
  "parent": { "type": "organization", "id": "1234567890" },
  "labels": { "env": "prod" }
}
```

เมื่อไม่ส่ง `.project` ปลั๊กอินใช้ `gcloud config get-value project` ถ้าค่านั้นก็ไม่ได้ตั้ง จะออกด้วยรหัส `2` พร้อม `no project ...`

### `set-default`

```json
{
  "ok": true,
  "plugin": "gcloud-project",
  "operation": "set-default",
  "previous": "my-proj-1",
  "current": "my-proj-2",
  "note": "Local gcloud config only; no remote API mutation."
}
```

`previous` เป็น `null` เมื่อไม่มี default ตั้งไว้ก่อน

## คลาสข้อผิดพลาด

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq` หรือไม่ได้ติดตั้ง `gcloud-auth/gcloud.sh` พี่น้องเคียงข้าง หรือไม่มี `gcloud` (ผ่าน `gcloud_assert_cli`) |
| `2` | usage | operation ไม่รู้จัก, `set-default` ไม่มี `.project` หรือ `show` ไม่มี project และไม่มี default ใน `gcloud config` |
| `3` | not-authenticated | ไม่มี credential `gcloud` ที่ active (ผ่าน `gcloud_assert_authenticated`) |
| `6` | gcloud-error | คำสั่ง `gcloud` ภายในล้มเหลว; ข้อความวินิจฉัยที่ตัดสั้นอยู่บน stderr |

การไม่มี `gcloud` CLI ล้มเหลวอย่าง **สะอาด**: ข้อความ stderr ชัดเจน + ออกด้วยรหัสไม่ใช่ศูนย์; ปลั๊กอินไม่เคย panic

## การตั้งค่า

```toml
# workspace.toml
[plugins.gcloud-project]
enabled = true
```

ไม่มี `[config.schema]` — บริบท project ถูก query สด surface ระดับ workspace เพียงตัวเดียวคือคีย์สากล `enabled`

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `workflow` คือ **เอเจนต์** ที่เรียกออกไป (ผ่าน CLI `bwoc gcloud`) `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ นอกจากที่ `gcloud` เองแคชไว้

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยาย; source helpers พี่น้อง; ตรวจว่ามี `jq` บน PATH |
| `invoke` | อ่านคำขอ, เรียก `gcloud projects ...` (หรือ `gcloud config set project`), ปล่อย JSON |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

- `list` และ `show` อ่านอย่างเดียว
- `set-default` เป็น idempotent: การตั้ง project เป็นค่าเดิมคือ no-op สำหรับ `gcloud` config และ envelope ของปลั๊กอินรายงาน `previous == current` ในกรณีนั้น การเล่นซ้ำหลัง `gcloud` ผิดพลาดชั่วคราวลู่เข้า

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `workflow/gcloud-project` ตัวแรกที่รันได้; ทั้งสาม verb ทำงาน จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-55`) และ smoke test ทดสอบ verb แบบ end-to-end กับ `gcloud` ในเครื่องที่ยืนยันตัวตนแล้ว

> [!warning] ช่องว่างการทดสอบจริง การยืนยัน end-to-end กับ project GCP จริงต้องรอ SA JSON sandbox ที่ผู้ดูแลจัดหามาวางที่ `.bwoc/secrets/gcloud-sa.json` (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gcloud.sh`, เส้นทาง `gcloud-auth` พี่น้องหายไปที่ error อย่างสะอาด, เส้นทาง unauthenticated ที่คืน diagnostic `not-authenticated` ที่มีโครงสร้าง และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "workflow"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "gcloud" / "Google Cloud" ปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อตาม [[../../../docs/th/PLUGINS.th#ข้อจำกัดความเป็นกลาง (HARD)|PLUGINS.th.md §ความเป็นกลาง]]) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../../../notes/2026-05-28_gcloud-workflow-plugin-architecture|บันทึกออกแบบ BWOC-51]] — framing ฉบับเต็ม (decisions 1–7)
- [[../gcloud-auth/SPEC.th|gcloud-auth SPEC.th]] — ปลั๊กอินพี่น้อง (สถานะ credential); helpers ถูก source ที่นี่
- [[../../../docs/th/PLUGINS.th|PLUGINS.th.md]] — สเปกปลั๊กอิน; แถว kind `workflow`
- [[auth|auth.toml]] — สัญญา auth (รูปร่างเท่านั้น ไม่มีค่า)
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
