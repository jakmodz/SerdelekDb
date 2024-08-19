use crate::table::{Row, Table, Value};
use serde::{Deserialize, Serialize};
use std::fs;
use rmp_serde::{decode, encode};
use crate::query::*;
use crate::select::{select_from_table, Selector, Statement};
#[derive(Serialize, Deserialize,Clone)]
pub struct  Db
{
    pub name:String,
    tables: Vec<Table>
}
impl Db
{
    pub fn new(name:String,path: &str)
    {
        let db = Self
        {
            tables: vec![],
            name
        };
        let final_path: String = path.to_owned() + "database.msgpack";
        let serialized = encode::to_vec(&db).unwrap();
        fs::write(final_path, serialized).unwrap();

    }
    pub fn add_table(&mut self,table:Table) -> Result<(),String>
    {
        for table_from_db  in self.tables.iter()
        {
            if table_from_db.table_name == table.table_name
            {
               return  Err(format!("there is table with this name: {}",table.table_name))
            }
        }
        self.tables.push(table);
        Ok(())
    }
    pub async fn open_database(path:&str)->Result<Db,String>
    {
        let content = fs::read(path);
        match content {
            Ok(content) => match decode::from_slice(&content)
            {
                Ok(db) => Ok(db),
                Err(error) => Err(error.to_string()),
            },
            Err(error) => Err(error.to_string()),
        }
    }
   pub async fn close_database(&mut self,path:&str) -> Result<(),String>
    {
        let serialized = encode::to_vec(&self).unwrap();
        return match fs::write(path,serialized)
        {
            Ok(_)=>
                {
                   Ok(())
                },
            Err(error)=>
                {
                    Err(error.to_string())
                }
        };
    }
    fn describe_table_by_name(&self, name:&String) -> String
    {

        for table in self.tables.iter()
        {
            if &table.table_name == name
            {
              return  table.describe_table();
            }
        }
        "There is no such table".to_string()
    }
    pub fn show_all_tables(&self) -> String
    {
        let mut tables = String::new();
        tables+= "Datbase name: ";
        tables+=&self.name;
        for table in self.tables.iter()
        {
            tables+="\n";
            tables+= &table.table_name;
        }
        tables
    }
    pub fn show_all_tables_describe(&self) -> String
    {
        let mut tables = String::new();
        tables+= "Datbase name: ";
        tables+=&self.name;
        for table in self.tables.iter()
        {
            tables+="\n";
            tables+= "tabel name: ";
            tables+= &table.table_name;
            tables+="\n";
            tables += &table.describe_table();

        }
        tables
    }
    pub fn delete_table(&mut self,name:String) -> Result<(),()>
    {
        for table in 0..self.tables.len()
        {
            if self.tables[table].table_name == name
            {
                self.tables.remove(table);
                return Ok(());
            }
        }
        Err(())
    }
    pub async fn execute_query(&mut self, query: String) -> Result<String, String>
    {
        let exprs = parse(tokenize(query));
        let mut current_table: Option<&mut Table> = None;
        let mut where_conditions: Vec<Selector> = Vec::new();
        let mut data_select: Option<Vec<String>> = None;
        let mut values: Option<Vec<Value>> = None;
        let mut update_fields: Option<Vec<String>> = None;
        let mut row: Option<Row> = None;
        for expr in exprs.clone()
        {
            match expr
            {
                Expr::Select(fields) =>
                {
                    data_select = Some(fields);
                }
                Expr::From(table_name) =>
                    {
                    current_table = self.tables.iter_mut().find(|t| t.table_name == table_name);
                    if current_table.is_none()
                    {
                        return Err(format!("Table '{}' not found.", table_name));
                    }
                }
                Expr::Where(field, op, value) =>
                    {
                    let selector = Selector {
                        name: field,
                        value,
                        statement: match op.as_str() {
                            "=" => Statement::Equals,
                            ">" => Statement::Bigger,
                            "<" => Statement::Smaller,
                            _ => return Err("Unsupported operator".to_string())
                        }
                    };
                    where_conditions.push(selector);
                }
                Expr::Update(table_name, fields, new_value) =>
                {
                    current_table = self.tables.iter_mut().find(|t| t.table_name == table_name);
                    if current_table.is_none()
                    {
                        return Err(format!("Table '{}' not found.", table_name));
                    }
                    update_fields = Some(fields);
                    values = Some(new_value);

                }
                Expr::Delete(table_name) =>
                    {

                        current_table = self.tables.iter_mut().find(|t| t.table_name == table_name);
                        if current_table.is_none() {
                            return Err(format!("Table '{}' not found.", table_name));
                        }

                }
                Expr::Drop(table_name)=>
                    {
                        return  match self.delete_table(table_name.clone())
                        {
                            Ok(_) => Ok("Table was deleted properly".to_string()),
                            Err(_) => Err("There was a problem deleting the table".to_string())
                        };
                    }
                Expr::Describe(table_name)=>
                    {
                        if table_name.to_lowercase() == "*"
                        {
                            return Ok(self.show_all_tables_describe());
                        }
                       return  Ok(self.describe_table_by_name(&table_name))
                    }
                Expr::Create(name,columns,data_types)=>
                    {
                        if  columns.len() != data_types.len()
                        {
                            return Err("the are too much columns or fields".to_string());
                        }
                        else if name.len() == 0
                        {
                            return Err("the are too much columns or fields".to_string());
                        }
                        let vec  = columns.into_iter().zip(data_types.into_iter()).collect();
                        let to_remove = '"';
                        return match Table::new(vec,name.chars().filter(|&c| c != to_remove ).collect() )
                        {
                            Ok(table) =>
                                {
                                   match self.add_table(table)
                                   {
                                       Ok(_)=>Ok("Table was added succesfully".to_string()),
                                       Err(error)=>Err(error)
                                   }
                                },
                            Err(error) =>
                                {
                                    Err(error)
                                }
                        }
                    }
                Expr::Insert(table_name,values)=>
                    {
                        current_table = self.tables.iter_mut().find(|t| t.table_name == table_name);
                        row = Some(Row::new(values))
                    }
            }
        };
        let table = current_table.ok_or("No table specified")?;
        //handling for more advanced query
        match &exprs[0] {
            Expr::Select(_) =>
                {
                if let Some(fields) = data_select
                {
                    let result = select_from_table(table, &fields, &where_conditions).await;
                    Ok(Self::prepare_string_from_select(result))
                }
                else
                {
                    Err("Select fields not specified".to_string())
                }
            }
            Expr::Update(_, _, _) =>
                {
                if let Some(fileds) = update_fields
                {
                    if let Some(values) = values
                    {

                       return match  table.update_table(fileds,values, &where_conditions)
                       {
                         Ok(_)=>  Ok("Update successful".to_string()),
                         Err(error) => Err(error)
                       }

                    }
                    Err("not fields specified".to_string())
                }
                else
                {
                    Err("Update fields not specified".to_string())
                }
            }
            Expr::Delete(_) =>
                {
                table.delete_row(&where_conditions)?;
                Ok("Rows deleted successfully".to_string())
                }
            Expr::Insert(_,_)=>
                {
                   return match row
                    {
                        Some(row)=>
                            {

                                match table.insert_row(row)
                                {
                                    Ok(_)=>
                                        {
                                            Ok("row was added successfully".to_string())
                                        },
                                    Err(error)=>
                                        {
                                            Err(error)
                                        }
                                }
                            },
                        None =>
                            {
                                Err("there is no values to insert".to_string())
                            }
                    }
                }

            _ => Err("Unsupported operation".to_string())
        }
    }
    fn prepare_string_from_select(selected: (Vec<String>, Vec<Vec<Value>>)) -> String
    {
        let (column_names, rows) = selected;

        if column_names.is_empty() || rows.is_empty()
        {
            return String::from("No data available");
        }
        let mut column_widths: Vec<usize> = column_names.iter()
            .map(|name| name.len())
            .collect();

        for row in &rows
        {
            for (i, value) in row.iter().enumerate()
            {
                let value_width = value.to_string().len();
                if i < column_widths.len() && value_width > column_widths[i]
                {
                    column_widths[i] = value_width;
                }
            }
        }

        let mut result = String::new();
        result.push_str("| ");
        for (i, name) in column_names.iter().enumerate()
        {
            result.push_str(&format!("{:<width$} | ", name, width = column_widths[i]));
        }
        result.push('\n');

        for width in &column_widths
        {
            result.push_str(&format!("{:-<width$}-", "", width = width + 2));
        }
        result.push_str("|\n");

        for row in rows
        {
            result.push_str("| ");
            for (i, value) in row.iter().enumerate()
            {
                if i < column_widths.len() {
                    result.push_str(&format!("{:<width$} | ", value, width = column_widths[i]));
                }
            }
            result.push('\n');
        }
        result
    }
}