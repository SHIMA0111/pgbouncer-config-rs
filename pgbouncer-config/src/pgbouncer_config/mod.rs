//! Core configuration model for PgBouncer.
//!
//! This module defines the typed configuration structures and traits used to
//! construct, render, and parse PgBouncer configuration files (commonly
//! called pgbouncer.ini). The primary entry point is [`PgBouncerConfig`], a
//! container that holds individual configuration sections implementing
//! [`Expression`].
//!
//! Sections currently provided by this crate:
//! - [`pgbouncer_setting`]: Typed representation of the [pgbouncer] section.
//! - [`databases_setting`]: Typed representation of the [databases] section.
//!
//! Rendering is driven by the [`Expression`] trait; parsing from INI text is
//! available via the [`ParserIniFromStr`] trait implementation for
//! [`PgBouncerConfig`].

use std::any::{type_name, Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display};
use std::ops::{Index, IndexMut};
use std::sync::{LazyLock, Mutex};
use heck::ToKebabCase;
use serde::{Deserialize, Serialize};
use crate::error::PgBouncerError;
#[cfg(feature = "io")]
use regex::Regex;
#[cfg(feature = "io")]
use crate::pgbouncer_config::databases_setting::DatabasesSetting;
#[cfg(feature = "io")]
use crate::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
#[cfg(feature = "io")]
use crate::utils::parser::{is_comment, ParserIniFromStr};
#[cfg(feature = "diff")]
use crate::utils::diff::Diffable;

pub mod pgbouncer_setting;
pub mod databases_setting;

static EXPRESSION_DEFAULT_SECTION_NAME: LazyLock<Mutex<HashMap<TypeId, &'static str>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Renderable configuration node.
///
/// Types implementing `Expression` can render themselves to the textual form
/// used by PgBouncer configuration files or sections. The return value is the
/// exact text that would appear in pgbouncer.ini for the given node.
#[cfg(feature = "diff")]
#[typetag::serde]
pub trait Expression: ExpressionClone + Any + Debug + Diffable {
    /// Renders this configuration node to its INI text representation.
    ///
    /// # Returns
    /// A `String` containing the text as it should appear in pgbouncer.ini.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    /// use pgbouncer_config::pgbouncer_config::Expression;
    /// let node = PgBouncerSetting::default();
    /// let text = node.expr();
    /// assert!(text.contains("[pgbouncer]"));
    /// ```
    fn expr(&self) -> String;

    /// Returns the name of the section corresponding to the struct's type.
    ///
    /// This method provides a default implementation that uses the structure's type name
    /// (as obtained via `type_name::<Self>()`) to determine the section name.
    ///
    /// Specifically:
    /// - If the fully-qualified type name contains namespace information (e.g., modules),
    ///   the method extracts and returns only the final part of the path (the struct name).
    /// - If no "::" delimiters are found in the type name (unlikely in regular use), it
    ///   simply returns the entire type name as-is.
    ///
    /// # Example
    ///
    /// ```
    /// struct MyStruct;
    ///
    /// impl MyStruct {
    ///     fn section_name(&self) -> String {
    ///         // Default implementation
    ///         let full_path = std::any::type_name::<Self>();
    ///         if let Some(struct_name) = full_path.split("::").last() {
    ///             struct_name.to_string()
    ///         } else {
    ///             full_path.to_string()
    ///         }
    ///     }
    /// }
    ///
    /// let instance = MyStruct;
    /// assert_eq!(instance.section_name(), "MyStruct".to_string());
    /// ```
    ///
    /// # Returns
    /// - A `String` containing the extracted name of the struct.
    fn section_name(&self) -> &'static str {
        let type_id = TypeId::of::<Self>();
        let mut cache_data = EXPRESSION_DEFAULT_SECTION_NAME.lock().unwrap();
        cache_data
            .entry(type_id)
            .or_insert_with(|| {
                let full_path = type_name::<Self>();
                let section_name = if let Some(struct_name) = full_path.split("::").last() {
                    struct_name
                } else {
                    full_path
                };
                let kebab_section_name = section_name.to_kebab_case();
                Box::leak(kebab_section_name.into_boxed_str())
            })
    }
}

/// Renderable configuration node.
///
/// Types implementing `Expression` can render themselves to the textual form
/// used by PgBouncer configuration files or sections. The return value is the
/// exact text that would appear in pgbouncer.ini for the given node.
#[cfg(not(feature = "diff"))]
#[typetag::serde]
pub trait Expression: ExpressionClone + Any + Debug {
    /// Renders this configuration node to its INI text representation.
    ///
    /// # Returns
    /// A `String` containing the text as it should appear in pgbouncer.ini.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    /// use pgbouncer_config::pgbouncer_config::Expression;
    /// let node = PgBouncerSetting::default();
    /// let text = node.expr();
    /// assert!(text.contains("[pgbouncer]"));
    /// ```
    fn expr(&self) -> String;

    /// Returns the name of the section corresponding to the struct's type.
    ///
    /// This method provides a default implementation that uses the structure's type name
    /// (as obtained via `type_name::<Self>()`) to determine the section name.
    ///
    /// Specifically:
    /// - If the fully-qualified type name contains namespace information (e.g., modules),
    ///   the method extracts and returns only the final part of the path (the struct name).
    /// - If no "::" delimiters are found in the type name (unlikely in regular use), it
    ///   simply returns the entire type name as-is.
    ///
    /// # Example
    ///
    /// ```
    /// struct MyStruct;
    ///
    /// impl MyStruct {
    ///     fn section_name(&self) -> String {
    ///         // Default implementation
    ///         let full_path = std::any::type_name::<Self>();
    ///         if let Some(struct_name) = full_path.split("::").last() {
    ///             struct_name.to_string()
    ///         } else {
    ///             full_path.to_string()
    ///         }
    ///     }
    /// }
    ///
    /// let instance = MyStruct;
    /// assert_eq!(instance.section_name(), "MyStruct".to_string());
    /// ```
    ///
    /// # Returns
    /// - A `String` containing the extracted name of the struct.
    fn section_name(&self) -> &'static str {
        let type_id = TypeId::of::<Self>();
        let mut cache_data = EXPRESSION_DEFAULT_SECTION_NAME.lock().unwrap();
        cache_data
            .entry(type_id)
            .or_insert_with(|| {
                let full_path = type_name::<Self>();
                let section_name = if let Some(struct_name) = full_path.split("::").last() {
                    struct_name
                } else {
                    full_path
                };
                let kebab_section_name = section_name.to_kebab_case();
                Box::leak(kebab_section_name.into_boxed_str())
            })
    }
}

/// Helper trait to enable cloning of trait objects (`Box<dyn Expression>`).
///
/// This trait is implemented for all `T: Expression + Clone + 'static` and is
/// used to provide `Clone` for `Box<dyn Expression>`.
pub trait ExpressionClone {
    /// Clones the current node and returns it as a boxed trait object.
    ///
    /// # Returns
    /// A boxed clone of `self` as `Box<dyn Expression>`.
    fn clone_box(&self) -> Box<dyn Expression>;
}

impl <T> ExpressionClone for T
where
    T: Expression + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Expression> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// High-level container for PgBouncer configuration sections.
///
/// `PgBouncerConfig` aggregates individual sections (such as
/// [`pgbouncer_setting::PgBouncerSetting`] and
/// [`databases_setting::DatabasesSetting`]) that implement [`Expression`], and
/// can render them into the INI format expected by PgBouncer.
///
/// # Fields
/// - settings: Internal list of configuration nodes in render order.
///
/// # Examples
/// Parse a minimal INI configuration and render it back:
/// ```rust
/// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
/// use pgbouncer_config::pgbouncer_config::databases_setting::DatabasesSetting;
/// use pgbouncer_config::builder::PgBouncerConfigBuilder;
///
/// let pgbouncer_setting = PgBouncerSetting::default();
/// let database_setting = DatabasesSetting::new();
/// let cfg = PgBouncerConfigBuilder::new(pgbouncer_setting, database_setting)
///     .unwrap()
///     .build();
///
/// let out = cfg.to_string();
/// assert!(out.contains("[pgbouncer]"));
/// // [databases] section is added by default
/// assert!(out.contains("[databases]"));
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PgBouncerConfig {
    #[serde(flatten)]
    pub(crate) settings: BTreeMap<String, Box<dyn Expression>>,
}

impl PgBouncerConfig {
    pub(crate) fn new() -> Self {
        Self {
            settings: BTreeMap::new(),
        }
    }

    /// Retrieves a typed reference to a contained configuration section.
    ///
    /// Use this to access a concrete section stored inside `PgBouncerConfig`,
    /// such as `pgbouncer_setting::PgBouncerSetting` or
    /// `databases_setting::DatabasesSetting`.
    ///
    /// # Parameters
    /// - T: Concrete type to retrieve from the internal settings list. Typically
    ///   one of the section types provided by this crate.
    ///
    /// # Returns
    /// A shared reference to the first stored section of type `T`.
    ///
    /// # Errors
    /// Returns an error if no section of type `T` is found.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::databases_setting::DatabasesSetting;
    /// use pgbouncer_config::pgbouncer_config::PgBouncerConfig;
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    /// use pgbouncer_config::pgbouncer_config::Expression;
    ///
    /// let pgbouncer_setting = PgBouncerSetting::default();
    /// let database_setting = DatabasesSetting::new();
    /// let cfg = PgBouncerConfigBuilder::new(pgbouncer_setting, database_setting)
    ///     .unwrap()
    ///     .build();
    /// let pgb: &PgBouncerSetting = cfg.get_config::<PgBouncerSetting>().unwrap();
    /// // Access via the Expression trait for demonstration
    /// assert!(pgb.expr().contains("[pgbouncer]"));
    /// ```
    ///
    /// # Notes
    /// - If multiple nodes of the same type are present, the first match is returned.
    /// - The returned reference is borrowed from `self`; standard borrowing rules apply.
    pub fn get_config<T: Any>(&self) -> crate::error::Result<&T> {
        for config in self.settings.values() {
            let down_cast = (config.as_ref() as &dyn Any).downcast_ref::<T>();
            match down_cast {
                Some(any_config) => return Ok(any_config),
                None => ()
            }
        }
        Err(PgBouncerError::PgBouncer("failed to get config".to_string()))
    }

    /// Retrieves a mutable typed reference to a contained configuration section.
    ///
    /// This allows modifying an existing section in place, such as
    /// `pgbouncer_setting::PgBouncerSetting` or `databases_setting::DatabasesSetting`.
    ///
    /// # Parameters
    /// - T: Concrete type to retrieve mutably from the internal settings list.
    ///
    /// # Returns
    /// An exclusive reference to the first stored section of type `T`.
    ///
    /// # Errors
    /// Returns an error if no section of type `T` is found.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::PgBouncerConfig;
    /// use pgbouncer_config::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;
    /// use pgbouncer_config::pgbouncer_config::databases_setting::DatabasesSetting;
    ///
    /// let pgbouncer_setting = PgBouncerSetting::default();
    /// let database_setting = DatabasesSetting::new();
    /// let mut cfg = PgBouncerConfigBuilder::new(pgbouncer_setting, database_setting)
    ///     .unwrap()
    ///     .build();
    /// {
    ///     let pgb: &mut PgBouncerSetting = cfg.get_config_mut::<PgBouncerSetting>().unwrap();
    ///     // Update a setting in place
    ///     pgb.set_listen_port(6433);
    /// }
    /// let text = cfg.to_string();
    /// assert!(text.contains("listen_port = 6433"));
    /// ```
    ///
    /// # Notes
    /// - If multiple nodes of the same type are present, the first match is returned.
    /// - The returned reference is a unique mutable borrow; you cannot hold any
    ///   other borrow of `self` at the same time.
    pub fn get_config_mut<T: Any>(&mut self) -> crate::error::Result<&mut T> {
        for config in self.settings.values_mut() {
            let down_cast_mut = (config.as_mut() as &mut dyn Any).downcast_mut::<T>();
            match down_cast_mut {
                Some(any_config) => return Ok(any_config),
                None => ()
            }
        }
        Err(PgBouncerError::PgBouncer("failed to get config".to_string()))
    }

    pub(crate) fn add_config<C: Expression + 'static>(&mut self, config: C) -> crate::error::Result<()> {
        if self.settings.contains_key(config.section_name()) {
            return Err(PgBouncerError::PgBouncer(format!("section {} already exists", config.section_name())));
        }
        self.settings.insert(config.section_name().to_string(), config.clone_box());

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.settings.len()
    }
}

impl Display for PgBouncerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr())
    }
}

impl Index<&str> for PgBouncerConfig {
    type Output = Box<dyn Expression>;

    fn index(&self, index: &str) -> &Self::Output {
        &self.settings[index]
    }
}

impl IndexMut<&str> for PgBouncerConfig {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.settings.get_mut(index).expect("index not found")
    }
}

impl From<&[&dyn Expression]> for PgBouncerConfig
{
    fn from(value: &[&dyn Expression]) -> Self {
        let configs = value
            .iter()
            .map(|config| (config.section_name().to_string(), config.clone_box()))
            .collect::<BTreeMap<String, Box<dyn Expression>>>();
        
        Self {
            settings: configs,
        }
    }
}

impl <C> From<&[C]> for PgBouncerConfig
where
    C: Expression + 'static,
{
    fn from(value: &[C]) -> Self {
        let configs = value
            .into_iter()
            .map(|config| (config.section_name().to_string(), config.clone_box()))
            .collect::<BTreeMap<String, Box<dyn Expression>>>();

        Self {
            settings: configs,
        }
    }
}


#[typetag::serde]
impl Expression for PgBouncerConfig {
    fn expr(&self) -> String {
        self.settings
            .iter()
            .map(|(_, config)| config.expr())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn section_name(&self) -> &'static str {
        "pgbouncer-config"
    }
}

#[cfg(feature = "io")]
impl ParserIniFromStr for PgBouncerConfig {
    type Error = PgBouncerError;

    fn parse_from_str(value: &str) -> Result<Self, Self::Error> {
        let section_re = Regex::new(r"(?m)^\[([^]\r\n]+)]\s*$")?;

        let mut headers = Vec::new();
        for caps in section_re.captures_iter(value) {
            let m = caps.get(0)
                .ok_or(PgBouncerError::PgBouncer("failed to parse section header".to_string()))?;
            let section_name = caps.get(1)
                .ok_or(PgBouncerError::PgBouncer("failed to parse section header".to_string()))?
                .as_str()
                .to_string();
            headers.push((section_name, m.start(), m.end()));
        }

        let mut sections = BTreeMap::new();
        for (i, (name, _hstart, hend)) in headers.iter().enumerate() {
            let body_start = *hend;
            let body_end = if let Some((_, next_hstart, _)) = headers.get(i + 1) {
                *next_hstart
            } else {
                value.len()
            };

            let mut body_lines = Vec::new();
            for line in value[body_start..body_end].lines() {
                if is_comment(line) || line.trim().is_empty() {
                    continue;
                }

                body_lines.push(line.to_string());
            }

            while body_lines.first().map_or(false, |l| l.trim().is_empty()) {
                body_lines.remove(0);
            }
            while body_lines.last().map_or(false, |l| l.trim().is_empty()) {
                body_lines.pop();
            }

            let body = body_lines.join("\n");

            sections.insert(name.to_string(), body);
        }

        let database_setting = if let Some(section_value) = sections.get("databases") {
            DatabasesSetting::parse_from_str(section_value)?
        } else {
            DatabasesSetting::new()
        };

        let pgbouncer_setting = if let Some(section_value) = sections.get("pgbouncer") {
            PgBouncerSetting::parse_from_str(section_value)?
        } else {
            PgBouncerSetting::default()
        };

        let mut pgbouncer_config = PgBouncerConfig::new();
        pgbouncer_config.add_config(pgbouncer_setting)?;
        pgbouncer_config.add_config(database_setting)?;

        Ok(pgbouncer_config)
    }
}

#[cfg(feature = "diff")]
#[typetag::serde]
impl Diffable for PgBouncerConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Serialize, Deserialize, Debug)]
    struct Dummy;

    #[derive(Clone, Serialize, Deserialize, Debug)]
    struct Dummy2;

    #[typetag::serde]
    impl Expression for Dummy {
        fn expr(&self) -> String { "dummy".to_string() }

        fn section_name(&self) -> &'static str {
            "dummy"
        }
    }

    #[cfg(feature = "diff")]
    #[typetag::serde]
    impl Diffable for Dummy {}

    #[typetag::serde]
    impl Expression for Dummy2 {
        fn expr(&self) -> String { "dummy2".to_string() }

        fn section_name(&self) -> &'static str {
            "dummy2"
        }
    }

    #[cfg(feature = "diff")]
    #[typetag::serde]
    impl Diffable for Dummy2 {}

    #[cfg(feature = "io")]
    fn minimal_pgbouncer_section() -> String {
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
    fn new_add_len_index_and_display() {
        let mut cfg = PgBouncerConfig::new();
        assert_eq!(cfg.len(), 0);
        cfg.add_config(Dummy).unwrap();
        assert_eq!(cfg.len(), 1);
        // Index and display via expr
        assert_eq!(cfg[&Dummy.section_name()].expr(), "dummy");
        assert_eq!(format!("{}", cfg), "dummy");

        // IndexMut allows modifying element in place
        cfg[&Dummy.section_name()] = Box::new(Dummy);
        assert_eq!(cfg[&Dummy.section_name()].expr(), "dummy");
    }

    #[test]
    fn from_same_slice_builds_config() {
        let arr = [Dummy, Dummy];
        let cfg = PgBouncerConfig::from(&arr[..]);
        // Duplicate section names are not allowed; constructing from a slice
        // with duplicates will deduplicate by section name.
        assert_eq!(cfg.len(), 1);
        assert!(cfg.expr().contains("dummy"));
    }

    #[test]
    fn from_dyn_slice_builds_config() {
        let arr: [&dyn Expression; 2] = [&Dummy, &Dummy2];
        let cfg = PgBouncerConfig::from(&arr[..]);
        assert_eq!(cfg.len(), 2);
        assert!(cfg.expr().contains("dummy"));
        assert!(cfg.expr().contains("dummy2"));
    }

    #[cfg(feature = "io")]
    #[test]
    fn parse_from_str_requires_pgbouncer_section() {
        let ini = "\
[databases]\n\
app = dbname=app host=127.0.0.1 port=5432\n\
";
        match PgBouncerConfig::parse_from_str(ini) {
            Ok(_) => (),
            Err(e) => panic!("error occurred: {}", e),
        };
    }

    #[cfg(feature = "io")]
    #[test]
    fn parse_from_str_minimal_pgbouncer_ok() {
        let ini = minimal_pgbouncer_section();
        let cfg = PgBouncerConfig::parse_from_str(&ini).expect("parse ok");
        let text = cfg.expr();
        assert!(text.contains("[pgbouncer]"));
        // [databases] may be empty/default but section from DatabasesSetting exists
        assert!(text.contains("[databases]"));
    }

    #[cfg(feature = "io")]
    #[test]
    fn parse_from_str_with_databases_and_comments() {
        let ini = "\
# a comment before section\n\
[pgbouncer]\n\
; inline comment line\n\
listen_addr = 127.0.0.1\n\
listen_port = 6432\n\
auth_type = md5\n\
max_client_conn = 100\n\
default_pool_size = 20\n\
pool_mode = session\n\
\n\
[databases]\n\
; ignore this line\n\
app = dbname=app host=127.0.0.1 port=5432\n\
";
        let cfg = PgBouncerConfig::parse_from_str(ini).expect("parse ok");
        let text = cfg.to_string();
        assert!(text.contains("[pgbouncer]"));
        assert!(text.contains("[databases]"));
        assert!(text.contains("dbname=app"));
        // Ensure comment markers are not present in rendered output
        assert!(!text.contains("# a comment"));
        assert!(!text.contains("; inline"));
    }
}
