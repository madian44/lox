use lox::FileLocation;

pub fn provide_definition(position: &lox::FileLocation, source: &str) -> LinkedList<lox::Token> {
    let reporter = LanguageReporter {};
    let ast = lox::ast(&reporter, source);
    definition_for_position(position, &ast)
}

struct LanguageReporter {}

impl lox::Reporter for LanguageReporter {
    fn add_diagnostic(&self, _start: &FileLocation, _end: &FileLocation, _message: &str) {}

    fn add_message(&self, _message: &str) {}

    fn has_diagnostics(&self) -> bool {
        false
    }
}

use std::collections::{HashMap, LinkedList};

struct Scopes<'t> {
    scopes: LinkedList<HashMap<&'t str, &'t lox::Token>>,
}

impl<'t> Scopes<'t> {
    fn new() -> Self {
        Self {
            scopes: LinkedList::new(),
        }
    }

    fn begin(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    fn end(&mut self) {
        self.scopes.pop_front();
    }

    fn define(&mut self, token: &'t lox::Token) {
        self.scopes
            .front_mut()
            .and_then(|m| m.insert(&token.lexeme, token));
    }

    fn find_token(&self, token: &lox::Token) -> Option<&lox::Token> {
        for scope in self.scopes.iter() {
            if scope.contains_key(token.lexeme.as_str()) {
                return scope.get(token.lexeme.as_str()).cloned();
            }
        }
        None
    }
}

struct Resolver<'a> {
    position: &'a lox::FileLocation,
    scopes: Scopes<'a>,
    definitions: LinkedList<lox::Token>,
}

fn definition_for_position(
    position: &lox::FileLocation,
    ast: &LinkedList<lox::Stmt>,
) -> LinkedList<lox::Token> {
    let mut resolver = Resolver::new(position);

    resolver.resolve_stmts(ast);

    resolver.definitions
}

impl<'a> Resolver<'a> {
    fn new(position: &'a lox::FileLocation) -> Self {
        let mut scopes = Scopes::new();
        scopes.begin();
        Self {
            position,
            scopes,
            definitions: LinkedList::new(),
        }
    }

    fn resolve_stmts(&mut self, statements: &'a LinkedList<lox::Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: &'a lox::Stmt) {
        match statement {
            lox::Stmt::Block { statements } => self.resolve_stmt_block(statements),
            lox::Stmt::Class {
                name,
                superclass,
                methods,
                ..
            } => self.resolve_stmt_class(name, superclass, methods),
            lox::Stmt::Expression { expression } => self.resolve_stmt_expression(expression),
            lox::Stmt::Function { function } => self.resolve_stmt_function(function),
            lox::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.resolve_stmt_if(condition, then_branch, else_branch),
            lox::Stmt::Print { value } => self.resolve_stmt_print(value),
            lox::Stmt::Return { keyword, value, .. } => self.resolve_stmt_return(keyword, value),
            lox::Stmt::Var { name, initialiser } => self.resolve_stmt_var(name, initialiser),
            lox::Stmt::While { condition, body } => self.resolve_stmt_while(condition, body),
        }
    }

    fn resolve_expr(&mut self, expression: &lox::Expr) {
        match expression {
            lox::Expr::Assign { name, value, .. } => self.resolve_expr_assign(name, value),
            lox::Expr::Binary { left, right, .. } => self.resolve_expr_binary(left, right),
            lox::Expr::Call {
                callee, arguments, ..
            } => self.resolve_expr_call(callee, arguments),
            lox::Expr::Get { object, .. } => self.resolve_expr_get(object),
            lox::Expr::Grouping { expression, .. } => self.resolve_expr_grouping(expression),
            lox::Expr::Literal { value, .. } => self.resolve_expr_literal(value),
            lox::Expr::Logical { left, right, .. } => self.resolve_expr_logical(left, right),
            lox::Expr::Set { object, value, .. } => self.resolve_expr_set(object, value),
            lox::Expr::Super { keyword, .. } => self.resolve_expr_super(keyword),
            lox::Expr::This { keyword, .. } => self.resolve_expr_this(keyword),
            lox::Expr::Unary { right, .. } => self.resolve_expr_unary(right),
            lox::Expr::Variable { name, .. } => self.resolve_expr_variable(name),
        }
    }

    fn resolve_stmt_block(&mut self, statements: &'a LinkedList<lox::Stmt>) {
        self.scopes.begin();
        self.resolve_stmts(statements);
        self.scopes.end();
    }

    fn resolve_stmt_class(
        &mut self,
        name: &'a lox::Token,
        superclass: &Option<lox::Expr>,
        methods: &'a LinkedList<lox::Stmt>,
    ) {
        self.scopes.define(name);

        if let Some(superclass) = superclass {
            self.resolve_superclass(name, superclass);
            self.scopes.begin();
            //            self.scopes.define("super");
        }

        self.scopes.begin();
        //        self.scopes.define("this");

        for method in methods {
            if let lox::Stmt::Function { function, .. } = method {
                self.resolve_function(function);
            }
        }

        self.scopes.end();

        if superclass.is_some() {
            self.scopes.end();
        }
    }

    fn resolve_superclass(&mut self, _class_name: &lox::Token, superclass: &lox::Expr) {
        self.resolve_expr(superclass);
    }

    fn resolve_stmt_expression(&mut self, expression: &lox::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_stmt_function(&mut self, function: &'a lox::Function) {
        self.scopes.define(function.name());
        self.resolve_function(function);
    }

    fn resolve_stmt_if(
        &mut self,
        condition: &lox::Expr,
        then_branch: &'a lox::Stmt,
        else_branch: &'a Option<lox::Stmt>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        else_branch.iter().for_each(|s| self.resolve_stmt(s));
    }

    fn resolve_stmt_print(&mut self, expression: &lox::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_stmt_return(&mut self, _keyword: &lox::Token, expression: &Option<lox::Expr>) {
        if let Some(expression) = expression {
            self.resolve_expr(expression);
        }
    }

    fn resolve_stmt_var(&mut self, name: &'a lox::Token, initialiser: &'a Option<lox::Expr>) {
        self.scopes.define(name);
        initialiser.iter().for_each(|e| self.resolve_expr(e));
    }

    fn resolve_stmt_while(&mut self, condition: &lox::Expr, body: &'a lox::Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
    }

    fn resolve_expr_assign(&mut self, name: &lox::Token, value: &lox::Expr) {
        self.resolve_expr(value);
        self.resolve_local(name);
    }

    fn resolve_expr_binary(&mut self, left: &lox::Expr, right: &lox::Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn resolve_expr_call(&mut self, callee: &lox::Expr, arguments: &[lox::Expr]) {
        self.resolve_expr(callee);
        arguments.iter().for_each(|a| self.resolve_expr(a));
    }

    fn resolve_expr_get(&mut self, object: &lox::Expr) {
        self.resolve_expr(object);
    }

    fn resolve_expr_grouping(&mut self, expression: &lox::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_expr_literal(&mut self, _: &lox::Token) {}

    fn resolve_expr_logical(&mut self, left: &lox::Expr, right: &lox::Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn resolve_expr_set(&mut self, object: &lox::Expr, value: &lox::Expr) {
        self.resolve_expr(object);
        self.resolve_expr(value);
    }

    fn resolve_expr_super(&mut self, keyword: &lox::Token) {
        self.resolve_local(keyword);
    }

    fn resolve_expr_this(&mut self, keyword: &lox::Token) {
        self.resolve_local(keyword);
    }

    fn resolve_expr_unary(&mut self, right: &lox::Expr) {
        self.resolve_expr(right);
    }

    fn resolve_expr_variable(&mut self, name: &lox::Token) {
        self.resolve_local(name);
    }

    fn resolve_local(&mut self, name: &lox::Token) {
        if self.is_token_at_position(name) {
            if let Some(token) = self.scopes.find_token(name) {
                self.definitions.push_back((*token).clone());
            }
        }
    }

    fn resolve_function(&mut self, function: &'a lox::Function) {
        self.scopes.begin();
        for param in function.params() {
            self.scopes.define(param);
        }
        self.resolve_stmts(function.body());
        self.scopes.end();
    }

    fn is_token_at_position(&mut self, token: &lox::Token) -> bool {
        self.token_starts_before_position(token) && self.token_ends_after_position(token)
    }

    fn token_starts_before_position(&self, token: &lox::Token) -> bool {
        token.start.line_number < self.position.line_number
            || (token.start.line_number == self.position.line_number
                && token.start.line_offset <= self.position.line_offset)
    }

    fn token_ends_after_position(&self, token: &lox::Token) -> bool {
        token.end.line_number > self.position.line_number
            || (token.end.line_number == self.position.line_number
                && token.end.line_offset >= self.position.line_offset)
    }
}
