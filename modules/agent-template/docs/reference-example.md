# memories/reference-postgres-conventions.md
---
type: reference
source: conventions/database.md#naming
date: 2026-05-22
verifiedAgainst: db/schema/users.sql@abc123
relatedFiles:
  - db/migrations/0042_add_users.sql
  - docs/database/naming.md
ttl: 90d
---

# PostgreSQL Naming Conventions (Verified)

## Tables
- `snake_case`, plural
- Example: `users`, `order_items`, `audit_logs`

## Columns
- `snake_case`
- Primary key: `id` (uuid)
- Foreign key: `<table_singular>_id` (e.g., `user_id`)
- Timestamps: `created_at`, `updated_at`, `deleted_at`

## Indexes
- Pattern: `idx_<table>_<col1>[_<col2>]`
- Unique: `uniq_<table>_<col>`
- Example: `idx_users_email`, `uniq_users_username`

## Constraints
- `fk_<from_table>_<to_table>` for foreign keys
- `chk_<table>_<rule>` for check constraints

---

## Sources
- conventions/database.md §3 (last updated 2026-04-10)
- Migration 0042 demonstrates pattern in practice
- PR #218 discussion confirmed conventions

## Revisit
- หาก convention ถูก update → re-verify
- ทุก 90 วัน → recheck กับ schema ปัจจุบัน
