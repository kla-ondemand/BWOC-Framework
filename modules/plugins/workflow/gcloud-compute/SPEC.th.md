---
title: gcloud-compute — วงจรชีวิต Google Cloud Compute Engine
aliases:
  - gcloud-compute
tags:
  - group/framework-plugins
  - type/plugin
  - kind/workflow
  - domain/integration
  - integration/gcloud
maturity: L1
---

# gcloud-compute — วงจรชีวิต Google Cloud Compute Engine

> [!abstract] ปลั๊กอินอ้างอิงสาย `workflow` ตัวแรกที่ **เขียนได้** ของตระกูล GCP (`BWOC-EPIC-9`) รับผิดชอบ **วงจรชีวิตของ Compute Engine instance** — แสดงรายการ instance, สั่ง start, สั่ง stop คำสั่ง: `list` (อ่าน), `start` / `stop` (**เขียน — มี gate ยืนยันโดยผู้ดูแลที่ระดับ CLI** และยังถูก guard ด้วย confirmation marker `BWOC_GCLOUD_CONFIRM` ที่ CLI ตั้งไว้) `delete` ตั้งใจ **ไม่** ส่งมา (เลื่อนออกไป — กู้คืนไม่ได้) source ฟังก์ชันช่วยด้านข้อมูลรับรองจากปลั๊กอินพี่น้อง [[../gcloud-auth/SPEC.th|`gcloud-auth`]] framing ฉบับเต็ม: [[../../../notes/2026-05-28_gcloud-compute-write-verbs|บันทึกออกแบบ BWOC-66]]

## ทำไมต้องเป็นปลั๊กอิน `workflow` ที่เขียนได้

EPIC-8 ส่ง foundation gcloud แบบอ่านเป็นหลัก (`gcloud-auth` + `gcloud-project`) และเลื่อน surface การเขียนทั้งหมดออกไป `gcloud-compute` เปิด surface นั้นด้วย slice ที่มี blast radius ต่ำสุด: start/stop instance มันยังเป็นปลั๊กอิน `workflow` — ไม่ใช่ kind `gcp` ใหม่ — เพราะเฟรมเวิร์กไม่ได้เป็นเจ้าของ schema Compute ที่เป็นบรรทัดฐาน เอเจนต์เรียกออกไปและ JSON ของ `gcloud` เองถูกส่งผ่าน (บันทึก §Decision 1) มัน source การ resolve ข้อมูลรับรอง **เดียวกัน** จาก `gcloud-auth/gcloud.sh` (รูปแบบ shared-helper จาก EPIC-8 §Decision 2) — เพิ่ม verb ไม่ใช่ kind และไม่ถือ auth ของตัวเอง

`start` / `stop` เป็น **verb แรกที่เอเจนต์เรียกถึงได้ซึ่งเปลี่ยนสถานะ infrastructure ระยะไกล** — มันมีค่าใช้จ่ายและรบกวน workload การออกแบบทั้งหมดจึง gate สิ่งนั้นอย่างปลอดภัยขณะที่คงเส้นทางอ่านให้ลื่นไหล

## คำสั่ง (Verbs)

| คำสั่ง | ทิศทาง | Auth | shell-out ไปยัง `gcloud` | Gate |
|---|---|---|---|---|
| `list` | อ่าน | ต้องมี | `gcloud compute instances list --format=json` (ตัวกรอง `--zones=` / `--project=` ทางเลือก) | ไม่มี |
| `start` | **เขียน** | ต้องมี | `gcloud compute instances start --zone=<z> [--project=<p>] --format=json -- <name>` — บูต VM (เกิดค่าใช้จ่าย) | **ยืนยันโดยผู้ดูแล** (ใน CLI `bwoc gcloud compute` — BWOC-68) + guard ด้วย confirmation marker ที่นี่ |
| `stop` | **เขียน** | ต้องมี | `gcloud compute instances stop --zone=<z> [--project=<p>] --format=json -- <name>` — หยุด VM (รบกวน workload) | **ยืนยันโดยผู้ดูแล** + guard ด้วย confirmation marker |

`start` / `stop` กู้คืนได้ (แต่ละตัวย้อนกันได้) `delete` — กู้คืนไม่ได้ (สูญ instance + disk) — ถูก **กันออก** จาก EPIC-9 และเลื่อนไปยัง slice อนาคตที่มี gate แข็งแรงกว่า (บันทึก §Decision 2) verb อ่าน **ไม่มี** gate

## Gate การเขียน (เป็นบรรทัดฐาน)

ตาม [[../../../docs/th/PLUGINS.th#Write verb — operator-confirm gate (normative)|PLUGINS.th.md §Write verb]] gate ยืนยันโดยผู้ดูแลอยู่ที่ **ขอบเขต CLI** (`bwoc gcloud compute`, BWOC-68) — จุดยืนยันจุดเดียว แสดงก่อนลงมือ (เป้าหมาย, zone, สถานะปัจจุบัน, คำสั่ง `gcloud` ตามจริง), ค่าเริ่มต้น **No**, `--yes` สำหรับบริบทที่ไม่โต้ตอบ ปลั๊กอินนี้ **ไม่** ถาม y/N ซ้ำ

มัน **มี** guard เชิงป้องกันเชิงลึก (บันทึก §Decision 3): verb เขียนปฏิเสธที่จะทำงานเว้นแต่ confirmation marker `BWOC_GCLOUD_CONFIRM` ที่ CLI ตั้งไว้ปรากฏอยู่ (ไม่ว่าง) ใน environment ดังนั้นการเรียกปลั๊กอินโดยตรง — ข้าม gate ของ CLI — จะถูกปฏิเสธด้วย envelope "no change" ที่มีโครงสร้าง (`ok: false`, `changed: false`, `reason: "unconfirmed-write"`) แทนที่จะเขียนเงียบ ๆ หรือ fail แบบเปล่า ๆ (ธัมมานุปัสสนา) marker คือการ coupling เพียงอย่างเดียวระหว่าง gate กับปลั๊กอิน ส่วนตาราง `[[verb]]` ใน manifest ประกาศว่า verb ใดเป็นการเขียนเพื่อให้ทั้ง `bwoc check` (BWOC-70) และ gate ของ CLI มองเห็น classification

## การทำงาน

CLI `bwoc gcloud compute` (`BWOC-68`) เรียก `gcloud.sh` จากไดเรกทอรีนี้:

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GCLOUD_OPERATION` (env) | `list` \| `start` \| `stop` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_GCLOUD_CONFIRM` (env) | confirmation marker — ตั้งเป็นค่าที่ไม่ว่างโดย CLI **หลังจาก** ผู้ดูแลยืนยันการเขียน verb อ่านไม่สนใจมัน; `start` / `stop` ปฏิเสธถ้าไม่มี |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace; ใช้ resolve path ของ SA JSON พี่น้อง |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้; ใช้หา helpers พี่น้องที่ `../gcloud-auth/gcloud.sh` |
| stdin | คำขอ JSON บรรทัดเดียว เช่น `{"operation":"list"}`, `{"operation":"start","instance":"vm-1","zone":"us-central1-a"}`, `{"operation":"stop","instance":"vm-1","zone":"us-central1-a","project":"my-proj"}` |

เมื่อสำเร็จ: ออกด้วยรหัส `0`, JSON หนึ่งอ็อบเจกต์บน stdout เมื่อผิดพลาด: ข้อความวินิจฉัยบน stderr + ออกด้วยรหัสไม่ใช่ศูนย์

### การ harden อาร์กิวเมนต์ (#92)

ทุก **positional** ที่ผู้ใช้ระบุ (ชื่อ instance) ถูกส่งไปยัง `gcloud` หลังตัวคั่น end-of-options `--`; ทุก **ค่า flag** ที่ผู้ใช้ระบุ (zone, project) ถูกผูกด้วย `=` ใน argv token เดียว ทั้งคู่ไม่อาจถูก `gcloud` ตีความเป็น flag — ชื่อ instance หรือ zone ที่ขึ้นต้นด้วย `-` ถูกทำให้เป็นกลาง สะท้อนการ [[../../../notes/2026-05-28_gcloud-option-injection-hardening|harden #91/#92]] ที่ปลั๊กอินพี่น้องใช้

## การยืนยันตัวตน

ปลั๊กอินนี้ **ไม่เคยอ่านค่า credential ใด ๆ** มัน source ฟังก์ชันช่วยพี่น้อง [[../gcloud-auth/SPEC.th#การยืนยันตัวตน|`gcloud-auth`]] (`gcloud_assert_cli`, `gcloud_assert_authenticated`) ตอนเริ่ม และให้ `gcloud` ลงมือ ลำดับความสำคัญของ credential เหมือนกับ [[../gcloud-auth/SPEC.th#การยืนยันตัวตน|`gcloud-auth §การยืนยันตัวตน`]]: ADC → `.bwoc/secrets/gcloud-sa.json` → `BWOC_GCLOUD_*` env ปลั๊กอินล้มเหลวเร็วพร้อมข้อความชัดเจนถ้าไม่มี credential ที่ active

ต่างจากปลั๊กอินพี่น้อง `gcloud-compute` **ไม่** ส่ง `auth.toml` — มันไม่ถือ auth ของตัวเอง (บันทึก §Decision 4) สัญญา credential เป็นของพี่น้อง `bwoc check` ไม่ audit ปลั๊กอิน workflow ที่ไม่ส่ง `auth.toml`

> [!danger] **ศีล — อทินนาทาน** ไม่มี token ใดเข้ามาใน address space ของปลั๊กอินนี้ เราเพียงเรียก `gcloud` และส่งผ่านผลลัพธ์ `start` / `stop` ไม่ส่ง, ไม่ log และไม่เก็บ credential ในรูปแบบใด

## รูปร่างผลลัพธ์

### `list`

```json
{
  "ok": true,
  "plugin": "gcloud-compute",
  "operation": "list",
  "total": 2,
  "instances": [
    { "name": "vm-1", "zone": "us-central1-a", "machine_type": "e2-medium", "status": "RUNNING",     "internal_ip": "10.128.0.2", "creation_timestamp": "2026-05-01T08:00:00.000-07:00" },
    { "name": "vm-2", "zone": "us-central1-b", "machine_type": "e2-small",  "status": "TERMINATED",  "internal_ip": "10.128.0.3", "creation_timestamp": "2026-05-02T09:00:00.000-07:00" }
  ]
}
```

`zone` และ `machine_type` ถูกย่อจาก resource URL เต็มของ `gcloud` เหลือแค่ path segment สุดท้าย `total` คือความยาว array ที่ `gcloud` คืนมา; ปลั๊กอินไม่ทำ paging ซ้ำ

### `start` / `stop`

```json
{
  "ok": true,
  "plugin": "gcloud-compute",
  "operation": "start",
  "instance": "vm-1",
  "zone": "us-central1-a",
  "changed": true,
  "result": { "...": "ผลลัพธ์ gcloud --format=json ดิบ" }
}
```

`result` พา JSON output ของ `gcloud` มาตรง ๆ (workflow passthrough — BWOC ไม่เป็นเจ้าของรูปร่าง Compute Mapping) `stop` คืน envelope เดียวกันโดย `"operation": "stop"`

### การเขียนที่ถูกปฏิเสธ (ไม่มี confirmation marker)

```json
{
  "ok": false,
  "plugin": "gcloud-compute",
  "operation": "start",
  "changed": false,
  "reason": "unconfirmed-write",
  "message": "write verb 'start' requires operator confirmation; the bwoc gcloud compute CLI sets BWOC_GCLOUD_CONFIRM after a y/N prompt. Direct plugin invocation of a write verb is refused — no instance was changed."
}
```

ปล่อยออกมา (ออกด้วยรหัส `5`) เมื่อ verb เขียนถูกเรียกโดยไม่มี `BWOC_GCLOUD_CONFIRM` ในเส้นทาง CLI ปกติ marker จะถูกตั้งเสมอหลังผู้ดูแลยืนยัน ดังนั้นกรณีนี้จะเกิดเฉพาะตอน direct-invoke ที่พยายาม bypass

## คลาสข้อผิดพลาด

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq`, หรือ helpers พี่น้อง `gcloud-auth/gcloud.sh` ไม่ได้ติดตั้งข้าง ๆ, หรือไม่มี `gcloud` (ตาม `gcloud_assert_cli`) |
| `2` | usage | operation ไม่รู้จัก, หรือ `start` / `stop` ถูกเรียกโดยไม่มี `.instance` หรือ `.zone` |
| `3` | not-authenticated | ไม่มี credential `gcloud` ที่ active (ตาม `gcloud_assert_authenticated`) |
| `5` | unconfirmed-write | verb เขียน (`start` / `stop`) ถูกเรียกโดยไม่มี marker `BWOC_GCLOUD_CONFIRM` — ปฏิเสธ ไม่มีการเปลี่ยนแปลง |
| `6` | gcloud-error | คำสั่ง `gcloud compute` ที่อยู่เบื้องล่างล้มเหลว (เช่น ไม่พบ instance, ไม่มีสิทธิ์, 4xx); ข้อความวินิจฉัยที่ถูกตัดอยู่บน stderr |

การไม่มี `gcloud` CLI ล้มเหลวอย่าง **สะอาด**: ข้อความ stderr ชัดเจน + ออกด้วยรหัสไม่ใช่ศูนย์; ปลั๊กอินไม่เคย panic

## การตั้งค่า

```toml
# workspace.toml
[plugins.gcloud-compute]
enabled = true
```

ไม่มี `[config.schema]` — สถานะ compute อ่าน live จาก `gcloud` surface ระดับ workspace เพียงตัวเดียวคือคีย์สากล `enabled` ตาราง `[[verb]]` ใน `manifest.toml` ประกาศ classification การเขียน (บริโภคโดย `bwoc check` BWOC-70 + gate ของ CLI BWOC-68) ไม่ใช่ config ของผู้ดูแล

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `workflow` คือ **เอเจนต์** ที่เรียกออกไป (ผ่าน CLI `bwoc gcloud compute`) `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ นอกจากที่ `gcloud` แคชไว้

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยายต่อการเรียก; source helpers พี่น้อง; ตรวจว่ามี `jq` บน PATH |
| `invoke` | อ่านคำขอ, (สำหรับการเขียน) ตรวจ confirmation marker, เรียก `gcloud compute instances ...`, ปล่อย JSON |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

- `list` อ่านอย่างเดียว
- `start` เป็น idempotent: สั่ง start instance ที่ `RUNNING` อยู่แล้วเป็น no-op สำหรับ `gcloud` และคืนสำเร็จ `stop` ก็เช่นกันกับ instance ที่ `TERMINATED` อยู่แล้ว การเล่นซ้ำหลัง error ชั่วคราวของ `gcloud` ลู่เข้าสู่สถานะวงจรชีวิตที่ร้องขอ

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `workflow/gcloud-compute` ตัวแรกที่รันได้; ทั้งสาม verb ทำงาน, gate การเขียนถูกต่อสาย จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-70`) และ smoke test ทดสอบ verb แบบ end-to-end กับ instance จริง

> [!warning] ช่องว่างการทดสอบจริง การยืนยัน end-to-end (start/stop VM จริง) ต้องรอ project GCP + instance ทดสอบแบบใช้แล้วทิ้งที่ผู้ดูแลจัดหา (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gcloud.sh`, เส้นทาง gcloud-auth-พี่น้อง-หายไปที่ error อย่างสะอาด, เส้นทางยังไม่ยืนยันตัวตนที่คืนการวินิจฉัย `not-authenticated` ที่มีโครงสร้าง, เส้นทาง unconfirmed-write ที่คืนการปฏิเสธ `unconfirmed-write` ที่มีโครงสร้าง และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "workflow"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "gcloud" / "Google Cloud" / "Compute Engine" ปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อตาม [[../../../docs/th/PLUGINS.th#ข้อจำกัดความเป็นกลาง (HARD)|PLUGINS.th.md §ความเป็นกลาง]]) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../../../notes/2026-05-28_gcloud-compute-write-verbs|บันทึกออกแบบ BWOC-66]] — risk matrix ของ verb เขียน + framing ของ confirm-gate (decisions 1–5)
- [[../gcloud-auth/SPEC.th|gcloud-auth SPEC.th]] — ปลั๊กอินพี่น้อง (สถานะ credential); source helpers ที่นี่
- [[../gcloud-project/SPEC.th|gcloud-project SPEC.th]] — ปลั๊กอินพี่น้อง (บริบท project); precedent ของ shared-helper + write-gate เดียวกัน
- [[../../../docs/th/PLUGINS.th|PLUGINS.th.md]] — สเปกปลั๊กอิน; แถว kind `workflow` + §Gate การเขียน
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
