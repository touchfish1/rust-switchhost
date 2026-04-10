use reqwest::Url;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub fn validate_remote_url(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("远程 URL 不能为空".to_string());
    }

    let url = Url::parse(trimmed).map_err(|_| "远程 URL 格式不正确".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("远程 URL 仅支持 http 或 https 协议".to_string()),
    }

    let host = url
        .host_str()
        .ok_or_else(|| "远程 URL 必须包含有效主机名".to_string())?;

    if is_private_host(host) {
        return Err("远程 URL 不能指向本机、内网或链路本地地址".to_string());
    }

    Ok(())
}

pub fn validate_hosts_content(content: &str) -> Result<(), String> {
    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let content_part = raw_line.split('#').next().unwrap_or("").trim();
        if content_part.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = content_part.split_whitespace().collect();
        if tokens.len() < 2 {
            return Err(format!("第 {} 行缺少主机名映射", line_number));
        }

        let ip = tokens[0];
        IpAddr::from_str(ip).map_err(|_| format!("第 {} 行 IP 地址无效: {}", line_number, ip))?;

        for hostname in &tokens[1..] {
            if !is_valid_hostname(hostname) {
                return Err(format!("第 {} 行主机名无效: {}", line_number, hostname));
            }
        }
    }

    Ok(())
}

fn is_valid_hostname(hostname: &str) -> bool {
    if hostname.is_empty() || hostname.len() > 253 {
        return false;
    }

    if hostname == "localhost" {
        return true;
    }

    hostname.split('.').all(is_valid_label)
}

fn is_valid_label(label: &str) -> bool {
    if label.is_empty() || label.len() > 63 {
        return false;
    }

    let bytes = label.as_bytes();
    let first = bytes[0] as char;
    let last = bytes[bytes.len() - 1] as char;

    if !is_valid_label_char(first) || !is_valid_label_char(last) {
        return false;
    }

    bytes
        .iter()
        .all(|byte| is_valid_label_char(*byte as char) || *byte == b'-')
}

fn is_valid_label_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_private_host(host: &str) -> bool {
    let normalized = host.trim_matches(['[', ']']);

    if normalized.eq_ignore_ascii_case("localhost") {
        return true;
    }

    match IpAddr::from_str(normalized) {
        Ok(IpAddr::V4(ip)) => is_private_ipv4(ip),
        Ok(IpAddr::V6(ip)) => is_private_ipv6(ip),
        Err(_) => false,
    }
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
}

fn is_private_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified()
}

#[cfg(test)]
mod tests {
    use super::{validate_hosts_content, validate_remote_url};

    #[test]
    fn validates_remote_urls() {
        assert!(validate_remote_url("https://example.com/hosts.txt").is_ok());
        assert!(validate_remote_url("https://raw.githubusercontent.com/example/hosts.txt").is_ok());
        assert!(validate_remote_url("ftp://example.com/file").is_err());
        assert!(validate_remote_url("not-a-url").is_err());
    }

    #[test]
    fn rejects_private_remote_urls() {
        assert!(validate_remote_url("http://127.0.0.1:8080/hosts").is_err());
        assert!(validate_remote_url("http://10.0.0.12/hosts").is_err());
        assert!(validate_remote_url("http://172.16.10.5/hosts").is_err());
        assert!(validate_remote_url("http://192.168.1.2/hosts").is_err());
        assert!(validate_remote_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(validate_remote_url("http://[::1]/hosts").is_err());
        assert!(validate_remote_url("http://localhost/hosts").is_err());
    }

    #[test]
    fn validates_hosts_content() {
        let valid = r#"
127.0.0.1 localhost
192.168.1.10 api.example.com web.example.com
::1 localhost
"#;
        assert!(validate_hosts_content(valid).is_ok());

        let invalid_ip = "localhost example.com";
        assert!(validate_hosts_content(invalid_ip).is_err());

        let invalid_host = "127.0.0.1 -bad.example.com";
        assert!(validate_hosts_content(invalid_host).is_err());

        let missing_host = "127.0.0.1";
        assert!(validate_hosts_content(missing_host).is_err());
    }
}
