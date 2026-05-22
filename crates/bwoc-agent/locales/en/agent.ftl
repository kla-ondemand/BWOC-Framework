# bwoc-agent — liveness banner.
# Format: Project Fluent (https://projectfluent.org/).
# Add new locales by dropping <lang>/agent.ftl and a match arm in i18n::bundle_for.

# Manifest-driven liveness (printed when bwoc-agent runs in an incarnated dir).
liveness-alive = I am alive: { $agent_id }
liveness-role = role:     { $role }
liveness-model = model:    { $model }
liveness-fallback = fallback: { $fallback }
liveness-memory = memory:   { $memory_path }
liveness-version = version:  { $version }

# Error: cwd is not an incarnated agent (no config.manifest.json).
error-missing-manifest = bwoc-agent: no config.manifest.json in { $cwd } — run from inside an incarnated agent directory
