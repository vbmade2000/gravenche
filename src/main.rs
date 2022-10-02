use std::{env, io::Error, path::PathBuf, process};

/// Processes command line arguments. Exits the process with code 1 if argument count is less than 2.
fn get_command_line_args() -> Vec<String> {
    // Process command line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("ERROR: Please provide csv filename");
        process::exit(1);
    }
    args
}

/// Generates absolute path for supplied csv filename. Also, checks if filename exists.
fn get_csv_path(filename: &str) -> Result<PathBuf, Error> {
    std::path::Path::new(filename).canonicalize()
}

fn main() {
    // Process command line args
    let args = get_command_line_args();

    // Get absolute path of CSV filename.
    let csv_filepath = get_csv_path(&args[1]).unwrap();

    println!("{:?}", csv_filepath);
}
