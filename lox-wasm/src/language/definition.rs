use lox::FileLocation;

pub fn provide_definition(position: &lox::FileLocation, source: &str) -> LinkedList<lox::Token> {
    let reporter = LanguageReporter {};
    let ast = lox::ast(&reporter, source);
    definition_for_position(position, &ast)
}

fn definition_for_position(
    position: &lox::FileLocation,
    ast: &LinkedList<lox::Stmt>,
) -> LinkedList<lox::Token> {
    let mut resolver = Resolver::new(position);

    resolver.resolve_stmts(ast);

    resolver.definitions
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

struct Class<'t> {
    name: &'t lox::Token,
    superclass: Option<&'t str>,
    methods: HashMap<&'t str, &'t lox::Token>,
}

impl<'t> Class<'t> {
    fn new(name: &'t lox::Token, superclass: Option<&'t str>) -> Self {
        Self {
            name,
            superclass,
            methods: HashMap::new(),
        }
    }

    fn add_method(&mut self, method: &'t lox::Token) {
        self.methods.insert(&method.lexeme, method);
    }

    fn find_method(&self, scopes: &'t Scopes, method: &str) -> Option<&'t lox::Token> {
        if self.methods.contains_key(method) {
            return self.methods.get(method).cloned();
        }
        self.superclass
            .and_then(|s| scopes.find_class(s))
            .and_then(|c| c.methods.get(method))
            .cloned()
    }
}

struct Scope<'t> {
    identifiers: HashMap<&'t str, &'t lox::Token>,
    types: HashMap<&'t str, &'t str>,
    classes: HashMap<&'t str, Class<'t>>,
}

impl<'t> Scope<'t> {
    fn new() -> Self {
        Self {
            identifiers: HashMap::new(),
            types: HashMap::new(),
            classes: HashMap::new(),
        }
    }
}

struct Scopes<'t> {
    scopes: LinkedList<Scope<'t>>,
}

impl<'t> Scopes<'t> {
    fn new() -> Self {
        Self {
            scopes: LinkedList::new(),
        }
    }

    fn begin(&mut self) {
        self.scopes.push_front(Scope::new());
    }

    fn end(&mut self) {
        self.scopes.pop_front();
    }

    fn define_identifier(&mut self, token: &'t lox::Token) {
        self.scopes
            .front_mut()
            .and_then(|m| m.identifiers.insert(&token.lexeme, token));
    }

    fn define_type_for_identifier(&mut self, name: &'t str, typ: &'t str) {
        self.scopes
            .front_mut()
            .and_then(|m| m.types.insert(name, typ));
    }

    fn define_class(&mut self, class: &'t lox::Token, superclass: Option<&'t str>) {
        self.scopes.front_mut().and_then(|m| {
            m.classes
                .insert(&class.lexeme, Class::new(class, superclass))
        });
    }

    fn add_method_to_class(&mut self, class_name: &str, method: &'t lox::Token) {
        for scope in self.scopes.iter_mut() {
            if let Some(class) = scope.classes.get_mut(class_name) {
                class.add_method(method);
                return;
            }
        }
    }

    fn find(&self, name: &str) -> Option<&lox::Token> {
        for scope in self.scopes.iter() {
            if scope.identifiers.contains_key(name) {
                return scope.identifiers.get(name).cloned();
            }
            if scope.classes.contains_key(name) {
                return scope.classes.get(name).map(|c| c.name);
            }
        }
        None
    }

    fn find_class(&self, name: &str) -> Option<&Class> {
        for scope in self.scopes.iter() {
            if scope.classes.contains_key(name) {
                return scope.classes.get(name);
            }
        }
        None
    }

    fn find_type_for_identifier(&self, name: &str) -> Option<&str> {
        for scope in self.scopes.iter() {
            if scope.types.contains_key(name) {
                return scope.types.get(name).cloned();
            }
        }
        None
    }
}

struct Resolver<'a> {
    position: &'a lox::FileLocation,
    scopes: Scopes<'a>,
    definitions: LinkedList<lox::Token>,
    current_class: Option<&'a str>,
}

impl<'a> Resolver<'a> {
    fn new(position: &'a lox::FileLocation) -> Self {
        let mut scopes = Scopes::new();
        scopes.begin();
        Self {
            position,
            scopes,
            definitions: LinkedList::new(),
            current_class: None,
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
            lox::Expr::Get { object, name, .. } => self.resolve_expr_get(object, name),
            lox::Expr::Grouping { expression, .. } => self.resolve_expr_grouping(expression),
            lox::Expr::Literal { value, .. } => self.resolve_expr_literal(value),
            lox::Expr::Logical { left, right, .. } => self.resolve_expr_logical(left, right),
            lox::Expr::Set {
                object,
                name,
                value,
                ..
            } => self.resolve_expr_set(object, name, value),
            lox::Expr::Super {
                keyword, method, ..
            } => self.resolve_expr_super(keyword, method),
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
        superclass: &'a Option<lox::Expr>,
        methods: &'a LinkedList<lox::Stmt>,
    ) {
        let superclass_name = if let Some(lox::Expr::Variable {
            name: superclass_name,
            ..
        }) = superclass
        {
            Some(superclass_name.lexeme.as_str())
        } else {
            None
        };
        self.scopes.define_class(name, superclass_name);
        let enclosing_class = self.current_class;
        self.current_class = Some(&name.lexeme);

        if let Some(superclass) = superclass {
            self.resolve_superclass(name, superclass);
            self.scopes.begin();
            //            self.scopes.define("super");
        }

        self.scopes.begin();
        //        self.scopes.define("this");

        for method in methods {
            if let lox::Stmt::Function { function, .. } = method {
                self.scopes
                    .add_method_to_class(&name.lexeme, function.name());
                self.resolve_function(function);
            }
        }

        self.scopes.end();

        if superclass.is_some() {
            self.scopes.end();
        }
        self.current_class = enclosing_class;
    }

    fn resolve_superclass(&mut self, _class_name: &lox::Token, superclass: &lox::Expr) {
        self.resolve_expr(superclass);
    }

    fn resolve_stmt_expression(&mut self, expression: &lox::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_stmt_function(&mut self, function: &'a lox::Function) {
        self.scopes.define_identifier(function.name());
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
        self.scopes.define_identifier(name);
        if let Some(initialiser) = initialiser {
            if let lox::Expr::Call { callee, .. } = initialiser {
                if let lox::Expr::Variable { name: class, .. } = callee.as_ref() {
                    if self.scopes.find_class(&class.lexeme).is_some() {
                        self.scopes
                            .define_type_for_identifier(&name.lexeme, &class.lexeme);
                    }
                }
            }
            self.resolve_expr(initialiser);
        }
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

    fn resolve_expr_get(&mut self, object: &lox::Expr, name: &lox::Token) {
        if self.is_at_position(name) {
            // position is a property of something
            let method = if let lox::Expr::Variable { name: target, .. } = object {
                self.scopes
                    .find_type_for_identifier(&target.lexeme)
                    .and_then(|t| self.scopes.find_class(t))
                    .and_then(|c| c.find_method(&self.scopes, &name.lexeme))
            } else if let lox::Expr::Call { callee, .. } = object {
                if let lox::Expr::Variable { name: target, .. } = callee.as_ref() {
                    self.scopes
                        .find_class(&target.lexeme)
                        .and_then(|c| c.find_method(&self.scopes, &name.lexeme))
                } else {
                    None
                }
            } else if let lox::Expr::This { .. } = object {
                if let Some(class) = self.current_class.and_then(|c| self.scopes.find_class(c)) {
                    class.find_method(&self.scopes, &name.lexeme)
                } else {
                    None
                }
            } else {
                None
            };
            method
                .into_iter()
                .for_each(|m| self.definitions.push_back((*m).clone()));
        } else {
            self.resolve_local(name);
            self.resolve_expr(object);
        }
    }

    fn resolve_expr_grouping(&mut self, expression: &lox::Expr) {
        self.resolve_expr(expression);
    }

    fn resolve_expr_literal(&mut self, _: &lox::Token) {}

    fn resolve_expr_logical(&mut self, left: &lox::Expr, right: &lox::Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn resolve_expr_set(&mut self, object: &lox::Expr, name: &lox::Token, value: &lox::Expr) {
        self.resolve_expr(object);
        self.resolve_local(name);
        self.resolve_expr(value);
    }

    fn resolve_expr_super(&mut self, keyword: &lox::Token, method: &lox::Token) {
        self.resolve_local(keyword);
        if self.is_at_position(method) {
            if let Some(class) = self.current_class.and_then(|c| self.scopes.find_class(c)) {
                if let Some(superclass) = class.superclass.and_then(|s| self.scopes.find_class(s)) {
                    superclass
                        .find_method(&self.scopes, &method.lexeme)
                        .iter()
                        .for_each(|m| self.definitions.push_back((*m).clone()));
                }
            }
        }
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
        if self.is_at_position(name) {
            if let Some(token) = self.scopes.find(&name.lexeme) {
                self.definitions.push_back((*token).clone());
            }
        }
    }

    fn resolve_function(&mut self, function: &'a lox::Function) {
        self.scopes.begin();
        for param in function.params() {
            self.scopes.define_identifier(param);
        }
        self.resolve_stmts(function.body());
        self.scopes.end();
    }

    fn is_at_position(&self, provider: &impl lox::ProvideLocation) -> bool {
        self.is_starts_before_position(provider) && self.is_ends_after_position(provider)
    }

    fn is_starts_before_position(&self, provider: &impl lox::ProvideLocation) -> bool {
        provider.start().line_number < self.position.line_number
            || (provider.start().line_number == self.position.line_number
                && provider.start().line_offset <= self.position.line_offset)
    }

    fn is_ends_after_position(&self, provider: &impl lox::ProvideLocation) -> bool {
        provider.end().line_number > self.position.line_number
            || (provider.end().line_number == self.position.line_number
                && provider.end().line_offset >= self.position.line_offset)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::zip;

    fn unindent_string(source: &str) -> String {
        let re = regex::Regex::new(r"\n\s+[|]").unwrap();
        re.replace_all(source, "\n").to_string()
    }

    #[test]
    fn test() {
        let tests = vec![
            ("fred = 1;", (0, 0), vec![]),
            (
                "var fred;
                |fred = 1;",
                (1, 1),
                vec![((0, 4), (0, 8))],
            ),
            (
                "class Test {
                |  hello() {}
                |}
                |Test().hello();",
                (3, 7),
                vec![((1, 2), (1, 7))],
            ),
            (
                "class Test {
                |  hello() {}
                |}
                |var test = Test();
                |test.hello();",
                (4, 5),
                vec![((1, 2), (1, 7))],
            ),
            (
                "class Base {
                |  hello() {}
                |}
                |class Test < Base {}
                |var test = Test();
                |test.hello();",
                (5, 5),
                vec![((1, 2), (1, 7))],
            ),
            (
                "class Base {
                |  hello() {}
                |}
                |class Test < Base {
                |  hello() {}
                |}
                |var test = Test();
                |test.hello();",
                (7, 5),
                vec![((4, 2), (4, 7))],
            ),
            (
                "class Base {
                |  hello() {}
                |}
                |class Test < Base {
                |  hello() { 
                |    super.hello();
                |  }
                |}",
                (5, 10),
                vec![((1, 2), (1, 7))],
            ),
            (
                "class Test {
                |  first() {}
                |  second() {
                |    this.first();
                |  }
                |}",
                (3, 9),
                vec![((1, 2), (1, 7))],
            ),
            (
                "fun Test(param) {
                |  print param;
                |}",
                (1, 9),
                vec![((0, 9), (0, 14))],
            ),
        ];
        for (source, (line_number, line_offset), locations) in tests {
            let result = provide_definition(
                &FileLocation {
                    line_number,
                    line_offset,
                },
                &unindent_string(source),
            );

            assert_eq!(
                result.len(),
                locations.len(),
                "Unexpected number of locations for {}",
                source
            );
            for (location, expected_location) in zip(result, locations) {
                if (location.start
                    != lox::FileLocation::new(expected_location.0 .0, expected_location.0 .1))
                    || (location.end
                        != lox::FileLocation::new(expected_location.1 .0, expected_location.1 .1))
                {
                    panic!("Missing location for {} {:?}", source, location);
                }
            }
        }
    }
}
