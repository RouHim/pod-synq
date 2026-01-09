/// Convert a list of subscription URLs to OPML format
pub fn to_opml(subscriptions: &[String]) -> String {
    let mut opml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head>
    <title>Podcast Subscriptions</title>
  </head>
  <body>
"#,
    );
    for url in subscriptions {
        opml.push_str(&format!(
            r#"    <outline type="rss" text="{}" xmlUrl="{}"/>"#,
            url, url
        ));
        opml.push('\n');
    }
    opml.push_str("  </body>\n</opml>");
    opml
}

/// Convert a list of subscription URLs to plain text format (one per line)
pub fn to_txt(subscriptions: &[String]) -> String {
    subscriptions.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_txt_empty() {
        let subs: Vec<String> = vec![];
        assert_eq!(to_txt(&subs), "");
    }

    #[test]
    fn test_to_txt_single() {
        let subs = vec!["https://example.com/feed.xml".to_string()];
        assert_eq!(to_txt(&subs), "https://example.com/feed.xml");
    }

    #[test]
    fn test_to_txt_multiple() {
        let subs = vec![
            "https://example.com/feed1.xml".to_string(),
            "https://example.com/feed2.xml".to_string(),
        ];
        assert_eq!(
            to_txt(&subs),
            "https://example.com/feed1.xml\nhttps://example.com/feed2.xml"
        );
    }

    #[test]
    fn test_to_opml_empty() {
        let subs: Vec<String> = vec![];
        let opml = to_opml(&subs);
        assert!(opml.contains("<opml version=\"2.0\">"));
        assert!(opml.contains("<title>Podcast Subscriptions</title>"));
        assert!(opml.contains("</opml>"));
        assert!(!opml.contains("<outline"));
    }

    #[test]
    fn test_to_opml_single() {
        let subs = vec!["https://example.com/feed.xml".to_string()];
        let opml = to_opml(&subs);
        assert!(opml.contains("<opml version=\"2.0\">"));
        assert!(opml.contains(
            r#"<outline type="rss" text="https://example.com/feed.xml" xmlUrl="https://example.com/feed.xml"/>"#
        ));
    }

    #[test]
    fn test_to_opml_multiple() {
        let subs = vec![
            "https://example.com/feed1.xml".to_string(),
            "https://example.com/feed2.xml".to_string(),
        ];
        let opml = to_opml(&subs);
        assert!(opml.contains("feed1.xml"));
        assert!(opml.contains("feed2.xml"));
    }
}
