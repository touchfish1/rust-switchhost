use super::HostEntry;

pub fn parse(content: &str) -> Vec<HostEntry> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_num, line)| {
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            
            let ip = parts[0].to_string();
            let domain = parts[1].to_string();
            let comment = if parts.len() > 2 {
                Some(parts[2..].join(" "))
            } else {
                None
            };
            
            Some(HostEntry {
                ip,
                domain,
                comment,
                enabled: true,
                line_number: line_num + 1,
            })
        })
        .collect()
}

pub fn serialize(entries: &[HostEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            let mut line = format!("{}    {}", entry.ip, entry.domain);
            if let Some(comment) = &entry.comment {
                line.push_str(&format!("    # {}", comment));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "127.0.0.1    localhost\n192.168.1.1    example.com";
        let entries = parse(content);
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].ip, "127.0.0.1");
        assert_eq!(entries[0].domain, "localhost");
        assert_eq!(entries[1].ip, "192.168.1.1");
        assert_eq!(entries[1].domain, "example.com");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "# This is a comment\n127.0.0.1    localhost    # local";
        let entries = parse(content);
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].comment, Some("local".to_string()));
    }

    #[test]
    fn test_serialize() {
        let entries = vec![
            HostEntry {
                ip: "127.0.0.1".to_string(),
                domain: "localhost".to_string(),
                comment: None,
                enabled: true,
                line_number: 1,
            },
        ];
        
        let result = serialize(&entries);
        assert_eq!(result, "127.0.0.1    localhost");
    }
}
