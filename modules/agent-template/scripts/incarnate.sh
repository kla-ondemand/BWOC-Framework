#!/usr/bin/env bash
# incarnate.sh — clone the BWOC agent template into a new agent repository
#
# Usage:
#   ./scripts/incarnate.sh <agent-name>
#   ./scripts/incarnate.sh <agent-name> /path/to/target
#
# Creates: ../agent-<name>/ (or target path)

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'

name="${1:-}"
if [[ -z "$name" ]]; then
  echo -e "${RED}Usage:${NC} ./scripts/incarnate.sh <agent-name> [target-path]"
  exit 1
fi

TEMPLATE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
target="${2:-$(dirname "$TEMPLATE_DIR")/agent-$name}"

if [[ -e "$target" ]]; then
  echo -e "${RED}Error:${NC} $target already exists"
  exit 1
fi

start_time=$(date +%s)
echo ""
echo "Incarnating agent: $name"
echo "Template:  $TEMPLATE_DIR"
echo "Target:    $target"
echo ""

# 1. Copy template (exclude git history and generated files)
rsync -a --exclude='.git' --exclude='*.example.*' "$TEMPLATE_DIR/" "$target/"

# 2. Create backend symlinks (remove any copied files first)
cd "$target"
for backend in AGY.md CODEX.md KIMI.md CLAUDE.md; do
  rm -f "$backend"
  ln -s AGENTS.md "$backend"
  echo -e "${GREEN}+${NC} $backend -> AGENTS.md"
done

# CLAUDE.md in agents should symlink (unlike the template which keeps a real guidance file)
# If you want Claude Code project-level guidance separate, replace CLAUDE.md with a real file.

# 3. Initialize fresh git history
git init -q
git add -A
git commit -q -m "Init: agent-$name from BWOC template v2"

echo -e "${GREEN}+${NC} git initialized"

# 4. Validate
echo ""
"$TEMPLATE_DIR/scripts/check-agent-neutrality.sh" "$target" || true

end_time=$(date +%s)
elapsed=$((end_time - start_time))

echo ""
echo "=============================="
echo -e "${GREEN}Done${NC} in ${elapsed}s"
echo ""
echo "Next steps:"
echo "  1. cd $target"
echo "  2. Edit AGENTS.md section 1 — fill {{placeholders}} with agent-specific values"
echo "  3. Edit config.manifest.json — resolve required placeholders"
echo "  4. Edit persona/README.md — define identity"
echo "  5. Run: ./scripts/check-agent-neutrality.sh"
echo "  6. Commit: git add -A && git commit -m 'Init: configure agent-$name'"
