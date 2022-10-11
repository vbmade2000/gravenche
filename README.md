### Gravenche
A toy payment transaction processor

### Why Gravenche?
Gravenche is an presumably extinct freshwater fish from France and Lake Geneva in Switzerland. The purpose of using this name is to spread awareness about extinct species.

### What does it do?
Gravenche accepts a CSV containing financial transactions and processes them.

### Compile and run application
```
$ git clone https://github.com/vbmade2000/gravenche.git
$ cd gravenche
$ cargo build --release # Integraion tests requires release binary present.
$ cargo run -- test_data.csv
```

### Run tests
Tests include few unit tests and a one integrated test.
```
$ git clone https://github.com/vbmade2000/gravenche.git
$ cd gravenche
$ cargo test # sample test csv is already included. It must be present in current directory for some tests to pass.
```

### Correctness of application.
The application is tested manually and automatically with some sample data. It also contains unit tests for some internal operations as well as integration test to verify that binary works as expected. Integration tests are located in **gravenche/tests** directory. We could have used Serde to deserialize csv record directly into some structure but that would make application somehow slow. We mostly ignore errors and ignore faulty transactions to continue the process. Ideally all the faulty transactions must be logged/tracked in a separate structure to be dealt with later.

### Safety and Robustness
No unsafe constructs are used. Error handling is done using [anyhow](https://docs.rs/anyhow/latest/anyhow/) crate. Mostly errors are ignored for processing to be continued.
