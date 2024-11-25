use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataSet {
    pub id: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct SQLiteDriverOptions {
    pub file_name: String,
    pub table_name: String,
}
