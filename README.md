# ChromoeDB

`chromoe-db` is an open-source, flexible, and scalable ecosystem designed for Rust-compatible database drivers. This library facilitates easy access, storage, and updating of data. Currently, all data is persistently stored using various supported databases, with **SQLite** being the only one available at this time.

## Installation

```sh
cargo add chromoe-db
```

## Examples

```rs
use chromoe_db::driver::sqlite_driver::SQLiteDriver;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct WorldData {
    time: String,
    money: i32,
}

fn main() {
    let driver = SQLiteDriver::new(None).expect("Failed to initialise SQLite driver");

    driver.set("name", "Reina").expect("Failed to set value");
    driver.set("world", Some( WorldData { time: "Day".to_string(), money: 15000 })).expect("Failed to set value");

    let world_value: Option<Value> = driver.get("world").expect("Failed to get value");
    println!("world: {:?}", world_value);

    driver.push("cart", vec!["Weapon A".to_string(), "Weapon B".to_string()]).expect("Failed to push values");
}
```

## License

This library is licensed under [MIT](https://github.com/reinacchi/chromoe-db/blob/master/LICENSE).