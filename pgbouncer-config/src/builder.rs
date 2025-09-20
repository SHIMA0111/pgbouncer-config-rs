use crate::error::PgBouncerError;
use crate::pgbouncer_config::{Expression, PgBouncerConfig};
use crate::pgbouncer_config::databases_setting::DatabasesSetting;
use crate::pgbouncer_config::pgbouncer_setting::PgBouncerSetting;

/// Fluent builder for assembling a [`PgBouncerConfig`].
///
/// Provides a convenient, step-by-step API to set or replace the
/// `[pgbouncer]` and `[databases]` sections and to append any additional
/// [`Expression`] nodes in a controlled order. The final configuration can be
/// obtained with [`PgBouncerConfigBuilder::build`].
///
/// # Examples
/// ```rust
/// use pgbouncer_config::builder::PgBouncerConfigBuilder;
/// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
///
/// let cfg = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new())
///     .unwrap()
///     .build();
/// assert!(cfg.to_string().contains("[pgbouncer]"));
/// ```
#[derive(Clone)]
pub struct PgBouncerConfigBuilder {
    config: PgBouncerConfig,
    pgbouncer_setting: bool,
    databases_setting: bool,
}

impl PgBouncerConfigBuilder {
    /// Constructs a builder from explicit `[pgbouncer]` and `[databases]` sections.
    ///
    /// # Parameters
    /// - pgbouncer_setting: The `[pgbouncer]` section to insert at index 0.
    /// - databases_setting: The `[databases]` section to insert at index 1.
    ///
    /// # Returns
    /// A new `Builder` pre-populated with both sections.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let b = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap();
    /// let cfg = b.build();
    /// assert!(cfg.to_string().contains("[pgbouncer]"));
    /// ```
    pub fn new(
        pgbouncer_setting: PgBouncerSetting,
        databases_setting: DatabasesSetting,
    ) -> crate::error::Result<Self> {
        let mut config = PgBouncerConfig::new();
        config.add_config(pgbouncer_setting)?;
        config.add_config(databases_setting)?;
        Ok(Self {
            config,
            // since we seeded both sections above, mark them as set
            pgbouncer_setting: true,
            databases_setting: true,
        })
    }

    /// Starts with an empty builder (no sections set yet).
    ///
    /// This is useful when you prefer to set sections incrementally.
    ///
    /// # Returns
    /// A new, empty `Builder`.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// let _b = PgBouncerConfigBuilder::builder();
    /// ```
    pub fn builder() -> Self {
        let config = PgBouncerConfig::new();
        Self {
            config,
            pgbouncer_setting: false,
            databases_setting: false,
        }
    }

    /// Sets the `[pgbouncer]` section once.
    ///
    /// # Parameters
    /// - pgbouncer_setting: Section to append to the configuration.
    ///
    /// # Returns
    /// A mutable reference to `self` for chaining.
    ///
    /// # Errors
    /// Returns an error if the `[pgbouncer]` section has already been set.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let mut b = PgBouncerConfigBuilder::builder();
    /// b.set_pgbouncer_setting(PgBouncerSetting::default()).unwrap();
    /// b.set_databases_setting(DatabasesSetting::new()).unwrap();
    /// let cfg = b.build();
    /// assert!(cfg.to_string().contains("[pgbouncer]"));
    /// ```
    pub fn set_pgbouncer_setting(&mut self, pgbouncer_setting: PgBouncerSetting) -> crate::error::Result<&mut Self> {
        if self.pgbouncer_setting {
            return Err(PgBouncerError::PgBouncer(
                "Cannot set pgbouncer-config setting twice".to_string()
            ))
        }
        self.pgbouncer_setting = true;
        self.add_config(pgbouncer_setting)?;
        Ok(self)
    }

    /// Sets the `[databases]` section once.
    ///
    /// # Parameters
    /// - databases_setting: Section to append to the configuration.
    ///
    /// # Returns
    /// A mutable reference to `self` for chaining.
    ///
    /// # Errors
    /// Returns an error if the `[databases]` section has already been set.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let mut b = PgBouncerConfigBuilder::builder();
    /// b.set_pgbouncer_setting(PgBouncerSetting::default()).unwrap();
    /// b.set_databases_setting(DatabasesSetting::new()).unwrap();
    /// let cfg = b.build();
    /// assert!(cfg.to_string().contains("[databases]"));
    /// ```
    pub fn set_databases_setting(&mut self, databases_setting: DatabasesSetting) -> crate::error::Result<&mut Self> {
        if self.databases_setting {
            return Err(PgBouncerError::PgBouncer(
                "Cannot set databases setting twice".to_string()
            ))
        }

        self.databases_setting = true;
        self.add_config(databases_setting)?;
        Ok(self)
    }

    /// Replaces the previously set `[pgbouncer]` section in place.
    ///
    /// # Parameters
    /// - pgbouncer_setting: New section value to replace the existing one.
    ///
    /// # Returns
    /// A mutable reference to `self` for chaining.
    ///
    /// # Errors
    /// Returns an error if `[pgbouncer]` has not been set yet.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let mut b = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap();
    /// b.replace_pgbouncer_setting(PgBouncerSetting::default()).unwrap();
    /// let _ = b.build();
    /// ```
    pub fn replace_pgbouncer_setting(&mut self, pgbouncer_setting: PgBouncerSetting) -> crate::error::Result<&mut Self> {
        if !self.pgbouncer_setting {
            return Err(PgBouncerError::PgBouncer(
                "Cannot replace pgbouncer-config setting before pgbouncer-config setting is set".to_string()
            ))
        }
        let section_name = pgbouncer_setting.section_name();
        self.config[&section_name] = Box::new(pgbouncer_setting);
        Ok(self)
    }

    /// Replaces the previously set `[databases]` section in place.
    ///
    /// # Parameters
    /// - databases_setting: New section value to replace the existing one.
    ///
    /// # Returns
    /// A mutable reference to `self` for chaining.
    ///
    /// # Errors
    /// Returns an error if `[databases]` has not been set yet.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let mut b = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap();
    /// b.replace_databases_setting(DatabasesSetting::new()).unwrap();
    /// let _ = b.build();
    /// ```
    pub fn replace_databases_setting(&mut self, databases_setting: DatabasesSetting) -> crate::error::Result<&mut Self> {
        if !self.databases_setting {
            return Err(PgBouncerError::PgBouncer(
                "Cannot replace databases setting before databases setting is set".to_string()
            ))
        }
        let section_name = databases_setting.section_name();
        self.config[&section_name] = Box::new(databases_setting);
        Ok(self)
    }

    /// Appends an additional configuration node implementing [`Expression`].
    ///
    /// # Parameters
    /// - config: Any configuration node to append.
    ///
    /// # Returns
    /// A mutable reference to `self` for chaining.
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Serialize, Deserialize};
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::Expression;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// #[cfg(feature = "diff")]
    /// use pgbouncer_config::utils::diff::Diffable;
    ///
    /// #[derive(Serialize, Deserialize, Clone, Debug)]
    /// struct Dummy;
    ///
    /// #[typetag::serde]
    /// impl Expression for Dummy {
    ///     fn expr(&self) -> String {
    ///         "dummy".to_string()
    ///     }
    /// }
    ///
    /// #[cfg(feature = "diff")]
    /// #[typetag::serde]
    /// impl Diffable for Dummy {}
    ///
    /// fn main() {
    ///     let mut b = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap();
    ///     b.add_config(Dummy).unwrap();
    ///     let conf = b.build();
    ///
    ///     assert!(conf.expr().contains("dummy"));
    ///     let dummy_ref = conf.get_config::<Dummy>().unwrap();
    ///     assert_eq!(dummy_ref.section_name(), "dummy");
    /// }
    /// ```
    pub fn add_config<C: Expression + 'static>(&mut self, config: C) -> crate::error::Result<&mut Self> {
        self.config.add_config(config)?;
        Ok(self)
    }

    /// Finalizes and returns the built configuration.
    ///
    /// # Returns
    /// A cloned `PgBouncerConfig` containing all accumulated sections.
    ///
    /// # Examples
    /// ```rust
    /// use pgbouncer_config::builder::PgBouncerConfigBuilder;
    /// use pgbouncer_config::pgbouncer_config::{pgbouncer_setting::PgBouncerSetting, databases_setting::DatabasesSetting};
    /// let cfg = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap().build();
    /// assert!(cfg.to_string().contains("[pgbouncer]"));
    /// ```
    pub fn build(&self) -> PgBouncerConfig {
        self.config.clone()
    }
}

#[test]
fn test_builder() {
    // Start with explicit sections via new(); replace should work since flags are set
    let mut builder = PgBouncerConfigBuilder::new(PgBouncerSetting::default(), DatabasesSetting::new()).unwrap();
    builder.replace_pgbouncer_setting(PgBouncerSetting::default()).unwrap();
    builder.replace_databases_setting(DatabasesSetting::new()).unwrap();

    // Adding duplicate sections should now be rejected
    assert!(builder.add_config(PgBouncerSetting::default()).is_err());
    assert!(builder.add_config(DatabasesSetting::new()).is_err());

    let config = builder.build();

    // Only two unique sections should exist
    assert_eq!(config.len(), 2);
    assert!(config[&PgBouncerSetting::default().section_name()].expr().contains("pgbouncer"));
    assert!(config[&DatabasesSetting::new().section_name()].expr().contains("databases"));
}