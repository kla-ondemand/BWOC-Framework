# bwoc-agent — สตริงภาษาไทยสำหรับ liveness banner
# รูปแบบ: Project Fluent

# Liveness ที่อิง manifest (พิมพ์เมื่อ bwoc-agent รันใน directory ของ agent ที่ incarnate แล้ว)
liveness-alive = ฉันยังมีชีวิตอยู่: { $agent_id }
liveness-role = role:     { $role }
liveness-model = model:    { $model }
liveness-fallback = fallback: { $fallback }
liveness-memory = memory:   { $memory_path }
liveness-version = version:  { $version }

# Error: cwd ไม่ใช่ agent ที่ incarnate (ไม่มี config.manifest.json)
error-missing-manifest = bwoc-agent: ไม่มี config.manifest.json ใน { $cwd } — รันจากภายใน directory ของ agent ที่ incarnate แล้ว
