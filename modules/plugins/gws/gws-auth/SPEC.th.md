---
title: gws-auth — รากฐานข้อมูลรับรอง OAuth ของ Google Workspace
aliases:
  - gws-auth
tags:
  - group/framework-plugins
  - type/plugin
  - kind/gws
  - domain/integration
  - integration/google-workspace
maturity: L1
---

# gws-auth — รากฐานข้อมูลรับรอง OAuth ของ Google Workspace

> [!abstract] **รากฐานข้อมูลรับรอง (credential foundation)** ของ plugin kind `gws` (`BWOC-EPIC-13`) รับผิดชอบ surface ของ OAuth2 — การมี token, scope ที่ได้รับ, บัญชี, และวันหมดอายุ — และ export ฟังก์ชันช่วย Bearer-auth + rate-limit + refresh ให้ปลั๊กอินรายบริการ source ไปใช้ คำสั่ง: `status` (อ่านอย่างเดียว; **ไม่เคยพิมพ์ค่า token**) เป็นคู่ขนานฝั่ง Workspace ของ [[../../workflow/gcloud-auth/SPEC.th|`gcloud-auth`]] — แต่เป็นตระกูล auth คนละแบบโดยสิ้นเชิง: OAuth2 **scope ที่ผู้ใช้ยินยอม** ผ่าน Workspace REST API ไม่ใช่ ADC / service-account ผ่าน `gcloud` CLI ในเครื่อง จับคู่กับ [[../gws-drive/SPEC.th|`gws-drive`]] (และ `gws-gmail` / `gws-calendar` ในอนาคต) ที่ source ฟังก์ชันช่วยที่ประกาศไว้ที่นี่ เหตุผลฉบับเต็ม: [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]]

## ทำไมต้องปลั๊กอินรากฐาน ทำไม kind `gws`

`gws-auth` ส่งมาเป็นปลั๊กอิน **แยกต่างหาก** เพื่อให้ปลั๊กอินรายบริการทุกตัว (`gws-drive` และ `gws-gmail` / `gws-calendar` ในอนาคต) นำ surface OAuth หนึ่งเดียวไปใช้ใหม่ — การ resolve token, header `Authorization: Bearer`, การจัดการ 429, refresh-if-expired — โดยไม่ต้องเขียนซ้ำ (บันทึก §Decision 2, รูปแบบตระกูล `gcloud-*`) ส่วนการใช้ kind `gws` (แทนที่จะนำ `workflow` มาใช้ใหม่อย่าง `gcloud`) ถูกต้องเพราะ BWOC ถือ schema ที่เป็นบรรทัดฐานเหนือการเชื่อมต่อนี้: [[../../../docs/th/PLUGINS.th#Workspace Resource Schema|Workspace resource schema]] รายบริการ (ไฟล์ Drive, เธรด Gmail, อีเวนต์ Calendar) + โมเดล scope ของ OAuth กฎคือ **เป็น kind ของตัวเองเมื่อ BWOC นิยาม schema ที่เป็นบรรทัดฐานเหนือการเชื่อมต่อ; ใช้ `workflow` ซ้ำเมื่อเป็น passthrough ที่ไม่มีรูปร่างที่ BWOC เป็นเจ้าของ** (บันทึก §Decision 1)

มัน**ไม่ใช่** ส่วนหนึ่งของ `gcloud`: gcloud เข้าถึง *โครงสร้างพื้นฐาน* GCP ผ่าน `gcloud` CLI ในเครื่องด้วย ADC / service-account; `gws` เข้าถึง *แอป* productivity ผ่าน Workspace REST API ด้วย scope ที่ผู้ใช้ยินยอมแบบ OAuth2 คนละตระกูล auth คนละ surface คนละวงจรชีวิต

## คำสั่ง (Verbs)

| คำสั่ง | ทิศทาง | Auth | ผลข้างเคียง |
|---|---|---|---|
| `status` | อ่าน | ไม่ต้องมี | ไม่มี — รายงานการมี token, scope ที่ได้รับ, บัญชี, วันหมดอายุ, และว่า refresh ได้หรือไม่ **ไม่เคย** พิมพ์ค่า token |

`status` คือ verb อ่านหลักสำหรับเอเจนต์ การได้มาซึ่ง token (OAuth consent flow) เป็นการกระทำของ **ผู้ดูแล** นอกกระบวนการ — ปลั๊กอินบริโภค token ไม่ใช่ผู้สร้าง

## การทำงาน

CLI `bwoc gws` (`BWOC-74`) เรียก `gws.sh` จากไดเรกทอรีนี้ (สะท้อนวิธีที่ `bwoc gcloud` เรียก `gcloud-auth` และ `bwoc figma` เรียก `figma-rest`):

| ช่องทาง | สิ่งที่ส่งมา |
|---|---|
| `BWOC_GWS_OPERATION` (env) | `status` — fallback สำหรับ `.operation` เมื่อ stdin ว่าง |
| `BWOC_WORKSPACE` (env) | path สัมบูรณ์ของ workspace; ใช้ resolve path ของไฟล์ token |
| `BWOC_PLUGIN_DIR` (env) | path สัมบูรณ์ของไดเรกทอรีปลั๊กอินนี้ (เพื่อข้อมูล) |
| `BWOC_GWS_TOKEN` (env) | OAuth2 access token — **ความลับ**, precedence สูงสุด |
| stdin | คำขอ JSON บรรทัดเดียว เช่น `{"operation":"status"}` |

เมื่อสำเร็จ: ออกด้วยรหัส `0`, ปล่อย JSON หนึ่งอ็อบเจกต์ทาง stdout เมื่อผิดพลาด: ข้อความวินิจฉัยทาง stderr + ออกด้วยรหัสไม่ใช่ศูนย์ (CLI รายงานเป็น `plugin '<name>' exited <code>`)

## การยืนยันตัวตน

ปลั๊กอินนี้ **ไม่เคย serialize token ลงในผลลัพธ์ใด ๆ** มันอ่าน token เพียงเพื่อตั้ง header `Authorization: Bearer` บนคำขอ REST ขาออก (สำหรับปลั๊กอินรายบริการที่ source มัน) ลำดับความสำคัญ (แหล่งแรกที่ resolve ได้ชนะ; สะท้อนใน `status.active_source`):

1. **`BWOC_GWS_TOKEN`** env — transient / CI; ไม่มีเมทาดาทา (scope, expiry, account ไม่ทราบสำหรับ env token)
2. **ไฟล์ token** — `${BWOC_WORKSPACE}/.bwoc/secrets/gws-token.json` ถูก gitignore ที่ระดับ workspace (secret store ของ BWOC-53), `chmod 600`, ไม่เคย commit เป็น JSON object ที่มีอย่างน้อย `access_token` และทางเลือก `refresh_token` / `expiry` / `scopes` / `account` / `client_id` / `client_secret`

[[auth|auth.toml]] ประกาศ **รูปร่าง** — ชื่อ env var, path ไฟล์ token + ฟิลด์ที่รู้จัก, และ scope readonly รายบริการ — โดย **ไม่มีค่า** `bwoc check` (`BWOC-77`) จะปฏิเสธหากปรากฏฟิลด์ที่ดูเหมือนเป็นค่า

> [!danger] **ศีล — อทินนาทาน** `auth.toml` ที่ malformed ไม่สามารถทำให้ token รั่วได้ เพราะไม่มี token ใดอยู่ในไฟล์ที่ถูก track เลย token เข้ามาในปลั๊กอินผ่าน environment หรือไฟล์ owner-only ที่ถูก gitignore เท่านั้น และออกไปเป็น header คำขอ curl เท่านั้น มันไม่เคยถูก echo, ไม่เคยถูก log และไม่เคยปรากฏใน JSON envelope ใด ๆ verb `status` ตั้งใจเปิดเผยเมทาดาทา (scope, account, expiry, source) — **ไม่เคย** เปิดเผยค่า token

### Scopes

scope ของ OAuth เป็น **รายบริการและผูกกับการยินยอม**: token ที่ได้รับเพียง `drive.readonly` ไม่สามารถอ่าน Gmail หรือ Calendar ได้ `auth.toml [gws.auth.scopes]` ประกาศ scope readonly ที่แต่ละบริการต้องใช้:

| บริการ | scope ที่ต้องมี |
|---|---|
| Drive | `https://www.googleapis.com/auth/drive.readonly` |
| Gmail | `https://www.googleapis.com/auth/gmail.readonly` |
| Calendar | `https://www.googleapis.com/auth/calendar.readonly` |

`status.scopes` รายงาน scope ที่ token *ปัจจุบัน* ถืออยู่ (จากไฟล์ token; ว่างสำหรับ env token) verb ของบริการที่ไม่มี scope จะแสดง `token lacks <scope> for <service>` บน HTTP 403 ที่เกิดขึ้น ไม่ใช่ความล้มเหลวเปล่า ๆ (ดูการจัดการ error ของ [[../gws-drive/SPEC.th|gws-drive]])

## Refresh-if-expired

เมื่อไฟล์ token มี `expiry` ในอดีต **และ** มีชุดสามสำหรับ refresh (`refresh_token` + `client_id` + `client_secret`) `gws-auth` จะทำ refresh_token grant แบบ offline กับ OAuth2 endpoint ของ Google แล้วเขียนไฟล์ token ทับในที่เดิม (`access_token` ใหม่ + `expiry` ที่คำนวณใหม่; ฟิลด์อื่นคงไว้ทั้งหมด; เขียนผ่านไฟล์ชั่วคราว `chmod 600` + `mv` แบบ atomic เพื่อไม่ให้ความลับ world-readable แม้ชั่วครู่) สิ่งนี้เกิดขึ้นอย่างโปร่งใสภายใน `gws_curl` ก่อนทุกคำขอ ดังนั้น sibling จะไม่เห็น token ที่หมดอายุ เมื่อต้อง refresh แต่ทำไม่ได้ (env token หรือไม่มีชุดสาม) คำขอจะดำเนินต่อและ HTTP 401 ที่เกิดขึ้นจะถูกแสดงด้วยข้อความ "re-authorize" ที่ชัดเจน — ไม่เคย panic

## รูปร่างผลลัพธ์

### `status`

```json
{
  "ok": true,
  "plugin": "gws-auth",
  "operation": "status",
  "active_source": "secrets-file",
  "has_token": true,
  "account": "me@example.com",
  "scopes": ["https://www.googleapis.com/auth/drive.readonly"],
  "expiry": "2026-05-28T18:00:00Z",
  "expired": false,
  "refreshable": true,
  "sources": {
    "env":          { "present": false, "var": "BWOC_GWS_TOKEN" },
    "secrets_file": { "present": true, "path": "/abs/workspace/.bwoc/secrets/gws-token.json" }
  }
}
```

`active_source` เป็นหนึ่งใน `env | secrets-file | none` `account` / `expiry` เป็น `null` เมื่อไม่ทราบ; `scopes` เป็น `[]` เมื่อไม่ทราบ (เช่น env token) `expired` เป็น `true` เฉพาะเมื่อ `expiry` ที่ทราบอยู่ในอดีต; `refreshable` เป็น `true` เฉพาะเมื่อมีชุดสามสำหรับ refresh envelope ปล่อยออกมาอย่างสะอาดเสมอ (ไม่ panic) แม้ไม่มี token เลย

## ฟังก์ชันช่วย (Shared helpers)

`gws.sh` export surface ข้อมูลรับรอง OAuth ให้ปลั๊กอินพี่น้อง `gws/gws-*` source ใช้ (บันทึก §Decision 2):

| ฟังก์ชัน | คืนค่า / ผลข้างเคียง |
|---|---|
| `gws_resolve_token` | echo access token (env ก่อน แล้วจึงไฟล์ token) ว่างเมื่อไม่มี **ไม่ใช่ฟิลด์ผลลัพธ์** — จับเข้าตัวแปรเท่านั้น |
| `gws_auth_header` | echo บรรทัด header `Authorization: Bearer <token>` เต็มสำหรับ curl ว่างเมื่อไม่มี token |
| `gws_assert_token` | คืน `2` (พร้อม stderr ชัดเจน) เมื่อ resolve token ไม่ได้ |
| `gws_token_scopes` / `gws_token_account` / `gws_token_expiry` | ตัวอ่านเมทาดาทาจากไฟล์ token (ว่างสำหรับ env token) |
| `gws_refresh_if_expired` | refresh token ที่หมดอายุ+refresh ได้ ในที่เดิม; no-op ในกรณีอื่น |
| `gws_curl` | คำขอที่ยืนยันตัวตน: refresh → header Bearer + JSON Accept → backoff 429 `Retry-After` (สูงสุด 4 ครั้ง) ตั้ง `HTTP_STATUS` / `HTTP_BODY` |
| `gws_classify_status` | แม็พ `HTTP_STATUS` เป็นข้อความวินิจฉัยชัดเจน + exit code (401 auth, 403 scope, 404, 429, transport) |

ตัว dispatcher (`_gws_auth_main`) ถูก guard ด้วย `BASH_SOURCE` ดังนั้นการ source ไฟล์นี้เป็นการ import ที่บริสุทธิ์ — ไม่กิน stdin, ไม่รัน verb [[../gws-drive/gws|`gws-drive/gws.sh`]] source ไฟล์นี้ผ่าน `$BWOC_PLUGIN_DIR/../gws-auth/gws.sh` (พร้อม fallback ที่อิง path ของสคริปต์ เพื่อให้ปลั๊กอินยังทดสอบได้นอก dispatcher ของเฟรมเวิร์ก)

## คลาสข้อผิดพลาด

| Exit | คลาส | ความหมาย |
|---|---|---|
| `0` | สำเร็จ | JSON หนึ่งอ็อบเจกต์บน stdout |
| `1` | dependency | ไม่มี `jq` บน PATH |
| `2` | usage / no-token | operation ไม่รู้จัก / ไม่ได้ระบุ หรือ `gws_assert_token` ไม่พบ token |
| `3` | auth / scope | `gws_classify_status` พบ HTTP 401 (token ไม่ถูกต้อง) หรือ 403 (scope ขาด) |
| `4` | rate-limited | HTTP 429 หลังหมดงบ backoff |
| `5` | not-found | HTTP 404 |
| `6` | transport / unexpected | network ล้มเหลว หรือ HTTP status ที่ไม่ได้แม็พ |

การไม่มี `jq` ล้มเหลวอย่าง **สะอาด** ด้วยข้อความ stderr ชัดเจน; ปลั๊กอินไม่เคย panic หรือทิ้งกระบวนการในสถานะกึ่ง

## การตั้งค่า

```toml
# workspace.toml
[plugins.gws-auth]
enabled = true
```

ไม่มี `[config.schema]` — การ resolve credential ขับด้วย environment ไม่ใช่ config (บันทึก §Decision 3) surface ระดับ workspace เพียงตัวเดียวคือคีย์สากล `enabled`

## การจับคู่วงจรชีวิต

ตาม [[../../../docs/th/PLUGINS.th#วงจรชีวิต|PLUGINS.th.md §วงจรชีวิต]] เจ้าของ kind `gws` คือ **เอเจนต์** ที่เรียกออกไปผ่าน CLI `bwoc gws` `init`/`teardown` เกิดต่อการเรียกแต่ละครั้งรอบ ๆ `invoke` ปลั๊กอินไม่ถือสถานะภายในใด ๆ นอกจากไฟล์ token ที่ผู้ดูแลจัดหา

| เฟส | สิ่งที่ปลั๊กอินนี้ทำ |
|---|---|
| `init` | โดยปริยายต่อการเรียก; ตรวจว่ามี `jq` บน PATH |
| `invoke` | อ่านคำขอ, ตรวจสถานะ token, ปล่อย JSON (`status`); หรือเมื่อถูก source ทำคำขอที่ยืนยันตัวตนให้ sibling |
| `teardown` | โดยปริยาย; ไม่มีสถานะให้คืน |

## Idempotency

- `status` อ่านอย่างเดียว ลำดับคงที่ทั่วทุกการเล่นซ้ำ
- `gws_refresh_if_expired` ลู่เข้า: token ที่ไม่หมดอายุเป็น no-op; token ที่หมดอายุ+refresh ได้ถูก refresh หนึ่งครั้งต่อการเรียกและไฟล์ถูกเขียนทับแบบ atomic

## ระดับวุฒิภาวะ (Maturity)

ประกาศ **L1** — ปลั๊กอินอ้างอิง `gws/gws-auth` ตัวแรกที่รันได้; verb `status` + surface ฟังก์ชันช่วยทำงาน จะขยับเป็น **L2** เมื่อส่วนขยาย `bwoc check` (`BWOC-77`) และ smoke test ทดสอบ end-to-end กับ OAuth token ของผู้ดูแล

> [!warning] ช่องว่างการทดสอบจริง การยืนยัน end-to-end (token ที่ยินยอมจริงอ่านข้อมูล Workspace จริง) ต้องรอ OAuth token ที่ผู้ดูแลจัดหามาที่ `.bwoc/secrets/gws-token.json` หรือ `BWOC_GWS_TOKEN` (บันทึก §Status) v0.1.0 ยืนยันด้วย: `bash -n gws.sh`, envelope `status` แบบไม่มี token ที่ปล่อยออกมาอย่างสะอาด (`has_token:false`), เส้นทาง jq-หายไปที่ error อย่างสะอาด และ `bwoc check` ที่ยอมรับ manifest

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend หรือ model ใด `kind = "gws"` เป็นค่า enum ของเฟรมเวิร์กเอง คำว่า "Google Workspace" / ชื่อบริการปรากฏเฉพาะใน `description` (ที่อนุญาตชื่อเป้าหมายการเชื่อมต่อตาม [[../../../docs/th/PLUGINS.th#ข้อจำกัดความเป็นกลาง (HARD)|PLUGINS.th.md §ความเป็นกลาง]]) และในเนื้อความ SPEC นี้ — ไม่เคยอยู่ใน `kind`, `entry` หรือคีย์ config สอดคล้องกับ **สมานัตตตา**

## ดูเพิ่มเติม

- [[../../../notes/2026-05-28_google-workspace-plugin-architecture|บันทึกออกแบบ BWOC-72]] — framing ฉบับเต็มสำหรับ foundation ของ EPIC-13 (decisions 1–5)
- [[../gws-drive/SPEC.th|gws-drive SPEC.th]] — ปลั๊กอินบริการพี่น้อง (ไฟล์ Drive); source ฟังก์ชันช่วยที่ประกาศที่นี่
- [[../../workflow/gcloud-auth/SPEC.th|gcloud-auth SPEC.th]] — คู่ขนานฝั่ง *โครงสร้างพื้นฐาน* ของ Google; **ไม่ใช่** ตระกูล auth เดียวกัน
- [[../../../docs/th/PLUGINS.th|PLUGINS.th.md]] — สเปกปลั๊กอิน; แถว kind `gws` + Workspace Resource Schema
- [[auth|auth.toml]] — สัญญา auth (รูปร่างเท่านั้น ไม่มีค่า)
- [[SPEC|SPEC.md]] — คู่ฉบับภาษาอังกฤษ (ความเท่าเทียมสองภาษา)
