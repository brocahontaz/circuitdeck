#!/bin/sh
# Bootstrap for the CircuitDeck real-Factorio development mode.
#
# Runs inside the factoriotools/factorio image (as root, before its entrypoint
# drops privileges) and installs the CircuitDeck-owned pieces into the
# /factorio volume:
#
#   1. mods/mod-list.json                - vendored graftorio3 setup
#   2. config/server-settings.json       - LAN, no verify, RCON on
#
# The mod runs with its built-in defaults (graftorio3-server-save=true,
# graftorio3-nth-tick=300), so no mod-settings.dat is shipped. Everything is
# copied only when missing or when FORCE_BOOTSTRAP=true, so the save, mods and
# settings survive container restarts and image upgrades.
set -eu

VOL=/factorio
MODS="$VOL/mods"
CONFIG="$VOL/config"

echo "circuitdeck-factorio-bootstrap: installing mod files"
mkdir -p "$MODS" "$CONFIG"
# Install the vendored mod zip + mod list on every start so image updates
# keep the installation in sync (the mod zip is immutable upstream code).
cp -f /circuitdeck/mods/graftorio3_2.0.0.zip "$MODS/"
cp -f /circuitdeck/mods/mod-list.json "$MODS/mod-list.json"
chown -R factorio:factorio "$MODS"

if [ ! -f "$CONFIG/server-settings.json" ] || [ "${FORCE_BOOTSTRAP:-false}" = "true" ]; then
  echo "circuitdeck-factorio-bootstrap: rendering server-settings.json"
  envsubst < /circuitdeck/server-settings.json > "$CONFIG/server-settings.json"
  chown factorio:factorio "$CONFIG/server-settings.json"
fi

# The image entrypoint passes --rcon-password "$(cat rconpw)" on the CLI,
# which overrides the settings file. Seed rconpw deterministically so
# FACTORIO_RCON_PASSWORD is actually what the server uses (only when
# missing, like the image's own generator, to avoid clobbering changes).
if [ ! -f "$CONFIG/rconpw" ]; then
  echo "circuitdeck-factorio-bootstrap: seeding rconpw"
  printf '%s\n' "${FACTORIO_RCON_PASSWORD:-circuitdeck}" > "$CONFIG/rconpw"
  chown factorio:factorio "$CONFIG/rconpw"
fi

echo "circuitdeck-factorio-bootstrap: done; handing over to image entrypoint"
exec /docker-entrypoint.sh