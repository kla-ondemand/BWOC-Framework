#!/usr/bin/env bash
# check-agent-neutrality.sh — validates backend neutrality for BWOC agent profiles
#
# Usage:
#   ./scripts/check-agent-neutrality.sh           # validate template root
#   ./scripts/check-agent-neutrality.sh <path>    # validate a specific agent
#
# Exit codes:
#   0 = all checks passed
#   1 = violations found

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET="${1:-$ROOT}"
[[ "$TARGET" != /* ]] && TARGET="$ROOT/$TARGET"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'
violations=0; warnings=0

fail()  { echo -e "${RED}FAIL${NC}  $1"; violations=$((violations + 1)); }
warn()  { echo -e "${YELLOW}WARN${NC}  $1"; warnings=$((warnings + 1)); }
pass()  { echo -e "${GREEN}PASS${NC}  $1"; }

# Hardcoded values that must not appear in AGENTS.md
HARDCODED_MODELS=(claude-opus claude-sonnet claude-haiku claude-3 claude-4 gemini-2 gemini-1 gemini-pro gpt-4 gpt-3 o3- o4- codex- kimi-k2)
HARDCODED_TOOLS=(mempalace chromadb pinecone pgvector weaviate)
BACKEND_PHRASES=("Claude will" "Claude can" "Gemini will" "Gemini can" "Codex will" "Kimi will")

echo ""
echo "BWOC Agent Neutrality Check"
echo "============================"
echo "Target: $TARGET"
echo ""

# 1. AGENTS.md exists and is a regular file
if [[ -f "$TARGET/AGENTS.md" ]]; then
  pass "AGENTS.md exists"
else
  fail "AGENTS.md not found — this is the single source of truth"
fi

# 2. Backend symlinks
for backend in GEMINI.md CODEX.md KIMI.md; do
  if [[ -L "$TARGET/$backend" ]]; then
    target_link=$(readlink "$TARGET/$backend")
    if [[ "$target_link" == "AGENTS.md" ]]; then
      pass "$backend -> AGENTS.md"
    else
      fail "$backend points to '$target_link' instead of AGENTS.md"
    fi
  else
    warn "$backend missing (create with: ln -s AGENTS.md $backend)"
  fi
done

# 3. CLAUDE.md — can be a symlink or a real guidance file
if [[ -L "$TARGET/CLAUDE.md" ]]; then
  target_link=$(readlink "$TARGET/CLAUDE.md")
  if [[ "$target_link" == "AGENTS.md" ]]; then
    pass "CLAUDE.md -> AGENTS.md"
  else
    fail "CLAUDE.md points to '$target_link' instead of AGENTS.md"
  fi
elif [[ -f "$TARGET/CLAUDE.md" ]]; then
  pass "CLAUDE.md exists (standalone guidance file)"
else
  warn "CLAUDE.md missing"
fi

# 4. config.manifest.json
if [[ -f "$TARGET/config.manifest.json" ]]; then
  if python3 -c "import json,sys; json.load(open('$TARGET/config.manifest.json'))" 2>/dev/null; then
    pass "config.manifest.json is valid JSON"
  else
    fail "config.manifest.json is not valid JSON"
  fi
else
  warn "config.manifest.json missing (recommended for cloning readiness)"
fi

if [[ -f "$TARGET/AGENTS.md" ]]; then
  agents="$TARGET/AGENTS.md"

  # 5. Required placeholders
  for ph in '{{agentId}}' '{{memoryPath}}' '{{taskId}}' '{{deepMemoryCmd}}'; do
    if grep -qF "$ph" "$agents" 2>/dev/null; then
      pass "AGENTS.md contains $ph"
    else
      warn "AGENTS.md missing recommended placeholder $ph"
    fi
  done

  # 6. No YAML frontmatter in AGENTS.md
  first_line=$(head -1 "$agents")
  if [[ "$first_line" == "---" ]]; then
    fail "AGENTS.md has YAML frontmatter — instruction files must use plain Markdown"
  else
    pass "AGENTS.md has no YAML frontmatter"
  fi

  # 7. No wikilinks in AGENTS.md
  if grep -qE '\[\[.*\]\]' "$agents" 2>/dev/null; then
    fail "AGENTS.md contains wikilinks — instruction files must use plain Markdown"
  else
    pass "AGENTS.md has no wikilinks"
  fi

  # 8. No hardcoded model IDs
  model_ok=true
  for model in "${HARDCODED_MODELS[@]}"; do
    if grep -qi "$model" "$agents" 2>/dev/null; then
      line=$(grep -im1 "$model" "$agents")
      # Allow in table examples and quoted backtick context
      if echo "$line" | grep -qE '`[^`]*'"$model"'[^`]*`|\|.*\|'; then
        warn "AGENTS.md mentions '$model' (verify it is only in an example context)"
      else
        fail "AGENTS.md contains hardcoded model ID '$model'"
        model_ok=false
      fi
    fi
  done
  $model_ok && pass "No hardcoded model IDs in AGENTS.md"

  # 9. No hardcoded tool names
  tool_ok=true
  for tool in "${HARDCODED_TOOLS[@]}"; do
    if grep -qi "$tool" "$agents" 2>/dev/null; then
      fail "AGENTS.md contains hardcoded tool name '$tool'"
      tool_ok=false
    fi
  done
  $tool_ok && pass "No hardcoded tool names in AGENTS.md"

  # 10. No backend-specific language
  lang_ok=true
  for phrase in "${BACKEND_PHRASES[@]}"; do
    if grep -q "$phrase" "$agents" 2>/dev/null; then
      fail "AGENTS.md contains backend-specific phrase '$phrase'"
      lang_ok=false
    fi
  done
  $lang_ok && pass "No backend-specific language in AGENTS.md"
fi

# --- Summary ---
echo ""
echo "=============================="
if [[ $violations -gt 0 ]]; then
  echo -e "${RED}${violations} violation(s)${NC}, ${YELLOW}${warnings} warning(s)${NC}"
  echo "Fix violations before merging."
  exit 1
else
  echo -e "${GREEN}0 violations${NC}, ${YELLOW}${warnings} warning(s)${NC}"
  echo "Neutrality check passed."
  exit 0
fi
