# BWOC CLI — English strings.
# Format: Project Fluent (https://projectfluent.org/).
# Phase 1 v2.0 scaffold; message keys land as commands are implemented.

scaffold-banner = bwoc (Phase 1 v2.0 scaffold) — lang={ $lang }
not-implemented = command not yet implemented in Phase 1
default-help-hint = bwoc (Phase 1 v2.0) — try `bwoc --help`

# bwoc new — incarnation report + interactive prompt
new-report-incarnated = Incarnated agent: { $agent_id }
new-report-target = Target:           { $path }
new-report-registered = Registered in workspace: { $path } (appended to .bwoc/agents.toml)
new-report-not-registered = No workspace found in ancestors — agent not registered in any agents.toml
new-report-next-steps-header = Next steps:
new-report-step-check = 1. cd { $path } && bwoc check . (verify backend neutrality)
new-report-step-edit-agents = 2. Edit AGENTS.md Section 1 — fill {"{{"}placeholders{"}}"} that aren't manifest fields.
new-report-step-edit-persona = 3. Edit persona/README.md — define identity, domains, boundaries.
new-report-step-git = 4. git init && git add -A && git commit -m 'feat(agent): incarnate'
new-prompt-format = { $key } ({ $desc }):{" "}

# bwoc check — header + PASS/WARN/FAIL labels + summaries
check-header = BWOC Agent Neutrality Check
check-target = Target: { $target }
check-label-pass = PASS
check-label-warn = WARN
check-label-fail = FAIL
check-summary-success = 0 violations, { $warnings } warning(s)
check-summary-success-tail = Neutrality check passed.
check-summary-failure = { $violations } violation(s), { $warnings } warning(s)
check-summary-failure-tail = Fix violations before merging.

# bwoc workspace validate — header + PASS/FAIL labels + summaries
validate-header = Workspace validation: { $path }
validate-label-pass = PASS
validate-label-fail = FAIL
validate-summary-success = { $passes } pass(es), 0 violation(s) — workspace is complete.
validate-summary-failure = { $passes } pass(es), { $violations } violation(s) — fix violations before operating on this workspace.

# bwoc workspace info — header + field labels + per-agent row
info-header = Workspace: { $path }
info-label-name = name
info-label-version = version
info-label-created = created
info-label-backend = backend
info-label-lang = lang
info-label-agents-dir = agents_dir
info-label-agents = agents
info-agent-row = { $id } ({ $status }) @ { $path }

# bwoc spawn — exec status (stderr)
spawn-exec-status = bwoc spawn: exec '{ $backend }' in { $path }

# bwoc list — agent registry display
list-empty = (no agents in workspace { $path })
list-col-id = ID
list-col-status = STATUS
list-col-backend = BACKEND
list-col-path = PATH

# bwoc init — success path
# (Fluent identifiers use `-`, not `.`, so we prefix instead of dotting.)
init-success-title = Initialized BWOC workspace at: { $path }
init-created-workspace-toml =   + .bwoc/workspace.toml
init-created-agents-toml =   + .bwoc/agents.toml
init-created-agents-dir =   + agents/  (default agents directory)
init-next-steps-header = Next steps:
init-next-step-validate =   bwoc workspace validate { $path }
init-next-step-new =   bwoc new <agent-name> ...        (incarnate the first agent here)
