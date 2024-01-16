use crate::{expr, reporter, stmt, token};
use std::collections::{HashMap, LinkedList};

struct ResolverError<'t> {
    token: &'t token::Token,
    message: String,
}

impl<'t> ResolverError<'t> {
    fn new(token: &'t token::Token, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

struct Scopes {
    scopes: LinkedList<HashMap<String, bool>>,
}

impl Scopes {
    fn new() -> Self {
        Self {
            // Initial scope is 'global' and not stored
            scopes: LinkedList::new(),
        }
    }

    fn begin(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    fn end(&mut self) {
        self.scopes.pop_front();
    }

    fn declare<'t>(&mut self, name: &'t token::Token) -> Result<(), ResolverError<'t>> {
        if self
            .scopes
            .front()
            .is_some_and(|s| s.contains_key(&name.lexeme))
        {
            return Err(ResolverError::new(
                name,
                &format!(
                    "Already a variable with the name '{}' is in scope",
                    name.lexeme
                ),
            ));
        }
        self.scopes
            .front_mut()
            .and_then(|m| m.insert(name.lexeme.clone(), false));
        Ok(())
    }

    fn define(&mut self, name: &token::Token) {
        self.scopes
            .front_mut()
            .and_then(|m| m.insert(name.lexeme.clone(), true));
    }

    fn is_declared_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .front()
            .iter()
            .any(|m| m.get(name).is_some_and(|o| !(*o)))
    }

    fn find_depth(&self, name: &str) -> Option<usize> {
        for (i, scope) in self.scopes.iter().enumerate() {
            if scope.contains_key(name) {
                return Some(i);
            }
        }
        None
    }
}

struct Resolver<'r> {
    reporter: &'r dyn reporter::Reporter,
    scopes: Scopes,
    depths: HashMap<usize, usize>,
    current_function: FunctionType,
}

pub fn resolve(
    reporter: &dyn reporter::Reporter,
    statements: &LinkedList<stmt::Stmt>,
) -> HashMap<usize, usize> {
    let mut resolver = Resolver::new(reporter);

    resolver.resolve_stmts(statements);

    resolver.depths
}

impl<'r> Resolver<'r> {
    fn new(reporter: &'r dyn reporter::Reporter) -> Self {
        Self {
            reporter,
            scopes: Scopes::new(),
            depths: HashMap::new(),
            current_function: FunctionType::None,
        }
    }

    fn resolve_stmts(&mut self, statements: &LinkedList<stmt::Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: &stmt::Stmt) {
        match statement {
            stmt::Stmt::Block { statements } => self.resolve_stmt_block(statements),
            stmt::Stmt::Expression { expression } => self.resolve_stmt_expression(expression),
            stmt::Stmt::Function { name, params, body } => {
                self.resolve_stmt_function(name, params, body)
            }
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.resolve_stmt_if(condition, then_branch, else_branch),
            stmt::Stmt::Print { value } => self.resolve_stmt_print(value),
            stmt::Stmt::Return { keyword, value, .. } => self.resolve_stmt_return(keyword, value),
            stmt::Stmt::Var { name, initialiser } => self.resolve_stmt_var(name, initialiser),
            stmt::Stmt::While { condition, body } => self.resolve_stmt_while(condition, body),
        }
    }

    fn resolve_expr(&mut self, expression: &expr::Expr) {
        match expression {
            expr::Expr::Assign { id, name, value } => self.resolve_expr_assign(id, name, value),
            expr::Expr::Binary { left, right, .. } => self.resolve_expr_binary(left, right),
            expr::Expr::Call {
                callee, arguments, ..
            } => self.resolve_expr_call(callee, arguments),
            expr::Expr::Grouping { expression, .. } => self.resolve_expr_grouping(expression),
            expr::Expr::Literal { value, .. } => self.resolve_expr_literal(value),
            expr::Expr::Logical { left, right, .. } => self.resolve_expr_logical(left, right),
            expr::Expr::Unary { right, .. } => self.resolve_expr_unary(right),
            expr::Expr::Variable { id, name } => self.resolve_expr_variable(id, name),
        }
    }

    fn resolve_stmt_block(&mut self, statements: &LinkedList<stmt::Stmt>) {
        self.scopes.begin();
        self.resolve_stmts(statements);
        self.scopes.end();
    }

    fn resolve_stmt_expression(&mut self, expression: &expr::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_stmt_function(
        &mut self,
        name: &token::Token,
        params: &LinkedList<token::Token>,
        body: &LinkedList<stmt::Stmt>,
    ) {
        if let Err(e) = self.scopes.declare(name) {
            self.reporter
                .add_diagnostic(&e.token.start, &e.token.end, &e.message);
        }
        self.scopes.define(name);
        self.resolve_function(FunctionType::Function, params, body);
    }

    fn resolve_stmt_if(
        &mut self,
        condition: &expr::Expr,
        then_branch: &stmt::Stmt,
        else_branch: &Option<stmt::Stmt>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        else_branch.iter().for_each(|s| self.resolve_stmt(s));
    }

    fn resolve_stmt_print(&mut self, expression: &expr::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_stmt_return(&mut self, keyword: &token::Token, expression: &Option<expr::Expr>) {
        if self.current_function == FunctionType::None {
            self.reporter.add_diagnostic(
                &keyword.start,
                &keyword.end,
                "Cannot return from top-level code",
            );
        }
        expression.iter().for_each(|e| self.resolve_expr(e));
    }

    fn resolve_stmt_var(&mut self, name: &token::Token, initialiser: &Option<expr::Expr>) {
        if let Err(e) = self.scopes.declare(name) {
            self.reporter
                .add_diagnostic(&e.token.start, &e.token.end, &e.message);
        }
        initialiser.iter().for_each(|e| self.resolve_expr(e));
        self.scopes.define(name);
    }

    fn resolve_stmt_while(&mut self, condition: &expr::Expr, body: &stmt::Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
    }

    fn resolve_expr_assign(&mut self, id: &usize, name: &token::Token, value: &expr::Expr) {
        self.resolve_expr(value);
        self.resolve_local(id, name);
    }

    fn resolve_expr_binary(&mut self, left: &expr::Expr, right: &expr::Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn resolve_expr_call(&mut self, callee: &expr::Expr, arguments: &[expr::Expr]) {
        self.resolve_expr(callee);
        arguments.iter().for_each(|a| self.resolve_expr(a));
    }

    fn resolve_expr_grouping(&mut self, expression: &expr::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_expr_literal(&mut self, _: &token::Token) {}

    fn resolve_expr_logical(&mut self, left: &expr::Expr, right: &expr::Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn resolve_expr_unary(&mut self, right: &expr::Expr) {
        self.resolve_expr(right);
    }

    fn resolve_expr_variable(&mut self, id: &usize, name: &token::Token) {
        if self.scopes.is_declared_in_current_scope(&name.lexeme) {
            self.add_diagnostic(name, "Cannot read local variable in its own initialiser");
        }
        self.resolve_local(id, name);
    }

    fn resolve_local(&mut self, id: &usize, name: &token::Token) {
        let depth = self.scopes.find_depth(&name.lexeme);
        depth.iter().for_each(|d| {
            self.depths.insert(*id, *d);
        });
    }

    fn resolve_function(
        &mut self,
        function_type: FunctionType,
        params: &LinkedList<token::Token>,
        body: &LinkedList<stmt::Stmt>,
    ) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.scopes.begin();
        for param in params {
            if let Err(e) = self.scopes.declare(param) {
                self.reporter
                    .add_diagnostic(&e.token.start, &e.token.end, &e.message);
            }
            self.scopes.define(param);
        }
        self.resolve_stmts(body);
        self.scopes.end();
        self.current_function = enclosing_function;
    }

    fn add_diagnostic(&self, t: &token::Token, message: &str) {
        self.reporter.add_diagnostic(&t.start, &t.end, message);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{location, parser, reporter::test::TestReporter, scanner, token};

    #[test]
    fn test_global_scope() {
        let mut scopes = Scopes::new();

        let blank_location = location::FileLocation::new(0, 0);

        let name = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let res = scopes.declare(&name);
        assert!(res.is_ok(), "Unexpected failure declaring 'name'");

        let res = scopes.is_declared_in_current_scope(&name.lexeme);
        assert!(!res, "Unexpected failure testing declartion of 'name'");

        scopes.define(&name);

        let res = scopes.is_declared_in_current_scope(&name.lexeme);
        assert!(!res, "Unexpected failure testing definition of 'name'");

        let res = scopes.find_depth(&name.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'name'");
    }

    #[test]
    fn test_declare_define() {
        let mut scopes = Scopes::new();

        let blank_location = location::FileLocation::new(0, 0);

        let name = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        scopes.begin();

        let res = scopes.declare(&name);
        assert!(res.is_ok(), "Unexpected failure declaring 'name'");

        let res = scopes.is_declared_in_current_scope(&name.lexeme);
        assert!(res, "Unexpected failure testing declartion of 'name'");

        scopes.define(&name);

        let res = scopes.is_declared_in_current_scope(&name.lexeme);
        assert!(!res, "Unexpected failure testing definition of 'name'");

        let res = scopes.find_depth(&name.lexeme);
        assert_eq!(res, Some(0), "Unexpected failure testing depth of 'name'");
    }

    #[test]
    fn test_depth() {
        let mut scopes = Scopes::new();

        let blank_location = location::FileLocation::new(0, 0);

        let name_global = token::Token::new(
            token::TokenType::Identifier,
            "global",
            blank_location,
            blank_location,
            None,
        );

        let name_first = token::Token::new(
            token::TokenType::Identifier,
            "first",
            blank_location,
            blank_location,
            None,
        );

        let name_second = token::Token::new(
            token::TokenType::Identifier,
            "second",
            blank_location,
            blank_location,
            None,
        );

        let res = scopes.declare(&name_global);
        assert!(res.is_ok(), "Unexpected failure declaring 'global'");

        scopes.begin();

        let res = scopes.declare(&name_first);
        assert!(res.is_ok(), "Unexpected failure declaring 'first'");

        let res = scopes.find_depth(&name_global.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'global'");

        let res = scopes.find_depth(&name_first.lexeme);
        assert_eq!(res, Some(0), "Unexpected failure testing depth of 'first'");

        scopes.begin();

        let res = scopes.declare(&name_second);
        assert!(res.is_ok(), "Unexpected failure declaring 'second'");

        let res = scopes.find_depth(&name_global.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'global'");

        let res = scopes.find_depth(&name_first.lexeme);
        assert_eq!(res, Some(1), "Unexpected failure testing depth of 'first'");

        let res = scopes.find_depth(&name_second.lexeme);
        assert_eq!(res, Some(0), "Unexpected failure testing depth of 'second'");

        scopes.end();

        let res = scopes.find_depth(&name_global.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'global'");

        let res = scopes.find_depth(&name_first.lexeme);
        assert_eq!(res, Some(0), "Unexpected failure testing depth of 'first'");

        let res = scopes.find_depth(&name_second.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'second'");

        scopes.end();

        let res = scopes.find_depth(&name_global.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'global'");

        let res = scopes.find_depth(&name_first.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'first'");

        let res = scopes.find_depth(&name_second.lexeme);
        assert_eq!(res, None, "Unexpected failure testing depth of 'second'");
    }

    #[test]
    fn test_errors() {
        let tests = vec![
            (
                "{ var a = 1; var a = 2; }",
                "Already a variable with the name 'a' is in scope",
            ),
            (
                "fun broken(a, a) {}",
                "Already a variable with the name 'a' is in scope",
            ),
            (
                "fun broken(a) { var a ; }",
                "Already a variable with the name 'a' is in scope",
            ),
            (
                "fun broken(a) { var b = 1 ; var b = 2; }",
                "Already a variable with the name 'b' is in scope",
            ),
            (
                "{ var a = a; }",
                "Cannot read local variable in its own initialiser",
            ),
            ("return false;", "Cannot return from top-level code"),
        ];

        let reporter = TestReporter::build();

        for (src, expected_diagnostic) in tests {
            let tokens = scanner::scan_tokens(&reporter, src);
            let statements = parser::parse(&reporter, tokens);
            reporter.reset();
            let _ = resolve(&reporter, &statements);
            if !reporter.has_diagnostic(expected_diagnostic) {
                reporter.print_contents();
                panic!("Missing diagnostic '{}' for '{}'", expected_diagnostic, src);
            }
        }
    }
}
