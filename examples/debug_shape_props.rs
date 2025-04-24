// examples/debug_shape_props.rs

// Use the structs from your library
use gslides_rs::models::shape_properties::ShapeProperties; // Import the specific struct
use serde_json;

fn main() {
    // The JSON object for "shapeProperties" from your deserialization_error.json (lines 80-93)
    let json_str = r#"
        {
          "shapeBackgroundFill": {
            "propertyState": "INHERIT"
          },
          "outline": {
            "propertyState": "INHERIT"
          },
          "shadow": {
            "propertyState": "INHERIT"
          },
          "autofit": {
            "fontScale": 1
          }
        }
    "#;

    println!("Attempting to deserialize JSON into ShapeProperties struct:");
    println!("JSON: {}", json_str);

    // Attempt to deserialize directly into the ShapeProperties struct
    let result: Result<ShapeProperties, _> = serde_json::from_str(json_str);

    match result {
        Ok(shape_props) => {
            println!("\nSuccessfully deserialized into ShapeProperties:");
            println!("{:#?}", shape_props); // Pretty-print the resulting struct
        }
        Err(e) => {
            println!("\nFailed to deserialize directly into ShapeProperties struct:");
            println!("Error: {}", e); // If this fails, the problem is within ShapeProperties or its direct dependencies
        }
    }
}
