use serde::Serialize;
use term_data_table::Table;

#[derive(Serialize)]
struct MyData {
    name: String,
    address: Vec<String>,
    age: u16,
}

impl MyData {
    fn new(name: impl Into<String>, address: &str, age: u16) -> Self {
        Self {
            name: name.into(),
            address: address.split(",").map(|s| s.trim().to_string()).collect(),
            age,
        }
    }
}

fn main() {
    let data = vec![
        MyData::new("john doe", "3 the close, CA", 15),
        MyData::new("jane smith", "3 the close, CA", 27),
        MyData::new("alexander jones", "3 the close, CA", 54),
    ];

    println!("{}", Table::from_serde(&data).unwrap().for_terminal());
}
