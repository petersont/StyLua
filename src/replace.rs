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

struct Replacer
{
    expressions: Vec<(Expression, Expression)>
}

impl Replacer
{

fn replace_function_args(self:&Self, function_args: &FunctionArgs) -> FunctionArgs
{
    match function_args
    {
        FunctionArgs::Parentheses {parentheses, arguments} => FunctionArgs::Parentheses {
            parentheses: parentheses.clone(),
            arguments: self.replace_punctuated_expressions(arguments),
        },

        FunctionArgs::String(token_reference) => FunctionArgs::String(token_reference.clone()),

        FunctionArgs::TableConstructor(table_constructor) => FunctionArgs::TableConstructor(
            self.replace_table_constructor(&table_constructor)),

        &_ => todo!(),
    }
}

fn replace_call(self:&Self, call: &Call) -> Call
{
    match call
    {
        Call::AnonymousCall(function_args) => Call::AnonymousCall(self.replace_function_args(function_args)),

        Call::MethodCall(method_call) =>
        {
            Call::MethodCall(
            method_call.clone().with_args(self.replace_function_args(method_call.args())))
        },

        &_ => todo!(),
    }
}

fn replace_prefix(self:&Self, prefix: &Prefix) -> Prefix
{
    match prefix
    {
        Prefix::Expression(expression_box) =>
            Prefix::Expression(Box::new(self.replace_expression(&*expression_box))),

        Prefix::Name(token_reference) => Prefix::Name(token_reference.clone()),

        &_ => todo!(),
    }
}

fn replace_suffix(self:&Self, suffix: &Suffix) -> Suffix
{
    match suffix
    {
        Suffix::Call(call) => Suffix::Call(self.replace_call(call)),
        Suffix::Index(index) => Suffix::Index(index.clone()),

        #[cfg(feature = "luau")]
        Suffix::TypeInstantiation(type_instantiation) => Suffix::TypeInstantiation(
            type_instantiation.clone()),

        &_ => todo!()
    }
}

fn replace_function_call(self:&Self, function_call: &FunctionCall) -> FunctionCall
{
    let prefix = self.replace_prefix(function_call.prefix());
    let suffixes = function_call.suffixes().map(|suffix| self.replace_suffix(suffix)).collect();
    function_call.clone().with_prefix(prefix).with_suffixes(suffixes)
}

fn replace_parentheses(self:&Self, contained: &ContainedSpan, expression: &Expression) -> Expression
{
    Expression::Parentheses{
        contained: contained.clone(),
        expression: Box::new(self.replace_expression(expression))
    }
}

fn replace_unary_operator(self:&Self, unop: &UnOp, expression: &Expression) -> Expression
{
    Expression::UnaryOperator{unop: unop.clone(), expression: Box::new(
        self.replace_expression(expression))}
}

fn replace_binary_operator(self:&Self, left_expression : &Expression, binop: &BinOp, right_expression : &Expression) -> Expression
{
    Expression::BinaryOperator{
        lhs: Box::new(self.replace_expression(left_expression)),
        binop: binop.clone(),
        rhs: Box::new(self.replace_expression(right_expression)),
    }
}

fn replace_anonymous_function(self:&Self, anonymous_function: &AnonymousFunction) -> Expression
{
    let new_body = self.replace_function_body(anonymous_function.body());
    Expression::Function(
        Box::new(anonymous_function.clone().with_body(new_body)))
}

fn replace_field(self:&Self, field: &Field) -> Field
{
    match field
    {
        Field::ExpressionKey { brackets, key, equal, value } => Field::ExpressionKey{
            brackets: brackets.clone(),
            key: self.replace_expression(&key),
            equal: equal.clone(),
            value: self.replace_expression(&value),
        },

        Field::NameKey { key, equal, value } => Field::NameKey{
            key: key.clone(),
            equal: equal.clone(),
            value: self.replace_expression(value),
        },

        Field::NoKey(expression) => Field::NoKey(self.replace_expression(expression)),

        &_ => todo!(),
    }
}

fn replace_punctuated_fields(self:&Self, punctuated_fields: &Punctuated<Field>) -> Punctuated<Field>
{
    let mut new_punctuated_fields = punctuated_fields.clone();
    for pair in new_punctuated_fields.pairs_mut()
    {
        *pair.value_mut() = self.replace_field(&pair.value());
    }
    new_punctuated_fields
}

fn replace_table_constructor(self:&Self, table_constructor: &TableConstructor) -> TableConstructor
{
    let new_fields = self.replace_punctuated_fields(table_constructor.fields());
    table_constructor.clone().with_fields(new_fields)
}

fn replace_suffixes<'a>(self:&Self, suffixes: impl Iterator<Item = &'a Suffix>) -> Vec<Suffix>
{
    suffixes.map(|item| {self.replace_suffix(item)}).collect()
}

fn replace_var_expression(self:&Self, var_expression: &VarExpression) -> VarExpression
{
    var_expression.clone()
        .with_prefix(self.replace_prefix(var_expression.prefix()))
        .with_suffixes(self.replace_suffixes(var_expression.suffixes()))
}

fn replace_var(self:&Self, var: &Var) -> Var
{
    match var
    {
        Var::Expression(var_expression_box) => Var::Expression(
            Box::new(self.replace_var_expression(&*var_expression_box))),

        Var::Name(token_reference) => Var::Name(token_reference.clone()),

        &_ => todo!(),
    }
}

fn replace_expression(self:&Self, expression: &Expression) -> Expression
{
    match expression
    {
        Expression::FunctionCall(function_call) =>
            Expression::FunctionCall(self.replace_function_call(&function_call)),

        Expression::Parentheses{contained, expression} =>
            self.replace_parentheses(&contained, &*expression),

        Expression::UnaryOperator{unop, expression} =>
            self.replace_unary_operator(&unop, &expression),

        Expression::BinaryOperator{lhs, binop, rhs} =>
            self.replace_binary_operator(&lhs, &binop, &rhs),

        Expression::Function(anonymous_function_box) =>
            self.replace_anonymous_function(&*anonymous_function_box),

        Expression::TableConstructor(table_constructor) =>
            Expression::TableConstructor(self.replace_table_constructor(table_constructor)),

        Expression::Var(var) =>
            Expression::Var(self.replace_var(var)),

        _ => expression.clone(),
    }
}

fn replace_punctuated_expressions(self:&Self, punctuated_expressions :&Punctuated<Expression>) -> Punctuated<Expression>
{
    let mut new_punctuated_expressions = punctuated_expressions.clone();
    for pair in new_punctuated_expressions.pairs_mut()
    {
        *pair.value_mut() = self.replace_expression(&pair.value());
    }
    new_punctuated_expressions
}

fn replace_local_assignment(self:&Self, local_assignment: &LocalAssignment) -> LocalAssignment
{
    local_assignment.clone().with_expressions(
        self.replace_punctuated_expressions(local_assignment.expressions()))
}

fn replace_assignment(self:&Self, assignment: &Assignment) -> Assignment
{
    assignment.clone().with_expressions(
        self.replace_punctuated_expressions(assignment.expressions()))
}

fn replace_while_loop(self:&Self, while_loop: &While) -> While
{
    let new_condition = self.replace_expression(while_loop.condition());
    let new_block = self.replace_block(while_loop.block());
    while_loop.clone()
        .with_condition(new_condition)
        .with_block(new_block)
}

fn replace_repeat(self:&Self, repeat: &Repeat) -> Repeat
{
    let new_block = self.replace_block(repeat.block());
    let new_until = self.replace_expression(repeat.until());

    repeat.clone()
        .with_block(new_block)
        .with_until(new_until)
}

fn replace_do(self:&Self, do_obj: &Do) -> Do
{
    let new_block = self.replace_block(do_obj.block());
    do_obj.clone()
        .with_block(new_block)
}

fn replace_function_body(self:&Self, function_body: &FunctionBody) -> FunctionBody
{
    let new_block = self.replace_block(function_body.block());
    function_body.clone().with_block(new_block)
}

fn replace_function_declaration(self:&Self, function_declaration: &FunctionDeclaration) -> FunctionDeclaration
{
    let new_body = self.replace_function_body(function_declaration.body());
    function_declaration.clone().with_body(new_body)
}

fn replace_local_function(self:&Self, local_function: &LocalFunction) -> LocalFunction
{
    let new_body = self.replace_function_body(local_function.body());
    local_function.clone().with_body(new_body)
}

fn replace_numeric_for(self:&Self, numeric_for: &NumericFor) -> NumericFor
{
    let new_start = self.replace_expression(numeric_for.start());
    let new_end = self.replace_expression(numeric_for.end());
    let new_block = self.replace_block(numeric_for.block());

    numeric_for.clone()
        .with_start(new_start)
        .with_end(new_end)
        .with_block(new_block)
}

fn replace_generic_for(self:&Self, generic_for: &GenericFor) -> GenericFor
{
    let new_block = self.replace_block(generic_for.block());
    let new_expressions = self.replace_punctuated_expressions(generic_for.expressions());

    generic_for.clone()
        .with_block(new_block)
        .with_expressions(new_expressions)
}

fn replace_else_ifs(self:&Self, else_ifs: &Vec<ElseIf>) -> Vec<ElseIf>
{
    else_ifs.iter().map(|else_if|{
        else_if.clone()
            .with_condition(self.replace_expression(else_if.condition()))
            .with_block(self.replace_block(else_if.block()))
    }).collect()
}

fn replace_if_object(self:&Self, if_object: &If) -> If
{
    if_object.clone()
        .with_condition(self.replace_expression(if_object.condition()))
        .with_block(self.replace_block(if_object.block()))
        .with_else_if(match if_object.else_if()
        {
            None => None,
            Some(else_ifs) => Some(self.replace_else_ifs(else_ifs)),
        })
        .with_else(match if_object.else_block()
        {
            None => None,
            Some(block) => Some(self.replace_block(block)),
        })
}

fn replace_statement(self:&Self, statement: &Stmt) -> Stmt
{
    match statement
    {
        Stmt::LocalAssignment(local_assignment) =>
            Stmt::LocalAssignment(self.replace_local_assignment(&local_assignment)),

        Stmt::Assignment(assignment) =>
            Stmt::Assignment(self.replace_assignment(&assignment)),

        Stmt::If(if_object) =>
            Stmt::If(self.replace_if_object(if_object)),

        Stmt::While(while_loop) =>
            Stmt::While(self.replace_while_loop(&while_loop)),

        Stmt::FunctionCall(function_call) =>
            Stmt::FunctionCall(self.replace_function_call(&function_call)),

        Stmt::FunctionDeclaration(function_declaration) =>
            Stmt::FunctionDeclaration(self.replace_function_declaration(&function_declaration)),

        Stmt::LocalFunction(local_function) =>
            Stmt::LocalFunction(self.replace_local_function(&local_function)),

        Stmt::NumericFor(numeric_for) =>
            Stmt::NumericFor(self.replace_numeric_for(&numeric_for)),

        Stmt::GenericFor(generic_for) =>
            Stmt::GenericFor(self.replace_generic_for(&generic_for)),

        Stmt::Repeat(repeat) =>
            Stmt::Repeat(self.replace_repeat(&repeat)),

        Stmt::Do(do_block) =>
            Stmt::Do(self.replace_do(&do_block)),

        _ => statement.clone(),
    }
}

fn replace_block(self:&Self, block: &Block) -> Block
{
    let mut new_stmts = vec![];

    for (statement, token_reference) in block.stmts_with_semicolon()
    {
        new_stmts.push((
            self.replace_statement(&statement),
            token_reference.clone()
        ));
    }

    block.clone().with_stmts(new_stmts)
}

pub fn replace_ast(self:&Self, input_ast: &Ast) -> Ast
{
    input_ast.clone().with_nodes(self.replace_block(input_ast.nodes()))
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

fn replace_code(code: &str) -> String
{
    let config = Config::default();
    let replacer = Replacer{expressions:vec![]};
    format_ast(replacer.replace_ast(&parse_fallible(code, config.syntax.into()).into_result().unwrap()),
        config, None, OutputVerification::None).unwrap().to_string()
}

fn input_output(input: &str, expected_output: &str)
{
    assert_eq!(replace_code(input), expected_output);
}

#[test]
fn assignment() {
    println!("Hey");
    input_output(
        "local x = 1\n",
        "local x = 1\n");
}

}
