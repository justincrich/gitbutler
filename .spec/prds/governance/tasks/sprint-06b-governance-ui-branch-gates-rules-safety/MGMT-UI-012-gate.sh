#!/usr/bin/env bash
#
# MGMT-UI-012 build gate — forbid SvelteKit server-side files.
#
# Widened by REMEDIATE-UI-2 (closes red-hat M3) from the original
# +page.server.ts-only check to also cover +layout.server.ts and +server.ts.
#
# adapter-static (apps/desktop/src): ALL server files are forbidden —
#   +page.server.ts, +layout.server.ts, and +server.ts.
#
# adapter-vercel (apps/web/src): +server.ts is a LEGITIMATE API route
#   pattern (e.g. routes/install.sh/+server.ts serves the install script).
#   Only +page.server.ts and +layout.server.ts are checked here as
#   defense-in-depth for the governance surface; +server.ts is allowed
#   because apps/web is not an adapter-static app.
#
# Exit 0 when the tree is clean; exit 1 with the offending paths on stderr.

set -u

desktop_violations=$(find apps/desktop/src -type f \
    \( -name '+page.server.ts' -o -name '+layout.server.ts' -o -name '+server.ts' \) \
    2>/dev/null || true)

web_violations=$(find apps/web/src -type f \
    \( -name '+page.server.ts' -o -name '+layout.server.ts' \) \
    2>/dev/null || true)

if [ -n "${desktop_violations}" ] || [ -n "${web_violations}" ]; then
    echo "MGMT-UI-012 FAILED: forbidden SvelteKit server files found:" >&2
    [ -n "${desktop_violations}" ] && printf '%s\n' "${desktop_violations}" >&2
    [ -n "${web_violations}" ] && printf '%s\n' "${web_violations}" >&2
    exit 1
fi

exit 0
