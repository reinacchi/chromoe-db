use rusqlite::{params, Connection, Error as RusqliteError, OptionalExtension, Result};
use serde::Serialize;
use serde_json::{from_str, to_string, Error as SerdeJsonError, Value};

pub use crate::structure::DataSet;
pub use crate::structure::SQLiteDriverOptions;

#[derive(Debug)]
pub struct SQLiteDriver {
    pub name: String,
    pub options: SQLiteDriverOptions,
    pub table: String,
    pub database: Connection,
}

impl SQLiteDriver {
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

    pub fn delete_all(&self) -> Result<bool> {
        self.delete_rows()
    }

    fn delete_row_key(&self, key: &str) -> Result<bool> {
        self.database
            .prepare(&format!("DELETE FROM {} WHERE ID = ?", self.table))?
            .execute(params![key])?;
        Ok(true)
    }

    fn delete_rows(&self) -> Result<bool> {
        self.database
            .prepare(&format!("DELETE FROM {}", self.table))?
            .execute([])?;
        Ok(true)
    }

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

    pub fn has(&self, key: &str) -> Result<bool> {
        Ok(self.get::<Value>(key)?.is_some())
    }

    pub fn pull<T>(&self, key: &str, value: T) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + std::cmp::PartialEq + Clone + Serialize,
    {
        let mut arr: Vec<T> = self.get(key)?.unwrap_or_default();

        arr.retain(|x| x != &value);

        self.set(key, arr.clone())?;

        Ok(arr)
    }

    pub fn push<T>(&self, key: &str, value: T) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned + Clone + Serialize,
    {
        let mut arr: Vec<T> = self.get(key)?.unwrap_or_default();

        arr.push(value);

        self.set(key, arr.clone())?;

        Ok(arr)
    }

    pub fn set<T>(&self, key: &str, value: T) -> Result<T>
    where
        T: Serialize,
    {
        let json = to_string(&value).map_err(|e: SerdeJsonError| {
            RusqliteError::ToSqlConversionFailure(Box::new(e)) // Only pass the boxed error, no second argument needed
        })?;

        let data_exists = self.has(key)?;

        if data_exists {
            self.database
                .prepare(&format!("UPDATE {} SET JSON = ? WHERE ID = ?", self.table))?
                .execute(params![json, key])?;
        } else {
            self.database
                .prepare(&format!(
                    "INSERT INTO {} (ID, JSON) VALUES (?, ?)",
                    self.table
                ))?
                .execute(params![key, json])?;
        }

        Ok(value)
    }

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
