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
            tuple((field, multispace1, tag("@OO"), multispace1, value_list)),
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

pub fn execute_query<T: serde::Serialize + Clone>(
    data: &HashMap<String, T>,
    expr: &Expr,
) -> Result<Vec<T>, serde_json::Error> {
    data.iter()
        .filter_map(|(_key, value)| {
            // Serialize the value to a JSON Value for evaluation.
            let json_value = serde_json::to_value(value);
            match json_value {
                Ok(val) => {
                    // If the expression evaluates true, return the original object.
                    if evaluate(expr, &val) {
                        Some(Ok(value.clone()))  // Cloning the value if it meets the condition.
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(e)),  // Return error during serialization
            }
        })
        // Collect results and propagate serialization errors.
        .collect()
}


pub(crate) fn evaluate(expr: &Expr, item: &serde_json::Value) -> bool {
    match expr {
        Expr::Condition(field, operator, values) => {
            if let Some(value) = item.get(field) {
                match operator {
                    Operator::Eq => values.iter().any(|v| value == &serde_json::Value::String(v.clone())),
                    Operator::Gt | Operator::Lt | Operator::Gte | Operator::Lte => {
                        if let Some(val_str) = value.as_str() {
                            values.iter().any(|v| {
                                let value_num: f64 = val_str.parse().unwrap_or(f64::NAN);
                                let target_num: f64 = v.parse().unwrap_or(f64::NAN);
                                match operator {
                                    Operator::Gt => value_num > target_num,
                                    Operator::Lt => value_num < target_num,
                                    Operator::Gte => value_num >= target_num,
                                    Operator::Lte => value_num <= target_num,
                                    _ => false,
                                }
                            })
                        } else {
                            false
                        }
                    },
                    Operator::StartsWith => {
                        values.iter().any(|v| {
                            if let Some(val_str) = value.as_str() {
                                val_str.starts_with(v)
                            } else {
                                false
                            }
                        })
                    },
                    Operator::EndsWith => {
                        values.iter().any(|v| {
                            if let Some(val_str) = value.as_str() {
                                val_str.ends_with(v)
                            } else {
                                false
                            }
                        })
                    },
                    Operator::Contains => {
                        values.iter().any(|v| {
                            if let Some(val_str) = value.as_str() {
                                val_str.contains(v)
                            } else {
                                false
                            }
                        })
                    },
                    Operator::OneOf => {
                        values.iter().any(|v| &serde_json::Value::String(v.clone()) == value)
                    },
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
