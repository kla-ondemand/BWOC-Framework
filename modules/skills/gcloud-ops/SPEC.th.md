---
title: gcloud Ops
aliases:
  - gcloud-ops
tags:
  - group/framework-skills
  - type/skill
  - domain/gcp
  - domain/integration
maturity: L1
---

# gcloud Ops

> [!abstract] Skill แบบ **skill-on-multiple-plugins** ตัวแรกของเฟรมเวิร์ก ให้ agent มี GCP self-orientation operation แบบอ่านเป็นหลัก 3 ตัว — `whoami`, `current-project`, `switch-project` — แต่ละตัวเป็นการเรียก verb ของ `bwoc gcloud` ที่ dispatch plugin kind `workflow` สองตัว คือ [[../../modules/plugins/workflow/gcloud-auth/SPEC|`gcloud-auth`]] และ [[../../modules/plugins/workflow/gcloud-project/SPEC|`gcloud-project`]] แบบบาง ๆ Skill เป็นเจ้าของ *ความหมายของการระบุตัวตน/บริบทของ agent*; plugin เป็นเจ้าของ *การ integrate* (การ resolve credential, การ shell-out ไป `gcloud`, การ introspect โปรเจกต์) Skill พึ่งพา plugin kind `workflow` ผ่านฟิลด์ contract [`requires_plugins`](#dependency-model--skill-on-multiple-plugins)

## Skill นี้ทำอะไร

ห่อ CLI `bwoc gcloud` ที่ operator ใช้ (plugin kind `workflow` สองตัวของ gcloud, [[../../docs/th/PLUGINS.th#ประเภทของปลั๊กอิน|PLUGINS.th.md §ประเภทของปลั๊กอิน]]) ด้วยคำศัพท์ self-orientation ของ agent เพื่อให้ agent ตอบได้ว่า "ฉันคือใคร, อยู่ที่ไหน, และสลับไปที่ไหน" โดยไม่ต้อง derive การ resolve credential หรือการ introspect โปรเจกต์ใหม่ มี operation ที่เปิด 3 ตัว แต่ละตัวบอกว่า *agent ต้องรู้หรือเปลี่ยนอะไร* และ delegate *วิธีไปถึง GCP* ให้ verb ของ `bwoc gcloud` ภายใต้ฝากระโปรง

- **`whoami`** — รายงาน credential ที่ active (source + อีเมลบัญชี) และโปรเจกต์ default ปัจจุบันในมุมมองเดียว **อ่านอย่างเดียว** ประกอบ `bwoc gcloud auth` (→ `gcloud-auth status`) + `bwoc gcloud project show` (→ `gcloud-project show` บน default) ไม่เคยเปิดเผยค่า credential
- **`current-project`** — รายงานเฉพาะ descriptor ของโปรเจกต์ default ปัจจุบัน **อ่านอย่างเดียว** เส้นทางที่เร็วกว่า `whoami` เมื่อสนใจแค่โปรเจกต์ ใช้ `bwoc gcloud project show` ภายใต้ฝากระโปรง
- **`switch-project`** — เปลี่ยนโปรเจกต์ default ในเครื่อง ประกอบ `bwoc gcloud project set-default` (→ `gcloud-project set-default`) **เขียนแบบ gated** — relay prompt ขอ operator confirm ที่ CLI บังคับ; skill ไม่เพิ่ม gate ที่สองและไม่ถอดออก

สิ่งที่ skill นี้ **จงใจไม่เปิด** (บันทึกออกแบบ [[../../notes/2026-05-28_gcloud-workflow-plugin-architecture|BWOC-51]] §Decision 5):

- **`login`** — `gcloud-auth login` เปิดเบราว์เซอร์และ agent ขับเองอย่างปลอดภัยไม่ได้ agent ที่ต้อง authenticate ให้ surface "กรุณารัน `gcloud auth login`" ให้ operator เท่านั้น
- **อะไรก็ตามนอกเหนือ plugin foundation สองตัว** — ไม่มี compute, storage, หรือ IAM สิ่งเหล่านั้นเป็น skill/epic ในอนาคตที่สร้างบน foundation นี้

## ทำไมจึงมี Skill นี้

การ bundle self-orientation เข้าไปใน plugin gcloud จะบังคับให้ทุก agent ที่อยากรู้บริบท GCP ของตัวเองต้องแบกเรื่อง resolve credential และ shell-out ไป `gcloud` ไปด้วย — ผิดชั้นสำหรับ capability ของ agent ([[../../notes/2026-05-28_gcloud-workflow-plugin-architecture|บันทึกออกแบบ BWOC-51]] §Decision 5) การแยกออกทำให้ผู้บริโภค *หลายราย* — skill นี้, skill GCP ในอนาคต, หรือการใช้ `bwoc gcloud` ตรง ๆ ของ operator — นั่งบน plugin **สองตัวเดียวกัน** ได้ และทำให้ plugin ถูก install, ตั้งค่า, audit, และทดสอบแยกจาก agent ใด ๆ Substrate เดียวกัน, ผู้เรียกต่างกัน — แกน [[../../docs/th/PLUGINS.th#Skill กับ Plugin|Skill กับ Plugin]] นำมาใช้ตรง ๆ

## ขอบเขต Skill ↔ Plugin

นี่คือเส้นแบ่งที่สำคัญที่สุด ต้องคมชัด

| | `gcloud-ops` (skill นี้) | plugin `workflow` ของ gcloud |
|---|---|---|
| ชั้น | capability ของ agent | การ integrate ของเฟรมเวิร์ก |
| opt-in ผ่าน | `<agent>/config.manifest.json` (`skills.framework[]`) | `workspace.toml [plugins.<name>]` |
| ผู้เรียก | agent ระหว่างการทำงานของตัวเอง | runtime ของเฟรมเวิร์ก / CLI `bwoc gcloud` |
| รู้ | ความหมายของ self-orientation — ฉันคือใคร, อยู่ที่ไหน, สลับบริบท | ลำดับ ADC vs service-account, การเรียก `gcloud` CLI, รูปร่าง JSON ของโปรเจกต์, gate ขอ operator confirm |
| **ไม่** รู้ | credential resolve ยังไง, `gcloud` ถูกเรียกยังไง, รูปร่าง descriptor ของโปรเจกต์ | อะไรก็ตามเกี่ยวกับ self-orientation ของ agent |

กฎสองข้อที่ตามมาจากตารางและ **ต่อรองไม่ได้**:

1. **การพึ่งพาเป็นทางเดียว: skill → plugins** Skill เรียก verb ของ plugin ผ่าน `bwoc gcloud`; plugin ไม่รู้จัก skill เลย plugin gcloud ใช้งานได้เต็มที่โดยไม่มี skill (operator รัน `bwoc gcloud` ตรง ๆ)
2. **Skill ไม่เคยอ่าน credential และไม่เคยเรียก `gcloud` ตรง ๆ** เข้าถึง GCP *ผ่าน plugin โดยอ้อมเท่านั้น* — plugin เป็นผู้เรียก `gcloud` รายเดียว, และ credential resolve อยู่ใน `gcloud-auth` จาก ADC / `.bwoc/secrets/gcloud-sa.json` / env `BWOC_GCLOUD_*` ไม่เคยผ่าน skill ([[../../notes/2026-05-28_gcloud-workflow-plugin-architecture|บันทึก BWOC-51]] §Decision 3)

## สัญญา Operation

แต่ละ operation ประกอบ verb ของ `bwoc gcloud` หนึ่งตัวขึ้นไป gate ของ verb นั้นถูกสืบทอด — verb เขียนตัวเดียว (`switch-project`) แบก gate ขอ operator confirm ของ CLI ([[../../modules/plugins/workflow/gcloud-project/SPEC|gcloud-project SPEC.md]] §Verbs); skill ไม่เพิ่ม gate ที่สอง

| Operation | สิ่งที่ agent ต้องการ | verb `bwoc gcloud` ภายใต้ฝากระโปรง | ทิศทาง | Gate |
|---|---|---|---|---|
| `whoami` | ฉัน authenticate เป็นใคร และชี้ไปที่ไหน? | `auth` + `project show` | อ่าน | ไม่มี — อ่านได้ฟรี |
| `current-project` | โปรเจกต์ default ปัจจุบันคืออะไร? | `project show` | อ่าน | ไม่มี — อ่านได้ฟรี |
| `switch-project` | ชี้ฉันไปที่โปรเจกต์ default อื่น | `project set-default` | เขียน (ในเครื่อง) | operator confirm (ใน CLI) |

verb อ่านรองรับ self-orientation ก่อนทำ GCP action ใด ๆ: agent เรียก `whoami` เพื่อยืนยันว่า authenticate แล้วและชี้ไปที่โปรเจกต์ที่ตั้งใจก่อนทำอย่างอื่น ทุก operation ถูกสังเกตด้วย `Kāyānupassanā` (สถานะ credential/โปรเจกต์ที่ plugin รายงาน) และ `Dhammānupassanā` (gate ตัวไหนมีผล) ความล้มเหลว surface operation, ต้นเหตุ, และทางแก้ — ไม่ใช่แค่ "ล้มเหลว" — รวมถึง "ไม่มี credential resolve; รัน `gcloud auth login`" ที่ actionable เมื่อ `gcloud-auth status` รายงาน `active_source = none`

## Dependency Model — skill-on-multiple-plugins

Skill นี้เป็น **skill ตัวแรกของเฟรมเวิร์กที่ประกอบ plugin มากกว่าหนึ่งตัว** plugin ทั้งสองที่มันขับ — `gcloud-auth` และ `gcloud-project` — เป็น kind `workflow` ทั้งคู่ การพึ่งพาจึงแสดงครั้งเดียวผ่านฟิลด์ kind-based ที่มีอยู่:

```toml
[contract]
requires         = []                  # ชื่อ SKILL ของเฟรมเวิร์ก (ฟิลด์เดิม)
requires_plugins = ["workflow"]        # KIND ของ plugin ที่ skill นี้ต้องการให้เปิด
```

- **`requires_plugins` ระบุ _kind_ ของ plugin ไม่ใช่ _ชื่อ_** ([[../../docs/th/SKILLS.th#Skill-on-plugin dependency|SKILLS.th.md §Skill-on-plugin dependency]]) `"workflow"` คือค่า enum ของ kind จาก [[../../docs/th/PLUGINS.th#ประเภทของปลั๊กอิน|PLUGINS.th.md]] Skill พึ่งพาการมี kind `workflow` อยู่; plugin **เฉพาะ** ที่มันประกอบ (`gcloud-auth` + `gcloud-project`) ถูกระบุใน [สัญญา Operation](#สัญญา-operation) ของ SPEC นี้ ไม่ใช่ใน manifest
- **ทำไม kind-level ไม่ใช่ name-level** resolver การพึ่งพาของเฟรมเวิร์กเป็น kind-based โดยออกแบบ — เพื่อให้ skill เป็นกลางและ adapter สลับได้ ([[../../docs/th/SKILLS.th#Skill-on-multiple-plugins|SKILLS.th.md §Skill-on-multiple-plugins]]) Skill ที่ประกอบ plugin หลายตัวของ kind เดียวระบุ kind นั้นครั้งเดียว; SPEC ระบุตัว instance การบังคับ **name-level** (ยืนยันว่าทั้ง `gcloud-auth` และ `gcloud-project` เปิดอยู่จริง ไม่ใช่แค่ plugin `workflow` *บางตัว*) เป็น future extension ที่บันทึกไว้ — ที่ L1 skill ล้มเหลวอย่างนุ่มนวลตอน invoke ถ้า plugin ที่ประกอบขาดไป โดย surface ว่า verb `bwoc gcloud` ตัวไหน dispatch ไม่ได้
- **resolve ตอน agent spawn** — ถ้า `gcloud-ops` เปิดแต่ไม่มี plugin kind `workflow` เปิดใน workspace, spawn ล้มเหลวทันทีพร้อม diagnostic ที่ระบุ kind ที่ขาด จับได้ก่อนด้วย `bwoc skill verify gcloud-ops`

dependency model เต็มอยู่ใน [[../../docs/th/SKILLS.th#Skill-on-plugin dependency|SKILLS.th.md §Skill-on-plugin dependency]]; การปรับแบบ multiple-plugins อยู่ใน [[../../docs/th/SKILLS.th#Skill-on-multiple-plugins|§Skill-on-multiple-plugins]]

## การ Map Lifecycle

ตาม [[../../docs/th/SKILLS.th#Lifecycle|SKILLS.th.md §Lifecycle]]:

```
init       → resolve การพึ่งพา plugin kind workflow; cache handle dispatch ของ bwoc gcloud
             ไม่เรียก gcloud, ไม่อ่าน credential — Anattā
invoke     → แต่ละ operation ประกอบ verb bwoc gcloud หนึ่งตัวขึ้นไป; verb เขียนตัวเดียว
             (switch-project) สืบทอด gate operator-confirm ของ CLI operation อ่านเป็น idempotent;
             switch-project converge (ตั้ง default ปัจจุบันซ้ำเป็น no-op)
teardown   → no-op skill ไม่ถือ state ภายนอก — สถานะ credential + โปรเจกต์อยู่ใน config
             ของ gcloud เอง อ่านผ่าน plugin โดยอ้อม
```

Skill ไม่ถือ global state ระหว่าง invocation Replay-safe

## Maturity

ประกาศ **L1** — spec + scaffold; สัญญา operation ตายตัวแต่ยังไม่ verify end-to-end การ verify จริงกับโปรเจกต์ GCP จริงถูก gate ด้วย sandbox ที่ operator ให้ (service-account JSON ที่ `.bwoc/secrets/gcloud-sa.json` หรือ ADC ในเครื่องที่ login แล้ว) — ความเสี่ยง `BWOC-EPIC-8` เดียวกับที่ plugin [[../../modules/plugins/workflow/gcloud-auth/SPEC|gcloud-auth]] + [[../../modules/plugins/workflow/gcloud-project/SPEC|gcloud-project]] แบก ขยับเป็น L2 เมื่อมี agent อย่างน้อยหนึ่งตัวขับ `whoami` → `switch-project` ครบ end-to-end; เป็น L3 เมื่อ `bwoc skill verify gcloud-ops` ถูกต่อสายและผ่านใน CI

## ความเป็นกลาง (Neutrality)

ค่าใน manifest ไม่ระบุ LLM backend, model, หรือ vendor CLI ใด `requires_plugins = ["workflow"]` อ้างค่า enum ของ plugin-**kind** ของเฟรมเวิร์กเอง ไม่ใช่ vendor "GCP" / "Google Cloud" / "gcloud" ปรากฏแค่ใน `description` และ prose ของ SPEC นี้ในฐานะชื่อเป้าหมายการ integrate ยอมรับได้ภายใต้กฎเดียวกับที่ยอมให้ plugin ระบุเป้าหมายของมัน `bwoc gcloud` และ `bwoc skill verify` เป็นคำสั่งของเฟรมเวิร์ก ไม่ใช่คำสั่ง backend สอดคล้อง **Samānattatā**

## ดูเพิ่ม

- [[../../docs/th/SKILLS.th|SKILLS.th.md]] — spec ที่ skill นี้ conform; dependency model แบบ skill-on-plugin + skill-on-multiple-plugins
- [[../../modules/plugins/workflow/gcloud-auth/SPEC|gcloud-auth SPEC.md]] — plugin credential-state ที่ skill นี้ขับ (verb `status`)
- [[../../modules/plugins/workflow/gcloud-project/SPEC|gcloud-project SPEC.md]] — plugin project-context ที่ skill นี้ขับ (verb `show`, `set-default`)
- [[../../docs/th/PLUGINS.th|PLUGINS.th.md]] — แถว kind `workflow` + แกน Skill กับ Plugin
- [[../../notes/2026-05-28_gcloud-workflow-plugin-architecture|2026-05-28_gcloud-workflow-plugin-architecture.md]] — กรอบ EPIC-8 (§Decision 5 ขอบเขต skill, §Decision 2 การแยก plugin สองตัว)
- [[../scrum-via-jira/SPEC|scrum-via-jira]] — skill-on-plugin ตัวแรก; รูปร่างที่ตัวนี้ขยายไปสู่ plugin หลายตัว
- [[SPEC|SPEC.md]] — คู่ภาษาอังกฤษ (bilingual parity)
