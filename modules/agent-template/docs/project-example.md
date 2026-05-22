# memories/project-billing-service.md
---
type: project
project: billing-service
date: 2026-05-22
status: active
ownerAgent: agent-database-schema
contributorAgents:
  - agent-api-builder
  - agent-test-author
---

# Project Memory — Billing Service

## ภาพรวม (Overview)
Billing service handles subscription, invoicing, and payment events.

## Domain Conventions ที่ตกลง

### Subscription States
```
trial → active → past_due → canceled → expired
                    ↓
                 paused → active
```

### Invoice Numbering
- Format: `INV-YYYY-NNNNNNN`
- Reset yearly
- Stored in `invoices.invoice_number` (immutable after creation)

### Idempotency
- ทุก payment intent มี `idempotency_key`
- Key TTL = 24h
- Implemented at `services/billing/payment_processor.ts`

## บทเรียนเฉพาะ project นี้

1. **Stripe webhooks อาจซ้ำ** — ใช้ event ID + DB constraint
2. **Tax calculation** — delegate ไป tax service (อย่าง compute เอง)
3. **Refunds** — partial refunds ต้อง audit log แต่ละครั้ง

## Files ที่ต้องระวัง

| File | Why |
|---|---|
| `db/migrations/0078_subscription_state_machine.sql` | State machine encoded — modify ด้วยความระวัง |
| `services/billing/invoice_generator.ts` | Numbering logic — race conditions |
| `webhooks/stripe.ts` | External contract — เปลี่ยน schema = breaking |

## Related Memories
- `reference-stripe-webhooks.md`
- `decision-2026-03-10-idempotency-approach.md`
- `feedback-PROJ-38-duplicate-charges.md`

## Cross-Agent Coordination
- Schema changes → agent-database-schema (me)
- API surface → agent-api-builder
- Test coverage → agent-test-author
