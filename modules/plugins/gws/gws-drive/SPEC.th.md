---
title: gws-drive — ไฟล์ Google Drive (อ่านเป็นหลัก)
aliases:
  - gws-drive
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-drive — ไฟล์ Google Drive (อ่านเป็นหลัก)

> [!abstract] ปลั๊กอินรายบริการของ kind `gws` (`BWOC-EPIC-13`) อ่าน **Google Drive** — `list` (Drive `files.list`) และ `get` (Drive `files.get` เมทาดาทา) — แล้วฉายผลแต่ละรายการเป็น [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|รูปร่างไฟล์ Drive ที่เป็นบรรทัดฐาน]] มัน**ไม่เคย**เขียนกลับไปยัง Drive (อ่านเป็นหลักโดยการออกแบบ — slice เขียนอย่าง upload ถูกเลื่อนออกไป, บันทึก §Decision 4) มัน source ฟังก์ชันช่วยข้อมูลรับรอง OAuth จาก foundation [[../gws-auth/SPEC.th|`gws-auth`]] จึงไม่มีโค้ด auth ของตัวเอง ต้องมี scope `drive.readonly` เหตุผลฉบับเต็ม: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]]

## คำสั่ง (Verbs)

| คำสั่ง | ทิศทาง | endpoint ของ Drive | ผลข้างเคียง |
|---|---|---|---|
| `list` | อ่าน | `GET /drive/v3/files` (`files.list`) | ไม่มี — แบ่งหน้าภายใน; `--max` จำกัดจำนวนผลรวม |
| `get` | อ่าน | `GET /drive/v3/files/{fileId}` (`files.get`) | ไม่มี — เมทาดาทาเท่านั้น; ไม่เคยดาวน์โหลดเนื้อหา |

ทั้งคู่ฉายอ็อบเจกต์ REST ของ Drive เป็นรูปร่างไฟล์ Drive (`file_id`, `name`, `mime_type`, `modified_time`, ทางเลือก `owners` / `web_view_link`) `get` คืนหนึ่งรายการ; `list` คืนเป็นอาร์เรย์

## การทำงาน

CLI `bwoc gws` (`BWOC-74`) เรียก `gws.sh` จากไดเรกทอรีนี้:

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `list` \| `get` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace (resolve ไฟล์ token ผ่าน sibling) |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้ — ใช้หา `../gws-auth/gws.sh` |
| `BWOC_GWS_TOKEN` (env) | OAuth2 access token — **ความลับ**, บริโภคโดยฟังก์ชันช่วยของ sibling |
| stdin | คำขอ JSON บรรทัดเดียว — ดูตัวอย่างสัญญาด้านล่าง |

```jsonc
{"operation":"list"}
{"operation":"list","query":"mimeType='application/pdf'","max":50}
{"operation":"get","file_id":"1AbC_dEfGhIjKlMnOpQrStUvWxYz"}
```

`.query` คือ [query ค้นหา](https://developers.google.com/drive/api/guides/search-files) ของ Drive ที่ส่งเป็นพารามิเตอร์ `q`; `.max` จำกัดจำนวนไฟล์ที่คืน (ค่าเริ่มต้น `100`)

เมื่อสำเร็จ: ออกด้วยรหัส `0`, JSON หนึ่งอ็อบเจกต์บน stdout เมื่อผิดพลาด: ข้อความวินิจฉัยทาง stderr + ออกด้วยรหัสไม่ใช่ศูนย์

## การยืนยันตัวตนและ scope

ปลั๊กอินนี้ **ไม่มีโค้ดข้อมูลรับรอง** มัน source `gws-auth/gws.sh` และเรียก `gws_curl` ซึ่ง resolve token (env → ไฟล์ token), refresh เมื่อหมดอายุ, ตั้ง `Authorization: Bearer`, และจัดการ rate limit ดูโมเดล token ที่ [[../gws-auth/SPEC.th|gws-auth]]

มันต้องมี scope **`https://www.googleapis.com/auth/drive.readonly`** เพราะ scope ของ OAuth เป็นรายบริการและผูกกับการยินยอม token ที่ได้รับเพียง scope Gmail หรือ Calendar จะคืน HTTP 403 ที่นี่ — แสดงเป็น `token lacks the required scope for Drive files` ไม่ใช่ความล้มเหลวเปล่า ๆ

> [!danger] **ศีล — อทินนาทาน** token ไม่เคยเข้าสู่ผลลัพธ์ของปลั๊กอินนี้ มันถูกส่งให้ curl โดยฟังก์ชันช่วยของ sibling เป็น header คำขอเท่านั้น; ปลั๊กอินฉายการตอบกลับ JSON ของ Drive — ไม่ใช่ข้อมูลรับรอง — เป็นรายการไฟล์ Drive

## รูปร่างผลลัพธ์

### `list`

```json
{
  "ok": true,
  "plugin": "gws-drive",
  "operation": "list",
  "total": 2,
  "files": [
    {
      "file_id": "1AbC_dEfGhIjKlMnOpQrStUvWxYz",
      "name": "BWOC Architecture.gdoc",
      "mime_type": "application/vnd.google-apps.document",
      "modified_time": "2026-05-27T09:00:00Z",
      "web_view_link": "https://docs.google.com/document/d/1AbC_dEfGhIjKlMnOpQrStUvWxYz/edit"
    },
    {
      "file_id": "2XyZ...",
      "name": "notes.pdf",
      "mime_type": "application/pdf",
      "modified_time": "2026-05-26T11:00:00Z",
      "owners": ["me@example.com"]
    }
  ]
}
```

### `get`

```json
{
  "ok": true,
  "plugin": "gws-drive",
  "operation": "get",
  "file": {
    "file_id": "1AbC_dEfGhIjKlMnOpQrStUvWxYz",
    "name": "BWOC Architecture.gdoc",
    "mime_type": "application/vnd.google-apps.document",
    "modified_time": "2026-05-27T09:00:00Z",
    "web_view_link": "https://docs.google.com/document/d/1AbC_dEfGhIjKlMnOpQrStUvWxYz/edit"
  }
}
```

ฟิลด์ทางเลือก (`owners`, `web_view_link`) **ถูกละเว้นเมื่อไม่มี** ไม่ใช่ `null` — ตามแบบแผน resource-schema ของเฟรมเวิร์ก

## การแบ่งหน้าและ rate limit

`list` แบ่งหน้าภายใน: ขอทีละหน้าได้สูงสุด 100 ไฟล์ ตาม `nextPageToken` ของ Drive และหยุดทันทีที่ถึง `--max` (หรือรายการหมด) มันคืน envelope ที่มีขอบเขตเดียวเพื่อให้เอเจนต์ไม่ดึง Drive แบบไม่มีขอบเขต การจัดการ rate limit (HTTP 429) ทำโดย `gws_curl` ของ sibling — มันเคารพ `Retry-After` พร้อม fallback กำลังสอง สูงสุดสี่ครั้ง ก่อนแสดง error ที่ลองใหม่ได้

## คลาสข้อผิดพลาด

สืบทอดจาก `gws_classify_status` ของ sibling (ดังนั้นปลั๊กอิน `gws-*` ทุกตัวแม็พ HTTP เหมือนกัน):

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq` หรือ `curl` บน PATH |
| `2` | usage / no-token | operation ไม่รู้จัก / ไม่ได้ระบุ, ไม่มี `.file_id`, `file_id` ไม่ถูกต้อง, หรือ resolve token ไม่ได้ |
| `3` | auth / scope | HTTP 401 (token ไม่ถูกต้อง) หรือ 403 (ขาด `drive.readonly`) |
| `4` | rate-limited | HTTP 429 หลังหมดงบ backoff |
| `5` | not-found | HTTP 404 (ไม่มีไฟล์นั้น) |
| `6` | transport / unexpected | network ล้มเหลว หรือ HTTP status ที่ไม่ได้แม็พ |

`file_id` ที่ถูกประดิษฐ์ขึ้นไม่สามารถ inject เข้า URL คำขอได้ — `get` ปฏิเสธ id ใด ๆ ที่อยู่นอก `[A-Za-z0-9_-]` ก่อนยิงคำขอ

## การตั้งค่า

```toml
# workspace.toml
[plugins.gws-drive]
enabled = true
```

ไม่มี `[config.schema]` — ปลั๊กอินไม่มี config ของตัวเอง; ข้อมูลรับรอง resolve ผ่าน foundation `gws-auth`

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `gws` คือ **เอเจนต์** ที่เรียกออกไปผ่าน CLI `bwoc gws` `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยายต่อการเรียก; ตรวจว่ามี `jq` + `curl` บน PATH และมีฟังก์ชันช่วยของ sibling |
| `invoke` | อ่านคำขอ, เรียก Drive ผ่าน `gws_curl` ของ sibling, ฉายการตอบกลับเป็นรายการไฟล์ Drive |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

ทั้งสอง verb อ่านอย่างเดียวและลำดับคงที่ทั่วทุกการเล่นซ้ำ `list` ให้ผลแน่นอนสำหรับสถานะ Drive + query + `max` ที่กำหนด; การแบ่งหน้าเป็นเรื่องภายในและไม่เคยแก้ไขสิ่งใดบางส่วน

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `gws/gws-drive` ตัวแรกที่รันได้; ทั้งสอง verb ทำงาน จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-77`) และ smoke test ทดสอบ end-to-end กับ OAuth token ของผู้ดูแลที่มี `drive.readonly`

> [!warning] ช่องว่างการทดสอบจริง การยืนยันจริง (token `drive.readonly` จริงอ่านไฟล์จริง) ต้องรอ OAuth token ที่ผู้ดูแลจัดหา (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gws.sh`, เส้นทาง dependency-หายไป + token-หายไป + `file_id`-ผิด ที่ error อย่างสะอาด และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "gws"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "Google Drive" / "Google Workspace" ปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อ) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../gws-auth/SPEC.th|gws-auth SPEC.th]] — foundation ข้อมูลรับรอง OAuth ที่ปลั๊กอินนี้ source
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]] — framing EPIC-13 ฉบับเต็ม (decisions 1–5)
- [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|PLUGINS.th.md §Workspace Resource Schema]] — รูปร่างไฟล์ Drive ที่เป็นบรรทัดฐานที่ปลั๊กอินนี้ปล่อย
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
