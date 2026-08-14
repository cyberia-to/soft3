#!/usr/bin/env python3
"""spacepussy-test chaosnet surface — product RPC for soft3 / true-cyber.

Serves:
  GET /status  — JSON (compatible with soft3 probe + cosmos-shaped fields)
  GET /health  — ok
  GET /        — short blurb

Height advances every BLOCK_SECS from genesis wall-clock.
State dir: $SOFT3_HOME or ~/.spacepussy-test
Bind: $SOFT3_BIND or 127.0.0.1:7780
"""

from __future__ import annotations

import json
import os
import socket
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

CHAIN_ID = "spacepussy-test"
BLOCK_SECS = 5
VERSION = "0.5.0"


def home_dir() -> Path:
    raw = os.environ.get("SOFT3_HOME") or os.path.expanduser("~/.spacepussy-test")
    p = Path(raw)
    p.mkdir(parents=True, exist_ok=True)
    return p


def load_genesis(home: Path) -> int:
    path = home / "genesis.json"
    if path.exists():
        try:
            data = json.loads(path.read_text())
            return int(data.get("genesis_time", time.time()))
        except Exception:
            pass
    t = int(time.time())
    path.write_text(
        json.dumps(
            {
                "chain_id": CHAIN_ID,
                "genesis_time": t,
                "network": "spacepussy-test",
                "protocol": "soft3/spacepussy-test/v0",
            },
            indent=2,
        )
        + "\n"
    )
    return t


def tip_height(genesis: int) -> int:
    return max(0, (int(time.time()) - genesis) // BLOCK_SECS)


def moniker() -> str:
    return os.environ.get("SOFT3_MONIKER") or socket.gethostname() or "soft3-node"


class Handler(BaseHTTPRequestHandler):
    genesis_time: int = 0

    def log_message(self, fmt: str, *args) -> None:
        sys_stderr = __import__("sys").stderr
        print(f"  {self.address_string()} {fmt % args}", file=sys_stderr)

    def _send(self, code: int, ctype: str, body: bytes) -> None:
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        path = self.path.split("?", 1)[0]
        if path in ("/status", "/status/"):
            height = tip_height(self.genesis_time)
            payload = {
                "jsonrpc": "2.0",
                "id": -1,
                "result": {
                    "node_info": {
                        "network": CHAIN_ID,
                        "moniker": moniker(),
                        "version": VERSION,
                        "protocol": "soft3/spacepussy-test/v0",
                    },
                    "sync_info": {
                        "latest_block_height": str(height),
                        "earliest_block_height": "1",
                        "catching_up": False,
                        "genesis_time": self.genesis_time,
                    },
                    "soft3": {
                        "role": "soft3 chaosnet (product default)",
                        "denom": "testpussy",
                        "prefix": "pussy",
                        "block_secs": BLOCK_SECS,
                    },
                },
            }
            body = (json.dumps(payload, indent=2) + "\n").encode()
            self._send(200, "application/json", body)
            return
        if path in ("/health", "/health/"):
            self._send(200, "text/plain", b"ok\n")
            return
        if path in ("/",):
            height = tip_height(self.genesis_time)
            text = (
                f"soft3 · spacepussy-test\n"
                f"moniker {moniker()}\n"
                f"height {height}\n"
                f"rpc /status\n"
            )
            self._send(200, "text/plain", text.encode())
            return
        self._send(404, "text/plain", b"not found\n")


def main() -> None:
    home = home_dir()
    genesis = load_genesis(home)
    bind = os.environ.get("SOFT3_BIND", "127.0.0.1:7780")
    host, _, port_s = bind.rpartition(":")
    host = host or "127.0.0.1"
    port = int(port_s or "7780")

    Handler.genesis_time = genesis
    httpd = ThreadingHTTPServer((host, port), Handler)
    print(
        f"soft3 node · {CHAIN_ID} · moniker={moniker()} · home={home}",
        flush=True,
    )
    print(f"  listening  {host}:{port}", flush=True)
    print(f"  tip height {tip_height(genesis)}", flush=True)
    print("  GET /status  /health  /", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()
