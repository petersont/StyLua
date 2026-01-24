#[cfg(test)]
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
use full_moon::ast::MethodCall;
use full_moon::ast::ElseIf;
use full_moon::ast::punctuated::Punctuated;
use full_moon::ast::punctuated::Pair;
use full_moon::ast::span::ContainedSpan;
use full_moon::tokenizer::TokenReference;

fn eq_function_args(a: &FunctionArgs, b: &FunctionArgs) -> bool
{
    match (a, b)
    {
        (
            FunctionArgs::Parentheses {parentheses:_a_parentheses, arguments:a_arguments},
            FunctionArgs::Parentheses {parentheses:_b_parentheses, arguments:b_arguments}
        ) => 
        {
            eq_punctuated_expressions(a_arguments, b_arguments)
        },

        (
            FunctionArgs::String(a_token_reference),
            FunctionArgs::String(b_token_reference)
        ) =>
        {
            eq_token_reference(a_token_reference, b_token_reference)
        },

        (
            FunctionArgs::TableConstructor(a_table_constructor),
            FunctionArgs::TableConstructor(b_table_constructor)
        ) =>
        {
            eq_table_constructor(a_table_constructor, b_table_constructor)
        },

        (&_, &_) => false,
    }
}

fn eq_call(a: &Call, b: &Call) -> bool
{
    match (a, b)
    {
        (
            Call::AnonymousCall(a_function_args),
            Call::AnonymousCall(b_function_args)
        ) => 
            eq_function_args(a_function_args, b_function_args),

        (
            Call::MethodCall(a_method_call),
            Call::MethodCall(b_method_call)
        ) =>
            eq_method_call(a_method_call, b_method_call),

        (&_, &_) => todo!(),
    }
}

fn eq_prefix(a: &Prefix, b: &Prefix) -> bool
{
    match (a, b)
    {
        (
            Prefix::Expression(a_expression_box),
            Prefix::Expression(b_expression_box)
        ) =>
        {
            eq_expression(&*a_expression_box, &*b_expression_box)
        },

        (
            Prefix::Name(a_token_reference),
            Prefix::Name(b_token_reference)
        ) =>
        {
            a_token_reference == b_token_reference
        },

        (&_, &_) => todo!(),
    }
}

fn eq_method_call(a: &MethodCall, b: &MethodCall) -> bool
{
    eq_token_reference(a.name(), b.name())
        && eq_function_args(a.args(), b.args())
}

fn eq_suffix(a: &Suffix, b: &Suffix) -> bool
{
    match (a, b)
    {
        (
            Suffix::Call(a_call),
            Suffix::Call(b_call)
        ) =>
            eq_call(a_call, b_call),

        (
            Suffix::Index(_a_index),
            Suffix::Index(_b_index),
        ) =>
        {
            todo!();
        },

        #[cfg(feature = "luau")]
        (
            Suffix::TypeInstantiation(a_type_instantiation),
            Suffix::TypeInstantiation(b_type_instantiation)
        ) =>
        {
            todo!();
        },

        (&_, &_) => 
        {
            panic!("unexpected something else as suffix");
        }
    }
}

fn eq_function_call(a: &FunctionCall, b: &FunctionCall) -> bool
{
    eq_prefix(a.prefix(), b.prefix()) && eq_suffixes(a.suffixes(), b.suffixes())
}

fn eq_parentheses(
    _a_contained: &ContainedSpan, a_expression: &Expression,
    _b_contained: &ContainedSpan, b_expression: &Expression) -> bool
{
    return eq_expression(a_expression, b_expression);
}

fn eq_unary_operator(
    a_unop: &UnOp, a_expression: &Expression,
    b_unop: &UnOp, b_expression: &Expression) -> bool
{
    a_unop == b_unop 
    && eq_expression(a_expression, b_expression)
}

fn eq_binary_operator(
    a_left_expression : &Expression, a_binop: &BinOp, a_right_expression : &Expression,
    b_left_expression : &Expression, b_binop: &BinOp, b_right_expression : &Expression) -> bool
{
    a_binop == b_binop
    && eq_expression(a_left_expression, b_left_expression)
    && eq_expression(a_right_expression, b_right_expression)
}

fn eq_anonymous_function(
    a: &AnonymousFunction,
    b: &AnonymousFunction) -> bool
{
    eq_function_body(a.body(), b.body())
}

fn eq_field(a: &Field, b: &Field) -> bool
{
    match (a, b)
    {
        (
            Field::ExpressionKey { brackets:_a_brackets, key:a_key, equal:_a_equal, value:a_value },
            Field::ExpressionKey { brackets:_b_brackets, key:b_key, equal:_b_equal, value:b_value }
        ) =>
        {
            eq_expression(a_key, b_key) && eq_expression(a_value, b_value)
        },

        (
            Field::NameKey { key:a_key, equal:_a_equal, value:a_value },
            Field::NameKey { key:b_key, equal:_b_equal, value:b_value }
        ) =>
        {
            eq_token_reference(a_key, b_key) && eq_expression(a_value, b_value)
        },

        (
            Field::NoKey(a_expression),
            Field::NoKey(b_expression)
        )
        =>
        {
            eq_expression(a_expression, b_expression)
        },

        _ => false,
    }
}

fn eq_punctuated_fields(a: &Punctuated<Field>, b: &Punctuated<Field>) -> bool
{
    if a.len() != b.len()
    {
        return false;
    }

    for (a_pair, b_pair) in a.pairs().zip(b.pairs())
    {
        if !eq_field(a_pair.value(), b_pair.value())
        {
            return false;
        }
    }

    return true;
}

fn eq_table_constructor(a: &TableConstructor, b: &TableConstructor) -> bool
{
    eq_punctuated_fields(a.fields(), b.fields())
}

fn eq_suffixes<'a>(
    a_suffixes: impl Iterator<Item = &'a Suffix>,
    b_suffixes: impl Iterator<Item = &'a Suffix>) -> bool
{
    let a_vec: Vec<&Suffix> = a_suffixes.collect();
    let b_vec: Vec<&Suffix> = b_suffixes.collect();

    if a_vec.len() != b_vec.len()
    {
        return false;
    }

    for (ax, bx) in a_vec.iter().zip(b_vec.iter())
    {
        if ! eq_suffix(ax, bx)
        {
            return false;
        }
    }

    return true;
}

fn eq_var_expression(a: &VarExpression, b: &VarExpression) -> bool
{
    if ! eq_prefix(a.prefix(), b.prefix())
    {
        return false;
    }

    eq_suffixes(a.suffixes(), b.suffixes())
}

fn eq_var(a: &Var, b: &Var) -> bool
{
    match (a, b)
    {
        (
            Var::Expression(a_var_expression_box),
            Var::Expression(b_var_expression_box)
        ) =>
        {
            eq_var_expression(&*a_var_expression_box, &*b_var_expression_box)
        }

        (
            Var::Name(a_token_reference),
            Var::Name(b_token_reference)
        ) =>
        {
            eq_token_reference(a_token_reference, b_token_reference)
        },

        (&_, &_) => false,
    }
}

pub fn eq_expression(a: &Expression, b: &Expression) -> bool
{
    match (a, b)
    {
        (
            Expression::FunctionCall(a_function_call),
            Expression::FunctionCall(b_function_call)
        ) =>
        {
            eq_function_call(a_function_call, b_function_call)
        },

        (
            Expression::Parentheses{contained:a_contained, expression:a_expression},
            Expression::Parentheses{contained:b_contained, expression:b_expression}
        ) =>
        {
            eq_parentheses(
                &a_contained, &*a_expression,
                &b_contained, &*b_expression)
        },

        (
            Expression::UnaryOperator{unop:a_unop, expression:a_expression},
            Expression::UnaryOperator{unop:b_unop, expression:b_expression}
        ) =>
        {
            eq_unary_operator(&a_unop, &a_expression, &b_unop, &b_expression)
        },

        (
            Expression::BinaryOperator{lhs:a_lhs, binop:a_binop, rhs:a_rhs},
            Expression::BinaryOperator{lhs:b_lhs, binop:b_binop, rhs:b_rhs}
        ) =>
        {
            eq_binary_operator(
                &a_lhs, &a_binop, &a_rhs,
                &b_lhs, &b_binop, &b_rhs)
        },

        (
            Expression::Function(a_anonymous_function_box),
            Expression::Function(b_anonymous_function_box)
        ) =>
            eq_anonymous_function(
                &*a_anonymous_function_box,
                &*b_anonymous_function_box),

        (
            Expression::TableConstructor(a_table_constructor),
            Expression::TableConstructor(b_table_constructor)
        ) =>
            eq_table_constructor(
                &a_table_constructor,
                &b_table_constructor),

        (
            Expression::Number(a_token_reference),
            Expression::Number(b_token_reference)
        ) =>
            eq_token_reference(&a_token_reference, &b_token_reference),

        (
            Expression::String(a_token_reference),
            Expression::String(b_token_reference)
        ) =>
        {
            eq_token_reference(&a_token_reference, &b_token_reference)
        },

        (
            Expression::Symbol(a_token_reference),
            Expression::Symbol(b_token_reference)
        ) =>
            eq_token_reference(&a_token_reference, &b_token_reference),

        (
            Expression::Var(a_var),
            Expression::Var(b_var)
        ) =>
            eq_var(&a_var, &b_var),

        _ =>
        {
            // so far only luau stuff is not handled
            false
        },
    }
}

fn eq_punctuated_expressions(a :&Punctuated<Expression>, b :&Punctuated<Expression>) -> bool
{
    if a.len() != b.len()
    {
        return false;
    }

    for (a_pair, b_pair) in a.pairs().zip(b.pairs())
    {
        if ! eq_expression(a_pair.value(), b_pair.value())
        {
            return false;
        }
    }
    true
}

fn eq_token_reference(
    a: &TokenReference,
    b: &TokenReference) -> bool
{
    a.token().to_string() == b.token().to_string()
}

fn eq_punctuated_token_reference(
    a: &Punctuated<TokenReference>,
    b: &Punctuated<TokenReference>) -> bool
{
    let a_pair_vec : Vec<&Pair<TokenReference>> = a.pairs().collect();
    let b_pair_vec : Vec<&Pair<TokenReference>> = a.pairs().collect();

    if a_pair_vec.len() != b_pair_vec.len()
    {
        return false;
    }

    for (a_pair, b_pair) in a.pairs().zip(b.pairs())
    {
        if ! eq_token_reference(a_pair.value(), b_pair.value())
        {
            return false;
        }
    }
    true
}

fn eq_local_assignment(a: &LocalAssignment, b: &LocalAssignment) -> bool
{
    if ! eq_punctuated_token_reference(a.names(), b.names())
    {
        return false;
    }

    if a.expressions().len() != b.expressions().len()
    {
        return false;
    }

    for (ax, bx) in a.expressions().pairs().zip(b.expressions().pairs())
    {
        if ! eq_expression(ax.value(), bx.value())
        {
            return false;
        }
    }

    true
}

fn eq_punctuated_var(
    a: &Punctuated<Var>,
    b: &Punctuated<Var>) -> bool
{
    let a_pair_vec : Vec<&Pair<Var>> = a.pairs().collect();
    let b_pair_vec : Vec<&Pair<Var>> = a.pairs().collect();

    if a_pair_vec.len() != b_pair_vec.len()
    {
        return false;
    }

    for (a_pair, b_pair) in a.pairs().zip(b.pairs())
    {
        if ! eq_var(a_pair.value(), b_pair.value())
        {
            return false;
        }
    }
    true
}

fn eq_assignment(a: &Assignment, b: &Assignment) -> bool
{
    if ! eq_punctuated_var(a.variables(), b.variables())
    {
        return false;
    }

    if a.expressions().len() != b.expressions().len()
    {
        return false;
    }

    for (ax, bx) in a.expressions().pairs().zip(b.expressions().pairs())
    {
        if ! eq_expression(ax.value(), bx.value())
        {
            return false;
        }
    }

    true
}

fn eq_else_if(a: &ElseIf, b: &ElseIf) -> bool
{
    eq_expression(a.condition(), b.condition())
        && eq_block(a.block(), b.block())
}

fn eq_else_if_vec(a: &Vec<ElseIf>, b: &Vec<ElseIf>) -> bool
{
    println!("Making it here");

    if a.len() != b.len()
    {
        return false
    }

    for (else_if_a, else_if_b) in a.iter().zip(b.iter())
    {
        if ! eq_else_if(else_if_a, else_if_b)
        {
            return false;
        }
    }

    true
}

fn eq_if_statement(a: &If, b: &If) -> bool
{
    eq_expression(a.condition(), b.condition())
        && eq_block(a.block(), b.block())
        && match (a.else_if(), b.else_if())
        {
            (None, None) => true,
            (Some(a_else_ifs), Some(b_else_ifs)) =>
                eq_else_if_vec(a_else_ifs, b_else_ifs),
            _ => false,
        }
        && match (a.else_block(), b.else_block())
        {
            (None, None) => true,
            (Some(a_else_block), Some(b_else_block)) =>
                eq_block(a_else_block, b_else_block),
            _ => false,
        }
}

fn eq_while_loop(a: &While, b: &While) -> bool
{
    eq_expression(a.condition(), b.condition()) && eq_block(a.block(), b.block())
}

fn eq_repeat(a: &Repeat, b: &Repeat) -> bool
{
    eq_expression(a.until(), b.until()) && eq_block(a.block(), b.block())
}

fn eq_do(a: &Do, b: &Do) -> bool
{
    eq_block(a.block(), b.block())
}

fn eq_function_body(a: &FunctionBody, b: &FunctionBody) -> bool
{
    eq_block(a.block(), b.block())
}

fn eq_function_declaration(a: &FunctionDeclaration, b: &FunctionDeclaration) -> bool
{
    a.name() == b.name() && eq_function_body(a.body(), b.body())
}

fn eq_local_function(a: &LocalFunction, b: &LocalFunction) -> bool
{
    a.name() == b.name() && eq_function_body(a.body(), b.body())
}

fn eq_numeric_for(a: &NumericFor, b: &NumericFor) -> bool
{
    eq_expression(a.start(), b.start())
        && eq_expression(a.end(), b.end())
        && eq_block(a.block(), b.block())
}

fn eq_generic_for(a: &GenericFor, b: &GenericFor) -> bool
{
    eq_punctuated_token_reference(a.names(), b.names())
        && eq_punctuated_expressions(a.expressions(), b.expressions())
        && eq_block(a.block(), b.block())
}

fn eq_statement(a: &Stmt, b: &Stmt) -> bool
{
    match (a,b)
    {
        (
            Stmt::LocalAssignment(a_local_assignment),
            Stmt::LocalAssignment(b_local_assignment)
        ) =>
            eq_local_assignment(&a_local_assignment, &b_local_assignment),

        (
            Stmt::Assignment(a_assignment),
            Stmt::Assignment(b_assignment)
        ) =>
            eq_assignment(&a_assignment, &b_assignment),

        (
            Stmt::If(a_if),
            Stmt::If(b_if)
        ) =>
            eq_if_statement(&a_if, &b_if),

        (
            Stmt::While(a_while),
            Stmt::While(b_while)
        ) =>
            eq_while_loop(&a_while, &b_while),

        (
            Stmt::FunctionCall(a_function_call),
            Stmt::FunctionCall(b_function_call)
        ) =>
        {
            eq_function_call(&a_function_call, &b_function_call)
        },

        (
            Stmt::FunctionDeclaration(a_function_declaration),
            Stmt::FunctionDeclaration(b_function_declaration)
        ) =>
            eq_function_declaration(&a_function_declaration, &b_function_declaration),

        (
            Stmt::LocalFunction(a_local_function),
            Stmt::LocalFunction(b_local_function)
        ) =>
            eq_local_function(&a_local_function, &b_local_function),

        (
            Stmt::NumericFor(a_numeric_for),
            Stmt::NumericFor(b_numeric_for)
        ) =>
            eq_numeric_for(&a_numeric_for, &b_numeric_for),

        (
            Stmt::GenericFor(a_generic_for),
            Stmt::GenericFor(b_generic_for)
        ) =>
            eq_generic_for(&a_generic_for, &b_generic_for),

        (
            Stmt::Repeat(a_repeat),
            Stmt::Repeat(b_repeat)
        ) =>
            eq_repeat(&a_repeat, b_repeat),

        (
            Stmt::Do(a_do),
            Stmt::Do(b_do)
        ) =>
            eq_do(&a_do, &b_do),

        _ => false,
    }
}

fn eq_block(a: &Block, b: &Block) -> bool
{
    let a_stmts_with_semicolon_vec: Vec<&(Stmt, Option<TokenReference>)> = a.stmts_with_semicolon().collect();
    let b_stmts_with_semicolon_vec: Vec<&(Stmt, Option<TokenReference>)> = a.stmts_with_semicolon().collect();

    if a_stmts_with_semicolon_vec.len() != b_stmts_with_semicolon_vec.len()
    {
        return false;
    }

    for ((a_statement, _a_token_reference), (b_statement, _b_token_reference)) in a.stmts_with_semicolon().zip(b.stmts_with_semicolon())
    {
        if !eq_statement(a_statement, b_statement)
        {
            return false;
        }
    }

    true
}

#[cfg(test)]
pub fn eq_ast(a: &Ast, b: &Ast) -> bool
{
    eq_block(a.nodes(), b.nodes())
}

#[cfg(test)]
mod tests
{
use full_moon::parse_fallible;
use crate::equivalent::eq_ast;
use crate::Config;

fn eq_code(a: &str, b: &str) -> bool
{
    let config = Config::default();
    eq_ast(
        &parse_fallible(a, config.syntax.into()).into_result().unwrap(),
        &parse_fallible(b, config.syntax.into()).into_result().unwrap())
}

#[test]
fn call_equal()
{
    assert!(eq_code("foo()", "foo()"))
}

#[test]
fn call_names_not_equal()
{
    assert!(!eq_code("bar()", "foo()"))
}

#[test]
fn call_args_equal()
{
    assert!(eq_code("foo(x, y)", "foo(x, y)"))
}

#[test]
fn call_args_not_equal()
{
    assert!(!eq_code("foo(x, 3)", "foo(x, y)"))
}

#[test]
fn call_args_equal_strings()
{
    assert!(eq_code("foo\"apple\"", "foo\"apple\""))
}

#[test]
fn call_args_not_equal_strings()
{
    assert!(!eq_code("foo\"apple\"", "foo\"banana\""))
}

#[test]
fn call_args_equal_tables()
{
    assert!(eq_code("foo{x=1, y=2}", "foo{x=1, y=2}"))
}

#[test]
fn call_args_not_equal_numbers_in_tables()
{
    assert!(!eq_code("foo{x=1, y=3}", "foo{x=1, y=2}"))
}

#[test]
fn call_args_equal_strings_in_tables()
{
    assert!(eq_code("foo{x=1, y=\"apples\"}", "foo{x=1, y=\"apples\"}"))
}

#[test]
fn call_args_not_equal_strings_in_tables()
{
    assert!(!eq_code("foo{x=1, y=\"apples\"}", "foo{x=1, y=\"bananas\"}"))
}

#[test]
fn call_args_equal_lists()
{
    assert!(eq_code("foo{x, y}", "foo{x, y}"))
}

#[test]
fn call_args_not_equal_single_var_arg()
{
    assert!(!eq_code("foo(x)", "foo(y)"))
}

#[test]
fn call_args_not_equal_lists()
{
    assert!(!eq_code("foo{x, y}", "foo{x}"))
}

#[test]
fn call_args_not_equal_tables_switch()
{
    assert!(!eq_code("foo{y=2, x=1}", "foo{x=1, y=2}"))
}

#[test]
fn method_call_equal()
{
    assert!(eq_code("obj:foo()", "obj:foo()"))
}

#[test]
fn method_call_names_not_equal()
{
    assert!(!eq_code("obj:foo()", "subj:foo()"))
}

#[test]
fn method_call_function_not_equal()
{
    assert!(!eq_code("obj:foo()", "subj:foo()"))
}

#[test]
fn method_call_with_string_and_bool_equal()
{
    assert!(eq_code("obj:foo(\"name\", false)", "obj:foo(  \"name\",  false )  "))
}

#[test]
fn method_call_with_string_and_bool_not_equal()
{
    assert!(!eq_code("obj:foo(\"name\", false)", "obj:foo(\"name\", true)"))
}

#[test]
fn method_call_with_string_not_equal_and_bool()
{
    assert!(!eq_code("obj:foo(\"name\", false)", "obj:foo(\"other\", false)"))
}

#[test]
fn if_statement_equal()
{
    assert!(eq_code("if true then print(\"true\") end", "if true then print(\"true\") end"))
}

#[test]
fn if_statement_not_equal_condition()
{
    assert!(!eq_code("if false then print(\"true\") end", "if true then print(\"true\") end"))
}

#[test]
fn if_statement_not_equal_body()
{
    assert!(!eq_code("if false then print(\"true\") end", "if true then print(\"true\") end"))
}

#[test]
fn if_statement_with_elseif_equal()
{
    assert!(eq_code("\
if false then
    print(\"false\")
elseif is_true() then
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
elseif is_true() then
    print(\"is_true\")
end"))
}

#[test]
fn if_statement_with_elseif_condition_not_equal()
{
    assert!(!eq_code("\
if false then
    print(\"false\")
elseif is_false() then
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
elseif is_true() then
    print(\"is_true\")
end"))
}

#[test]
fn if_statement_with_elseif_body_not_equal()
{
    assert!(!eq_code("\
if false then
    print(\"false\")
elseif is_true() then
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
elseif is_true() then
    print(\"is_false\")
end"))
}

#[test]
fn if_statement_with_elseif_vs_without()
{
    assert!(!eq_code("\
if false then
    print(\"false\")
elseif is_false() then
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
end"))
}

#[test]
fn if_statement_with_else_equal()
{
    assert!(eq_code("\
if false then
    print(\"false\")
else
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
else
    print(\"is_true\")
end"))
}

#[test]
fn if_statement_with_else_vs_without()
{
    assert!(!eq_code("\
if false then
    print(\"false\")
else
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
end"))
}

#[test]
fn if_statement_with_else_not_equal_body()
{
    assert!(!eq_code("\
if false then
    print(\"false\")
else
    print(\"is_true\")
end",
    "\
if false then
    print(\"false\")
else
    print(\"is_false\")
end"))
}

}
