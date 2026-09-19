use anyhow::{Context, bail};
use deadpool_postgres::Config;

/// Value of `key` if it is set and not blank.
fn non_empty(get: &impl Fn(&str) -> Option<String>, key: &str) -> Option<String> {
    get(key).filter(|v| !v.trim().is_empty())
}

/// Resolve the database connection settings from environment-style lookups.
///
/// `DATABASE_URL` wins when set and non-empty. Otherwise the connection is built
/// from `POSTGRES_HOST` (default `localhost`), `POSTGRES_PORT` (default `5432`),
/// and the required `POSTGRES_USER`, `POSTGRES_PASSWORD` and `POSTGRES_DB`.
/// The discrete values are set on the config as-is, so passwords containing
/// characters such as `@`, `:`, `/` or `%` need no URL encoding.
pub fn resolve_database_config(get: impl Fn(&str) -> Option<String>) -> anyhow::Result<Config> {
    let mut config = Config::new();

    if let Some(url) = non_empty(&get, "DATABASE_URL") {
        config.url = Some(url);
        return Ok(config);
    }

    let user = non_empty(&get, "POSTGRES_USER");
    let password = non_empty(&get, "POSTGRES_PASSWORD");
    let dbname = non_empty(&get, "POSTGRES_DB");

    let missing: Vec<&str> = [
        ("POSTGRES_USER", user.is_none()),
        ("POSTGRES_PASSWORD", password.is_none()),
        ("POSTGRES_DB", dbname.is_none()),
    ]
    .into_iter()
    .filter_map(|(name, is_missing)| is_missing.then_some(name))
    .collect();

    if !missing.is_empty() {
        bail!(
            "Database is not configured: missing {}. \
             Set DATABASE_URL, or set POSTGRES_USER, POSTGRES_PASSWORD and POSTGRES_DB \
             (POSTGRES_HOST defaults to localhost, POSTGRES_PORT to 5432).",
            missing.join(", ")
        );
    }

    config.host = Some(non_empty(&get, "POSTGRES_HOST").unwrap_or_else(|| "localhost".to_string()));
    config.port = Some(match non_empty(&get, "POSTGRES_PORT") {
        Some(port) => port
            .trim()
            .parse()
            .with_context(|| format!("POSTGRES_PORT is not a valid port number: {port:?}"))?,
        None => 5432,
    });
    config.user = user;
    config.password = password;
    config.dbname = dbname;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn resolve(vars: &[(&str, &str)]) -> anyhow::Result<Config> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        resolve_database_config(|key| map.get(key).cloned())
    }

    #[test]
    fn database_url_wins_over_discrete_vars() {
        let config = resolve(&[
            ("DATABASE_URL", "postgresql://u:p@db:5432/app"),
            ("POSTGRES_USER", "other"),
            ("POSTGRES_PASSWORD", "other"),
            ("POSTGRES_DB", "other"),
        ])
        .unwrap();

        assert_eq!(config.url.as_deref(), Some("postgresql://u:p@db:5432/app"));
        assert_eq!(config.host, None);
        assert_eq!(config.user, None);
    }

    #[test]
    fn database_url_alone_needs_nothing_else() {
        let config = resolve(&[("DATABASE_URL", "postgresql://u:p@db/app")]).unwrap();
        assert!(config.url.is_some());
    }

    #[test]
    fn blank_database_url_falls_back_to_discrete_vars() {
        let config = resolve(&[
            ("DATABASE_URL", "  "),
            ("POSTGRES_USER", "u"),
            ("POSTGRES_PASSWORD", "p"),
            ("POSTGRES_DB", "d"),
        ])
        .unwrap();

        assert_eq!(config.url, None);
        assert_eq!(config.user.as_deref(), Some("u"));
    }

    #[test]
    fn discrete_vars_are_assembled() {
        let config = resolve(&[
            ("POSTGRES_HOST", "db.internal"),
            ("POSTGRES_PORT", "6543"),
            ("POSTGRES_USER", "humidor"),
            ("POSTGRES_PASSWORD", "secret"),
            ("POSTGRES_DB", "humidor_db"),
        ])
        .unwrap();

        assert_eq!(config.url, None);
        assert_eq!(config.host.as_deref(), Some("db.internal"));
        assert_eq!(config.port, Some(6543));
        assert_eq!(config.user.as_deref(), Some("humidor"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.dbname.as_deref(), Some("humidor_db"));
    }

    #[test]
    fn host_and_port_have_defaults() {
        let config = resolve(&[
            ("POSTGRES_USER", "u"),
            ("POSTGRES_PASSWORD", "p"),
            ("POSTGRES_DB", "d"),
        ])
        .unwrap();

        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
    }

    #[test]
    fn password_with_special_characters_is_kept_verbatim() {
        let password = "p@ss:w/rd%40 #?&=+";
        let config = resolve(&[
            ("POSTGRES_USER", "u"),
            ("POSTGRES_PASSWORD", password),
            ("POSTGRES_DB", "d"),
        ])
        .unwrap();

        assert_eq!(config.password.as_deref(), Some(password));
        // The pool builds the connection from the discrete fields, not a URL.
        assert_eq!(config.url, None);
        let pg = config.get_pg_config().unwrap();
        assert_eq!(pg.get_password(), Some(password.as_bytes()));
        assert_eq!(pg.get_user(), Some("u"));
    }

    #[test]
    fn nothing_set_names_all_required_vars() {
        let err = resolve(&[]).unwrap_err().to_string();

        assert!(err.contains("missing POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_DB"));
        assert!(err.contains("DATABASE_URL"));
    }

    #[test]
    fn missing_password_is_named_and_not_defaulted() {
        let err = resolve(&[("POSTGRES_USER", "u"), ("POSTGRES_DB", "d")])
            .unwrap_err()
            .to_string();

        // Only the "missing ..." clause; the trailing hint lists all the variables.
        assert!(err.starts_with("Database is not configured: missing POSTGRES_PASSWORD. "));
    }

    #[test]
    fn empty_required_var_counts_as_missing() {
        let err = resolve(&[
            ("POSTGRES_USER", ""),
            ("POSTGRES_PASSWORD", "p"),
            ("POSTGRES_DB", "d"),
        ])
        .unwrap_err()
        .to_string();

        assert!(err.contains("missing POSTGRES_USER."));
    }

    #[test]
    fn invalid_port_is_rejected() {
        let err = resolve(&[
            ("POSTGRES_PORT", "not-a-port"),
            ("POSTGRES_USER", "u"),
            ("POSTGRES_PASSWORD", "p"),
            ("POSTGRES_DB", "d"),
        ])
        .unwrap_err()
        .to_string();

        assert!(err.contains("POSTGRES_PORT"));
    }
}
