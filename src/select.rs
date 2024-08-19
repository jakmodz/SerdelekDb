use crate::table::*;
use crate::enum_utils::*;

#[derive(Debug)]
pub struct Selector
{
    pub name: String,
    pub value: Value,
    pub statement: Statement,
}
#[derive(Debug)]
pub enum Statement
{
    Equals,
    Bigger,
    Smaller
}
pub async fn select_from_table(
    table: &Table,
    data: &Vec<String>,
    selectors: &Vec<Selector>,
) -> (Vec<String>, Vec<Vec<Value>>)
{
    let select_all = data.len() == 1 && data[0] == "*";

    let column_indices: Vec<_> = if !select_all
    {
        data.iter()
            .filter_map(|field| table.columns.iter().position(|(col_name, _)| col_name == field))
            .collect()
    }
    else
    {
        (0..table.columns.len()).collect()
    };

    let selected_columns: Vec<String> = column_indices
        .iter()
        .map(|&i| table.columns[i].0.clone())
        .collect();

    let rows = table
        .values
        .iter()
        .filter_map(|row| {
            let mut match_found = true;

            for selector in selectors
            {
                let col_idx = table.columns.iter().position(|(col_name, _)| col_name == &selector.name);
                if let Some(idx) = col_idx
                {
                    let row_value = &row.data[idx];
                    let type_match = get_variant_name(row_value) == get_variant_name(&selector.value);

                    let value_match = match selector.statement
                    {
                        Statement::Equals => *row_value == selector.value,
                        Statement::Bigger => Value::compare_values(&Statement::Bigger,row_value,&selector),
                        Statement::Smaller => Value::compare_values(&Statement::Smaller,row_value,&selector)
                    };

                    if !type_match || !value_match
                    {
                        match_found = false;
                        break;
                    }
                }
                else
                {
                    match_found = false;
                    break;
                }
            }

            if match_found
            {
                Some(column_indices.iter().map(|&i| row.data[i].clone()).collect())
            }
            else
            {
                None
            }
        })
        .collect();

    (selected_columns, rows)
}