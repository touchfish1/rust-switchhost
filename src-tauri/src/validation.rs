use reqwest::Url;
use std::net::IpAddr;
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

    if url.host_str().is_none() {
        return Err("远程 URL 必须包含有效主机名".to_string());
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

#[cfg(test)]
mod tests {
    use super::{validate_hosts_content, validate_remote_url};

    #[test]
    fn validates_remote_urls() {
        assert!(validate_remote_url("https://example.com/hosts.txt").is_ok());
        assert!(validate_remote_url("http://127.0.0.1:8080/hosts").is_ok());
        assert!(validate_remote_url("ftp://example.com/file").is_err());
        assert!(validate_remote_url("not-a-url").is_err());
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
