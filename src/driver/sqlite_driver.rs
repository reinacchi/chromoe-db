use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::Serialize;
use serde_json::{from_str, json, Error as SerdeJsonError, Value};

pub use crate::structure::DataSet;
pub use crate::structure::SQLiteDriverOptions;

/// SQLite database driver for storing and managing JSON data.
///
/// The `SQLiteDriver` provides methods for interacting with an SQLite database,
/// including adding, retrieving, updating, and deleting JSON data in a specified table.
///
/// It abstracts the database operations and allows the user to interact with
/// the database as if it were a key-value store, with the data being stored
/// as serialised JSON.
///
/// # Fields
///
/// - `name`: The name of the SQLite database file.
/// - `options`: Configuration options for the SQLite driver, including the
///   database file name and table name.
/// - `table`: The name of the table in the SQLite database to operate on.
/// - `database`: The connection to the SQLite database.
#[derive(Debug)]
pub struct SQLiteDriver {
    pub name: String,
    pub options: SQLiteDriverOptions,
    pub table: String,
    pub database: Connection,
}

impl SQLiteDriver {
    /// Creates a new instance of the `SQLiteDriver` with the provided options.
    /// If no options are provided, it defaults to using `json.sqlite` as the
    /// database file and `json` as the table name.
    ///
    /// # Parameters
    /// - `options`: Optional configuration options for the SQLite database.
    ///
    /// # Returns
    /// A `Result` containing either the `SQLiteDriver` instance or an error.
    pub fn new(options: Option<SQLiteDriverOptions>) -> Result<Self> {
        let options = options.unwrap_or_else(|| SQLiteDriverOptions {
            file_name: "json.sqlite".to_string(),
            table_name: "json".to_string(),
        });

        let database = Connection::open(&options.file_name)?;

        let driver = SQLiteDriver {
            name: options.file_name.clone(),
            options: options.clone(),
            table: options.table_name.clone(),
            database,
        };

        driver.prepare(&options.table_name)?;

        Ok(driver)
    }

    /// Prepares the SQLite database by creating the table if it doesn't already exist.
    ///
    /// # Parameters
    /// - `table`: The name of the table to create.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn prepare(&self, table: &str) -> Result<()> {
        self.database.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (ID TEXT PRIMARY KEY, JSON TEXT)",
                table
            ),
            [],
        )?;
        Ok(())
    }

    /// Adds a value to an existing entry or creates a new entry if it doesn't exist.
    /// The value is added to the current value of the entry (if it exists).
    ///
    /// # Parameters
    /// - `key`: The key for the entry to update.
    /// - `value`: The value to add to the current entry.
    ///
    /// # Returns
    /// The new value after adding `value` to the existing entry, or an error if
    /// the value is not finite (e.g., NaN or infinity).
    pub fn add(&self, key: &str, value: f64) -> Result<f64> {
        let current_value: f64 = self.get(key)?.unwrap_or(0.0);

        if !current_value.is_finite() {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Non-finite value"),
            )));
        }

        let new_value = current_value + value;
        self.set(key, new_value)?;
        Ok(new_value)
    }

    /// Retrieves all data entries from the database as a vector of tuples.
    ///
    /// # Returns
    /// A `Result` containing a vector of tuples where each tuple consists of
    /// a key (`String`) and a corresponding value (`serde_json::Value`).
    pub fn all(&self) -> Result<Vec<(String, Value)>> {
        let mut stmt = self
            .database
            .prepare(&format!("SELECT * FROM {}", self.table))?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let json_str: String = row.get(1)?;
            let json: Value = from_str(&json_str).unwrap_or(Value::Null);
            Ok((id, json))
        })?;

        let mut data = Vec::new();
        for row in rows {
            let (id, value) = row?;
            data.push((id, value));
        }

        Ok(data)
    }

    /// Deletes a specific entry by key. If the key refers to a nested value,
    /// it will remove the nested field within the JSON data.
    ///
    /// # Parameters
    /// - `key`: The key of the entry to delete.
    ///
    /// # Returns
    /// A `Result` indicating whether the deletion was successful.
    pub fn delete(&self, key: &str) -> Result<bool> {
        if key.contains('.') {
            let split: Vec<&str> = key.split('.').collect();
            let mut obj: Value = self.get(split[0])?.unwrap_or(Value::Null);
            obj.as_object_mut().map(|obj| obj.remove(split[1]));
            self.set(split[0], obj)?;
            return Ok(true);
        }

        self.delete_row_key(key)?;
        Ok(true)
    }

    /// Deletes all entries in the database.
    ///
    /// # Returns
    /// A `Result` indicating whether the deletion was successful.
    pub fn delete_all(&self) -> Result<bool> {
        self.delete_rows()
    }

    /// Deletes a specific row from the table by key.
    ///
    /// # Parameters
    /// - `key`: The key of the entry to delete.
    ///
    /// # Returns
    /// A `Result` indicating whether the deletion was successful.
    fn delete_row_key(&self, key: &str) -> Result<bool> {
        self.database
            .prepare(&format!("DELETE FROM {} WHERE ID = ?", self.table))?
            .execute(params![key])?;
        Ok(true)
    }

    /// Deletes all rows from the table.
    ///
    /// # Returns
    /// A `Result` indicating whether the deletion was successful.
    fn delete_rows(&self) -> Result<bool> {
        self.database
            .prepare(&format!("DELETE FROM {}", self.table))?
            .execute([])?;
        Ok(true)
    }

    /// Retrieves the value for a given key, potentially deserialising it into the specified type.
    ///
    /// # Parameters
    /// - `key`: The key of the entry to retrieve.
    ///
    /// # Returns
    /// A `Result` containing an `Option` of the deserialised value, or an error if the
    /// deserialisation fails.
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        if key.contains('.') {
            let split: Vec<&str> = key.split('.').collect();
            let val: Value = self.get_row_key(split[0])?.unwrap_or_default();
            let nested_value = val.pointer(&format!("/{}", split[1])).cloned();
            Ok(nested_value.map(|v| from_str(&v.to_string()).unwrap_or_default()))
        } else {
            self.get_row_key(key)
        }
    }

    /// Retrieves a value for a key, directly from the row.
    ///
    /// # Parameters
    /// - `key`: The key of the entry to retrieve.
    ///
    /// # Returns
    /// A `Result` containing the deserialised value, or `None` if the key doesn't exist.
    fn get_row_key<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut stmt = self
            .database
            .prepare(&format!("SELECT JSON FROM {} WHERE ID = ?", self.table))?;

        let row = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .optional()?;

        if let Some(json_str) = row {
            let json: Option<T> = from_str(&json_str).ok();
            Ok(json)
        } else {
            Ok(None)
        }
    }

    /// Checks if a given key exists in the database.
    ///
    /// # Parameters
    /// - `key`: The key to check.
    ///
    /// # Returns
    /// A `Result` containing a boolean indicating whether the key exists.
    pub fn has(&self, key: &str) -> Result<bool> {
        Ok(self.get::<Value>(key)?.is_some())
    }

    /// Removes a specific value from an array stored at the given key.
    ///
    /// # Parameters
    /// - `key`: The key of the entry where the array is stored.
    /// - `value`: The value to remove from the array.
    ///
    /// # Returns
    /// A `Result` containing the updated array after removal.
    pub fn pull<T>(&self, key: &str, value: T) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + std::cmp::PartialEq + Clone + Serialize,
    {
        let mut arr: Vec<T> = self.get(key)?.unwrap_or_default();

        arr.retain(|x| x != &value);

        self.set(key, arr.clone())?;

        Ok(arr)
    }

    /// Appends a value to an array stored at the given key.
    ///
    /// # Parameters
    /// - `key`: The key of the entry where the array is stored.
    /// - `value`: The value to append to the array.
    ///
    /// # Returns
    /// A `Result` containing the updated array after the value is appended.
    pub fn push<T>(&self, key: &str, value: T) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Clone + Serialize,
    {
        let mut arr: Vec<T> = self.get(key)?.unwrap_or_default();

        arr.push(value);

        self.set(key, arr.clone())?;

        Ok(arr)
    }

    /// Sets or updates the value for a given key in the database.
    ///
    /// # Parameters
    /// - `key`: The key for the entry.
    /// - `value`: The value to store, which will be serialised into JSON.
    ///
    /// # Returns
    /// A `Result` containing the value that was set.
    pub fn set<T>(&self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let parts: Vec<&str> = key.split('.').collect();
        let root_key = parts[0];

        let mut root_value: Value = self.get(root_key)?.unwrap_or_else(|| json!({}));

        let mut current = &mut root_value;
        for part in &parts[1..] {
            current = current
                .as_object_mut()
                .unwrap()
                .entry(part.to_string())
                .or_insert(json!({}));
        }
        *current = json!(value);

        let json_string = serde_json::to_string(&root_value).map_err(|e: SerdeJsonError| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
        self.database
            .prepare(&format!(
                "INSERT INTO {} (ID, JSON) VALUES (?, ?) ON CONFLICT(ID) DO UPDATE SET JSON = ?",
                self.table
            ))?
            .execute(params![root_key, json_string, json_string])?;

        Ok(())
    }

    /// Subtracts a value from an existing entry. If the entry does not exist,
    /// it initialises it with the result.
    ///
    /// # Parameters
    /// - `key`: The key of the entry to subtract from.
    /// - `value`: The value to subtract from the current value.
    ///
    /// # Returns
    /// The new value after subtraction.
    pub fn subtract(&self, key: &str, value: f64) -> Result<f64> {
        let current_value: f64 = self.get(key)?.unwrap_or(0.0);

        if !current_value.is_finite() {
            return Err(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Non-finite value"),
            )));
        }

        let new_value = current_value - value;
        self.set(key, new_value)?;
        Ok(new_value)
    }
}
