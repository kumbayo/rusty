use crate::ast::*;
use crate::expect;
use crate::lexer::Token::*;

use super::allow;
use super::RustyLexer;
use super::{slice_and_advance, unexpected_token};

#[cfg(test)]
mod tests;

pub fn parse_primary_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    parse_expression_list(lexer)
}

fn parse_expression_list(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_range_statement(lexer);
    if lexer.token == KeywordComma {
        let mut expressions = Vec::new();
        expressions.push(left?);
        // this starts an expression list
        while lexer.token == KeywordComma {
            lexer.advance();
            expressions.push(parse_range_statement(lexer)?);
        }
        return Ok(Statement::ExpressionList { expressions });
    }
    Ok(left?)
}

fn parse_range_statement(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let start = parse_or_expression(lexer)?;

    if lexer.token == KeywordDotDot {
        lexer.advance();
        let end = parse_or_expression(lexer)?;
        return Ok(Statement::RangeStatement {
            start: Box::new(start),
            end: Box::new(end),
        });
    }
    Ok(start)
}

// OR
fn parse_or_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_xor_expression(lexer)?;

    let operator = match lexer.token {
        OperatorOr => Operator::Or,
        _ => return Ok(left),
    };

    lexer.advance();

    let right = parse_or_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

// XOR
fn parse_xor_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_and_expression(lexer)?;

    let operator = match lexer.token {
        OperatorXor => Operator::Xor,
        _ => return Ok(left),
    };

    lexer.advance();

    let right = parse_xor_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

// AND
fn parse_and_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_equality_expression(lexer)?;

    let operator = match lexer.token {
        OperatorAnd => Operator::And,
        _ => return Ok(left),
    };

    lexer.advance();

    let right = parse_and_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

//EQUALITY  =, <>
fn parse_equality_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_compare_expression(lexer)?;
    let operator = match lexer.token {
        OperatorEqual => Operator::Equal,
        OperatorNotEqual => Operator::NotEqual,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_equality_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

//COMPARE <, >, <=, >=
fn parse_compare_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_additive_expression(lexer)?;
    let operator = match lexer.token {
        OperatorLess => Operator::Less,
        OperatorGreater => Operator::Greater,
        OperatorLessOrEqual => Operator::LessOrEqual,
        OperatorGreaterOrEqual => Operator::GreaterOrEqual,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_compare_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

// Addition +, -
fn parse_additive_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_multiplication_expression(lexer)?;
    let operator = match lexer.token {
        OperatorPlus => Operator::Plus,
        OperatorMinus => Operator::Minus,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_additive_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

// Multiplication *, /, MOD
fn parse_multiplication_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_unary_expression(lexer)?;
    let operator = match lexer.token {
        OperatorMultiplication => Operator::Multiplication,
        OperatorDivision => Operator::Division,
        OperatorModulo => Operator::Modulo,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_multiplication_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}
// UNARY -x, NOT x
fn parse_unary_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let operator = match lexer.token {
        OperatorNot => Some(Operator::Not),
        OperatorMinus => Some(Operator::Minus),
        _ => None,
    };

    if let Some(operator) = operator {
        lexer.advance();
        Ok(Statement::UnaryExpression {
            operator: operator,
            value: Box::new(parse_parenthesized_expression(lexer)?),
        })
    } else {
        parse_parenthesized_expression(lexer)
    }
}

// PARENTHESIZED (...)
fn parse_parenthesized_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    match lexer.token {
        KeywordParensOpen => {
            lexer.advance();
            let result = parse_primary_expression(lexer);
            expect!(KeywordParensClose, lexer);
            lexer.advance();
            result
        }
        _ => parse_leaf_expression(lexer),
    }
}

// Literals, Identifiers, etc.
fn parse_leaf_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let current = match lexer.token {
        Identifier => parse_reference(lexer),
        LiteralInteger => parse_literal_number(lexer),
        LiteralTrue => parse_bool_literal(lexer, true),
        LiteralFalse => parse_bool_literal(lexer, false),
        _ => Err(unexpected_token(lexer)),
    };

    if current.is_ok() && lexer.token == KeywordAssignment {
        lexer.advance();
        return Ok(Statement::Assignment {
            left: Box::new(current?),
            right: Box::new(parse_range_statement(lexer)?),
        });
    };
    current
}

fn parse_bool_literal(lexer: &mut RustyLexer, value: bool) -> Result<Statement, String> {
    lexer.advance();
    Ok(Statement::LiteralBool { value })
}

pub fn parse_reference(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let reference = Statement::Reference {
        name: slice_and_advance(lexer).to_string(),
    };

    if allow(KeywordParensOpen, lexer) {
        let statement_list = if allow(KeywordParensClose, lexer) {
            None
        } else {
            let list = parse_expression_list(lexer).unwrap();
            expect!(KeywordParensClose, lexer);
            lexer.advance();
            Some(list)
        };
        Ok(Statement::CallStatement {
            operator: Box::new(reference),
            parameters: Box::new(statement_list),
        })
    } else {
        Ok(reference)
    }
}

fn parse_literal_number(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let result = slice_and_advance(lexer);
    if allow(KeywordDot, lexer) {
        return parse_literal_real(lexer, result);
    }

    Ok(Statement::LiteralInteger { value: result })
}

fn parse_literal_real(lexer: &mut RustyLexer, integer: String) -> Result<Statement, String> {
    expect!(LiteralInteger, lexer);
    let fractional = slice_and_advance(lexer);
    let exponent = if lexer.token == LiteralExponent {
        slice_and_advance(lexer)
    } else {
        "".to_string()
    };
    let result = format!("{}.{}{}", integer, fractional, exponent);

    Ok(Statement::LiteralReal { value: result })
}