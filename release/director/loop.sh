#!/bin/bash
# every 15 minutes, one tick; the loop itself never thinks
export PATH="$HOME/.local/bin:$PATH" DIR="$HOME/director"
while true; do bash "$DIR/tick.sh" || echo "$(date -u +%H:%M:%S) tick failed $?" >> "$DIR/state/tick.log"; sleep 900; done
