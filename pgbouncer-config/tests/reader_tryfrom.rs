#[cfg(feature = "io")]
mod tests {
    use pgbouncer_config::io::read::{Reader, Readers};
    use std::path::PathBuf;

    #[test]
    fn readers_file_returns_err_for_missing_path() {
        let missing = PathBuf::from("/this/path/should/not/exist/pgbouncer.ini");
        assert!(!missing.exists(), "Test assumes the path does not exist");
        let res = Reader::try_from(Readers::File(&missing));
        assert!(res.is_err(), "Expected error for missing file path");
    }
}
