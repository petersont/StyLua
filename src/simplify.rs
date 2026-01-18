use full_moon::ast::Ast;
use full_moon::ast::punctuated::Punctuated;
use full_moon::ast::punctuated::Pair;
use full_moon::ast::BinOp::Or;
use full_moon::ast::BinOp::And;
use full_moon::ast::UnOp::Not;
use full_moon::ast::FunctionCall;

use full_moon::ast::Expression;
use full_moon::ast::Prefix;
use full_moon::ast::Suffix;
use full_moon::ast::Call;
use full_moon::ast::FunctionArgs;
use full_moon::ast::LocalAssignment;
use full_moon::ast::If;
use full_moon::ast::ElseIf;
use full_moon::ast::Do;
use full_moon::ast::While;
use full_moon::ast::FunctionDeclaration;
use full_moon::ast::LocalFunction;
use full_moon::ast::FunctionBody;
use full_moon::ast::Stmt;
use full_moon::ast::Block;
use full_moon::ast::UnOp;
use full_moon::ast::BinOp;
use full_moon::ast::span::ContainedSpan;

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
        Call::MethodCall(method_call) =>
        {
            Call::MethodCall(
            method_call.clone().with_args(simplify_function_args(method_call.args())))
        },
        &_ => todo!(),
    }
}

fn simplify_prefix(prefix: &Prefix) -> Prefix
{
    match prefix
    {
        Prefix::Expression(expression_box) =>
            Prefix::Expression(Box::new(simplify_expression(&*expression_box))),

        Prefix::Name(_token_reference) =>
            prefix.clone(),

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

fn simplify_function_call(function_call: &FunctionCall) -> FunctionCall
{
    let prefix = simplify_prefix(function_call.prefix());
    let suffixes = function_call.suffixes().map(|suffix| simplify_suffix(suffix)).collect();
    function_call.clone().with_prefix(prefix).with_suffixes(suffixes)
}

fn simplify_parentheses(contained: &ContainedSpan, expression: &Expression) -> Expression
{
    let new_inside_expression = simplify_expression(expression);
    match new_inside_expression
    {
        Expression::Symbol(_) => new_inside_expression,
        _ => Expression::Parentheses{
            contained: contained.clone(),
            expression: Box::new(new_inside_expression),
        }
    }
}

fn simplify_unary_operator(unop: &UnOp, expression: &Expression) -> Expression
{
    let new_expression = simplify_expression(expression);
    match unop
    {
        Not(_) =>
        {
            match new_expression
            {
                Expression::Symbol(ref token_reference) =>
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
                                    return Expression::Symbol(TokenReference::new(
                                        leading_trivia,
                                        Token::new(Symbol{symbol:False}),
                                        trailing_trivia,
                                    )),

                                False =>
                                    return Expression::Symbol(TokenReference::new(
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
        }
        _ => {},
    }

    return Expression::UnaryOperator{unop: unop.clone(), expression: Box::new(new_expression)};
}

fn simplify_binary_operator(left_expression : &Expression, binop: &BinOp, right_expression : &Expression) -> Expression
{
    let new_left_expression = simplify_expression(left_expression);
    let new_right_expression = simplify_expression(right_expression);

    match binop
    {
        Or(_) =>
        {
            match left_expression
            {
                Expression::Symbol(token_reference) =>
                {
                    match token_reference.token().token_type()
                    {
                        full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                        {
                            match symbol
                            {
                                True =>
                                    return Expression::Symbol(token_reference.clone()),

                                False =>
                                    return new_right_expression,

                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },

                _ => {},
            }

            match right_expression
            {
                Expression::Symbol(token_reference) =>
                {
                    match token_reference.token().token_type()
                    {
                        full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                        {
                            match symbol
                            {
                                True =>
                                    return Expression::Symbol(token_reference.clone()),

                                False =>
                                    return new_left_expression,

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
            match left_expression
            {
                Expression::Symbol(token_reference) =>
                {
                    match token_reference.token().token_type()
                    {
                        full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                        {
                            match symbol
                            {
                                False =>
                                    return Expression::Symbol(token_reference.clone()),

                                True =>
                                    return new_right_expression,
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                },

                _ => {},
            }

            match right_expression
            {
                Expression::Symbol(token_reference) =>
                {
                    match token_reference.token().token_type()
                    {
                        full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                        {
                            match symbol
                            {
                                False =>
                                    return Expression::Symbol(token_reference.clone()),

                                True =>
                                    return new_left_expression,
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

    Expression::BinaryOperator{
        lhs: Box::new(new_left_expression),
        binop: binop.clone(),
        rhs: Box::new(new_right_expression),
    }
}

fn simplify_expression(expression: &Expression) -> Expression
{
    match expression
    {
        Expression::FunctionCall(function_call) =>
            return Expression::FunctionCall(simplify_function_call(&function_call)),

        Expression::Parentheses{contained, expression} =>
            return simplify_parentheses(&contained, &*expression),

        Expression::UnaryOperator{unop, expression} =>
            return simplify_unary_operator(&unop, &expression),

        Expression::BinaryOperator{lhs, binop, rhs} =>
            return simplify_binary_operator(&lhs, &binop, &rhs),

        _ => {},
    }

    expression.clone()
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
        |expression| simplify_expression(expression)).collect())
}

fn simplify_local_assignment(local_assignment: &LocalAssignment) -> Vec<Stmt>
{
    vec![
        Stmt::LocalAssignment(
            local_assignment.clone().with_expressions(
                simplify_punctuated_expressions(local_assignment.expressions())
    ))]
}

fn is_just_true(new_condition: &Expression) -> bool
{
    match new_condition
    {
        Expression::Symbol(ref token_reference) =>
        {
            match token_reference.token().token_type()
            {
                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                {
                    match symbol
                    {
                        True => return true,
                        _ => false,
                    }
                },
                _ => false,
            }
        },
        _ => false,
    }
}

fn is_just_false(new_condition: &Expression) -> bool
{
    match new_condition
    {
        Expression::Symbol(ref token_reference) =>
        {
            match token_reference.token().token_type()
            {
                full_moon::tokenizer::TokenType::Symbol{ symbol } =>
                {
                    match symbol
                    {
                        False => return true,
                        _ => false,
                    }
                },
                _ => false,
            }
        },
        _ => false,
    }
}

enum Predicate
{
    True,
    Expression(Expression),
}

struct Clause
{
    predicate: Predicate,
    block: Block,
}

fn clauses_to_statements(mut clauses: Vec<Clause>) -> Vec<Stmt>
{
    let mut citer = clauses.drain(..);
    let first = match citer.next()
    {
        None => return vec![],
        Some(first) => first,
    };

    let mut new_if = match first.predicate
    {
        Predicate::True =>
            return vec![Stmt::Do(Do::new().with_block(first.block))],

        Predicate::Expression(expression) =>
            If::new(expression).with_block(first.block),
    };

    let mut new_elseifs = Vec::new();
    for clause in citer
    {
        match clause.predicate
        {
            Predicate::True =>
            {
                if new_elseifs.len() > 0
                {
                    new_if = new_if.with_else_if(Some(new_elseifs));
                }
                return vec![Stmt::If(new_if
                    .with_else_token(Some(TokenReference::symbol("else").unwrap()))
                    .with_else(Some(clause.block)))];
            },

            Predicate::Expression(expression) =>
            {
                new_elseifs.push(ElseIf::new(expression).with_block(clause.block))
            },
        }
    }

    if new_elseifs.len() > 0
    {
        new_if = new_if.with_else_if(Some(new_elseifs));
    }

    vec![Stmt::If(new_if)]
}

fn to_predicate(expression: Expression) -> Predicate
{
    if is_just_true(&expression)
    {
        return Predicate::True;
    }

    return Predicate::Expression(expression);
}

fn simplify_if_statement(if_statement: &If) -> Vec<Stmt>
{
    let mut clauses = vec![];

    let new_condition = simplify_expression(if_statement.condition());
    let new_block = simplify_block(if_statement.block());

    if ! is_just_false(&new_condition)
    {
        clauses.push(Clause{
            predicate: to_predicate(new_condition),
            block: new_block
        });
    }

    if let Some(elseifs) = if_statement.else_if()
    {
        for else_if_clause in elseifs
        {
            let new_condition = simplify_expression(else_if_clause.condition());
            if is_just_false(&new_condition)
            {
                continue;
            }

            let new_block = simplify_block(else_if_clause.block());
            clauses.push(Clause{
                predicate: to_predicate(new_condition),
                block: new_block
            });
        }
    }

    if let Some(else_block) = if_statement.else_block()
    {
        clauses.push(Clause{
            predicate: Predicate::True,
            block: simplify_block(&else_block)
        });
    }

    clauses_to_statements(clauses)
}

fn simplify_while_loop(while_loop: &While) -> Vec<Stmt>
{
    let new_condition = simplify_expression(while_loop.condition());
    let new_block = simplify_block(while_loop.block());

    vec![
        Stmt::While(
            while_loop.clone()
                .with_condition(new_condition)
                .with_block(new_block)
        )
    ]
}

fn simplify_function_body(function_body: &FunctionBody) -> FunctionBody
{
    let new_block = simplify_block(function_body.block());
    function_body.clone().with_block(new_block)
}

fn simplify_function_declaration(function_declaration: &FunctionDeclaration) -> Vec<Stmt>
{
    let new_body = simplify_function_body(function_declaration.body());
    vec![
        Stmt::FunctionDeclaration(
            function_declaration.clone()
                .with_body(new_body)
        )
    ]
}

fn simplify_local_function(local_function: &LocalFunction) -> Vec<Stmt>
{
    let new_body = simplify_function_body(local_function.body());
    vec![
        Stmt::LocalFunction(
            local_function.clone().with_body(new_body)
        )
    ]
}

fn simplify_statement(statement: &Stmt) -> Vec<Stmt>
{
    match statement
    {
        Stmt::LocalAssignment(local_assignment) =>
            simplify_local_assignment(&local_assignment),

        Stmt::If(if_statement) =>
            simplify_if_statement(&if_statement),

        Stmt::While(while_loop) =>
            simplify_while_loop(&while_loop),

        Stmt::FunctionDeclaration(function_declaration) =>
            simplify_function_declaration(&function_declaration),

        Stmt::LocalFunction(local_function) =>
            simplify_local_function(&local_function),

        _ => vec![statement.clone()],
    }
}

fn simplify_block(block: &Block) -> Block
{
    let mut new_stmts = vec![];

    for (statement, token_reference) in block.stmts_with_semicolon()
    {
        for new_statement in simplify_statement(&statement)
        {
            new_stmts.push((
                new_statement,
                token_reference.clone()
            ));
        }
    }

    block.clone().with_stmts(new_stmts)
}

pub fn simplify_ast(input_ast: Ast) -> Ast
{
    input_ast.clone().with_nodes(simplify_block(input_ast.nodes()))
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

fn check_unchanged(input: &str)
{
    assert_eq!(simplify_code(input), input);
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
    fn bool_or_left_expression() {
        input_output(
            "local x = true or y\n",
            "local x = true\n");
    }

    #[test]
    fn bool_or_right_expression() {
        input_output(
            "local x = y or true\n",
            "local x = true\n");
    }

    #[test]
    fn bool_and_left_expression() {
        input_output(
            "local x = false and z\n",
            "local x = false\n");
    }

    #[test]
    fn bool_and_right_expression() {
        input_output(
            "local x = y and false\n",
            "local x = false\n");
    }

    #[test]
    fn bool_or_identity_left_expression() {
        input_output(
            "local x = false or apple\n",
            "local x = apple\n");
    }

    #[test]
    fn bool_or_identity_left_expression_compound() {
        input_output(
            "local x = false or true and cherry\n",
            "local x = cherry\n");
    }

    #[test]
    fn bool_or_identity_right_expression() {
        input_output(
            "local x = banana or false\n",
            "local x = banana\n");
    }

    #[test]
    fn bool_or_identity_right_expression_compound() {
        input_output(
            "local x = false or fig or false\n",
            "local x = fig\n");
    }

    #[test]
    fn bool_and_identity_left_expression() {
        input_output(
            "local x = true and grapefruit\n",
            "local x = grapefruit\n");
    }

    #[test]
    fn bool_and_identity_left_expression_compound() {
        input_output(
            "local x = true and (false or grapefruit)\n",
            "local x = grapefruit\n");
    }

    #[test]
    fn bool_and_identity_right_expression() {
        input_output(
            "local x = cantaloupe and true\n",
            "local x = cantaloupe\n");
    }

    #[test]
    fn bool_and_identity_right_expression_compound() {
        input_output(
            "local x = durian and true and true\n",
            "local x = durian\n");
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
    fn negative_function_of_bool_expression() {
        input_output(
            "local x = -foo(true and true)\n",
            "local x = -foo(true)\n");
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

    #[test]
    fn expression_in_argument_of_method() {
        input_output(
            "local x = obj:foo(true or y)\n",
            "local x = obj:foo(true)\n");
    }

    #[test]
    fn expression_in_argument_of_method_on_result_of_getter() {
        input_output(
            "local x = getObj():foo(true or y)\n",
            "local x = getObj():foo(true)\n");
    }

    #[test]
    fn expression_in_back_of_method_call() {
        input_output(
            "local x = (true and obj):foo()\n",
            "local x = (obj):foo()\n");
    }

    #[test]
    fn bool_literal_in_if_statement() {
        input_output(
            "if true then foo() else bar() end\n",
            "do\n\tfoo()\nend\n")
    }

    #[test]
    fn if_block_continues_to_simplify() {
        input_output("\
if first then
    local x = true and true
end",
            "if first then\n\tlocal x = true\nend\n")
    }

    #[test]
    fn elseif_block_continues_to_simplify() {
        input_output("\
if first then
    foo()
elseif second then
    local x = true and true
end",
            "if first then\n\tfoo()\nelseif second then\n\tlocal x = true\nend\n")
    }

    #[test]
    fn else_block_continues_to_simplify() {
        input_output("\
if first then
    foo()
else
    local x = true and true
end",
            "if first then\n\tfoo()\nelse\n\tlocal x = true\nend\n")
    }

    #[test]
    fn bool_true_expression_in_if_statement() {
        input_output(
            "if true and true then foo() else bar() end\n",
            "do\n\tfoo()\nend\n")
    }

    #[test]
    fn literal_true_in_if_statement_with_elseifs() {
        input_output(
"if true then
    foo()
elseif something then
    bar1()
elseif something then
    bar2()
end\n",
            "do\n\tfoo()\nend\n")
    }

    #[test]
    fn elseifs_all_stay_no_else() {
        check_unchanged("\
if first then
\tfoo()
elseif second then
\tbar1()
elseif third then
\tbar2()
end\n");
    }

    #[test]
    fn elseifs_all_stay_and_else() {
        check_unchanged("\
if first then
\tfoo()
elseif second then
\tbar1()
elseif third then
\tbar2()
else
\tbar3()
end\n");
    }

    #[test]
    fn literal_true_in_elseif_condition() {
        input_output("\
if first then
\tfoo()
elseif true then
\tbar1()
elseif third then
\tbar2()
end\n",
"\
if first then
\tfoo()
else
\tbar1()
end\n")
    }

    #[test]
    fn first_condition_is_false() {
        input_output(
"\
if false then
    foo()
elseif something then
    bar1()
elseif something then
    bar2()
end",

"\
if something then
\tbar1()
elseif something then
\tbar2()
end\n")
    }

    #[test]
    fn just_if_false() {
        input_output(
"\
local x = 1
if false then
    foo()
end",

"local x = 1\n")
    }

    #[test]
    fn just_if_false_else() {
        input_output(
            "if false then foo() else bar() end",
            "do\n\tbar()\nend\n"
        )
    }

    #[test]
    fn if_false_else_if_true() {
        input_output(
"\
if false then
\tfoo()
elseif true then
\tbar1()
elseif third then
\tbar2()
else
\tbar3()
end\n",

"do\n\tbar1()\nend\n")
    }

    #[test]
    fn while_loop_continues_into_condition()
    {
        input_output(
            "while true or true do end",
            "while true do\nend\n"
        )
    }

    #[test]
    fn while_loop_continues_into_body()
    {
        input_output(
            "while true do local x = false or false end",
            "while true do\n\tlocal x = false\nend\n",
        )
    }

    #[test]
    fn function_continues_into_body()
    {
        input_output(
            "function foo() local x = true and true end",
            "function foo()\n\tlocal x = true\nend\n",
        )
    }

    #[test]
    fn local_function_continues_into_body()
    {
        input_output(
            "local function foo() local x = true and true end",
            "local function foo()\n\tlocal x = true\nend\n",
        )
    }
}