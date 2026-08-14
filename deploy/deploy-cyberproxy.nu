#!/usr/bin/env nu
# Deploy spacepussy-test node surface to cyberproxy and (re)start systemd.

const HOST = "cyberproxy"
const REMOTE = "/home/cyber/spacepussy-test"

def main [] {
    print $"deploying spacepussy-test → ($HOST):($REMOTE)"
    ^ssh $HOST $"mkdir -p ($REMOTE)/data"
    ^scp [
        "deploy/spacepussy_test_node.py"
        "deploy/spacepussy-test.service"
        "deploy/nginx-rpc.spacepussy-test.soft3.org.conf"
    ] $"($HOST):($REMOTE)/"
    ^ssh $HOST $"chmod +x ($REMOTE)/spacepussy_test_node.py"

    # systemd unit
    ^ssh $HOST $"sudo cp ($REMOTE)/spacepussy-test.service /etc/systemd/system/spacepussy-test.service"
    ^ssh $HOST "sudo systemctl daemon-reload"
    ^ssh $HOST "sudo systemctl enable --now spacepussy-test.service"
    ^ssh $HOST "sudo systemctl restart spacepussy-test.service"
    ^ssh $HOST "systemctl --no-pager --full status spacepussy-test.service | head -20"

    # nginx vhost (HTTP first; TLS after DNS A record exists)
    ^ssh $HOST $"sudo cp ($REMOTE)/nginx-rpc.spacepussy-test.soft3.org.conf /etc/nginx/sites-available/rpc.spacepussy-test.soft3.org"
    ^ssh $HOST "sudo ln -sfn /etc/nginx/sites-available/rpc.spacepussy-test.soft3.org /etc/nginx/sites-enabled/rpc.spacepussy-test.soft3.org"
    ^ssh $HOST "sudo nginx -t && sudo systemctl restart nginx"

    print ""
    print "local probe on host:"
    ^ssh $HOST "curl -sS http://127.0.0.1:7780/status | head -c 400; echo"
    print ""
    print "public (after DNS A → 167.235.28.94):"
    print "  http://rpc.spacepussy-test.soft3.org/status"
    print "TLS:"
    print "  ssh cyberproxy 'sudo certbot --nginx -d rpc.spacepussy-test.soft3.org'"
}
