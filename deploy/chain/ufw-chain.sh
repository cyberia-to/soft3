#!/usr/bin/env bash
# Open (or close) the radio QUIC/UDP port of one chain instance in ufw, idempotently, with a log line.
#
#   sudo ./ufw-chain.sh open  pussy     # reads SOFT3_RADIO from /etc/soft3/pussy.env
#   sudo ./ufw-chain.sh close pussy
#   sudo ./ufw-chain.sh show  pussy
#
# Only the UDP port is touched. The HTTP API stays on loopback behind nginx (80/443 are already open
# on every cybernode machine). jupiter has no passwordless sudo: run this there as root by hand.
set -euo pipefail

action="${1:-}"; chain="${2:-}"
[[ "$action" =~ ^(open|close|show)$ && -n "$chain" ]] || { echo "usage: $0 open|close|show <chain>" >&2; exit 2; }
env_file="/etc/soft3/$chain.env"
[[ -r "$env_file" ]] || { echo "missing $env_file" >&2; exit 2; }
# shellcheck disable=SC1090
source "$env_file"
port="${SOFT3_RADIO##*:}"
[[ "$port" =~ ^[0-9]+$ && "$port" -ge 1024 && "$port" -le 65535 ]] || { echo "SOFT3_RADIO has no sane port: '$SOFT3_RADIO'" >&2; exit 2; }
comment="soft3-chain-$chain-radio"
log="/var/log/soft3-chain/ufw.log"
mkdir -p "$(dirname "$log")"

present() { ufw status | grep -qE "^$port/udp\s+ALLOW IN\s+Anywhere" ; }

case "$action" in
  show)
    ufw status numbered | grep -E "$port/udp|$comment" || echo "no rule for $port/udp"
    ;;
  open)
    if present; then echo "already open: $port/udp ($chain)"; exit 0; fi
    ufw allow proto udp to any port "$port" comment "$comment"
    echo "$(date -u +%FT%TZ) open $port/udp $chain by $(logname 2>/dev/null || echo root)" >> "$log"
    echo "opened $port/udp ($chain)"
    ;;
  close)
    if ! present; then echo "not open: $port/udp ($chain)"; exit 0; fi
    ufw delete allow proto udp to any port "$port" comment "$comment" >/dev/null 2>&1 \
      || ufw delete allow "$port/udp"
    echo "$(date -u +%FT%TZ) close $port/udp $chain by $(logname 2>/dev/null || echo root)" >> "$log"
    echo "closed $port/udp ($chain)"
    ;;
esac
