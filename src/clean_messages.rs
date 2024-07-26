use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref TLD_REGEX: Regex = {
        let common_tlds: [&str; 20] = [
            "com", "org", "net", "ru", "de", "jp", "uk", "br", "pl", "in", "it", "fr", "au",
            "info", "cn", "nl", "eu", "biz", "za", "io",
        ];
        let tld_pattern = common_tlds
            .iter()
            .map(|&tld| format!(r"\.{}", regex::escape(tld.to_lowercase().as_str())))
            .collect::<Vec<_>>()
            .join("|");
        Regex::new(&tld_pattern).unwrap()
    };
}

// Remove naughty content from discord messages so our logger isn't unhappy
pub(crate) fn clean_message(message: &str) -> String {
    // Pattern for mentions (e.g., @username)
    let mention_pattern = r"@(\w+)";
    // Pattern for URLs (http:// or https:// followed by non-whitespace characters)
    let url_pattern = r"https?://(\S+)";

    let re_mention = Regex::new(mention_pattern).unwrap();
    let re_url = Regex::new(url_pattern).unwrap();

    let mut clean_message = message.to_string();

    // Remove '@' from mentions
    clean_message = re_mention.replace_all(&clean_message, "$1").to_string();

    // Modify URLs
    clean_message = re_url
        .replace_all(&clean_message, |caps: &regex::Captures| {
            let url = &caps[1];
            // Remove the TLD from the URL
            let modified_url = TLD_REGEX.replace(url, "").to_string();
            modified_url
        })
        .to_string();

    clean_message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_message() {
        // Collection of tuples containing test messages and their expected cleaned versions
        let test_cases = vec![
            (
                "Hello @user123, visit https://example.com or https://test-site.io for info.",
                "Hello user123, visit example or test-site for info.",
            ),
            (
                "@admin: Check out our new site at http://cool-stuff.com!",
                "admin: Check out our new site at cool-stuff!",
            ),
            (
                "Contact @support or @help, and visit https://dangerous-place.net for more.",
                "Contact support or help, and visit dangerous-place for more.",
            ),
            (
                "Click here: https://www.youtube.com/watch?v=dQw4w9WgXcQ",
                "Click here: www.youtube/watch?v=dQw4w9WgXcQ",
            ),
        ];

        // Iterate over the test cases and assert the expected outcome
        for (input, expected) in test_cases {
            assert_eq!(
                clean_message(input),
                expected,
                "Failed on message: {}",
                input
            );
        }
    }
}
