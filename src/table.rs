use std::fmt::Debug;
use crate::enum_utils::*;
use std::fmt;
use std::cmp::PartialEq;
use serde::{Deserialize, Serialize};
use crate::select::*;

#[derive(Debug,PartialEq,Serialize,Deserialize,Clone)]
pub enum DataTypes
{
    Int,
    Float,
    Text,
    Key(Key)
}
#[derive(Debug,PartialEq,Serialize,Deserialize,Clone)]
pub enum Key
{
    IntAutoIncrement,
    Int,
}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub enum Value {
    Int(i32),
    Float(f64),
    Text(String),
    Key(i64),
}
impl PartialEq for Value
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Key(a), Value::Key(b)) => a == b,
            _ => false,
        }
    }
}
impl Value {
    pub fn get_int(&self) -> Option<i32>
    {
        match self
        {
            Value::Int(value) => Some(*value),
            _ => None,
        }
    }
    pub fn get_float(&self) -> Option<f64>
    {
        match self
        {
            Value::Float(value) => Some(*value),
            _ => None,
        }
    }
    pub fn get_text(self) -> Option<String>
    {
        match self
        {
            Value::Text(value) => Some(value),
            _ => None,
        }
    }
    pub fn get_key(&self) -> Option<i64>
    {
        match self
        {
            Value::Key(value) => Some(*value),
            _ => None,
        }
    }
    pub  fn compare_values(statement: &Statement, row_value: &Value, selector: &Selector) -> bool
     {
        return  match statement
        {
            Statement::Equals => {*row_value == selector.value},
            Statement::Bigger=>
                {
                    match (row_value, &selector.value)
                    {
                        (Value::Int(r), Value::Int(s)) => r > s,
                        (Value::Float(r), Value::Float(s)) => r > s,
                        (Value::Text(r), Value::Text(s)) => r > s,
                        (Value::Key(r), Value::Key(s)) => r > s,
                        _ => false,
                    }
                },
            Statement::Smaller=>
                {
                    match (row_value, &selector.value)
                    {
                        (Value::Int(r), Value::Int(s)) => r < s,
                        (Value::Float(r), Value::Float(s)) => r < s,
                        (Value::Text(r), Value::Text(s)) => r < s,
                        (Value::Key(r), Value::Key(s)) => r < s,
                        _ => false,
                    }
                },
        }
    }
}
impl fmt::Display for Value
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(value) => write!(f, "Int({})", value),
            Value::Float(value) => write!(f, "Float({})", value),
            Value::Text(value) => write!(f, "Text(\"{}\")", value),
            Value::Key(value) => write!(f, "Key({})", value),
        }
    }
}
    #[derive(Debug,Serialize,Deserialize)]
    pub struct Row
    {
        pub data: Vec<Value>
    }
    impl Row
    {
        pub fn new(values: Vec<Value>) -> Self
        {
            Row
            {
                data: values
            }
        }
        pub fn len(&self) -> usize
        {
            self.data.len()
        }
    }
    impl Clone for Row
    {
        fn clone(&self) -> Self
        {
            Row::new(self.data.clone())
        }
    }
#[derive(Serialize, Deserialize,Clone)]
    pub struct Table
    {
        pub table_name: String,
        pub columns: Vec<(String, DataTypes)>,
        pub values: Vec<Row>,

    }
    impl Table
    {
        pub fn find_key_index_in_columns(&self) -> Option<usize>
        {
            self.columns.iter().position(|(_, data_type)| matches!(data_type, DataTypes::Key(Key::IntAutoIncrement)))
        }

        fn contains_key_auto_increment(&self) -> bool
        {
            self.columns.iter().any(|(_, data_type)| *data_type == DataTypes::Key(Key::IntAutoIncrement))
        }
        pub fn new(columns: Vec<(String, DataTypes)>, name: String) -> Result<Table, String>
        {
            if columns.is_empty()
            {
                return Err(String::from("there is no column"));
            }

            let has_primary_key = columns.iter().any(|(_, data_type)|
            matches!(data_type, DataTypes::Key(Key::IntAutoIncrement | Key::Int))
            );

            if has_primary_key
            {
                Ok(Table {
                    columns,
                    values: Vec::new(),
                    table_name: name,
                })
            }
            else
            {
                Err(String::from("there is no primary key"))
            }
        }
        fn validate_row(&self, row: &Row) -> Result<(), String>
        {
            if self.columns.len() > row.len()
            {
                return Err(String::from("there is to much "));
            }
            else if self.columns.len() < row.len()
            {
                return Err(String::from("the is missing something"));
            }
            else
            {
                for ((_, datatype), value) in self.columns.iter().zip(row.data.iter())
                {
                    if get_variant_name(datatype) != get_variant_name(value)
                    {
                        return Err(String::from("the types are wrong!"));
                    }
                    if get_variant_name(datatype) == "Key" && get_variant_name(value) == "Key"
                    {
                        continue;
                    }
                }
            }
            Ok(())
        }
        pub fn insert_row(&mut self, mut row: Row) -> Result<(), String>
        {
            match self.validate_row(&row)
            {
                Ok(_) =>
                    {
                        if self.contains_key_auto_increment()
                        {
                            let index = self.find_key_index_in_columns().unwrap();
                            let new_key_value = if self.values.is_empty()
                            {
                                1
                            }
                            else
                            {
                                if let Value::Key(last_key) = self.values.last().unwrap().data[index]
                                {
                                    last_key + 1
                                }
                                else
                                {
                                    return Err("Expected a Key value for auto-increment.".to_string());
                                }
                            };

                            row.data[index] = Value::Key(new_key_value);
                        }

                        self.values.push(row);
                    },
                Err(error) =>
                    {
                        return Err(error);
                    }
            }
            Ok(())
        }
        pub fn delete_row(&mut self, selectors: &[Selector]) -> Result<(), String>
        {
            let matching_indices: Vec<usize> = self.values
                .iter()
                .enumerate()
                .filter(|(_, row)| self.matches_all_selectors(row, selectors))
                .map(|(index, _)| index)
                .collect();

            if matching_indices.is_empty()
            {
                return Err(String::from("No row found matching the selectors"));
            }

            for row_index in matching_indices.into_iter().rev() {
                self.values.remove(row_index);
            }

            Ok(())
        }
        pub fn describe_table(&self) -> String
        {
            let mut table = String::new();
            for (name, data_type) in self.columns.iter()
            {
                let formated_str = format!("{} : {}\n", name, get_variant_name(&data_type));
                table += formated_str.as_str();
            }
            table
        }
        fn matches_all_selectors(&self, row: &Row, selectors: &[Selector]) -> bool
        {
            selectors.iter().all(|selector|
            {
                if let Some(col_idx) = self.columns.iter().position(|(col_name, _)| col_name == &selector.name)
                {
                    let row_value = &row.data[col_idx];
                    Value::compare_values(&selector.statement, row_value, &selector)
                }
                else
                {
                    false
                }
            })
        }
        pub fn update_table(&mut self,fields:Vec<String> ,values: Vec<Value>, selectors: &[Selector]) -> Result<(), String>
        {
            if fields.len() != values.len()
            {
                return Err(String::from("Fields and values length do not match"));
            }
            let matching_indices: Vec<usize> = self
                .values
                .iter()
                .enumerate()
                .filter(|(_, row)| self.matches_all_selectors(row, selectors))
                .map(|(index, _)| index)
                .collect();

            if matching_indices.is_empty()
            {
                return Err(String::from("No row found matching the selectors"));
            }
            for row_index in matching_indices
            {
                let row = &mut self.values[row_index];
                for (field, value) in fields.iter().zip(values.iter())
                {
                    if let Some(field_index) = self.columns.iter().position(|(col_name, _)| col_name == field)
                    {
                        row.data[field_index] = value.clone();
                    }
                    else
                    {
                        return Err(format!("Field '{}' not found in the table", field));
                    }
                }
            }
            Ok(())
        }
    }
pub  fn setup_test_table() -> Table
{
    let columns = vec![
        ("id".to_string(), DataTypes::Key(Key::IntAutoIncrement)),
        ("name".to_string(), DataTypes::Text),
        ("age".to_string(), DataTypes::Int),
        ("salary".to_string(), DataTypes::Float),
    ];
    let mut table = Table::new(columns, "users".to_string()).unwrap();

    table.insert_row(Row::new(vec![
        Value::Key(1),
        Value::Text("Alice".to_string()),
        Value::Int(30),
        Value::Float(50000.0),
    ])).unwrap();

    table.insert_row(Row::new(vec![
        Value::Key(2),
        Value::Text("Bob".to_string()),
        Value::Int(25),
        Value::Float(45000.0),
    ])).unwrap();

    table.insert_row(Row::new(vec![
        Value::Key(3),
        Value::Text("Charlie".to_string()),
        Value::Int(35),
        Value::Float(60000.0),
    ])).unwrap();
    table
}