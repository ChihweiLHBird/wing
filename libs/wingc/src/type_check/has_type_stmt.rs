use crate::{
	ast::{Ast, FunctionDefinition, Stmt, StmtKind},
	visit::{self, Visit},
};

// List in this file the fields that are initialized in a class constructor.
pub struct HasStatementVisitor<'a> {
	ast: &'a Ast,
	pub seen_return: bool,
	pub seen_throw: bool,
}

impl<'a> HasStatementVisitor<'a> {
	pub fn new(ast: &'a Ast) -> Self {
		Self {
			ast,
			seen_return: false,
			seen_throw: false,
		}
	}

	pub fn visit<I>(&mut self, statements: I)
	where
		I: IntoIterator<Item = &'a Stmt>,
	{
		for stmt in statements {
			self.visit_stmt(stmt);
		}
	}
}

impl<'a> Visit<'a> for HasStatementVisitor<'a> {
	fn ast(&self) -> &'a Ast {
		self.ast
	}

	fn visit_stmt(&mut self, node: &'a Stmt) {
		match &node.kind {
			StmtKind::Throw(_) => self.seen_throw = true,
			StmtKind::Return(_) => self.seen_return = true,
			_ => (),
		}
		visit::visit_stmt(self, node);
	}

	fn visit_function_definition(&mut self, _: &'_ FunctionDefinition) {
		// Don't recurse into functions. This way our search will ignore stmts in inner functions.
		return;
	}
}
