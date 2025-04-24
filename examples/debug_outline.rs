// examples/debug_outline.rs

// Use the structs from your library
use gslides_rs::models::shape_properties::{Outline, PropertyState}; // Import necessary items

fn main() {
    // The exact JSON value for the "outline" key from your error file (line 84-86)
    let json_str = r#"
        {
            "propertyState": "INHERIT"
        }
    "#;

    println!("Attempting to deserialize JSON into Outline struct:");
    println!("JSON: {}", json_str);

    // Attempt to deserialize directly into the Outline struct
    let result: Result<Outline, _> = serde_json::from_str(json_str);

    match result {
        Ok(outline_struct) => {
            println!("\nSuccessfully deserialized into Outline:");
            println!("{:#?}", outline_struct); // Pretty-print the resulting struct

            // Check if the propertyState field was correctly parsed
            assert_eq!(outline_struct.property_state, Some(PropertyState::Inherit));
            println!("\nAssertion passed: propertyState is Inherit.");
        }
        Err(e) => {
            println!("\nFailed to deserialize directly into Outline struct:");
            println!("Error: {}", e);
        }
    }

    println!("\nAttempting to deserialize JSON into Option<Outline>:");
    // Attempt to deserialize directly into Option<Outline> (should also work)
    let result_option: Result<Option<Outline>, _> = serde_json::from_str(json_str);
    match result_option {
        Ok(outline_option) => {
            println!("\nSuccessfully deserialized into Option<Outline>:");
            println!("{:#?}", outline_option);
        }
        Err(e) => {
            println!("\nFailed to deserialize directly into Option<Outline>:");
            println!("Error: {}", e);
        }
    }
}
