use full_moon::ast::Ast;
use full_moon::ast::punctuated::Punctuated;
use full_moon::ast::punctuated::Pair;
use full_moon::ast::Expression;

fn foo_expression(expression: full_moon::ast::Expression) -> full_moon::ast::Expression
{
    match expression
    {
        full_moon::ast::Expression::FunctionCall(function_call) =>
        {
            return full_moon::ast::Expression::FunctionCall(function_call);
        }
        full_moon::ast::Expression::Parentheses{ref contained, ref expression} =>
        {
            return Expression::Parentheses{
                contained: contained.clone(),
                expression: Box::new(foo_expression(*expression.clone())),
            };
        },
        full_moon::ast::Expression::BinaryOperator{ref lhs, ref binop, ref rhs} =>
        {
            match binop
            {
                full_moon::ast::BinOp::Or(_) =>
                {
                    let lhs_clone = *lhs.clone();
                    match lhs_clone
                    {
                        full_moon::ast::Expression::Symbol(token_reference) =>
                        {
                            match token_reference.token().token_type()
                            {
                                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                                {
                                    match symbol
                                    {
                                        full_moon::tokenizer::Symbol::True =>
                                            return Expression::Symbol(token_reference),

                                        full_moon::tokenizer::Symbol::False =>
                                            return *rhs.clone(),

                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                        },

                        _ => {},
                    }

                    let rhs_clone = *rhs.clone();
                    match rhs_clone
                    {
                        full_moon::ast::Expression::Symbol(token_reference) =>
                        {
                            match token_reference.token().token_type()
                            {
                                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                                {
                                    match symbol
                                    {
                                        full_moon::tokenizer::Symbol::True =>
                                            return Expression::Symbol(token_reference),

                                        full_moon::tokenizer::Symbol::False =>
                                            return *lhs.clone(),
                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                        },

                        _ => {},
                    }
                },

                full_moon::ast::BinOp::And(_) =>
                {
                    let lhs_clone = *lhs.clone();
                    match lhs_clone
                    {
                        full_moon::ast::Expression::Symbol(token_reference) =>
                        {
                            match token_reference.token().token_type()
                            {
                                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                                {
                                    match symbol
                                    {
                                        full_moon::tokenizer::Symbol::False =>
                                            return Expression::Symbol(token_reference),

                                        full_moon::tokenizer::Symbol::True =>
                                            return *rhs.clone(),
                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                        },

                        _ => {},
                    }

                    let rhs_clone = *rhs.clone();
                    match rhs_clone
                    {
                        full_moon::ast::Expression::Symbol(token_reference) =>
                        {
                            match token_reference.token().token_type()
                            {
                                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                                {
                                    match symbol
                                    {
                                        full_moon::tokenizer::Symbol::False =>
                                            return Expression::Symbol(token_reference),

                                        full_moon::tokenizer::Symbol::True =>
                                            return *lhs.clone(),
                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                        },

                        _ => {},
                    }
                },

                _ => {},
            }

            return Expression::BinaryOperator{
                lhs: Box::new(foo_expression(*lhs.clone())),
                binop: binop.clone(),
                rhs: Box::new(foo_expression(*rhs.clone())),
            };
        },
        _ => {},
    }

    expression
}

fn replace_expressions(
    punctuated_expressions: Punctuated<Expression>,
    expressions: Vec<Expression>) -> Punctuated<Expression>
{
    let mut new_expressions = Punctuated::new();

    for (pair, expression) in punctuated_expressions.pairs().zip(expressions)
    {
        match pair.punctuation()
        {
            Some(punctuation) =>
                new_expressions.push_punctuated(expression, punctuation.clone()),

            None =>
                new_expressions.push(Pair::End(expression)),
        }
    }

    new_expressions
}

fn foo_local_assignment(local_assignment: full_moon::ast::LocalAssignment) -> full_moon::ast::LocalAssignment
{
    local_assignment.clone().with_expressions(replace_expressions(local_assignment.expressions().clone(),
        local_assignment.expressions().iter().map(
        |expression| foo_expression(expression.clone())).collect()))
}

fn foo_statement(statement: full_moon::ast::Stmt) -> full_moon::ast::Stmt
{
    match statement
    {
        full_moon::ast::Stmt::LocalAssignment(local_assignment) =>
        {
            return full_moon::ast::Stmt::LocalAssignment(
                foo_local_assignment(local_assignment.clone()))
        },
        _ => statement,
    }
}

fn foo_block(block: full_moon::ast::Block) -> full_moon::ast::Block
{
    block.clone().with_stmts(
        block.stmts_with_semicolon().map(
            |(statement, token_reference)|
                (foo_statement(statement.clone()), token_reference.clone())
        ).collect())
}

pub fn foo(input_ast: Ast) -> Ast
{
    input_ast.clone().with_nodes(foo_block(input_ast.nodes().clone()))
}

#[cfg(test)]
mod tests
{
use full_moon::parse_fallible;
use crate::simplify::foo;
use crate::Config;
use crate::format_ast;
use crate::OutputVerification;


fn simplify_code(code: &str) -> String {
    let config = Config::default();
    format_ast(foo(parse_fallible(code, config.syntax.into()).into_result().unwrap()),
        config, None, OutputVerification::None).unwrap().to_string()
}

fn input_output(input: &str, expected_output: &str)
{
    assert_eq!(simplify_code(input), expected_output);
}

#[test]
fn basic_assignment() {
    input_output(
        "local x = 1\n",
        "local x = 1\n");
}

    #[test]
    fn test_format_code_basic() {
        input_output( "\
local function f()
    return false;
end

return jeff
", "\
local function f()
\treturn false
end

return jeff
");
    }

    #[test]
    fn test_format_code_with_bool_or_lhs() {
        input_output(
            "local x = true or y\n",
            "local x = true\n");
    }

    #[test]
    fn test_format_code_with_bool_or_rhs() {
        input_output(
            "local x = y or true\n",
            "local x = true\n");
    }

    #[test]
    fn test_format_code_with_bool_and_lhs() {
        input_output(
            "local x = false and z\n",
            "local x = false\n");
    }

    #[test]
    fn test_format_code_with_bool_and_rhs() {
        input_output(
            "local x = y and false\n",
            "local x = false\n");
    }

    #[test]
    fn test_format_code_with_bool_or_identity_lhs() {
        input_output(
            "local x = false or y\n",
            "local x = y\n");
    }

    #[test]
    fn test_format_code_with_bool_or_identity_rhs() {
        input_output(
            "local x = y or false\n",
            "local x = y\n");
    }

    #[test]
    fn test_format_code_with_bool_and_identity_lhs() {
        input_output(
            "local x = true and y\n",
            "local x = y\n");
    }

    #[test]
    fn test_format_code_with_bool_and_identity_rhs() {
        input_output(
            "local x = y and true\n",
            "local x = y\n");
    }

    #[test]
    fn test_format_code_with_bools_true() {
        input_output(
            "local x = true or b and c\n",
            "local x = true\n");
    }

    #[test]
    fn test_format_code_with_bools_or_subexpression() {
        input_output(
            "local x = a or b and false\n",
            "local x = a or false\n");
    }

    #[test]
    fn test_format_code_with_expression_parentheses() {
        input_output(
            "local x = (true)\n",
            "local x = true\n");
    }

    #[test]
    fn test_format_code_with_expression_in_parentheses() {
        input_output(
            "local x = (true or y)\n",
            "local x = true\n");
    }

    #[test]
    fn test_format_code_with_expression_in_argument_of_function() {
        input_output(
            "local x = foo(true or y)\n",
            "local x = true\n");
    }
}