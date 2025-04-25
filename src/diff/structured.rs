use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use treediff::{value::Key, Delegate};

/// Represents a simplified view of a value involved in a change.
/// Based on Section 5 of the design document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValueRepr {
    String(String),
    Number(serde_json::Number), // Keep numeric precision
    Boolean(bool),
    Null,
    // Summaries for complex types
    Array(String), // e.g., "[Array len=5]"
    Object(String), // e.g., "{Object}"
                   // Add more specific types if needed (e.g., Color, TransformSummary)
}

impl ValueRepr {
    /// Helper to convert treediff's JsonValue to our ValueRepr.
    /// Summarizes complex types.
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
                    .replace('\\', "\\\\") // Must escape backslash first!
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
                    .replace('\'', "\\'"); // Escape single quote as we use it
                format!("'{}'", escaped_s)
            }
            ValueRepr::Number(n) => n.to_string(),
            ValueRepr::Boolean(b) => b.to_string(),
            ValueRepr::Null => "null".to_string(),
            ValueRepr::Array(s) | ValueRepr::Object(s) => s.clone(),
        }
    }
}

/// The type of difference detected.
/// Based on Section 5 of the design document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

/// Represents a single difference found between two structures.
/// Based on Section 5 of the design document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Change {
    /// Dot-separated path with bracket notation for array indices.
    /// Example: "slides[1].pageElements[0].shape.text.textElements[0].textRun.content"
    pub path: String,
    /// The type of change.
    pub change_type: ChangeType,
    /// The value before the change (None for Added).
    pub old_value: Option<ValueRepr>,
    /// The value after the change (None for Removed).
    pub new_value: Option<ValueRepr>,
}

/// treediff Delegate implementation to collect changes into `Vec<Change>`.
/// Based on Section 4 of the design document.
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

    /// Helper to format the current path stack into a string.
    /// Uses dot notation for fields and bracket notation for indices.
    fn format_path(&self) -> String {
        self.current_path.join("") // Segments now include their own separators (.) or brackets ([])
    }
}

// *** Implement Delegate with correct method names and lifetimes ***
// The Delegate trait requires a lifetime parameter 'a bound to the values being compared.
// The methods removed/added also take a lifetime 'b for the key.
impl<'a> Delegate<'a, Key, JsonValue> for ChangeCollector {
    // Use `push` instead of `push_path_segment`
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

    // Use `pop` instead of `pop_path_segment`
    fn pop(&mut self) {
        self.current_path.pop();
    }

    // Add lifetimes 'a and 'b as required by the trait
    fn removed<'b>(&mut self, _key: &'b Key, value: &'a JsonValue) {
        let path = self.format_path();
        self.changes.push(Change {
            path,
            change_type: ChangeType::Removed,
            old_value: Some(ValueRepr::from_json_value(value)),
            new_value: None,
        });
    }

    // Add lifetimes 'a and 'b as required by the trait
    fn added<'b>(&mut self, _key: &'b Key, value: &'a JsonValue) {
        let path = self.format_path();
        self.changes.push(Change {
            path,
            change_type: ChangeType::Added,
            old_value: None,
            new_value: Some(ValueRepr::from_json_value(value)),
        });
    }

    // Add lifetime 'a as required by the trait
    fn modified(&mut self, old: &'a JsonValue, new: &'a JsonValue) {
        let path = self.format_path();
        self.changes.push(Change {
            path,
            change_type: ChangeType::Modified,
            old_value: Some(ValueRepr::from_json_value(old)),
            new_value: Some(ValueRepr::from_json_value(new)),
        });
    }

    // Implement `unchanged` if needed, otherwise default is fine
    // fn unchanged(&mut self, _v: &'a JsonValue) {}
}
