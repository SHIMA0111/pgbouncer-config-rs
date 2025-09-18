//! Diff utilities for comparing serializable values.
//!
//! This module provides a lightweight, tree-structured diff built on top of
//! `serde_json::Value`. It detects additions, removals, and changes across
//! objects (maps), arrays (by index), and scalar values.

use std::collections::BTreeMap;
use serde::Serialize;

/// Structured diff between two serializable values.
///
/// The diff is produced by converting both inputs to `serde_json::Value` and
/// comparing recursively:
/// - Objects: The union of keys is examined; per-key diffs are computed. Missing
///   keys become `Added` or `Removed` entries.
/// - Arrays: Compared by index up to the maximum length; only differing indices
///   are included.
/// - Scalars: If equal → `Same`; otherwise `Changed { old, new }` with JSON-serialized
///   strings for the values.
///
/// # Notes
/// - String values are JSON-serialized; expect surrounding quotes in `old`/`new`
///   (e.g., `"foo"`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum Diff {
    /// Both sides are equal (no difference).
    Same,
    /// A scalar value or element changed.
    ///
    /// The `old` and `new` fields contain JSON-serialized representations
    /// (via `serde_json::Value::to_string()`).
    ///
    /// # Fields
    /// - old: Previous value as a JSON string.
    /// - new: New value as a JSON string.
    Changed { old: String, new: String },
    /// A value was added on the right-hand side.
    ///
    /// # Fields
    /// - new: Added value as a JSON string.
    Added { new: String },
    /// A value was removed from the right-hand side.
    ///
    /// # Fields
    /// - old: Removed value as a JSON string.
    Removed { old: String },
    /// Difference of an object (map-like value).
    ///
    /// # Fields
    /// - fields: Map of field name to nested diff. If empty, the result would
    ///   be `Same` instead of `Object`.
    Object {
        fields: BTreeMap<String, Diff>,
    },
    /// Difference of an array (list-like value).
    ///
    /// # Fields
    /// - items: List of `(index, Diff)` entries for differing indices only.
    Array {
        items: Vec<(usize, Diff)>
    },
}

/// Computes a structured diff between two serializable values.
///
/// # Parameters
/// - old: Previous value to compare.
/// - new: New value to compare.
///
/// # Returns
/// A [`Diff`] describing how `new` differs from `old`. If identical, returns
/// [`Diff::Same`].
///
/// # Errors
/// Returns an error if serialization via `serde_json::to_value` fails for either
/// input. This is uncommon for standard types but can occur with custom `Serialize`
/// implementations.
///
/// # Examples
/// ```rust
/// use pgbouncer_config::utils::diff::{compute_diff, Diff};
/// let d = compute_diff(&1, &2).unwrap();
/// assert!(matches!(d, Diff::Changed { .. }));
///
/// let d2 = compute_diff(&"a", &"a").unwrap();
/// assert!(matches!(d2, Diff::Same));
/// ```
///
/// A map/object example with added and removed keys:
/// ```rust
/// use pgbouncer_config::utils::diff::{compute_diff, Diff};
/// use std::collections::BTreeMap;
/// let mut old = BTreeMap::new();
/// old.insert("a".to_string(), 1);
/// let mut new = BTreeMap::new();
/// new.insert("b".to_string(), 1);
/// let d = compute_diff(&old, &new).unwrap();
/// if let Diff::Object { fields } = d {
///     assert!(matches!(fields.get("a"), Some(Diff::Removed { .. })));
///     assert!(matches!(fields.get("b"), Some(Diff::Added { .. })));
/// } else { panic!("expected object diff"); }
/// ```
pub fn compute_diff<T: Serialize>(old: &T, new: &T) -> crate::error::Result<Diff> {
    let old_v = serde_json::to_value(old)?;
    let new_v = serde_json::to_value(new)?;
    Ok(diff_value(&old_v, &new_v))
}

fn diff_value(old: &serde_json::Value, new: &serde_json::Value) -> Diff {
    match (old, new) {
        (serde_json::Value::Object(old), serde_json::Value::Object(new)) => {
            let mut keys: BTreeMap<String, ()> = BTreeMap::new();
            for k in old.keys() {
                keys.insert(k.clone(), ());
            }
            for k in new.keys() {
                keys.insert(k.clone(), ());
            }

            let mut fields = BTreeMap::new();
            for k in keys.keys() {
                match (old.get(k), new.get(k)) {
                    (Some(old), Some(new)) => {
                        fields.insert(k.clone(), diff_value(old, new));
                    },
                    (Some(old), None) => {
                        fields.insert(k.clone(), Diff::Removed { old: old.to_string() });
                    },
                    (None, Some(new)) => {
                        fields.insert(k.clone(), Diff::Added { new: new.to_string() });
                    },
                    (None, None) => {},
                }
            }
            if fields.is_empty() {
                Diff::Same
            } else {
                Diff::Object { fields }
            }
        },
        (serde_json::Value::Array(old), serde_json::Value::Array(new)) => {
            let len = old.len().max(new.len());
            let mut items = Vec::new();
            for i in 0..len {
                match (old.get(i), new.get(i)) {
                    (Some(old_val), Some(new_val)) => {
                        let d = diff_value(old_val, new_val);
                        if !matches!(d, Diff::Same) {
                            items.push((i, d));
                        }
                    },
                    (Some(old_val), None) => {
                        items.push((i, Diff::Removed { old: old_val.to_string() }));
                    },
                    (None, Some(new_val)) => {
                        items.push((i, Diff::Added { new: new_val.to_string() }));
                    },
                    (None, None) => {},
                }
            }
            if items.is_empty() {
                Diff::Same
            } else {
                Diff::Array { items }
            }
        },
        _ => {
            if old == new {
                Diff::Same
            } else {
                Diff::Changed { old: old.to_string(), new: new.to_string() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn scalar_same_and_changed() {
        let a = 1i32;
        let b = 1i32;
        let c = 2i32;

        let d_ab = compute_diff(&a, &b).expect("ok");
        assert_eq!(d_ab, Diff::Same);

        let d_ac = compute_diff(&a, &c).expect("ok");
        assert_eq!(d_ac, Diff::Changed { old: "1".to_string(), new: "2".to_string() });

        let s1 = String::from("foo");
        let s2 = String::from("bar");
        let d_s = compute_diff(&s1, &s2).expect("ok");
        // serde_json::Value::String displays with quotes
        assert_eq!(d_s, Diff::Changed { old: "\"foo\"".to_string(), new: "\"bar\"".to_string() });
    }

    #[test]
    fn object_added_removed_and_changed() {
        let mut old: HashMap<String, i32> = HashMap::new();
        old.insert("a".into(), 1);
        old.insert("x".into(), 10);

        let mut new: HashMap<String, i32> = HashMap::new();
        new.insert("b".into(), 1);
        new.insert("x".into(), 11);

        let d = compute_diff(&old, &new).expect("ok");
        let fields = match d {
            Diff::Object { fields } => fields,
            other => panic!("expected object diff, got {:?}", other),
        };

        assert_eq!(fields.get("a"), Some(&Diff::Removed { old: "1".to_string() }));
        assert_eq!(fields.get("b"), Some(&Diff::Added { new: "1".to_string() }));
        assert_eq!(fields.get("x"), Some(&Diff::Changed { old: "10".to_string(), new: "11".to_string() }));
    }

    #[test]
    fn array_index_differences() {
        let old = vec![1, 2, 3];
        let new = vec![1, 4];

        let d = compute_diff(&old, &new).expect("ok");
        let items = match d {
            Diff::Array { items } => items,
            other => panic!("expected array diff, got {:?}", other),
        };

        // Expect a change at index 1 and a removal at index 2
        assert!(items.contains(&(1, Diff::Changed { old: "2".to_string(), new: "4".to_string() })));
        assert!(items.contains(&(2, Diff::Removed { old: "3".to_string() })));
        // Index 0 should not be present (same)
        assert!(!items.iter().any(|(idx, _)| *idx == 0));
    }

    #[test]
    fn nested_object_and_array_diff() {
        let old = serde_json::json!({
            "a": {"x": 1},
            "b": [1, 2]
        });
        let new = serde_json::json!({
            "a": {"x": 2},
            "b": [1, 2, 3]
        });

        let d = compute_diff(&old, &new).expect("ok");
        let fields = match d {
            Diff::Object { fields } => fields,
            other => panic!("expected object diff, got {:?}", other),
        };

        // a.x changed
        match fields.get("a") {
            Some(Diff::Object { fields: a_fields }) => {
                assert_eq!(a_fields.get("x"), Some(&Diff::Changed { old: "1".to_string(), new: "2".to_string() }));
            }
            other => panic!("expected nested object diff for 'a', got {:?}", other),
        }

        // b has an added element at index 2
        match fields.get("b") {
            Some(Diff::Array { items }) => {
                assert!(items.contains(&(2usize, Diff::Added { new: "3".to_string() })));
            }
            other => panic!("expected array diff for 'b', got {:?}", other),
        }
    }

    #[test]
    fn arrays_equal_are_same() {
        let v1 = vec!["a", "b"];
        let v2 = vec!["a", "b"];
        let d = compute_diff(&v1, &v2).expect("ok");
        assert_eq!(d, Diff::Same);
    }
}
