use std::{
    env,
    fs::File,
    io::{BufReader, Error},
    path::PathBuf,
    process,
};

mod client;

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

/// Read CSV file
fn read_csv(csv_filepath: &PathBuf) {
    let file = File::open(csv_filepath).unwrap();

    // Use of BufReader prevents
    let buf_reader = BufReader::new(file);

    // We re-use this store record.
    let mut record = csv::StringRecord::new();

    let mut csv_reader = csv::Reader::from_reader(buf_reader);

    // Using an existing variable to store a record prevents memory allocation every time.
    while csv_reader.read_record(&mut record).unwrap() {
        println!("-> {:?}", &record);
    }
}

fn main() {
    // Process command line args
    let args = get_command_line_args();

    // Get absolute path of CSV filename.
    let csv_filepath = get_csv_path(&args[1]).unwrap();

    // println!("{:?}", csv_filepath);

    read_csv(&csv_filepath);
}
