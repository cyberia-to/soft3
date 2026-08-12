# ---
# tags: soft3, site, nu
# crystal-type: source
# crystal-domain: cyber
# ---
# verify every external href in site/index.html answers 200 on the live web.
# run before every site deploy: `nu site/check-links.nu`
# exits non-zero and lists the dead links if any target 404s — the guard that
# keeps the component links from drifting away from the published graph again.

def main [] {
    let html = open --raw ($env.FILE_PWD | path join "index.html")
    # bare-domain hrefs are rel=preconnect hints (fonts), not pages — skip them
    let links = ($html
        | parse --regex 'href="(https://[^"]+)"'
        | get capture0 | uniq
        | where {|u| ($u | url parse | get path) not-in ["", "/"] })
    let results = ($links | par-each {|u|
        let code = (do { http get --max-time 15sec --full --allow-errors $u | get status } | default 0)
        {url: $u, code: $code}
    })
    let dead = ($results | where code != 200)
    print $"checked ($links | length) links"
    if ($dead | is-empty) {
        print "all live"
    } else {
        print "DEAD:"
        print ($dead | table)
        exit 1
    }
}
