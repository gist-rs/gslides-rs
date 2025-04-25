use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use treediff::{value::Key, Delegate};

/// Represents a simplified view of a value involved in a change.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValueRepr {
    String(String),
    Number(serde_json::Number), // Keep numeric precision
    Boolean(bool),
    Null,
    // Summaries for complex types
    Array(String),  // e.g., "[Array len=5]"
    Object(String), // e.g., "{Object}"
}

impl ValueRepr {
    /// Helper to convert treediff's JsonValue to our ValueRepr. Summarizes complex types.
    fn from_json_value(val: &JsonValue) -> Self {
        match val {
            JsonValue::Null => ValueRepr::Null,
            JsonValue::Bool(b) => ValueRepr::Boolean(*b),
            JsonValue::Number(n) => ValueRepr::Number(n.clone()),
            JsonValue::String(s) => ValueRepr::String(s.clone()),
            JsonValue::Array(arr) => ValueRepr::Array(format!("[Array len={}]", arr.len())),
            JsonValue::Object(_map) => ValueRepr::Object("{Object}".to_string()), // Simple object summary
        }
    }

    /// Formats the ValueRepr for display.
    pub fn format_for_display(&self) -> String {
        match self {
            ValueRepr::String(s) => {
                let escaped_s = s
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
                    // NOTE: No need to escape single quote ' anymore if not using them for wrapping
                    // .replace('\'', "\\'");
                    // Also escape backticks if they appear in the string itself
                    .replace('`', "\\`");

                // Truncate long strings for readability in diffs
                if escaped_s.len() > 60 {
                    format!("{}...", &escaped_s[..57])
                } else {
                    escaped_s // Keep the original escaped string
                }
            }
            ValueRepr::Number(n) => n.to_string(),
            ValueRepr::Boolean(b) => b.to_string(),
            ValueRepr::Null => "null".to_string(),
            ValueRepr::Array(s) | ValueRepr::Object(s) => s.clone(),
        }
    }
}

/// The type of difference detected.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

/// Represents a single difference found between two structures.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Change {
    pub path: String,
    pub change_type: ChangeType,
    pub old_value: Option<ValueRepr>, // Uses generic ValueRepr
    pub new_value: Option<ValueRepr>, // Uses generic ValueRepr
}

/// treediff Delegate implementation to collect changes into `Vec<Change>`.
#[derive(Debug)]
pub(crate) struct ChangeCollector {
    pub(crate) changes: Vec<Change>,
    current_path: Vec<String>, // Stack to build the path string
}

impl ChangeCollector {
    pub(crate) fn new() -> Self {
        ChangeCollector {
            changes: Vec::new(),
            current_path: Vec::new(),
        }
    }
    fn format_path(&self) -> String {
        self.current_path.join("")
    }
}

impl<'a> Delegate<'a, Key, JsonValue> for ChangeCollector {
    fn push(&mut self, segment: &Key) {
        let segment_str = match segment {
            Key::String(s) => {
                if self.current_path.is_empty() {
                    s.clone()
                } else {
                    format!(".{}", s)
                }
            }
            Key::Index(i) => format!("[{}]", i),
        };
        self.current_path.push(segment_str);
    }

    fn pop(&mut self) {
        self.current_path.pop();
    }

    fn removed<'b>(&mut self, _key: &'b Key, value: &'a JsonValue) {
        let path = self.format_path();
        self.changes.push(Change {
            path,
            change_type: ChangeType::Removed,
            old_value: Some(ValueRepr::from_json_value(value)), // Use generic
            new_value: None,
        });
    }

    fn added<'b>(&mut self, _key: &'b Key, value: &'a JsonValue) {
        let path = self.format_path();
        self.changes.push(Change {
            path,
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(ValueRepr::from_json_value(value)), // Use generic
        });
    }

    fn modified(&mut self, old: &'a JsonValue, new: &'a JsonValue) {
        let path = self.format_path();
        let old_repr = ValueRepr::from_json_value(old); // Use generic
        let new_repr = ValueRepr::from_json_value(new); // Use generic

        // Only record if values actually differ after representation
        if old_repr != new_repr {
            self.changes.push(Change {
                path,
                change_type: ChangeType::Modified,
                old_value: Some(old_repr),
                new_value: Some(new_repr),
            });
        }
    }
}
