# tsumugi (紬)

## Example

```ini
❯ tsumugi --url https://github.com/SagerNet/sing-geoip/raw/refs/heads/rule-set/geoip-cn.srs systemd iproute2 rule
[Unit]
Description=tsumugi iproute2 rule (ip rule add)
# e.g. sing-box.service
Requires=place_holder.service
After=place_holder.service

[Service]
Type=oneshot
RemainAfterExit=yes

ExecStartPre=/usr/bin/tsumugi --url https://github.com/SagerNet/sing-geoip/raw/refs/heads/rule-set/geoip-cn.srs \
            convert --output /tmp/.tsumugi_iproute2_rule_cache.db srs
ExecStart=/bin/sh -ec "/usr/bin/tsumugi -f /tmp/.tsumugi_iproute2_rule_cache.db generate iproute2 route \
            --table main | /usr/bin/ip -batch -"

ExecReload=/bin/sh -ec "/bin/mv /tmp/.tsumugi_iproute2_rule_cache.db /tmp/.tsumugi_iproute2_rule_cache.db.bak; \
            /usr/bin/tsumugi --url https://github.com/SagerNet/sing-geoip/raw/refs/heads/rule-set/geoip-cn.srs \
            convert --output /tmp/.tsumugi_iproute2_rule_cache.db srs; \
            /usr/bin/tsumugi -f /tmp/.tsumugi_iproute2_rule_cache.db.bak generate iproute2 route \
            --delete --table main | /usr/bin/ip -batch -; \
            /bin/rm /tmp/.tsumugi_iproute2_rule_cache.db.bak; \
            /usr/bin/tsumugi -f /tmp/.tsumugi_iproute2_rule_cache.db generate iproute2 route \
            --table main | /usr/bin/ip -batch -"

ExecStop=/bin/sh -ec "/usr/bin/tsumugi -f /tmp/.tsumugi_iproute2_rule_cache.db generate iproute2 route \
            --delete --table main | /usr/bin/ip -batch -"
ExecStop=/bin/rm -f /tmp/.tsumugi_iproute2_rule_cache.db

[Install]
WantedBy=multi-user.target
```

## Usage

```console
Simple tool for interactive with *ray geoip.dat and sing-box ruleset

Usage: tsumugi [OPTIONS] <--file <FILE>|--url <URL>> <COMMAND>

Commands:
  generate  Generate things, e.g. nftables script
  convert   Convert from one format to another
  systemd   Generate a systemd service unit
  help      Print this message or the help of the given subcommand(s)

Options:
  -f, --file <FILE>      Url of the file to download
  -u, --url <URL>        Path of the file to read
  -c, --code <CODE>      Country code
  -o, --output <OUTPUT>  Output path
  -h, --help             Print help
  -V, --version          Print version
```