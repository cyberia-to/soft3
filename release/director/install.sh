#!/bin/bash
# put the director on agent-a under the claude body and start the loop in tmux (no systemd in the container)
set -euo pipefail
HOST=${HOST:-agent-a}; BODY=${BODY:-claude}; SRC=$(cd "$(dirname "$0")" && pwd); SSHF=${SSHF:-$HOME/cyber/cybernode/local/ssh_config}
ssh() { command ssh -F "$SSHF" -o BatchMode=yes "$@"; }; scp() { command scp -F "$SSHF" "$@"; }
ssh "$HOST" "install -d -o $BODY -g $BODY /srv/forge/$BODY/director"
scp -q "$SRC"/{README.md,tick.sh,director.yaml,loop.sh} "$HOST:/srv/forge/$BODY/director/"
ssh "$HOST" "chown -R $BODY:$BODY /srv/forge/$BODY/director; chmod +x /srv/forge/$BODY/director/*.sh; sudo -u $BODY -H bash -lc 'tmux has-session -t director 2>/dev/null && tmux kill-session -t director; tmux new-session -d -s director \"bash \$HOME/director/loop.sh\"; tmux ls'"
