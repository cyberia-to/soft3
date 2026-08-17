#!/usr/bin/env nu
# Deploy real soft3 node (cybergraph+bbg) to cyberproxy.

const HOST = "cyberproxy"

def main [] {
    print $"deploying soft3 node → ($HOST)"
    ^ssh $HOST 'export PATH=$HOME/.cargo/bin:$PATH; cargo install soft3 --force'
    ^scp "deploy/spacepussy-test.service" $"($HOST):/tmp/spacepussy-test.service"
    ^ssh $HOST 'sudo cp /tmp/spacepussy-test.service /etc/systemd/system/spacepussy-test.service'
    ^ssh $HOST 'sudo systemctl daemon-reload && sudo systemctl enable --now spacepussy-test && sudo systemctl restart spacepussy-test'
    ^ssh $HOST 'systemctl --no-pager status spacepussy-test | head -18'
    ^ssh $HOST 'curl -sS http://127.0.0.1:7780/status | head -c 500; echo'
    print "public: https://cyb.ai/spacepussy-test/status"
}
