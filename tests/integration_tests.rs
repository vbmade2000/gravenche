use std::process::Command;
use std::str;

#[test]
fn test_execution() {
    let output = Command::new("./target/release/gravenche")
        .args(["test_data.csv"])
        .output()
        .expect("Failed to execute Gravenche.");

    let _stdout = output.stdout;
    let _stderr = output.stderr;
    assert_eq!(output.status.code(), Some(0));
    assert_ne!(_stdout.len(), 0);
    assert_eq!(_stderr.len(), 0);
}
