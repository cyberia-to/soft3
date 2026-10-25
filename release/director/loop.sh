#!/bin/bash
# every 15 minutes, one tick; the loop itself never thinks
export PATH="$HOME/.local/bin:$PATH" DIR="$HOME/director"
# the body's subscription token (claude setup-token, 1 year); never in the repository
[ -f "$HOME/.config/cyber/claude.env" ] && . "$HOME/.config/cyber/claude.env"
while true; do if ! bash "$DIR/tick.sh"; then rc=$?; echo "$(date -u +%H:%M:%S) tick failed rc=$rc" >> "$DIR/state/tick.log"; fi; sleep 900; done
