use serde::{Serialize, Deserialize};

/// Represents a data entry in a dataset, typically used for storing and retrieving
/// structured data in a database.
///
/// This struct is used to represent an individual row or object in the dataset, where
/// each entry is identified by a unique `id` and can hold a dynamic value in the form of
/// a JSON object. The `value` field allows for flexible storage of various types of data
/// using the `serde_json::Value` type, which can represent different JSON structures like
/// strings, numbers, arrays, objects, or nulls.
///
/// # Fields
///
/// - `id`: A unique identifier for the dataset entry. This is typically used as a primary key
///   in database operations, ensuring that each entry can be retrieved, updated, or deleted
///   based on its unique identifier. The ID is represented as a `String`, which can be a UUID
///   or any other suitable format for identifying records.
///
/// - `value`: The actual data associated with this entry. This is stored as a `serde_json::Value`
///   type, which is a flexible and powerful representation of any valid JSON data. This allows the
///   driver to store various data types in the database without enforcing a rigid schema. The data
///   stored here can be a string, a number, an array, an object, or any other valid JSON structure.
///
/// # Example Usage
///
/// ```rust
/// use serde_json::{json, Value};
///
/// let data = DataSet {
///     id: "12345".to_string(),
///     value: json!({"key": "value", "age": 30}),
/// };
/// ```
///
/// In this example, `data.id` is the unique identifier `"12345"`, and `data.value` is a JSON object
/// containing a string and a number.
#[derive(Debug, Serialize, Deserialize)]
pub struct DataSet {
    /// Unique identifier for this data entry in the dataset.
    pub id: String,

    /// A flexible container for any JSON-compatible value.
    /// This can store a wide range of data structures such as strings, numbers, arrays, objects, etc.
    pub value: serde_json::Value,
}


/// Configuration options for the SQLite database driver.
///
/// This struct holds configuration options specific to the SQLite database driver.
/// It contains settings for the SQLite database file and the table name that the driver
/// will use for interactions. These options are passed when initializing the database driver
/// to configure the connection and operations on the SQLite database.
///
/// # Fields
///
/// - `file_name`: The path to the SQLite database file. This is the file on the filesystem
///   where the SQLite database is stored. If the file doesn't exist, it may be created
///   depending on the database driver's behavior or settings. The `file_name` is represented
///   as a `String`, allowing flexibility in specifying file paths or database names.
///
/// - `table_name`: The name of the table in the SQLite database that the driver will operate on.
///   This allows specifying which table to query or manipulate during database interactions.
///   The `table_name` is a `String` and should correspond to the actual table in the database.
///
/// # Example Usage
///
/// ```rust
/// let options = SQLiteDriverOptions {
///     file_name: "json.sqlite".to_string(),
///     table_name: "users".to_string(),
/// };
/// ```
///
/// In this example, the SQLite database is located in the file `"json.sqlite"`, and the
/// driver will interact with the `"users"` table within that database.
#[derive(Debug, Clone)]
pub struct SQLiteDriverOptions {
    /// Path to the SQLite database file.
    /// This file contains the SQLite database that the driver will connect to.
    pub file_name: String,

    /// Name of the table to operate on within the SQLite database.
    /// This should match an existing table in the database.
    pub table_name: String,
}
