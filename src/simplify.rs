use full_moon::ast::Ast;
use full_moon::ast::punctuated::Punctuated;
use full_moon::ast::punctuated::Pair;
use full_moon::ast::BinOp::Or;
use full_moon::ast::BinOp::And;
use full_moon::ast::UnOp::Not;
use full_moon::ast::FunctionCall;

use full_moon::ast::Expression;
use full_moon::ast::Expression::UnaryOperator;
use full_moon::ast::Suffix;
use full_moon::ast::Call;
use full_moon::ast::FunctionArgs;

use full_moon::tokenizer::TokenReference;
use full_moon::tokenizer::Token;
use full_moon::tokenizer::Symbol::True;
use full_moon::tokenizer::Symbol::False;
use full_moon::tokenizer::TokenType::Symbol;

fn simplify_function_args(function_args: &FunctionArgs) -> FunctionArgs
{
    match function_args
    {
        FunctionArgs::Parentheses {parentheses, arguments} => FunctionArgs::Parentheses {
            parentheses: parentheses.clone(),
            arguments: simplify_punctuated_expressions(arguments),
        },

        FunctionArgs::String(token_reference) => FunctionArgs::String(token_reference.clone()),
        FunctionArgs::TableConstructor(table_constructor) => FunctionArgs::TableConstructor(table_constructor.clone()),
        &_ => todo!(),
    }
}

fn simplify_call(call: &Call) -> Call
{
    match call
    {
        Call::AnonymousCall(function_args) => Call::AnonymousCall(simplify_function_args(function_args)),
        Call::MethodCall(method_call) => Call::MethodCall(method_call.clone()),
        &_ => todo!(),
    }
}

fn simplify_suffix(suffix: &Suffix) -> Suffix
{
    match suffix
    {
        Suffix::Call(call) =>
        {
            Suffix::Call(simplify_call(call))
        },

        Suffix::Index(index) =>
        {
            Suffix::Index(index.clone())
        },

        #[cfg(feature = "luau")]
        Suffix::TypeInstantiation(type_instantiation) =>
        {
            Suffix::TypeInstantiation(type_instantiation.clone())
        },

        &_ => todo!()
    }
}

fn simplify_function_call(function_call: FunctionCall) -> FunctionCall
{
    let suffixes = function_call.suffixes().map(|suffix| simplify_suffix(suffix)).collect();
    function_call.with_suffixes(suffixes)
}

fn simplify_expression(expression: full_moon::ast::Expression) -> full_moon::ast::Expression
{
    match expression
    {
        Expression::FunctionCall(function_call) =>
            return Expression::FunctionCall(simplify_function_call(function_call)),

        full_moon::ast::Expression::Parentheses{ref contained, ref expression} =>
        {
            let new_inside_expression = simplify_expression(*expression.clone());
            match new_inside_expression
            {
                Expression::Symbol(ref _symbol) =>
                {
                    return new_inside_expression;
                },
                _ => {},
            }

            return Expression::Parentheses{
                contained: contained.clone(),
                expression: Box::new(new_inside_expression),
            };
        },
        UnaryOperator{ref unop, ref expression} =>
        {
            match unop
            {
                Not(_) =>
                {
                    let new_expression = simplify_expression(*expression.clone());
                    match new_expression
                    {
                        full_moon::ast::Expression::Symbol(ref token_reference) =>
                        {
                            let leading_trivia = token_reference.leading_trivia().map(|x| x.clone()).collect();
                            let trailing_trivia = token_reference.trailing_trivia().map(|x| x.clone()).collect();

                            match token_reference.token().token_type()
                            {
                                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                                {
                                    match symbol
                                    {
                                        True =>
                                            return full_moon::ast::Expression::Symbol(TokenReference::new(
                                                leading_trivia,
                                                Token::new(Symbol{symbol:False}),
                                                trailing_trivia,
                                            )),

                                        False =>
                                            return full_moon::ast::Expression::Symbol(TokenReference::new(
                                                leading_trivia,
                                                Token::new(Symbol{symbol:True}),
                                                trailing_trivia,
                                            )),

                                        _ => {},
                                    }
                                },
                                _ => {},
                            }
                        },
                        _ => {},
                    }

                    return UnaryOperator{unop: unop.clone(), expression: Box::new(new_expression.clone())};
                }
                _ => {},
            }
        },
        full_moon::ast::Expression::BinaryOperator{ref lhs, ref binop, ref rhs} =>
        {
            match binop
            {
                Or(_) =>
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
                                        True =>
                                            return Expression::Symbol(token_reference),

                                        False =>
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
                                        True =>
                                            return Expression::Symbol(token_reference),

                                        False =>
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

                And(_) =>
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
                                        False =>
                                            return Expression::Symbol(token_reference),

                                        True =>
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
                                        False =>
                                            return Expression::Symbol(token_reference),

                                        True =>
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
                lhs: Box::new(simplify_expression(*lhs.clone())),
                binop: binop.clone(),
                rhs: Box::new(simplify_expression(*rhs.clone())),
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

fn simplify_punctuated_expressions(punctuated_expressions :&Punctuated<Expression>) -> Punctuated<Expression>
{
    replace_expressions(punctuated_expressions.clone(),
        punctuated_expressions.iter().map(
        |expression| simplify_expression(expression.clone())).collect())
}

fn simplify_local_assignment(local_assignment: full_moon::ast::LocalAssignment) -> full_moon::ast::LocalAssignment
{
    local_assignment.clone().with_expressions(simplify_punctuated_expressions(local_assignment.expressions()))
}

fn simplify_statement(statement: full_moon::ast::Stmt) -> full_moon::ast::Stmt
{
    match statement
    {
        full_moon::ast::Stmt::LocalAssignment(local_assignment) =>
        {
            return full_moon::ast::Stmt::LocalAssignment(
                simplify_local_assignment(local_assignment.clone()))
        },
        _ => statement,
    }
}

fn simplify_block(block: full_moon::ast::Block) -> full_moon::ast::Block
{
    block.clone().with_stmts(
        block.stmts_with_semicolon().map(
            |(statement, token_reference)|
                (simplify_statement(statement.clone()), token_reference.clone())
        ).collect())
}

pub fn simplify_ast(input_ast: Ast) -> Ast
{
    input_ast.clone().with_nodes(simplify_block(input_ast.nodes().clone()))
}

#[cfg(test)]
mod tests
{
use full_moon::parse_fallible;
use crate::simplify::simplify_ast;
use crate::Config;
use crate::format_ast;
use crate::OutputVerification;

fn simplify_code(code: &str) -> String {
    let config = Config::default();
    format_ast(simplify_ast(parse_fallible(code, config.syntax.into()).into_result().unwrap()),
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
    fn basic() {
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
    fn bool_or_lhs() {
        input_output(
            "local x = true or y\n",
            "local x = true\n");
    }

    #[test]
    fn bool_or_rhs() {
        input_output(
            "local x = y or true\n",
            "local x = true\n");
    }

    #[test]
    fn bool_and_lhs() {
        input_output(
            "local x = false and z\n",
            "local x = false\n");
    }

    #[test]
    fn bool_and_rhs() {
        input_output(
            "local x = y and false\n",
            "local x = false\n");
    }

    #[test]
    fn bool_or_identity_lhs() {
        input_output(
            "local x = false or y\n",
            "local x = y\n");
    }

    #[test]
    fn bool_or_identity_rhs() {
        input_output(
            "local x = y or false\n",
            "local x = y\n");
    }

    #[test]
    fn bool_and_identity_lhs() {
        input_output(
            "local x = true and y\n",
            "local x = y\n");
    }

    #[test]
    fn bool_and_identity_rhs() {
        input_output(
            "local x = y and true\n",
            "local x = y\n");
    }

    #[test]
    fn bools_true() {
        input_output(
            "local x = true or b and c\n",
            "local x = true\n");
    }

    #[test]
    fn bools_or_subexpression() {
        input_output(
            "local x = a or b and false\n",
            "local x = a or false\n");
    }

    #[test]
    fn bool_literal_in_parentheses() {
        input_output(
            "local x = (true)\n",
            "local x = true\n");
    }

    #[test]
    fn or_expression_in_parentheses() {
        input_output(
            "local x = (true or y)\n",
            "local x = true\n");
    }

    #[test]
    fn and_expression_in_parentheses() {
        input_output(
            "local x = (false and y)\n",
            "local x = false\n");
    }

    #[test]
    fn not_false() {
        input_output(
            "local x = not false\n",
            "local x = true\n");
    }

    #[test]
    fn not_true() {
        input_output(
            "local x = not true\n",
            "local x = false\n");
    }

    #[test]
    fn not_with_parenthetical_false() {
        input_output(
            "local x = not (false and y)\n",
            "local x = true\n");
    }

    #[test]
    fn not_with_parenthetical_true() {
        input_output(
            "local x = not (true or y)\n",
            "local x = false\n");
    }

    #[test]
    fn expression_in_argument_of_function() {
        input_output(
            "local x = foo(true or y)\n",
            "local x = foo(true)\n");
    }
}