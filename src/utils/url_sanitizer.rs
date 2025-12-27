/// Sanitizes a URL by validating it's HTTP or HTTPS protocol.
/// Returns empty string for invalid URLs.
pub fn sanitize_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }

    // Check if URL starts with http:// or https://
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        String::new()
    }
}

/// Sanitizes a list of URLs and returns both the sanitized URLs
/// and a list of update mappings [old_url, new_url] for changed URLs.
///
/// Returns: (sanitized_urls, update_urls)
/// where update_urls is an array of [original, sanitized] pairs
pub fn sanitize_urls(urls: &[String]) -> (Vec<String>, Vec<[String; 2]>) {
    let mut sanitized = Vec::new();
    let mut updates = Vec::new();

    for url in urls {
        let sanitized_url = sanitize_url(url);

        // Track URLs that were changed (rewritten to empty string)
        if url != &sanitized_url {
            updates.push([url.clone(), sanitized_url.clone()]);
        }

        sanitized.push(sanitized_url);
    }

    (sanitized, updates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_url_valid_http() {
        assert_eq!(
            sanitize_url("http://example.com/feed.rss"),
            "http://example.com/feed.rss"
        );
    }

    #[test]
    fn test_sanitize_url_valid_https() {
        assert_eq!(
            sanitize_url("https://example.com/feed.rss"),
            "https://example.com/feed.rss"
        );
    }

    #[test]
    fn test_sanitize_url_invalid_ftp() {
        assert_eq!(sanitize_url("ftp://example.com/file"), "");
    }

    #[test]
    fn test_sanitize_url_invalid_javascript() {
        assert_eq!(sanitize_url("javascript:alert(1)"), "");
    }

    #[test]
    fn test_sanitize_url_empty() {
        assert_eq!(sanitize_url(""), "");
    }

    #[test]
    fn test_sanitize_urls_mixed() {
        let urls = vec![
            "http://example.com/1".to_string(),
            "ftp://bad.com".to_string(),
            "https://example.com/2".to_string(),
        ];

        let (sanitized, updates) = sanitize_urls(&urls);

        assert_eq!(sanitized.len(), 3);
        assert_eq!(sanitized[0], "http://example.com/1");
        assert_eq!(sanitized[1], "");
        assert_eq!(sanitized[2], "https://example.com/2");

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0], ["ftp://bad.com".to_string(), "".to_string()]);
    }
}
