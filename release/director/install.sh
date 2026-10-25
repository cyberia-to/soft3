#!/bin/bash
# put the director on agent-a under the claude body and start the loop in tmux (no systemd in the container)
set -euo pipefail
HOST=${HOST:-agent-a}; BODY=${BODY:-claude}; SRC=$(cd "$(dirname "$0")" && pwd); SSHF=${SSHF:-$HOME/cyber/cybernode/local/ssh_config}
ssh() { command ssh -F "$SSHF" -o BatchMode=yes "$@"; }; scp() { command scp -F "$SSHF" "$@"; }
ssh "$HOST" "install -d -o $BODY -g $BODY /srv/forge/$BODY/director /srv/forge/$BODY/doctrine /srv/forge/$BODY/work"
# the shared doctrine of the home (~/cyber/AGENTS.md, CLAUDE.md) is not a repository; it travels with the install
scp -q "$HOME/cyber/AGENTS.md" "$HOME/cyber/CLAUDE.md" "$HOST:/srv/forge/$BODY/doctrine/"
scp -q "$SRC"/{README.md,tick.sh,director.yaml,loop.sh,ask-fable} "$HOST:/srv/forge/$BODY/director/"
ssh "$HOST" "chown -R $BODY:$BODY /srv/forge/$BODY/director /srv/forge/$BODY/doctrine /srv/forge/$BODY/work; chmod +x /srv/forge/$BODY/director/*.sh /srv/forge/$BODY/director/ask-fable; sudo -u $BODY -H bash -lc 'tmux has-session -t director 2>/dev/null && tmux kill-session -t director; tmux new-session -d -s director \"bash \$HOME/director/loop.sh\"; tmux ls'"
