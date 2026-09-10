#!/bin/sh
# CircuitDeck factorio-metrics exporter (development only).
#
# graftorio3 (running inside the Factorio server) writes a Prometheus
# textfile to /data/script-output/graftorio3/game.prom every few seconds
# (nth-tick=300 -> every 5 game seconds). This sidecar serves that file at
# /metrics with standard textfile-collector semantics so Prometheus can
# scrape it:
#
#   - the file is served byte-identical to what the mod wrote,
#   - a missing/empty file yields an empty exposition body (Prometheus
#     keeps the last samples for ~5 minutes, matching node_exporter),
#   - staleness is reported as an explicit exporter-health gauge,
#     mirroring node_exporter's textfile collector.
#
# Why not node_exporter directly? Its textfile collector reads a local
# directory; here the Factorio container owns the volume. A tiny busybox
# httpd avoids shipping extra binaries while keeping the exposition
# identical to what the mod wrote. /game.prom (raw mod output) is served
# as-is for debugging.

set -eu

PROM_DIR="${PROM_DIR:-/data/script-output/graftorio3}"
PROM_FILE="$PROM_DIR/game.prom"
PORT="${PORT:-9100}"
# Fresh mod output, copied into the served tree on startup and refreshed
# once per second afterwards (one stat + optional copy, bounded by the
# mod's own 5s write cadence).
SERVED=/srv/game.prom

mkdir -p /srv
cp -f "$PROM_FILE" "$SERVED" 2>/dev/null || : > "$SERVED"

# Render /metrics (mod exposition + exporter-health gauge) once per second;
# busybox httpd serves it statically. 1s staleness is well under the 15s
# Prometheus scrape interval. The health gauge measures the age of the MOD's
# own file (true write staleness), not the served copy.
(
  while true; do
    cp -f "$PROM_FILE" "$SERVED" 2>/dev/null || true
    {
      cat "$SERVED"
      now=$(date +%s)
      if [ -s "$SERVED" ]; then
        mtime=$(stat -c %Y "$PROM_FILE" 2>/dev/null || echo "$now")
        age=$((now - mtime))
        printf '# HELP factorio_exporter_file_age_seconds Seconds since graftorio3 last wrote game.prom.\n'
        printf '# TYPE factorio_exporter_file_age_seconds gauge\n'
        printf 'factorio_exporter_file_age_seconds %s\n' "$age"
      else
        printf '# HELP factorio_exporter_file_missing Whether graftorio3 has written game.prom yet (1 = missing).\n'
        printf '# TYPE factorio_exporter_file_missing gauge\n'
        printf 'factorio_exporter_file_missing 1\n'
      fi
    } > /tmp/metrics.next 2>/dev/null && mv -f /tmp/metrics.next /srv/metrics
    sleep 1
  done
) &
POLLER=$!
trap 'kill "$POLLER" 2>/dev/null || true' TERM INT

# Static /metrics plus CGI /metrics.sh (same payload); Prometheus scrapes
# /metrics.sh (see prometheus/prometheus.yml).
cat > /srv/metrics.sh <<'CGI'
#!/bin/sh
printf 'Content-Type: text/plain; version=0.0.4; charset=utf-8\r\n\r\n'
cat /srv/metrics 2>/dev/null || cat /srv/game.prom
CGI
chmod +x /srv/metrics.sh

exec busybox httpd -f -p "$PORT" -h /srv -c /circuitdeck/exporter/httpd.conf