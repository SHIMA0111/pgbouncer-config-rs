use regex::Regex;

pub trait ParserIniFromStr {
    type Error;

    fn parse_from_str(value: &str) -> Result<Self, Self::Error> where Self: Sized;
}

pub(crate) fn parse_key_value(value: &str) -> crate::error::Result<(String, String)> {
    let key_value_regex = Regex::new(
        r#"^\s*(?P<key>[^=]+?)\s*=\s*(?P<value>.+?)\s*$"#
    )?;
    let caps = key_value_regex.captures(value).ok_or(
        crate::error::PgBouncerError::PgBouncer(format!("Invalid format key=value: {}", value))
    )?;
    let key = caps.name("key").ok_or(
        crate::error::PgBouncerError::PgBouncer(format!("Invalid key: {}", value))
    )?.as_str().trim().to_string();
    let value = caps.name("value").ok_or(
        crate::error::PgBouncerError::PgBouncer(format!("Invalid value: {}", value))
    )?.as_str().trim().to_string();
    Ok((key, value))
}

pub(crate) fn is_comment(value: &str) -> bool {
    value.starts_with("#") || value.starts_with(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_comment_detects_hash_and_semicolon() {
        assert!(is_comment("# this is a comment"));
        assert!(is_comment("; also a comment"));
        assert!(!is_comment("not a comment"));
        assert!(!is_comment(" key = value"));
    }

    #[test]
    fn test_parse_key_value_parses_key_and_value() {
        let (k, v) = parse_key_value("mykey = some value").expect("should parse");
        assert_eq!(k, "mykey");
        assert_eq!(v, "some value");

        let (k2, v2) = parse_key_value("  another_key=  'quoted value'  ").expect("should parse");
        assert_eq!(k2, "another_key");
        assert_eq!(v2, "'quoted value'");
    }

    #[test]
    fn test_parse_key_value_format() {
        let (key, value) = parse_key_value("no-braces = value").unwrap();
        assert_eq!(key, "no-braces");
        assert_eq!(value, "value");
    }
}
