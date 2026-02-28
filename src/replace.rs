use full_moon::ast::Ast;
use full_moon::ast::FunctionCall;
use full_moon::ast::Expression;
use full_moon::ast::Prefix;
use full_moon::ast::Suffix;
use full_moon::ast::Call;
use full_moon::ast::FunctionArgs;
use full_moon::ast::LocalAssignment;
use full_moon::ast::Assignment;
use full_moon::ast::If;
use full_moon::ast::ElseIf;
use full_moon::ast::Do;
use full_moon::ast::While;
use full_moon::ast::FunctionDeclaration;
use full_moon::ast::LocalFunction;
use full_moon::ast::AnonymousFunction;
use full_moon::ast::NumericFor;
use full_moon::ast::GenericFor;
use full_moon::ast::Repeat;
use full_moon::ast::FunctionBody;
use full_moon::ast::Stmt;
use full_moon::ast::Block;
use full_moon::ast::UnOp;
use full_moon::ast::BinOp;
use full_moon::ast::Field;
use full_moon::ast::TableConstructor;
use full_moon::ast::Var;
use full_moon::ast::VarExpression;

use full_moon::ast::punctuated::Punctuated;
use full_moon::ast::span::ContainedSpan;
use crate::equivalent::eq_expression;

struct Replacer
{
    expressions: Vec<(Expression, Expression)>
}

impl Replacer
{

fn replace_in_function_args(self:&Self, function_args: &FunctionArgs) -> FunctionArgs
{
    match function_args
    {
        FunctionArgs::Parentheses {parentheses, arguments} => FunctionArgs::Parentheses {
            parentheses: parentheses.clone(),
            arguments: self.replace_in_punctuated_expressions(arguments),
        },

        FunctionArgs::String(token_reference) => FunctionArgs::String(token_reference.clone()),

        FunctionArgs::TableConstructor(table_constructor) => FunctionArgs::TableConstructor(
            self.replace_in_table_constructor(&table_constructor)),

        &_ => todo!(),
    }
}

fn replace_in_call(self:&Self, call: &Call) -> Call
{
    match call
    {
        Call::AnonymousCall(function_args) => Call::AnonymousCall(self.replace_in_function_args(function_args)),

        Call::MethodCall(method_call) =>
        {
            Call::MethodCall(
            method_call.clone().with_args(self.replace_in_function_args(method_call.args())))
        },

        &_ => todo!(),
    }
}

fn replace_in_prefix(self:&Self, prefix: &Prefix) -> Prefix
{
    match prefix
    {
        Prefix::Expression(expression_box) =>
            Prefix::Expression(Box::new(self.replace_in_expression(&*expression_box))),

        Prefix::Name(token_reference) => Prefix::Name(token_reference.clone()),

        &_ => todo!(),
    }
}

fn replace_in_suffix(self:&Self, suffix: &Suffix) -> Suffix
{
    match suffix
    {
        Suffix::Call(call) => Suffix::Call(self.replace_in_call(call)),
        Suffix::Index(index) => Suffix::Index(index.clone()),

        #[cfg(feature = "luau")]
        Suffix::TypeInstantiation(type_instantiation) => Suffix::TypeInstantiation(
            type_instantiation.clone()),

        &_ => todo!()
    }
}

fn replace_in_function_call(self:&Self, function_call: &FunctionCall) -> FunctionCall
{
    let prefix = self.replace_in_prefix(function_call.prefix());
    let suffixes = function_call.suffixes().map(|suffix| self.replace_in_suffix(suffix)).collect();
    function_call.clone().with_prefix(prefix).with_suffixes(suffixes)
}

fn replace_in_parentheses(self:&Self, contained: &ContainedSpan, expression: &Expression) -> Expression
{
    Expression::Parentheses{
        contained: contained.clone(),
        expression: Box::new(self.replace_in_expression(expression))
    }
}

fn replace_in_unary_operator(self:&Self, unop: &UnOp, expression: &Expression) -> Expression
{
    Expression::UnaryOperator{unop: unop.clone(), expression: Box::new(
        self.replace_in_expression(expression))}
}

fn replace_in_binary_operator(self:&Self, left_expression : &Expression, binop: &BinOp, right_expression : &Expression) -> Expression
{
    Expression::BinaryOperator{
        lhs: Box::new(self.replace_in_expression(left_expression)),
        binop: binop.clone(),
        rhs: Box::new(self.replace_in_expression(right_expression)),
    }
}

fn replace_in_anonymous_function(self:&Self, anonymous_function: &AnonymousFunction) -> Expression
{
    let new_body = self.replace_in_function_body(anonymous_function.body());
    Expression::Function(
        Box::new(anonymous_function.clone().with_body(new_body)))
}

fn replace_in_field(self:&Self, field: &Field) -> Field
{
    match field
    {
        Field::ExpressionKey { brackets, key, equal, value } => Field::ExpressionKey{
            brackets: brackets.clone(),
            key: self.replace_in_expression(&key),
            equal: equal.clone(),
            value: self.replace_in_expression(&value),
        },

        Field::NameKey { key, equal, value } => Field::NameKey{
            key: key.clone(),
            equal: equal.clone(),
            value: self.replace_in_expression(value),
        },

        Field::NoKey(expression) => Field::NoKey(self.replace_in_expression(expression)),

        &_ => todo!(),
    }
}

fn replace_in_punctuated_fields(self:&Self, punctuated_fields: &Punctuated<Field>) -> Punctuated<Field>
{
    let mut new_punctuated_fields = punctuated_fields.clone();
    for pair in new_punctuated_fields.pairs_mut()
    {
        *pair.value_mut() = self.replace_in_field(&pair.value());
    }
    new_punctuated_fields
}

fn replace_in_table_constructor(self:&Self, table_constructor: &TableConstructor) -> TableConstructor
{
    let new_fields = self.replace_in_punctuated_fields(table_constructor.fields());
    table_constructor.clone().with_fields(new_fields)
}

fn replace_in_suffixes<'a>(self:&Self, suffixes: impl Iterator<Item = &'a Suffix>) -> Vec<Suffix>
{
    suffixes.map(|item| {self.replace_in_suffix(item)}).collect()
}

fn replace_in_var_expression(self:&Self, var_expression: &VarExpression) -> VarExpression
{
    var_expression.clone()
        .with_prefix(self.replace_in_prefix(var_expression.prefix()))
        .with_suffixes(self.replace_in_suffixes(var_expression.suffixes()))
}

fn replace_in_var(self:&Self, var: &Var) -> Var
{
    match var
    {
        Var::Expression(var_expression_box) => Var::Expression(
            Box::new(self.replace_in_var_expression(&*var_expression_box))),

        Var::Name(token_reference) => Var::Name(token_reference.clone()),

        &_ => todo!(),
    }
}

fn replace_in_expression(self:&Self, expression: &Expression) -> Expression
{
    for (source, target) in self.expressions.iter()
    {
        if eq_expression(source, expression)
        {
            return target.clone();
        };
    }

    match expression
    {
        Expression::FunctionCall(function_call) =>
            Expression::FunctionCall(self.replace_in_function_call(&function_call)),

        Expression::Parentheses{contained, expression} =>
            self.replace_in_parentheses(&contained, &*expression),

        Expression::UnaryOperator{unop, expression} =>
            self.replace_in_unary_operator(&unop, &expression),

        Expression::BinaryOperator{lhs, binop, rhs} =>
            self.replace_in_binary_operator(&lhs, &binop, &rhs),

        Expression::Function(anonymous_function_box) =>
            self.replace_in_anonymous_function(&*anonymous_function_box),

        Expression::TableConstructor(table_constructor) =>
            Expression::TableConstructor(self.replace_in_table_constructor(table_constructor)),

        Expression::Var(var) =>
            Expression::Var(self.replace_in_var(var)),

        _ => expression.clone(),
    }
}

fn replace_in_punctuated_expressions(self:&Self, punctuated_expressions :&Punctuated<Expression>) -> Punctuated<Expression>
{
    let mut new_punctuated_expressions = punctuated_expressions.clone();
    for pair in new_punctuated_expressions.pairs_mut()
    {
        *pair.value_mut() = self.replace_in_expression(&pair.value());
    }
    new_punctuated_expressions
}

fn replace_in_local_assignment(self:&Self, local_assignment: &LocalAssignment) -> LocalAssignment
{
    local_assignment.clone().with_expressions(
        self.replace_in_punctuated_expressions(local_assignment.expressions()))
}

fn replace_in_assignment(self:&Self, assignment: &Assignment) -> Assignment
{
    assignment.clone().with_expressions(
        self.replace_in_punctuated_expressions(assignment.expressions()))
}

fn replace_in_while_loop(self:&Self, while_loop: &While) -> While
{
    let new_condition = self.replace_in_expression(while_loop.condition());
    let new_block = self.replace_in_block(while_loop.block());
    while_loop.clone()
        .with_condition(new_condition)
        .with_block(new_block)
}

fn replace_in_repeat(self:&Self, repeat: &Repeat) -> Repeat
{
    let new_block = self.replace_in_block(repeat.block());
    let new_until = self.replace_in_expression(repeat.until());

    repeat.clone()
        .with_block(new_block)
        .with_until(new_until)
}

fn replace_in_do(self:&Self, do_obj: &Do) -> Do
{
    let new_block = self.replace_in_block(do_obj.block());
    do_obj.clone()
        .with_block(new_block)
}

fn replace_in_function_body(self:&Self, function_body: &FunctionBody) -> FunctionBody
{
    let new_block = self.replace_in_block(function_body.block());
    function_body.clone().with_block(new_block)
}

fn replace_in_function_declaration(self:&Self, function_declaration: &FunctionDeclaration) -> FunctionDeclaration
{
    let new_body = self.replace_in_function_body(function_declaration.body());
    function_declaration.clone().with_body(new_body)
}

fn replace_in_local_function(self:&Self, local_function: &LocalFunction) -> LocalFunction
{
    let new_body = self.replace_in_function_body(local_function.body());
    local_function.clone().with_body(new_body)
}

fn replace_in_numeric_for(self:&Self, numeric_for: &NumericFor) -> NumericFor
{
    let new_start = self.replace_in_expression(numeric_for.start());
    let new_end = self.replace_in_expression(numeric_for.end());
    let new_block = self.replace_in_block(numeric_for.block());

    numeric_for.clone()
        .with_start(new_start)
        .with_end(new_end)
        .with_block(new_block)
}

fn replace_in_generic_for(self:&Self, generic_for: &GenericFor) -> GenericFor
{
    let new_block = self.replace_in_block(generic_for.block());
    let new_expressions = self.replace_in_punctuated_expressions(generic_for.expressions());

    generic_for.clone()
        .with_block(new_block)
        .with_expressions(new_expressions)
}

fn replace_in_else_ifs(self:&Self, else_ifs: &Vec<ElseIf>) -> Vec<ElseIf>
{
    else_ifs.iter().map(|else_if|{
        else_if.clone()
            .with_condition(self.replace_in_expression(else_if.condition()))
            .with_block(self.replace_in_block(else_if.block()))
    }).collect()
}

fn replace_in_if_object(self:&Self, if_object: &If) -> If
{
    if_object.clone()
        .with_condition(self.replace_in_expression(if_object.condition()))
        .with_block(self.replace_in_block(if_object.block()))
        .with_else_if(match if_object.else_if()
        {
            None => None,
            Some(else_ifs) => Some(self.replace_in_else_ifs(else_ifs)),
        })
        .with_else(match if_object.else_block()
        {
            None => None,
            Some(block) => Some(self.replace_in_block(block)),
        })
}

fn replace_in_statement(self:&Self, statement: &Stmt) -> Stmt
{
    match statement
    {
        Stmt::LocalAssignment(local_assignment) =>
            Stmt::LocalAssignment(self.replace_in_local_assignment(&local_assignment)),

        Stmt::Assignment(assignment) =>
            Stmt::Assignment(self.replace_in_assignment(&assignment)),

        Stmt::If(if_object) =>
            Stmt::If(self.replace_in_if_object(if_object)),

        Stmt::While(while_loop) =>
            Stmt::While(self.replace_in_while_loop(&while_loop)),

        Stmt::FunctionCall(function_call) =>
            Stmt::FunctionCall(self.replace_in_function_call(&function_call)),

        Stmt::FunctionDeclaration(function_declaration) =>
            Stmt::FunctionDeclaration(self.replace_in_function_declaration(&function_declaration)),

        Stmt::LocalFunction(local_function) =>
            Stmt::LocalFunction(self.replace_in_local_function(&local_function)),

        Stmt::NumericFor(numeric_for) =>
            Stmt::NumericFor(self.replace_in_numeric_for(&numeric_for)),

        Stmt::GenericFor(generic_for) =>
            Stmt::GenericFor(self.replace_in_generic_for(&generic_for)),

        Stmt::Repeat(repeat) =>
            Stmt::Repeat(self.replace_in_repeat(&repeat)),

        Stmt::Do(do_block) =>
            Stmt::Do(self.replace_in_do(&do_block)),

        _ => statement.clone(),
    }
}

fn replace_in_block(self:&Self, block: &Block) -> Block
{
    let mut new_stmts = vec![];

    for (statement, token_reference) in block.stmts_with_semicolon()
    {
        new_stmts.push((
            self.replace_in_statement(&statement),
            token_reference.clone()
        ));
    }

    block.clone().with_stmts(new_stmts)
}

pub fn replace_in_ast(self:&Self, input_ast: &Ast) -> Ast
{
    input_ast.clone().with_nodes(self.replace_in_block(input_ast.nodes()))
}
}

#[cfg(test)]
mod tests
{
use full_moon::parse_fallible;
use crate::Config;
use crate::replace::Replacer;
use crate::OutputVerification;
use crate::format_ast;
use full_moon::ast::Expression;
use full_moon::tokenizer::Token;
use full_moon::tokenizer::TokenType;
use full_moon::tokenizer::TokenReference;

fn replace_code(replacer: Replacer, code: &str) -> String
{
    let config = Config::default();
    format_ast(replacer.replace_in_ast(&parse_fallible(code, config.syntax.into()).into_result().unwrap()),
        config, None, OutputVerification::None).unwrap().to_string()
}

#[test]
fn assignment_replace_number()
{
    assert_eq!(replace_code(Replacer{expressions:vec![
            (Expression::Number(TokenReference::new(vec![], Token::new(TokenType::Number { text: "1".into() }), vec![])),
            Expression::Number(TokenReference::new(vec![], Token::new(TokenType::Number { text: "2".into() }), vec![])))
        ]},
        "local x = 1\n"),
        "local x = 2\n");
}

}
