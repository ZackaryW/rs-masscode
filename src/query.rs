use std::{collections::HashMap, fmt::Display};

use nom::{
    branch::alt,
    bytes::complete::{tag, is_not},
    character::complete::{alphanumeric1, char, multispace0, multispace1},
    combinator::{map, recognize},
    multi::separated_list0,
    sequence::{delimited, tuple, preceded},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Operator {
    Eq,
    Gt,
    Lt,
    Gte,
    Lte,
    StartsWith,
    EndsWith,
    Contains,
    OneOf,  // New operator for `one_of`
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Eq => write!(f, "=="),
            Operator::Gt => write!(f, ">"),
            Operator::Lt => write!(f, "<"),
            Operator::Gte => write!(f, ">="),
            Operator::Lte => write!(f, "<="),
            Operator::StartsWith => write!(f, "StartsWith"),
            Operator::EndsWith => write!(f, "EndsWith"),
            Operator::Contains => write!(f, "Contains"),
            Operator::OneOf => write!(f, "OneOf"),  
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Condition(String, Operator, Vec<String>),  // Adjusted to hold Vec<String> for one_of
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Condition(field, operator, values) => write!(f, "{} {} {}", field, operator, values.join(", ")),
            Expr::Not(inner) => write!(f, "NOT ({})", inner),
            Expr::And(left, right) => write!(f, "({} AND {})", left, right),
            Expr::Or(left, right) => write!(f, "({} OR {})", left, right),
        }
    }
}

fn field(input: &str) -> IResult<&str, String> {
    map(recognize(alphanumeric1), String::from)(input)
}

fn value(input: &str) -> IResult<&str, String> {
    map(recognize(alphanumeric1), String::from)(input)
}

// Parses a list of string values enclosed in brackets
fn value_list(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        char('['),
        separated_list0(tag(","), map(delimited(multispace0, is_not(",]"), multispace0), String::from)),
        char(']')
    )(input)
}

fn operator_and_negation(input: &str) -> IResult<&str, (bool, Operator)> {
    alt((
        map(tag("=="), |_| (false, Operator::Eq)),
        map(tag("!="), |_| (true, Operator::Eq)),
        map(tag(">="), |_| (false, Operator::Gte)),
        map(tag("<="), |_| (false, Operator::Lte)),
        map(tag(">"), |_| (false, Operator::Gt)),
        map(tag("<"), |_| (false, Operator::Lt)),
        map(tag("@SW"), |_| (false, Operator::StartsWith)),
        map(tag("@EW"), |_| (false, Operator::EndsWith)),
        map(tag("@CT"), |_| (false, Operator::Contains)),
        map(tag("@OO"), |_| (false, Operator::OneOf)), 
    ))(input)
}

fn condition(input: &str) -> IResult<&str, Expr> {
    alt((
        // Handle one_of condition separately
        map(
            tuple((field, multispace1, tag("@oo"), multispace1, value_list)),
            |(f, _, _, _, vals)| Expr::Condition(f, Operator::OneOf, vals),
        ),
        // Normal conditions
        map(
            tuple((field, multispace1, operator_and_negation, multispace1, value)),
            |(f, _, (negate, op), _, v)| {
                let basic_condition = Expr::Condition(f, op, vec![v]);
                if negate {
                    Expr::Not(Box::new(basic_condition))
                } else {
                    basic_condition
                }
            },
        ),
    ))(input)
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
    delimited(
        char('('),
        logical_expr,
        char(')'),
    )(input)
}

fn logical_expr(input: &str) -> IResult<&str, Expr> {
    alt((
        map(tuple((parse_expr, multispace0, tag("&"), multispace0, parse_expr)),
            |(left, _, _, _, right)| Expr::And(Box::new(left), Box::new(right))),
        map(tuple((parse_expr, multispace0, tag("|"), multispace0, parse_expr)),
            |(left, _, _, _, right)| Expr::Or(Box::new(left), Box::new(right))),
        map(preceded(tuple((tag("~"), multispace1)), parse_expr),
            |expr| Expr::Not(Box::new(expr))),
        condition,  // To handle conditions without explicit parentheses
    ))(input)
}


pub fn parse_query(input: &str) -> IResult<&str, Expr> {
    // Parsing logic as previously discussed, integrating the root function of your parser
    logical_expr(input) // Assuming this is your top-level parsing function
}

pub fn execute_query(data: &HashMap<String, HashMap<String, String>>, expr: &Expr) -> Vec<String> {
    data.iter()
        .filter_map(|(key, values)| if evaluate(expr, values) { Some(key.clone()) } else { None })
        .collect()
}

pub(crate) fn evaluate<T: serde::Serialize>(expr: &Expr, item: &T) -> bool {
    // Serialize the item into serde_json::Value to access fields dynamically
    let value = serde_json::to_value(item).unwrap();

    match expr {
        Expr::Condition(field, operator, values) => {
            if let Some(value) = value.get(field).and_then(|v| v.as_str()) {
                match operator {
                    Operator::Eq => values.contains(&value.to_string()),
                    Operator::Gt => value > &values[0],
                    Operator::Lt => value < &values[0],
                    Operator::Gte => value >= &values[0],
                    Operator::Lte => value <= &values[0],
                    Operator::StartsWith => value.starts_with(&values[0]),
                    Operator::EndsWith => value.ends_with(&values[0]),
                    Operator::Contains => value.contains(&values[0]),
                    Operator::OneOf => values.iter().any(|v| v == value),
                }
            } else {
                false
            }
        },
        Expr::Not(inner) => !evaluate(inner, item),
        Expr::And(left, right) => evaluate(left, item) && evaluate(right, item),
        Expr::Or(left, right) => evaluate(left, item) || evaluate(right, item),
    }
}
