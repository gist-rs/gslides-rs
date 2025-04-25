use crate::diff::formatting::{generate_git_diff, generate_readable_summary};
use crate::diff::structured::{Change, ChangeCollector};
use crate::Presentation;
use serde_json::Value as JsonValue;
use treediff::diff;

use super::error::DiffError;

/// Builder for creating a `Comparer`.
/// Sets the initial "base" presentation for comparison.
#[derive(Default)]
pub struct ComparerBuilder {
    base: Option<Presentation>,
    is_simplify: bool,
}

impl ComparerBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the base presentation to compare against.
    pub fn set_base(mut self, base: Presentation) -> Self {
        self.base = Some(base);
        self
    }

    // Set simplified result or not.
    pub fn set_simplify(mut self, is_simplify: bool) -> Self {
        self.is_simplify = is_simplify;
        self
    }

    /// Builds the `Comparer`.
    /// Returns an error if the base presentation was not set.
    pub fn build(self) -> Result<Comparer, DiffError> {
        let base = self
            .base
            .ok_or_else(|| DiffError::InvalidPath("Base presentation not set".to_string()))?; // Corrected error message
        Ok(Comparer {
            base,
            // Pass the flag from the builder to the Comparer
            is_simplify: self.is_simplify,
        })
    }
}

/// Compares presentations against a stored base.
pub struct Comparer {
    base: Presentation,
    is_simplify: bool,
}

impl Comparer {
    /// Compares the stored base presentation against a new presentation.
    ///
    /// Returns a `ComparisonResult` containing the structured diff.
    pub fn compare(&self, other: &Presentation) -> Result<ComparisonResult, DiffError> {
        // Convert Presentation structs to serde_json::Value for treediff
        let base_val: JsonValue = serde_json::to_value(&self.base)?;
        let other_val: JsonValue = serde_json::to_value(other)?;

        // Perform the diff using the ChangeCollector delegate
        let mut collector = ChangeCollector::new();
        diff(&base_val, &other_val, &mut collector);

        Ok(ComparisonResult {
            base: self.base.clone(),
            compared: other.clone(),
            changes: collector.changes,
            is_simplify: self.is_simplify,
        })
    }
}

/// Holds the results of a comparison between two presentations.
pub struct ComparisonResult {
    base: Presentation,
    compared: Presentation,
    changes: Vec<Change>,
    is_simplify: bool,
}

impl ComparisonResult {
    /// Returns the structured list of changes found.
    pub fn get_structured_diff(&self) -> &[Change] {
        &self.changes
    }

    /// Generates and returns a Git-style text diff.
    pub fn get_git_diff(&self) -> Result<String, DiffError> {
        generate_git_diff(&self.base, &self.compared, &self.changes)
    }

    /// Generates and returns a human-readable summary of the differences.
    pub fn get_readable_diff(&self) -> Result<String, DiffError> {
        generate_readable_summary(
            &self.base,     // Pass old presentation
            &self.compared, // Pass new presentation
            &self.changes,
            self.is_simplify,
        )
    }
}
