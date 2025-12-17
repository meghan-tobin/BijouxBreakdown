use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write, Seek, SeekFrom};
use std::convert::From;
use regex::Regex;
use std::fs::OpenOptions;

// - Supply Configuration
//     - Start with an empty configuration/json
//     - Create json entries from user input
//     - Have save functionaility which will lock the json
// - Item Creation
//     - Automattically grab configuration
//     - Have basic functionaility to add an item based on user input
//     - Result should calulate accurate results (one calc function run multiple times)

struct SupplyConfiguration {
    name: String,
    entries: Vec<Item>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum UnitOfMeasurment {
    Yards,
    Cms,
    Feet,
    Inches,
    NotApplicable,
    Unsupported
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    cost: f32,
    quanity: i32,
    unit: UnitOfMeasurment
}

fn parse_cost(cost_input: &str) -> Result<f32, String> {
    cost_input.trim().parse::<f32>().map_err(|e| format!("Failed to parse cost: {}", e))
}

fn parse_quantity(quantity_input: &str) -> Result<i32, String> {
    quantity_input.trim().parse::<i32>().map_err(|e| format!("Failed to parse quantity: {}", e))
}

fn determine_unit(unit_input: &str) -> Result<UnitOfMeasurment, String> {
    let unit = match unit_input.to_lowercase().as_str() {
        "yards" => UnitOfMeasurment::Yards,
        "cms" => UnitOfMeasurment::Cms,
        "feet" => UnitOfMeasurment::Feet,
        "in" => UnitOfMeasurment::Inches,
        "n/a" => UnitOfMeasurment::NotApplicable,
        _ => return Err(format!("unsupported unit: {}", unit_input))
    };

    Ok(unit)
}

fn create_item_and_validate(name_input: &str, cost_input: &str, quantity_input: &str, unit_input: &str) ->  Result<Item, String> {
    let converted_cost = parse_cost(cost_input)?;
    let converted_quantity = parse_quantity(quantity_input)?;
    let unit = determine_unit(unit_input)?;

   Ok(
        Item {name: name_input.to_string(),
        cost: converted_cost,
        quanity: converted_quantity,
        unit}
    )
}

fn validate_user_input(name_input: &str, cost_input: &str, quantity_input: &str, unit_input: &str) -> Result<bool, String> {
    let converted_cost = parse_cost(cost_input)?;
    let converted_quantity = parse_quantity(quantity_input)?;
    let unit = determine_unit(unit_input)?;

    if converted_cost < 0.0 {
        return Err("Cost cannot be negative.".to_string());
    } else if converted_quantity < 0 {
        return Err("Quantity cannot be negative.".to_string());
    }

    Ok(true)
}

fn listener() {
    // infinate loop 
    // waiting for an "add" to start which triggers create_configuration_json
    // "entry" triggers create_json_entry()
    // write prompts for gathering user input
    // for now just have them enter { name: <>, cost: <>, quanity: <>, unit: <>}

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        
        println!("Starting configuration");
        let item_parts: Vec<&str> = input.trim().split(',').map(|s| s.trim()).collect();

        if item_parts.len() != 4 {
            println!("Please provide exactly four values.");
            continue;
        }

        match validate_user_input(item_parts[0], item_parts[1], item_parts[2], item_parts[3]) {
            Ok(valid) => {
                if valid {
                    match create_item_and_validate(item_parts[0], item_parts[1], item_parts[2], item_parts[3]) {
                        Ok(item) => {
                            create_json_entry(item.clone(), "supplies2"); 
                            println!("Item created: {:?}", item);
                        },
                        Err(e) => println!("Error creating item: {}", e)
                    }
                }
            },
            Err(e) => println!("Validation error: {}", e),
        }
    }
}

fn create_json_entry(item: Item, config_name: &str) {  // Change this to accept Item, not a reference
    // //let file_path = "/Users/meghan/BijouxBreakdown/breakdown/src/supplies.json"; // this needs to be the name of a configuration
    // let mut file = create_json_file(config_name);
    // // // Try to open the file or create it if it doesn't exist
    // // create_json_file(String::from("supplies2"));
    // let file_reader = file.try_clone().expect("Failed to clone file for reading");

    // // Attempt to read existing items from the file
    // let mut items: Vec<Item> = match serde_json::from_reader(file_reader) {
    //     Ok(existing_items) => existing_items,
    //     Err(_) => Vec::new(), // If there's an error, start with an empty vector
    // };

    // items.push(item);  // Push the owned Item directly

    // // Move the file cursor back to the start before writing
    // file.set_len(0).expect("Failed to truncate file"); // Clear the file
    // file.seek(SeekFrom::Start(0)).expect("Failed to seek to start"); // Seek to the beginning

    // // Write the updated items back to the file
    // // let file = File::create(file_path).expect("Failed to create file");
    // //let mut file = create_json_file(file_name);
    // serde_json::to_writer_pretty(file, &items).expect("Failed to write to file");

    let file_path = format!("/Users/meghan/BijouxBreakdown/breakdown/src/{}.json", config_name);

    // Attempt to read existing items from the file
    let mut items: Vec<Item> = match File::open(&file_path) {
        Ok(file_reader) => {
            serde_json::from_reader(file_reader).unwrap_or_else(|_| Vec::new())
        }
        Err(_) => Vec::new(), // If the file doesn't exist, start with an empty vector
    };

    // Add the new item to the list
    items.push(item);

    // Reopen the file for writing, truncating it first
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true) // Create the file if it doesn't exist
        .open(&file_path)
        .expect("Failed to open file for writing");

    // Write the updated items back to the file
    serde_json::to_writer_pretty(&mut file, &items).expect("Failed to write to file");
}

fn lock_json() {
    // lock the json file
}

fn read_json() {
    // read the json and create user input based off of it
}

fn main() {
    listener();
}