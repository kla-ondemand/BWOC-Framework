---
title: gws-calendar — ปฏิทิน & เหตุการณ์ Google Calendar (อ่านเป็นหลัก)
aliases:
  - gws-calendar
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-calendar — ปฏิทิน & เหตุการณ์ Google Calendar (อ่านเป็นหลัก)

> [!abstract] ปลั๊กอินรายบริการของ kind `gws` (`BWOC-EPIC-13`) อ่าน **Google Calendar** — `calendars` (Calendar `calendarList.list`) และ `events` (Calendar `events.list`) — แล้วฉายแต่ละเหตุการณ์เป็น [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|รูปร่างเหตุการณ์ปฏิทินที่เป็นบรรทัดฐาน]] มัน**ไม่เคย**สร้างหรือแก้เหตุการณ์ (อ่านเป็นหลักโดยการออกแบบ — slice เขียนอย่าง `events.insert` ถูกเลื่อนออกไป, บันทึก §Decision 4) มัน source ฟังก์ชันช่วยข้อมูลรับรอง OAuth จาก foundation [[../gws-auth/SPEC.th|`gws-auth`]] จึงไม่มีโค้ด auth ของตัวเอง ต้องมี scope `calendar.readonly` เหตุผลฉบับเต็ม: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]]

## คำสั่ง (Verbs)

| คำสั่ง | ชื่อพ้อง | ทิศทาง | endpoint ของ Calendar | ผลข้างเคียง |
|---|---|---|---|---|
| `calendars` | `list` | อ่าน | `GET /users/me/calendarList` (`calendarList.list`) | ไม่มี — แบ่งหน้าภายใน |
| `events` | — | อ่าน | `GET /calendars/{calendarId}/events` (`events.list`) | ไม่มี — แบ่งหน้าภายใน; `--max` จำกัดจำนวนผลรวม |

`calendars` คืนปฏิทินที่ token มองเห็น (`calendar_id`, `summary`, ทางเลือก `primary` / `access_role`) `events` ฉายแต่ละเหตุการณ์เป็นรูปร่างเหตุการณ์ปฏิทิน (`event_id`, `calendar_id`, `summary`, `start`, `end`, ทางเลือก `attendees_count`) และคืนอาร์เรย์ใต้คีย์ `events`

> [!note] ชื่อคำสั่ง CLI `bwoc gws` (`BWOC-74`) เรียก operation `calendars` เบื้องหลัง subcommand `calendar list`; `list` ถูกรับเป็นชื่อพ้องเพื่อให้เรียกตรง ๆ ได้ทั้งสองแบบ

## การทำงาน

CLI `bwoc gws` (`BWOC-74`) เรียก `gws.sh` จากไดเรกทอรีนี้:

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `calendars` \| `events` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace (resolve ไฟล์ token ผ่าน sibling) |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้ — ใช้หา `../gws-auth/gws.sh` |
| `BWOC_GWS_TOKEN` (env) | OAuth2 access token — **ความลับ**, บริโภคโดยฟังก์ชันช่วยของ sibling |
| stdin | คำขอ JSON บรรทัดเดียว — ดูตัวอย่างสัญญาด้านล่าง |

```jsonc
{"operation":"calendars"}
{"operation":"events"}
{"operation":"events","calendar_id":"primary","max":50}
{"operation":"events","calendar_id":"team@group.calendar.google.com"}
```

`.calendar_id` เลือกปฏิทินที่จะอ่านเหตุการณ์ (ค่าเริ่มต้น `primary`); `.max` จำกัดจำนวนเหตุการณ์ที่คืน (ค่าเริ่มต้น `100`)

เมื่อสำเร็จ: ออกด้วยรหัส `0`, JSON หนึ่งอ็อบเจกต์บน stdout เมื่อผิดพลาด: ข้อความวินิจฉัยทาง stderr + ออกด้วยรหัสไม่ใช่ศูนย์

## การยืนยันตัวตนและ scope

ปลั๊กอินนี้ **ไม่มีโค้ดข้อมูลรับรอง** มัน source `gws-auth/gws.sh` และเรียก `gws_curl` ซึ่ง resolve token (env → ไฟล์ token), refresh เมื่อหมดอายุ, ตั้ง `Authorization: Bearer`, และจัดการ rate limit ดูโมเดล token ที่ [[../gws-auth/SPEC.th|gws-auth]]

มันต้องมี scope **`https://www.googleapis.com/auth/calendar.readonly`** เพราะ scope ของ OAuth เป็นรายบริการและผูกกับการยินยอม token ที่ได้รับเพียง scope Drive หรือ Gmail จะคืน HTTP 403 ที่นี่ — แสดงเป็น `token lacks the required scope for calendar 'primary' events` ไม่ใช่ความล้มเหลวเปล่า ๆ

> [!danger] **ศีล — อทินนาทาน** token ไม่เคยเข้าสู่ผลลัพธ์ของปลั๊กอินนี้ มันถูกส่งให้ curl โดยฟังก์ชันช่วยของ sibling เป็น header คำขอเท่านั้น; ปลั๊กอินฉายการตอบกลับ JSON ของ Calendar — ไม่ใช่ข้อมูลรับรอง — เป็นรายการเหตุการณ์

## รูปร่างผลลัพธ์

### `calendars`

```json
{
  "ok": true,
  "plugin": "gws-calendar",
  "operation": "calendars",
  "total": 2,
  "calendars": [
    { "calendar_id": "primary", "summary": "me@example.com", "primary": true, "access_role": "owner" },
    { "calendar_id": "team@group.calendar.google.com", "summary": "BWOC Team", "access_role": "reader" }
  ]
}
```

### `events`

```json
{
  "ok": true,
  "plugin": "gws-calendar",
  "operation": "events",
  "total": 2,
  "events": [
    {
      "event_id": "abc123def456",
      "calendar_id": "primary",
      "summary": "Sprint 13 review",
      "start": "2026-05-28T09:00:00Z",
      "end": "2026-05-28T10:00:00Z",
      "attendees_count": 4
    },
    {
      "event_id": "ghi789jkl012",
      "calendar_id": "primary",
      "summary": "All-day offsite",
      "start": "2026-06-01",
      "end": "2026-06-02"
    }
  ]
}
```

ฟิลด์ทางเลือก (`primary`, `access_role`, `attendees_count`) **ถูกละเว้นเมื่อไม่มี** ไม่ใช่ `null` — ตามแบบแผน resource-schema ของเฟรมเวิร์ก `start` / `end` เป็น date-time สำหรับเหตุการณ์มีเวลา และเป็น date สำหรับเหตุการณ์ทั้งวัน

## การแบ่งหน้าและ rate limit

ทั้งสอง verb แบ่งหน้าภายใน: `calendars` ตาม `nextPageToken` ของ `calendarList.list`; `events` ขอทีละหน้าได้สูงสุด 100 เหตุการณ์ (`singleEvents=true`, `orderBy=startTime` เพื่อลำดับที่แน่นอนและกระจายเหตุการณ์ซ้ำ), ตาม `nextPageToken` และหยุดทันทีที่ถึง `--max` (หรือรายการหมด) แต่ละอันคืน envelope ที่มีขอบเขตเดียวเพื่อให้เอเจนต์ไม่ดึงปฏิทินแบบไม่มีขอบเขต การจัดการ rate limit (HTTP 429) ทำโดย `gws_curl` ของ sibling — มันเคารพ `Retry-After` พร้อม fallback กำลังสอง สูงสุดสี่ครั้ง ก่อนแสดง error ที่ลองใหม่ได้

## คลาสข้อผิดพลาด

สืบทอดจาก `gws_classify_status` ของ sibling (ดังนั้นปลั๊กอิน `gws-*` ทุกตัวแม็พ HTTP เหมือนกัน):

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq` หรือ `curl` บน PATH |
| `2` | usage / no-token | operation ไม่รู้จัก / ไม่ได้ระบุ, `calendar_id` ไม่ถูกต้อง, หรือ resolve token ไม่ได้ |
| `3` | auth / scope | HTTP 401 (token ไม่ถูกต้อง) หรือ 403 (ขาด `calendar.readonly`) |
| `4` | rate-limited | HTTP 429 หลังหมดงบ backoff |
| `5` | not-found | HTTP 404 (ไม่มีปฏิทินนั้น) |
| `6` | transport / unexpected | network ล้มเหลว หรือ HTTP status ที่ไม่ได้แม็พ |

`calendar_id` ที่ถูกประดิษฐ์ขึ้นไม่สามารถ inject เข้า URL คำขอได้ — `events` ปฏิเสธ id ใด ๆ ที่อยู่นอก `[A-Za-z0-9_.@-]` และ percent-encode มันเข้า path ก่อนยิงคำขอ

## การตั้งค่า

```toml
# workspace.toml
[plugins.gws-calendar]
enabled = true
```

ไม่มี `[config.schema]` — ปลั๊กอินไม่มี config ของตัวเอง; ข้อมูลรับรอง resolve ผ่าน foundation `gws-auth`

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `gws` คือ **เอเจนต์** ที่เรียกออกไปผ่าน CLI `bwoc gws` `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยายต่อการเรียก; ตรวจว่ามี `jq` + `curl` บน PATH และมีฟังก์ชันช่วยของ sibling |
| `invoke` | อ่านคำขอ, เรียก Calendar ผ่าน `gws_curl` ของ sibling, ฉายการตอบกลับเป็นรายการปฏิทิน / เหตุการณ์ |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

ทั้งสอง verb อ่านอย่างเดียวและลำดับคงที่ทั่วทุกการเล่นซ้ำ `events` ให้ผลแน่นอนสำหรับสถานะปฏิทิน + `calendar_id` + `max` ที่กำหนด; การแบ่งหน้าเป็นเรื่องภายในและไม่เคยแก้ไขสิ่งใดบางส่วน

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `gws/gws-calendar` ตัวแรกที่รันได้; ทั้งสอง verb ทำงาน จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-77`) และ smoke test ทดสอบ end-to-end กับ OAuth token ของผู้ดูแลที่มี `calendar.readonly`

> [!warning] ช่องว่างการทดสอบจริง การยืนยันจริง (token `calendar.readonly` จริงอ่านเหตุการณ์จริง) ต้องรอ OAuth token ที่ผู้ดูแลจัดหา (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gws.sh`, เส้นทาง dependency-หายไป + token-หายไป + `calendar_id`-ผิด ที่ error อย่างสะอาด และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "gws"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "Google Calendar" / "Google Workspace" ปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อ) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../gws-auth/SPEC.th|gws-auth SPEC.th]] — foundation ข้อมูลรับรอง OAuth ที่ปลั๊กอินนี้ source
- [[../gws-gmail/SPEC.th|gws-gmail SPEC.th]] — ปลั๊กอิน Gmail พี่น้อง (รูปทรงตระกูลเดียวกัน)
- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]] — framing EPIC-13 ฉบับเต็ม (decisions 1–5)
- [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|PLUGINS.th.md §Workspace Resource Schema]] — รูปร่างเหตุการณ์ปฏิทินที่เป็นบรรทัดฐานที่ปลั๊กอินนี้ปล่อย
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
