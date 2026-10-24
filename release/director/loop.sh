#!/bin/bash
# every 15 minutes, one tick; the loop itself never thinks
export PATH="$HOME/.local/bin:$PATH" DIR="$HOME/director"
while true; do if ! bash "$DIR/tick.sh"; then rc=$?; echo "$(date -u +%H:%M:%S) tick failed rc=$rc" >> "$DIR/state/tick.log"; fi; sleep 900; done
