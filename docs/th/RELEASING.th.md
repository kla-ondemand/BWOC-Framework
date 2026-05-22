---
title: การปล่อย Release BWOC
aliases:
  - Release Process
tags:
  - group/framework
  - type/process
  - meta/operations
---

# การปล่อย Release BWOC

> [!abstract] วิธีปล่อยรีลีสของชุดเครื่องมือ bwoc (`bwoc` CLI + `bwoc-agent` daemon) — Release ใช้ tag เป็นตัวกระตุ้น: push tag ที่ตรงตามรูปแบบ `v*` แล้ว build pipeline จะทำการ build ทุก platform แล้ว upload ไปยัง GitHub Release โดยอัตโนมัติ

## ก่อน tag

ก่อนกด tag — ผู้ดูแลควรตรวจสอบ:

- [ ] **CI ผ่าน** บน `main` ของ commit ล่าสุด — ดูที่ [Actions → CI](https://github.com/bemindlabs/BWOC-Framework/actions/workflows/ci.yml)
- [ ] **`CHANGELOG.md`** มี section สำหรับเวอร์ชันที่จะปล่อย — เปลี่ยน `[Unreleased]` block เป็น `[X.Y.Z] — YYYY-MM-DD` แล้วเพิ่ม `[Unreleased]` ใหม่ที่ว่างเปล่าไว้ด้านบน
- [ ] **`VERSION.md`** `Software-Version` ตรงกับ tag (auto-bump โดย hook ทุกครั้งที่แก้โค้ด ควรจะตรงกับ workspace `Cargo.toml` อยู่แล้ว)
- [ ] **`Cargo.toml`** workspace version ตรงกับ tag

## กด tag

```bash
git tag v0.1.0            # หรือ v0.2.0-rc1
git push origin v0.1.0
```

Tag จะกระตุ้น [`.github/workflows/release.yml`](../../.github/workflows/release.yml):

1. **Build แบบ matrix** — 4 target ทำงานพร้อมกัน:
   - `x86_64-unknown-linux-gnu`
   - `aarch64-apple-darwin` (macOS Apple Silicon)
   - `x86_64-apple-darwin` (macOS Intel)
   - `x86_64-pc-windows-msvc`
2. **Package** แต่ละตัวเป็น `bwoc-<tag>-<target>.{tar.gz|zip}` มี `bwoc`, `bwoc-agent`, `README.md`, `LICENSE`, `CHANGELOG.md`
3. **Sidecar** ไฟล์ `.sha256` ข้าง archive แต่ละตัว
4. **สร้าง GitHub Release อัตโนมัติ** พร้อม note จาก commit range ตั้งแต่ tag ก่อนหน้า
5. **Upload** artifact ทั้งหมดไปยัง release

## Pre-release vs final

Tag ที่มีเครื่องหมาย `-` จะถูกทำเครื่องหมายเป็น **prerelease** อัตโนมัติ:

| Tag | ประเภท |
|---|---|
| `v0.1.0` | final |
| `v0.1.0-rc1` | prerelease (release candidate) |
| `v0.2.0-beta.3` | prerelease (beta) |

GitHub ใช้ flag prerelease กับเครื่องมือ auto-update และป้าย "Latest release"

## นโยบายเวอร์ชัน

BWOC ใช้ [SemVer 2.0.0](https://semver.org/) สำหรับ binary ของ bwoc toolkit ดู [`VERSION.md`](../../VERSION.md) สำหรับนโยบายฉบับเต็มและความแตกต่างระหว่าง phase กับ version (Phase 1 / Phase 2 / ... เป็นแนวคิด roadmap ไม่ใช่แกนเวอร์ชัน)

## สิ่งที่ยังไม่อยู่ใน pipeline

- **Code signing** — Apple notarization (macOS) และ Windows Authenticode ยังไม่ได้ตั้งค่า binary ปล่อยแบบไม่ signed ผู้ใช้จะเห็น prompt "untrusted developer" ตอน launch ครั้งแรก การเพิ่ม signing ต้องให้ maintainer จัดการ cert และเก็บ signing key ไว้ใน GitHub Actions secrets
- **Linux ARM / musl** — มีเฉพาะ `x86_64-unknown-linux-gnu` ARM Linux (`aarch64-unknown-linux-gnu`) และ musl-libc (`x86_64-unknown-linux-musl`) เพิ่มเข้า matrix ได้เมื่อมีความต้องการจริงจากผู้ใช้
- **Homebrew formula / Scoop manifest / cargo binstall metadata** — distribution อยู่ใน ecosystem ของตัวเอง

## Rolling back

ถ้า release ที่ tag ไปแล้วมี artifact เสีย:

1. **อย่าลบ** tag — GitHub Release ยังเก็บ binary ที่เสียอยู่ และผู้ใช้อาจดาวน์โหลดไปแล้ว
2. กด patch tag ใหม่ที่มีการแก้ (เช่น `v0.1.1` หลังจาก `v0.1.0`)
3. แก้ release note ของ release ที่เสียให้ชี้ไปยังตัวที่มาทดแทน

แบบนี้สอดคล้องกับนโยบาย SemVer: patch ไม่ย้อนกลับ

## ดูเพิ่ม

- [`.github/workflows/release.yml`](../../.github/workflows/release.yml) — workflow ที่เอกสารนี้อธิบาย
- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — gate ของแต่ละ commit ต้องเป็นสีเขียวก่อนกด tag
- [`CHANGELOG.md`](../../CHANGELOG.md) — สิ่งที่ต้อง update ก่อน tag
- [`VERSION.md`](../../VERSION.md) — เวอร์ชันปัจจุบัน + นโยบาย SemVer
- [`ROADMAP.th.md`](ROADMAP.th.md) — phase (ไม่กำหนดเวอร์ชัน)
