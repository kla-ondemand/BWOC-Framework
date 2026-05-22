# Glossary — Pali Terms in BWOC

This is the fast-lookup reference for every Pali term used in BWOC. Each entry gives the **engineering meaning in one line**. For the full mapping (which framework owns the term, what it determines, how it composes with others), see [`PHILOSOPHY.en.md`](../../modules/agent-template/docs/en/PHILOSOPHY.en.md).

**Reading aid:** Pali terms are *labels*. The engineering meaning is what matters in code and reviews. No religious literacy is required to use BWOC.

---

## How to Use

- **Looking up a term you read in a doc or commit?** Find it in the alphabetical table below.
- **Wondering which framework a term belongs to?** Group is named in the Engineering Meaning column.
- **Want the full treatment of a framework?** Click through to `PHILOSOPHY.en.md` (linked at the bottom).

---

## Alphabetical Index

| Pali | English | Engineering meaning |
|---|---|---|
| **Acinteyya 4** | Four Unthinkables | Things deliberately not modeled — boundaries of speculation. |
| **Adinnādānā veramaṇī** | Abstain from taking what is not given | CoC: respect attribution, license, IP. No plagiarism. |
| **Anattā** | Non-self | Release stale state. Cleanup worktrees, branches, memory; no clinging. |
| **Anicca** | Impermanence | Everything mutates; memory and caches need pruning and timestamps. |
| **Aparihāniya-dhamma 7** | Seven Non-Decline Principles | Fleet governance — what keeps a multi-agent system from decay. |
| **Ariya-dhana 7** | Seven Noble Treasures | Capability maturity levels (L1 → L7). |
| **Ariyasacca 4** | Four Noble Truths | Problem-solving spine. Every task: Dukkha → Samudaya → Nirodha → Magga. |
| **Attanutata** | Knowing self | Capability declaration — agent states what it can and cannot do, up front. |
| **Bhāvanā 4** | Four Cultivations | Agent lifecycle stages (growth → maturity → mentoring → release). |
| **Bhava-taṇhā** | Craving to persist | Threat category: persistence, privilege escalation. |
| **Brahmavihāra 4** | Four Divine Abidings | Error/UX disposition: Mettā · Karuṇā · Muditā · Upekkhā. |
| **Dukkha** | Suffering / problem | (1) First Noble Truth — concrete problem statement; (2) one of the Three Marks. |
| **Iddhipāda 4** | Four Bases of Power | Engine of work — drive, persistence, attention, investigation. |
| **Jāti** | Birth | Optional gross-lifecycle synonym for [Uppāda]. |
| **Kamma 3** | Three Doors of Action | Audit logging — action, speech, intent. Maps to commit, message, plan. |
| **Kalyāṇamitta 7** | Seven Qualities of a Good Friend | Inter-agent trust scoring. |
| **Kāma-taṇhā** | Craving for stimulus | Threat category: influence attacks (prompt injection, social engineering). |
| **Kāmesumicchācārā veramaṇī** | Abstain from improper conduct | CoC: respect boundaries; no unwanted advances or sexualized content in project channels. |
| **Karuṇā** | Compassion | UX: suggest a fix, not only the error. |
| **Khandha 5** | Five Aggregates | Architecture model — Rūpa · Vedanā · Saññā · Saṅkhāra · Viññāṇa → file/IO/memory/logic/runtime. |
| **Magga** | Path | (1) Fourth Noble Truth — the plan; (2) shorthand for Magga 8. |
| **Magga 8** | Noble Eightfold Path | Functional requirements — eight pillars in the SRS. |
| **Maraṇa** | Death | Optional gross-lifecycle synonym for [Vaya]. |
| **Mattaññutā** | Right amount | Lean discipline — `MEMORY.md` ≤ 200 lines; smallest spec wins. |
| **Mettā** | Loving-kindness | UX: friendly, direct tone. |
| **Muditā** | Sympathetic joy | UX: acknowledge when others were right; welcome new contributors. |
| **Musāvādā veramaṇī** | Abstain from false speech | CoC: no impersonation, no falsified results, no misleading commits. |
| **Nirodha** | Cessation | (1) Third Noble Truth — measurable success state; (2) the *vaya* phase action. |
| **Padhāna 4** | Four Right Efforts | Effort discipline — restrain, abandon, develop, maintain. |
| **Paññā 3** | Three Roots of Wisdom | Self-improvement — sutamaya (learning), cintāmaya (reasoning), bhāvanāmaya (cultivation). |
| **Pāṇātipātā veramaṇī** | Abstain from harm | CoC: no harassment, threats, doxxing, hate speech. |
| **Paṭiccasamuppāda** | Dependent Origination | Failure analysis — trace conditions backward. The visible problem is rarely the root. |
| **Sammā-ājīva** | Right livelihood | Trust and neutrality — no vendor lock-in, no preferential backend. |
| **Sammā-diṭṭhi** | Right view | Persona, identity. |
| **Sammā-kammanta** | Right action | Worktree discipline, commits. |
| **Sammā-samādhi** | Right concentration | Focus, session scope. |
| **Sammā-saṅkappa** | Right intention | Goal setting, planning. |
| **Sammā-sati** | Right mindfulness | Memory system. |
| **Sammā-vācā** | Right speech | Inter-agent communication. |
| **Sammā-vāyāma** | Right effort | Verification gates (lint, format, test, regression, build). |
| **Samānattatā** | Equal treatment | All backends treated equally; no vendor favoritism in core docs. |
| **Samudaya** | Origin | Second Noble Truth — root cause. |
| **Saṅgahavatthu 4** | Four Bases of Sympathy | User relations — giving, kind speech, helpful action, equanimity. |
| **Saṅkhata** | Conditioned thing | Anything that arises and ceases. The basis for the arc (uppāda · ṭhiti · vaya). |
| **Sappurisadhamma 7** | Seven Qualities of a True Person | Context sensing — situation, audience, time, etc. |
| **Sāraṇīyadhamma 6** | Six Conditions of Cordiality | Inter-agent coordination protocol. |
| **Satipaṭṭhāna 4** | Four Foundations of Mindfulness | Observability — body, feeling, mind, dhamma → metrics, logs, traces, state. |
| **Sīla** | Moral conduct | Baseline safety discipline (often shorthand for Sīla 5). |
| **Sīla 5** | Five Precepts | Baseline forbidden actions — no harm, no taking, no improper conduct, no false speech, no heedlessness. |
| **Sīlasāmaññatā** | Communal convention | All agents follow the same shared rules. Conventions beat preferences. |
| **Surāmerayamajjapamādaṭṭhānā veramaṇī** | Abstain from heedlessness | CoC: no contributing under impaired judgment; no reckless commits. |
| **Taṇhā 3** | Three Cravings | Threat categories — Kāma · Bhava · Vibhava (stimulus · persistence · destruction). |
| **Ṭhiti** | Persisting-with-change | Arc phase 2 — agent operates, state evolves under discipline. |
| **Tilakkhaṇa** | Three Marks of existence | Anicca · Dukkha · Anattā — design must conform. |
| **Upekkhā** | Equanimity | UX: stay even when frustrated; disagree without escalating. |
| **Uppāda** | Arising | Arc phase 1 — identity created, manifest resolved, capabilities declared. |
| **Vaya** | Passing-away | Arc phase 3 — cleanup, branch release, memory prune, task closed. |
| **Vibhava-taṇhā** | Craving to destroy | Threat category: destruction, data loss. |
| **Yoniso Manasikāra** | Wise attention | Verify against current state before acting on remembered claims. |

---

## See Also

- [`PHILOSOPHY.en.md`](../../modules/agent-template/docs/en/PHILOSOPHY.en.md) — full mappings and conceptual core.
- [`PHILOSOPHY.en.md §0.1`](../../modules/agent-template/docs/en/PHILOSOPHY.en.md#01-the-arc--uppāda--ṭhiti--vaya) — the arc (uppāda · ṭhiti · vaya).
- [`VISION.md`](../../VISION.md) — why these terms in the first place.
- [Pali Text Society dictionary](https://www.palidictionary.org/) — for primary linguistic sources beyond engineering use.
