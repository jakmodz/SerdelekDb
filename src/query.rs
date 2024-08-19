use crate::table::*;
#[derive(Debug)]
pub enum Token
{
    Select,
    From,
    Drop,
    Where,
    Update,
    Set,
    Insert,
    Delete,
    Describe,
    Identifier(String),
    Operator(String),
    Number(Value),
    Create,

}
#[derive(Debug,Clone)]
pub enum Expr
{
    Select(Vec<String>),
    From(String),
    Drop(String),
    Where(String,String,Value),
    Update(String,Vec<String>,Vec<Value>),
    Delete(String),
    Describe(String),
    Create(String,Vec<String>,Vec<DataTypes>),
    Insert(String,Vec<Value>)
}

pub fn tokenize(query:String) -> Vec<Token>
{
    let mut tokens = Vec::new();
    let query = query.split_whitespace();
    for word in query
    {
        match word.to_lowercase().as_str()
        {
            "select" => tokens.push(Token::Select),
            "from" => tokens.push(Token::From),
            "where" => tokens.push(Token::Where),
            "update" => tokens.push(Token::Update),
            "insert"=> tokens.push(Token::Insert),
            "set" => tokens.push(Token::Set),
            "delete" => tokens.push(Token::Delete),
            "drop" => tokens.push(Token::Drop),
            "=" => tokens.push(Token::Operator(word.to_string())),
            "describe"=> tokens.push(Token::Describe),
            "create"=> tokens.push(Token::Create),
            ">" => tokens.push(Token::Operator(word.to_string())),
            "<" => tokens.push(Token::Operator(word.to_string())),
            _ => {
                if let Some(number) = check_data_type(word)
                {
                    tokens.push(Token::Number(number));
                }
                else
                {
                    tokens.push(Token::Identifier(word.to_string()));
                }
            }
        }
    }
    tokens
}
fn check_data_type(s: &str) -> Option<Value>
{
    let mut return_value = None;
    if let Ok(value) = s.parse::<i32>()
    {
        return_value =  Some(Value::Int(value))
    } else if let Ok(value) = s.parse::<f64>()
    {
       return_value =  Some(Value::Float(value))
    }
    else if  s.starts_with("'") && s.ends_with("'")
    {
        let to_remove = '\'';
        return_value =   Some(Value::Text(s.chars().filter(|&c| c != to_remove ).collect()))
    }
    else if  s.starts_with("key_")
    {
        if let Ok(value) = s[4..].parse::<i64>()
        {
            return_value = Some(Value::Key(value))
        }
    }
    return_value
}

pub fn parse(tokens: Vec<Token>) -> Vec<Expr>
{
    let mut expressions = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next()
    {
        match token
        {
            Token::Select =>
                {
                let mut fields = Vec::new();
                while let Some(Token::Identifier(field)) = iter.peek()
                {
                    fields.push(field.clone());
                    iter.next();
                }
                expressions.push(Expr::Select(fields));
            }
            Token::From =>
                {
                if let Some(Token::Identifier(table)) = iter.next()
                {
                    expressions.push(Expr::From(table.clone()));
                }
            }
            Token::Where =>
                if let Some(Token::Identifier(field)) = iter.next()
                {
                    if let Some(Token::Operator(op)) = iter.next()
                    {
                        if let Some(Token::Number(value)) = iter.next()
                        {
                            expressions.push(Expr::Where(field.clone(), op.clone(),value.clone() ));
                        }
                    }
                }
            Token::Update =>
                {
                    let mut name = String::new();
                    let mut fields: Vec<String> = Vec::new();
                    let mut values = Vec::new();
                    while let Some(token) = iter.next()
                    {
                        match token
                        {
                            Token::Identifier(field) =>
                                {
                                    if field.as_str() == "|"
                                    {
                                        // Parse the fields until the next token
                                        while let Some(token) = iter.next()
                                        {
                                            match token
                                            {
                                                Token::Identifier(field) => fields.push(field.clone()),
                                                Token::Number(value) =>
                                                    {
                                                        values.push(value.clone());
                                                    },
                                                _ => break,
                                            }
                                        }
                                    }
                                    else
                                    {
                                        if name.len() == 0
                                        {
                                            name = field.clone();
                                        }
                                        else
                                        {
                                            break;
                                        }
                                    }
                                }
                            Token::Number(value) =>
                                {
                                    values.push(value.clone());
                                }
                            _ => {}
                        }
                    }
                    expressions.push(Expr::Update(name, fields, values));
                }
            Token::Delete =>
                {
                if let Some(Token::Identifier(table)) = iter.next()
                {
                    expressions.push(Expr::Delete(table.clone()));
                }
            }
            Token::Drop=>
                {
                    if let Some(Token::Identifier(table)) = iter.next()
                    {
                        expressions.push(Expr::Drop(table.clone()));
                    }
                }
            Token::Describe=>
                {
                    if let Some(Token::Identifier(table)) = iter.next()
                    {
                        expressions.push(Expr::Describe(table.clone()));
                    }
                }
            Token::Insert =>
                {
                    let mut name = String::new();
                    let mut row = Vec::new();
                    while let Some(token) = iter.next()
                    {
                        match token
                        {
                            Token::Identifier(table_name)=>
                                {
                                    name = table_name.clone();
                                }
                            Token::Number(value)=>
                                {
                                    row.push(value.clone())
                                }
                            _ =>{}
                        }
                    }
                    expressions.push(Expr::Insert(name,row));
                }
            Token::Create =>
                {
                    let mut columns:Vec<String> = Vec::new();
                    let mut name = String::new();
                    let mut data_types = Vec::new();
                    while let Some(token) = iter.next()
                    {
                        match token
                        {
                            Token::Identifier(column) =>
                                {
                                    if column.starts_with("\"") && column.ends_with("\"")
                                    {
                                        name = column.clone();
                                    }
                                    else if column == "|"
                                    {
                                        while let Some(data_type_token) = iter.next()
                                        {
                                            match data_type_token
                                            {
                                                Token::Identifier(data) =>
                                                    {
                                                        match data.to_lowercase().as_str()
                                                        {
                                                            "int" => data_types.push(DataTypes::Int),
                                                            "text" => data_types.push(DataTypes::Text),
                                                            "float" => data_types.push(DataTypes::Float),
                                                            "key_auto" => data_types.push(DataTypes::Key(Key::IntAutoIncrement)),
                                                            "key" => data_types.push(DataTypes::Key(Key::Int)),
                                                            _ => break,
                                                        }
                                                    },
                                                _ => break,
                                            }
                                        }
                                    }
                                    else
                                    {
                                        columns.push(column.clone());
                                    }
                                },
                            _ => {}
                        }
                    }
                    if name.len()> 0 && columns.len() > 0 && data_types.len() > 0
                    {
                        expressions.push(Expr::Create(name,columns,data_types))
                    }
                }
            _ => {}
        }
    }
    expressions
}