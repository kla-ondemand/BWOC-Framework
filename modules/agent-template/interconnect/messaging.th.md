---
title: การสื่อสารระหว่าง Agent
aliases:
  - Messaging
  - สาราณียธรรม 6
tags:
  - group/agents
  - type/design
  - meta/template
status: ฉบับร่าง (v2026.5.23 — sender identity + สาราณียธรรม 6)
canonical-source: อังคุตตรนิกาย 6.11–12 (สาราณียสูตร)
---

# การสื่อสารระหว่าง Agent

> [!abstract] การสื่อสาร agent → agent ต่อยอดจากช่อง inbox ที่ใช้ user → agent ([`send.rs`](../../../crates/bwoc-cli/src/send.rs)) ด้วยการระบุตัวตน sender ผู้รับสามารถปฏิเสธโดยอิงโปรไฟล์ความน่าเชื่อถือ Kalyāṇamitta 7 ของผู้ส่ง ([`trust.th.md`](trust.th.md)) ส่วนกติกาการพูดคุยอย่างนอบน้อมมาจาก **สาราณียธรรม 6** (อังคุตตรนิกาย 6.11–12) — 6 เงื่อนไขแห่งความสามัคคีที่แปลงเป็นกฎทางวิศวกรรม

## เหตุผล

`bwoc send` ส่ง envelope โดย hardcode `"from": "user"` ค่าเดียว — ถูกสำหรับช่อง human → agent แต่การประสานระหว่าง agent ต้องให้ผู้ส่งแสดงตัวตน เพื่อให้ผู้รับ:

1. ใช้กลไกตรวจ Kalyāṇamitta 7 (ทำเสร็จแล้วใน `bwoc-agent --serve` — ดู [`trust.th.md`](trust.th.md))
2. แสดงประวัติ inbox ที่มีความหมาย (`bwoc inbox` เห็น sender จริง ไม่ใช่ "user" รวมๆ)
3. ตรวจสอบนโยบายการปฏิเสธได้ — refusals อ้างชื่อ peer จริง ไม่ใช่ placeholder

นอกจากนี้ การมี sender identity ทำให้ framework บังคับ **สาราณียธรรม 6** ได้ — 6 เงื่อนไขแห่งความสามัคคีจากมหาปรินิพพานสูตรและ อังคุตตรนิกาย 6.11 ซึ่งเป็นแนวคำสอนหลักของพุทธะเกี่ยวกับการอยู่ร่วมกันในชุมชน เมื่อแปลงสู่การสื่อสารระหว่าง agent:

| บาลี | ความหมาย | ในระบบ |
|---|---|---|
| Mettā-kāya-kamma | เมตตากายกรรม | API stability — ไม่ทำ schema envelope พังกลางทาง |
| Mettā-vacī-kamma | เมตตาวจีกรรม | เนื้อหา `message` สุภาพ ตรงไปตรงมา ไม่ตะโกน ไม่หยาบคาย ไม่ดูถูก |
| Mettā-mano-kamma | เมตตามโนกรรม | ตีความ envelope ของ peer ในทางดี — malformed ≠ malicious |
| Sādhāraṇa-bhogī | สาธารณโภคี | สถานะมองเห็นได้ — เขียนลง JSONL inbox ไม่มีช่องลับ |
| Sīla-sāmaññatā | สีลสามัญญตา | Sīla 5 baseline + manifest schema เดียวกันทั้งสองฝั่ง |
| Diṭṭhi-sāmaññatā | ทิฏฐิสามัญญตา | อ้างอิงกราฟ `PHILOSOPHY.th.md` เดียวกันเมื่อให้เหตุผล |

ข้อจำกัดการออกแบบ 3 ข้อสำหรับ v1:

1. **Sender identity แสดงตัว ไม่ใช่พิสูจน์** ขั้นที่ 4 ของ trust ได้ ship แบบ v1 ที่ผู้ส่งแสดงเองใน manifest signed envelope (HMAC over workspace-local secret ฯลฯ) เลื่อนไป v2
2. **Trust gating เป็น opt-in ฝั่งผู้รับ** ผู้รับที่ `requiredTrust = []` รับทุก envelope ที่ well-formed ทั้งจาก agent หรือ user — strict-by-default จะทำให้ flow ที่มีอยู่พังตอน rollout
3. **ไม่มี file shape ใหม่** envelope schema เปลี่ยนความหมายของ `from` แต่ JSONL บน disk ยังเหมือนเดิม

## Envelope Schema

Envelope บน disk = 1 บรรทัด JSONL ต่อหนึ่งข้อความใน `<recipient>/.bwoc/inbox.jsonl` — schema:

```json
{
  "ts":      "<ISO 8601 UTC>",
  "from":    "user" | "agent-<sender-name>",
  "to":      "agent-<recipient-name>",
  "message": "<UTF-8 text>"
}
```

ความหมายของ `from` ตามค่า:

| ค่า `from` | ความหมาย | Trust gate |
|---|---|---|
| `"user"` | มนุษย์ผู้ใช้งาน (`bwoc send` default) | ผ่านเสมอ (ผู้รับปฏิเสธ user ไม่ได้) |
| `"agent-<name>"` | agent อื่นในเดียวกัน workspace | อยู่ภายใต้ `requiredTrust` ของผู้รับเมื่อ gating เปิด |
| อื่นๆ | สงวนไว้สำหรับ identity source ในอนาคต (signed external sender ฯลฯ) | ถูกปฏิเสธด้วย `reason: "unknown_sender"` |

ฝั่ง runtime (daemon poll + refusal logic) จัดการทั้ง 3 กรณีแล้วตั้งแต่ trust step 4 spec นี้แค่ระบุ contract เป็นทางการ

## CLI Surface

```
bwoc send <to> <message>                    # from=user (default)
bwoc send <to> <message> --from <agent>     # from=agent-<name>
```

กฎ resolution ของ `--from`:
- รับชื่อ agent (หรือ `agentId` เต็ม) prefix `agent-` เติมให้ถ้าไม่มี — mirror กับ `--to`
- ผู้ส่งที่ระบุ **ต้อง** มีอยู่ใน `agents.toml` ของ workspace ที่ครอบคลุม ถ้าไม่พบ → exit 2 พร้อม error ชัดเจน
- `config.manifest.json` ของผู้ส่ง **ต้อง** อ่านได้ ถ้าอ่านไม่ได้ → exit 1

Daemon ฝั่งผู้รับ resolve manifest ของผู้ส่งใหม่ตอน envelope มาถึง — ฉะนั้นผู้ส่งที่เปลี่ยน declaration ระหว่าง `send` กับ `inbox poll` จะถูกประเมินด้วยสถานะ *ปัจจุบัน* (trust เป็น property ของ claim ปัจจุบัน ไม่ใช่ของเวลาที่ส่ง)

## สาราณียธรรม 6 — กฎเชิงวิศวกรรม

6 เงื่อนไขไม่ได้ถูก framework บังคับวันนี้ มันเป็น **norms** ที่ `AGENTS.md` §3 (Communication / Sammā-vācā) ของ template ควรสะท้อน เจตนาคือให้ agent ซึมซับเป็นแนวทาง ไม่ใช่ให้ `bwoc check` gate

### 1. เมตตากายกรรม — API stability

> การกระทำทางกายด้วยเมตตา: ไม่เปลี่ยนพื้นใต้เท้า peer

- JSONL envelope schema เป็น **append-only** — เพิ่มฟิลด์ optional ได้ แต่ฟิลด์เดิมความหมายเดิมเสมอ ฟิลด์ required ไม่ถอด
- Path ที่ spec นี้เปิดเผย (`.bwoc/inbox.jsonl`, `.bwoc/inbox.refusals.jsonl`) เป็น contract — ย้ายมัน = breaking change
- Protocol changes ของ Unix socket daemon (`PING`/`STATUS`/`STOP`) ใช้ discipline เดียวกัน

### 2. เมตตาวจีกรรม — พูดดี

> วจีกรรมด้วยเมตตา: เนื้อหา `message` ต้องอ่านเหมือนคำชี้แนะแบบเพื่อนร่วมงาน ไม่ใช่การตะโกน

- ใช้ประโยคบอกเล่ามากกว่า imperative ("กรุณาทำ X" ดีกว่า "ทำ X เดี๋ยวนี้")
- ห้าม ALL CAPS, คำหยาบ, คำดูถูก — framework ไม่บังคับ ผู้ตรวจ + operator บังคับ
- "ฉันทำไม่ได้" ที่ซื่อสัตย์ ดีกว่า "OK" ที่หลอก — ดู Vattā ใน [trust spec](trust.th.md)

### 3. เมตตามโนกรรม — คิดดี

> มโนกรรมด้วยเมตตา: ตีความ envelope ของ peer ในทางดี

- JSON malformed ≠ malicious — daemon poll จัดการ parse failure ด้วย warning + continue ไม่ใช่ระแวงว่าโจมตี
- ฟิลด์ optional ที่ขาด ≠ peer ไม่ปฏิบัติตาม — ใช้ default ตาม spec
- Sender ที่ไม่รู้จัก (`from: agent-x` ไม่อยู่ใน registry) ได้ structured refusal `reason: "unknown_sender"` ไม่ใช่ silent drop

### 4. สาธารณโภคี — แบ่งปันสิ่งที่ได้

> แบ่งปันทรัพยากรอย่างเป็นธรรม: state ต้องมองเห็นได้

- traffic ของ inbox ทั้งหมดอยู่ใน `inbox.jsonl` (version-able, grep-able, replay-able)
- refusals อยู่ใน `inbox.refusals.jsonl` (audit ได้ — ไม่เคยถูกลบ; merge ตอน read)
- ไม่มี agent ซ่อนข้อความใน private channel ที่ workspace มองไม่เห็น

### 5. สีลสามัญญตา — ศีลเสมอกัน

> precept ชุดเดียวกันทั้งสองฝั่ง

- ทั้ง sender และ recipient แสดงการปฏิบัติตาม Sīla 5 ([`AGENTS.md` §9](../AGENTS.md))
- ทั้งคู่ผ่าน `bwoc check` ก่อนเข้าร่วม flow ระหว่าง agent
- ทั้งคู่ใช้ manifest schema version ที่ compatible (mismatch `schemaVersion` เป็น refusal reason ใน trust v2 — ผ่อนปรนใน v1)

### 6. ทิฏฐิสามัญญตา — เห็นตรงกัน

> เป้าหมายตรงกัน: อ้างอิงปรัชญาเดียวกัน

- ข้อความ `message` ที่อ้าง Buddhist framework **ควร** link ไปที่รายการ canonical ใน `PHILOSOPHY.th.md` — เป็น convention ไม่ใช่ข้อบังคับใน wire format
- spec ระหว่าง agent (ไฟล์นี้, `trust.th.md`, `capabilities.md`) อยู่ใต้ `interconnect/` ฉะนั้น template ทุกตัว ship ที่ path เดียวกัน

## Backward Compatibility

- `bwoc send <to> <message>` แบบเดิมยังเขียน `from: "user"` เหมือนเดิม — ไม่มี behavior change
- `--from` default = `user` เมื่อไม่ใส่ — scripts ที่ไม่ผ่าน flag ใหม่ไม่ได้รับผลกระทบ
- Envelope เก่า (ก่อน spec) ที่มี `from: "user"` deserialize เหมือนเดิม — codepath user-passthrough ของ daemon ฝั่งผู้รับยังเป็นเส้นทางเดิม

## ลำดับการ Implement

1. ✓ `bwoc-agent --serve` daemon-side refusal สำหรับ sender ที่ไม่ใช่ `user` — **ship แล้วใน trust step 4** ฝั่ง runtime เสร็จแล้ว; spec นี้แค่บันทึก contract
2. `bwoc send --from <agent>` — flag sender identity ใน `bwoc-cli` (iter นี้)
3. Tests + live verification ของ flow agent → agent กับ trust gate (iter นี้)
4. CHANGELOG + ROADMAP cross-reference (iter นี้)
5. **เลื่อน (v2):** signed envelopes, sender identity proof, cross-workspace messaging, broadcast (`bwoc send --all`)

## สิ่งที่ Spec นี้ ไม่ ครอบคลุม

- **Signed envelopes / identity proof** Workspace-local secret HMAC ครอบ envelope JSON เป็นแนวทาง v2 ที่ชัดเจน threat model วันนี้ยอมรับว่า clone ที่ประสงค์ร้ายเขียน `from: agent-bob` ได้ทั้งๆ ที่เป็น agent ตัวอื่น — การตรวจ trust วันนี้ทำกับ *manifest* ของผู้ส่งซึ่งเป็นไฟล์ต่อ agent บน disk
- **Cross-workspace messaging** Trust เป็น per-workspace ([`trust.th.md`](trust.th.md)) — envelope ที่ส่งหา agent ใน workspace อื่นเป็น undefined behavior ใน v1
- **Broadcast / fan-out** `bwoc send --all <message>` เป็น operator surface ที่มีประโยชน์ แต่ไม่ใช่เรื่อง sender identity — ขึ้น queue แยก
- **Routing ผ่านตัวกลาง** การส่งทั้งหมดเป็น point-to-point — agent ที่อยาก relay ต้องอ่านจาก inbox ตัวเองแล้วส่งต่อโดยชัดแจ้ง

## ประวัติการแก้ Spec

- **v1 / 2026-05-23 (ฉบับร่างแรก):** Envelope schema + `--from <agent>` CLI surface + แผนที่ สาราณียธรรม 6 → กฎเชิงวิศวกรรม Trust gate integration ใช้งานได้แล้วจาก trust step 4 ที่ ship ก่อนหน้านี้ในวันเดียวกัน

## เอกสารอ้างอิง

- [`trust.th.md`](trust.th.md) — Kalyāṇamitta 7 trust model — refusal gate ทำงานบนฟิลด์ `from` ที่ spec นี้กำหนด
- [`capabilities.md`](capabilities.md) — capability declaration — peer ที่มี skill ที่ใช่ + trust ที่ต้องการ + posture สาราณียธรรมที่สะอาด = ภาพรวมที่สมบูรณ์
- [`AGENTS.md` §3 (Communication)](../AGENTS.md) — หลัก Sammā-vācā กับการพูดต่อ user — กฎเดียวกันใช้ระหว่าง peer
- [`PHILOSOPHY.th.md` #13. สาราณียธรรม 6](../docs/th/PHILOSOPHY.th.md) — อ้างอิง canonical สำหรับ 6 เงื่อนไข
- อังคุตตรนิกาย 6.11–12 — แหล่งดั้งเดิม ([SuttaCentral AN 6.11](https://suttacentral.net/an6.11), [AN 6.12](https://suttacentral.net/an6.12))