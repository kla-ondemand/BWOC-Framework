---
title: gws-gmail — เธรด & ป้ายกำกับ Google Gmail (อ่านเป็นหลัก)
aliases:
  - gws-gmail
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-gmail — เธรด & ป้ายกำกับ Google Gmail (อ่านเป็นหลัก)

> [!abstract] ปลั๊กอินรายบริการของ kind `gws` (`BWOC-EPIC-13`) อ่าน **Google Gmail** — `search` (Gmail `threads.list` เสริมรายเธรดด้วย `threads.get`), `show` (หนึ่งเธรดผ่าน `threads.get`), และ `labels` (`labels.list`) — แล้วฉายแต่ละเธรดเป็น [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|รูปร่างเธรด Gmail ที่เป็นบรรทัดฐาน]] มัน**ไม่เคย**ส่งเมลหรือแก้ป้ายกำกับ (อ่านเป็นหลักโดยการออกแบบ — slice เขียนอย่าง `send` ถูกเลื่อนออกไป, บันทึก §Decision 4) มัน source ฟังก์ชันช่วยข้อมูลรับรอง OAuth จาก foundation [[../gws-auth/SPEC.th|`gws-auth`]] จึงไม่มีโค้ด auth ของตัวเอง ต้องมี scope `gmail.readonly` เหตุผลฉบับเต็ม: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]]

## คำสั่ง (Verbs)

| คำสั่ง | ชื่อพ้อง | ทิศทาง | endpoint ของ Gmail | ผลข้างเคียง |
|---|---|---|---|---|
| `search` | `threads` | อ่าน | `GET /users/me/threads` (`threads.list`) + `threads.get` รายเธรด | ไม่มี — แบ่งหน้าภายใน; `--max` จำกัดจำนวนผลรวม |
| `show` | `message`, `messages` | อ่าน | `GET /users/me/threads/{id}` (`threads.get`, เมทาดาทา) | ไม่มี — เมทาดาทาของหนึ่งเธรด |
| `labels` | — | อ่าน | `GET /users/me/labels` (`labels.list`) | ไม่มี — ชุดป้ายกำกับของผู้ใช้ |

`search` และ `show` ฉายอ็อบเจกต์ REST ของ Gmail เป็นรูปร่างเธรด Gmail (`thread_id`, `subject`, `from`, `last_message_time`, ทางเลือก `snippet` / `labels`) `search` คืนอาร์เรย์ใต้คีย์ `threads`; `show` กระจายหนึ่งรายการลงใน envelope `labels` คืนอ็อบเจกต์ป้ายกำกับ (`label_id`, `name`, `type`)

> [!note] ชื่อคำสั่ง CLI `bwoc gws` (`BWOC-74`) เรียก `search` / `show` / `labels` ชื่อเชิงแนวคิดจาก brief ของ EPIC-13 — *threads* (`search`) และ *message* (`show`) — ถูกรับเป็นชื่อพ้องเพื่อให้เรียกตรง ๆ ได้ทั้งสองแบบ `search` resolve เมทาดาทาข้อความล่าสุดของแต่ละเธรดอยู่แล้ว จึงไม่จำเป็นต้องมี verb รายข้อความแยกสำหรับพื้นผิวอ่านเป็นหลัก

## การทำงาน

CLI `bwoc gws` (`BWOC-74`) เรียก `gws.sh` จากไดเรกทอรีนี้:

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `search` \| `show` \| `labels` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace (resolve ไฟล์ token ผ่าน sibling) |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้ — ใช้หา `../gws-auth/gws.sh` |
| `BWOC_GWS_TOKEN` (env) | OAuth2 access token — **ความลับ**, บริโภคโดยฟังก์ชันช่วยของ sibling |
| stdin | คำขอ JSON บรรทัดเดียว — ดูตัวอย่างสัญญาด้านล่าง |

```jsonc
{"operation":"search"}
{"operation":"search","query":"from:me is:unread","max":25}
{"operation":"show","thread_id":"18ab12cd34ef5678"}
{"operation":"labels"}
```

`.query` คือ [query ค้นหา](https://developers.google.com/gmail/api/guides/filtering) ของ Gmail ที่ส่งเป็นพารามิเตอร์ `q`; `.max` จำกัดจำนวนเธรดที่คืน (ค่าเริ่มต้น `100`)

เมื่อสำเร็จ: ออกด้วยรหัส `0`, JSON หนึ่งอ็อบเจกต์บน stdout เมื่อผิดพลาด: ข้อความวินิจฉัยทาง stderr + ออกด้วยรหัสไม่ใช่ศูนย์

## การยืนยันตัวตนและ scope

ปลั๊กอินนี้ **ไม่มีโค้ดข้อมูลรับรอง** มัน source `gws-auth/gws.sh` และเรียก `gws_curl` ซึ่ง resolve token (env → ไฟล์ token), refresh เมื่อหมดอายุ, ตั้ง `Authorization: Bearer`, และจัดการ rate limit ดูโมเดล token ที่ [[../gws-auth/SPEC.th|gws-auth]]

มันต้องมี scope **`https://www.googleapis.com/auth/gmail.readonly`** เพราะ scope ของ OAuth เป็นรายบริการและผูกกับการยินยอม token ที่ได้รับเพียง scope Drive หรือ Calendar จะคืน HTTP 403 ที่นี่ — แสดงเป็น `token lacks the required scope for Gmail threads` ไม่ใช่ความล้มเหลวเปล่า ๆ

> [!danger] **ศีล — อทินนาทาน** token ไม่เคยเข้าสู่ผลลัพธ์ของปลั๊กอินนี้ มันถูกส่งให้ curl โดยฟังก์ชันช่วยของ sibling เป็น header คำขอเท่านั้น; ปลั๊กอินฉายการตอบกลับ JSON ของ Gmail — ไม่ใช่ข้อมูลรับรอง — เป็นรายการเธรด

## รูปร่างผลลัพธ์

### `search`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "search",
  "total": 2,
  "threads": [
    {
      "thread_id": "18ab12cd34ef5678",
      "subject": "Sprint 13 review",
      "from": "jisoo@example.com",
      "snippet": "Closing EPIC-13 — last impl story…",
      "labels": ["INBOX", "IMPORTANT"],
      "last_message_time": "2026-05-28T09:00:00Z"
    },
    {
      "thread_id": "18ab99887766aabb",
      "subject": "Re: gws-auth helpers",
      "from": "lisa@example.com",
      "last_message_time": "2026-05-27T14:00:00Z"
    }
  ]
}
```

### `show`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "show",
  "thread_id": "18ab12cd34ef5678",
  "subject": "Sprint 13 review",
  "from": "jisoo@example.com",
  "snippet": "Closing EPIC-13 — last impl story…",
  "labels": ["INBOX", "IMPORTANT"],
  "last_message_time": "2026-05-28T09:00:00Z"
}
```

### `labels`

```json
{
  "ok": true,
  "plugin": "gws-gmail",
  "operation": "labels",
  "total": 2,
  "labels": [
    { "label_id": "INBOX", "name": "INBOX", "type": "system" },
    { "label_id": "Label_42", "name": "BWOC", "type": "user" }
  ]
}
```

ฟิลด์ทางเลือก (`snippet`, `labels`) **ถูกละเว้นเมื่อไม่มี** ไม่ใช่ `null` — ตามแบบแผน resource-schema ของเฟรมเวิร์ก

## การแบ่งหน้าและ rate limit

`search` แบ่งหน้าภายใน: ขอทีละหน้าได้สูงสุด 100 เธรด ตาม `nextPageToken` ของ Gmail และหยุดทันทีที่ถึง `--max` (หรือรายการหมด) แต่ละเธรดที่เก็บมาจะถูกเสริมด้วย `threads.get` (เมทาดาทาเท่านั้น — header Subject/From/Date) เพื่อเติมฟิลด์บังคับ `subject` / `from` / `last_message_time` มันคืน envelope ที่มีขอบเขตเดียวเพื่อให้เอเจนต์ไม่ดึงกล่องเมลแบบไม่มีขอบเขต การจัดการ rate limit (HTTP 429) ทำโดย `gws_curl` ของ sibling — มันเคารพ `Retry-After` พร้อม fallback กำลังสอง สูงสุดสี่ครั้ง ก่อนแสดง error ที่ลองใหม่ได้ เธรดเดียวที่ 404 ระหว่าง `list` กับ `get` (ถูกลบกลางคัน) จะถูกข้าม; error auth/rate เชิงระบบยังคงยกเลิกทั้งหมด

## คลาสข้อผิดพลาด

สืบทอดจาก `gws_classify_status` ของ sibling (ดังนั้นปลั๊กอิน `gws-*` ทุกตัวแม็พ HTTP เหมือนกัน):

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq` หรือ `curl` บน PATH |
| `2` | usage / no-token | operation ไม่รู้จัก / ไม่ได้ระบุ, ไม่มี `.thread_id`, `thread_id` ไม่ถูกต้อง, หรือ resolve token ไม่ได้ |
| `3` | auth / scope | HTTP 401 (token ไม่ถูกต้อง) หรือ 403 (ขาด `gmail.readonly`) |
| `4` | rate-limited | HTTP 429 หลังหมดงบ backoff |
| `5` | not-found | HTTP 404 (ไม่มีเธรดนั้น) |
| `6` | transport / unexpected | network ล้มเหลว หรือ HTTP status ที่ไม่ได้แม็พ |

`thread_id` ที่ถูกประดิษฐ์ขึ้นไม่สามารถ inject เข้า URL คำขอได้ — `show` ปฏิเสธ id ใด ๆ ที่อยู่นอก `[A-Za-z0-9_-]` ก่อนยิงคำขอ

## การตั้งค่า

```toml
# workspace.toml
[plugins.gws-gmail]
enabled = true
```

ไม่มี `[config.schema]` — ปลั๊กอินไม่มี config ของตัวเอง; ข้อมูลรับรอง resolve ผ่าน foundation `gws-auth`

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `gws` คือ **เอเจนต์** ที่เรียกออกไปผ่าน CLI `bwoc gws` `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยายต่อการเรียก; ตรวจว่ามี `jq` + `curl` บน PATH และมีฟังก์ชันช่วยของ sibling |
| `invoke` | อ่านคำขอ, เรียก Gmail ผ่าน `gws_curl` ของ sibling, ฉายการตอบกลับเป็นรายการเธรด / ป้ายกำกับ |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

ทั้งสาม verb อ่านอย่างเดียวและลำดับคงที่ทั่วทุกการเล่นซ้ำ `search` ให้ผลแน่นอนสำหรับสถานะกล่องเมล + query + `max` ที่กำหนด; การแบ่งหน้าและการเสริมรายเธรดเป็นเรื่องภายในและไม่เคยแก้ไขสิ่งใดบางส่วน

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `gws/gws-gmail` ตัวแรกที่รันได้; ทั้งสาม verb ทำงาน จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-77`) และ smoke test ทดสอบ end-to-end กับ OAuth token ของผู้ดูแลที่มี `gmail.readonly`

> [!warning] ช่องว่างการทดสอบจริง การยืนยันจริง (token `gmail.readonly` จริงอ่านเธรดจริง) ต้องรอ OAuth token ที่ผู้ดูแลจัดหา (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gws.sh`, เส้นทาง dependency-หายไป + token-หายไป + `thread_id`-ผิด ที่ error อย่างสะอาด และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "gws"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "Google Gmail" / "Google Workspace" ปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อ) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../gws-auth/SPEC.th|gws-auth SPEC.th]] — foundation ข้อมูลรับรอง OAuth ที่ปลั๊กอินนี้ source
- [[../gws-drive/SPEC.th|gws-drive SPEC.th]] — ปลั๊กอิน Drive พี่น้อง (รูปทรงตระกูลเดียวกัน)
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]] — framing EPIC-13 ฉบับเต็ม (decisions 1–5)
- [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|PLUGINS.th.md §Workspace Resource Schema]] — รูปร่างเธรด Gmail ที่เป็นบรรทัดฐานที่ปลั๊กอินนี้ปล่อย
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
