mod process;

use std::io::{self, Write}; // for reading user input

fn main() {
    // STEP 1: Build and display the process tree
    let process_tree = process::tree::build_process_tree();
    println!("====================");
    println!("  Linux Process Tree");
    println!("====================");
    process::tree::print_tree(&process_tree, 1, 0);

    // STEP 2: Ask user which PID to kill
    println!("\nEnter the PID of the process to kill (or press Enter to skip):");

    // STEP 3: Read input
    let mut input = String::new();
    print!("> "); // prompt
    io::stdout().flush().unwrap(); // make sure prompt shows up before input
    io::stdin().read_line(&mut input).expect("Failed to read line");

    // STEP 4: Clean input and check if it's empty (skip if so)
    let trimmed = input.trim();
    if trimmed.is_empty() {
        println!("No PID entered. Exiting.");
        return;
    }

    // STEP 5: Parse PID and try to kill it
    match trimmed.parse::<u32>() {
        Ok(pid) => {
            if process::control::kill_process(pid) {
                println!("✅ Process {} was killed successfully.", pid);
            } else {
                println!("❌ Failed to kill process {}. Are you root? Is the PID valid?", pid);
            }
        }
        Err(_) => {
            println!("Invalid PID entered.");
        }
    }
}
