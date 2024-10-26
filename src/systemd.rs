use anyhow::Result;
use which::which;

pub fn generate_nftables(
    url: &str,
    code: &str,
    table: &str,
    ipv4set: &str,
    ipv6set: &str,
) -> Result<String> {
    let nftables_exe = which("nft")?.to_string_lossy().into_owned();
    let current_exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let code_args = if !code.is_empty() {
        &format!("--code {code} ")
    } else {
        ""
    };
    Ok(format!(
        "\
[Unit]
Description=tsumugi nftables (nft -f)
Requires=nftables.service
After=nftables.service
# e.g. sing-box.service
Requires=place_holder.service
After=place_holder.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/bin/sh -ec \"{current_exe} \\
            --url {url} \\
            {code_args}generate nftables \\
            --table {table} --ipv4set {ipv4set} --ipv6set {ipv6set} | {nftables_exe} -f -\"

ExecReload=/bin/sh -ec \"{current_exe} \\
            --url {url} \\
            {code_args}generate nftables \\
            --table {table} --ipv4set {ipv4set} --ipv6set {ipv6set} | {nftables_exe} -f -\"

ExecStop={nftables_exe} flush set inet {table} {ipv4set}
ExecStop={nftables_exe} flush set inet {table} {ipv6set}

[Install]
WantedBy=multi-user.target
"
    ))
}

pub fn generate_iproute2_route(
    url: &str,
    code: &str,
    table: &str,
    ipv4_gateway: &str,
    ipv6_gateway: &str,
    dev: &str,
) -> Result<String> {
    let ip_exe = which("ip")?.to_string_lossy().into_owned();
    let current_exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let code_args = if !code.is_empty() {
        &format!("--code {code} ")
    } else {
        ""
    };
    let cache_path = "/tmp/.tsumugi_iproute2_route_cache.db";
    let orig_path = format!("{}.bak", cache_path);
    let generate_args = "generate iproute2 route";
    let table = format!("--table {}", table);
    let gateway_dev_args = format!(
        "--ipv4-gateway {} --ipv6-gateway {} --dev {}",
        ipv4_gateway, ipv6_gateway, dev
    );
    Ok(format!(
        "\
[Unit]
Description=tsumugi iproute2 route (ip route add)
# e.g. sing-box.service
Requires=place_holder.service
After=place_holder.service

[Service]
Type=oneshot
RemainAfterExit=yes

ExecStartPre={current_exe} --url {url} \\
            {code_args}convert --output {cache_path} srs
ExecStart=/bin/sh -ec \"{current_exe} -f {cache_path} {generate_args} \\
            {table} {gateway_dev_args}| {ip_exe} -batch -\"

ExecReload=/bin/sh -ec \"/bin/mv {cache_path} {orig_path}; \\
            {current_exe} -f {orig_path} {generate_args} \\
            {current_exe} --url {url} \\
            {code_args}convert --output {cache_path} srs; \\
            --delete {table} {gateway_dev_args} | {ip_exe} -batch -; \\
            /bin/rm {orig_path}; \\
            {current_exe} {cache_path} {generate_args} \\
            {table} {gateway_dev_args} | {ip_exe} -batch -\"

ExecStop=/bin/sh -ec \"{current_exe} {cache_path} {generate_args} \\
            --delete {table} {gateway_dev_args} | {ip_exe} -batch -\"
ExecStop=/bin/rm -f {cache_path}

[Install]
WantedBy=multi-user.target
"
    ))
}

pub fn generate_iproute2_rule(url: &str, code: &str, table: &str) -> Result<String> {
    let ip_exe = which("ip")?.to_string_lossy().into_owned();
    let current_exe = std::env::current_exe()?.to_string_lossy().into_owned();
    let code_args = if !code.is_empty() {
        &format!("--code {code} ")
    } else {
        ""
    };
    let cache_path = "/tmp/.tsumugi_iproute2_rule_cache.db";
    let orig_path = format!("{}.bak", cache_path);
    let generate_args = "generate iproute2 route";
    let table = format!("--table {}", table);
    Ok(format!(
        "\
[Unit]
Description=tsumugi iproute2 rule (ip rule add)
# e.g. sing-box.service
Requires=place_holder.service
After=place_holder.service

[Service]
Type=oneshot
RemainAfterExit=yes

ExecStartPre={current_exe} --url {url} \\
            {code_args}convert --output {cache_path} srs
ExecStart=/bin/sh -ec \"{current_exe} -f {cache_path} {generate_args} \\
            {table} | {ip_exe} -batch -\"

ExecReload=/bin/sh -ec \"/bin/mv {cache_path} {orig_path}; \\
            {current_exe} --url {url} \\
            {code_args}convert --output {cache_path} srs; \\
            {current_exe} -f {orig_path} {generate_args} \\
            --delete {table} | {ip_exe} -batch -; \\
            /bin/rm {orig_path}; \\
            {current_exe} -f {cache_path} {generate_args} \\
            {table} | {ip_exe} -batch -\"

ExecStop=/bin/sh -ec \"{current_exe} -f {cache_path} {generate_args} \\
            --delete {table} | {ip_exe} -batch -\"
ExecStop=/bin/rm -f /tmp/.tsumugi_iproute2_rule_cache.db

[Install]
WantedBy=multi-user.target
"
    ))
}
