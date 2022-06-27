use term_data_table::IntoRow;

#[derive(Debug, IntoRow)]
struct MyData<T: std::fmt::Display> {
    name: String,
    age: u16,
    extra: T,
}

impl MyData<String> {
    fn new(name: &str, age: u16) -> Self {
        Self {
            name: name.into(),
            age,
            extra: "some message".into(),
        }
    }
}

fn main() {
    let data = vec![MyData::new("John Doe", 28), MyData::new("Jane Foo", 72)];

    term_data_table::data_table(&data)
}
