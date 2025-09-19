#[cfg(feature = "io")]
mod test {
    use pgbouncer_config::pgbouncer_config::PgBouncerConfig;
    use pgbouncer_config::utils::parser::ParserIniFromStr;

    fn minimal_pgbouncer_ini() -> String {
        "\
    [pgbouncer]\n\
    listen_addr = 127.0.0.1\n\
    listen_port = 6432\n\
    auth_type = md5\n\
    max_client_conn = 100\n\
    default_pool_size = 20\n\
    pool_mode = session\n\
    ".to_string()
    }

    #[test]
    fn parse_minimal_pgbouncer_section_ok() {
        let ini = minimal_pgbouncer_ini();
        let cfg = PgBouncerConfig::parse_from_str(&ini).expect("should parse minimal [pgbouncer]");
        let text = cfg.to_string();
        assert!(text.contains("[pgbouncer]"));
    }

    #[test]
    fn parse_missing_pgbouncer_section_ok_and_defaults_applied() {
        let ini = "\
    [databases]\n\
    app = dbname=app host=127.0.0.1 port=5432\n\
    ";
        let cfg = PgBouncerConfig::parse_from_str(ini).expect("should parse without [pgbouncer]");
        let text = cfg.to_string();
        assert!(text.contains("[pgbouncer]")); // default section rendered
        assert!(text.contains("[databases]"));
        assert!(text.contains("dbname=app"));
    }

    #[test]
    fn parse_with_databases_and_comments_ignores_comments() {
        let ini = "\
    # comment\n\
    [pgbouncer]\n\
    listen_addr = 127.0.0.1\n\
    listen_port = 6432\n\
    auth_type = md5\n\
    max_client_conn = 100\n\
    default_pool_size = 20\n\
    pool_mode = session\n\
    \n\
    [databases]\n\
    ; another comment\n\
    {app} = dbname=app host=127.0.0.1 port=5432\n\
    ";
        let cfg = PgBouncerConfig::parse_from_str(ini).expect("should parse with comments");
        let text = cfg.to_string();
        assert!(text.contains("[pgbouncer]"));
        assert!(text.contains("[databases]"));
        assert!(text.contains("dbname=app"));
        assert!(!text.contains("# comment"));
        assert!(!text.contains("; another comment"));
    }
}
