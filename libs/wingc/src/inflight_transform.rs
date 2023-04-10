use std::cell::RefCell;

use crate::{
	ast::{
		ArgList, Class, Constructor, Expr, ExprKind, FunctionDefinition, FunctionSignature, Phase, Scope, Stmt, StmtKind,
		Symbol, TypeAnnotation, UserDefinedType,
	},
	fold::{self, Fold},
};

pub struct InflightTransformer {
	curr_phase: Phase,
}

impl InflightTransformer {
	pub fn new() -> Self {
		Self {
			curr_phase: Phase::Preflight,
		}
	}
}

impl Fold for InflightTransformer {
	fn fold_function_definition(&mut self, node: FunctionDefinition) -> FunctionDefinition {
		let old_phase = self.curr_phase;
		self.curr_phase = node.signature.phase;
		let new_node = fold::fold_function_definition(self, node);
		self.curr_phase = old_phase;
		new_node
	}

	fn fold_expr(&mut self, expr: Expr) -> Expr {
		// No preflight scopes can exist inside an inflight scope, so
		// we know that if we encounter any inflight closures, they won't
		// need to be transformed.
		if self.curr_phase == Phase::Inflight {
			return expr;
		}

		// If we encounter a non-inflight closure, we can don't need to
		// transform it into a resource, but we still want to recurse
		// in case its body contains any inflight closures.
		if let ExprKind::FunctionClosure(ref def) = expr.kind {
			if def.signature.phase != Phase::Inflight {
				return fold::fold_expr(self, expr);
			}
		}

		match expr.kind {
			ExprKind::FunctionClosure(def) => {
				// resource_def = resource {
				//   init() {}
				//   inflight handle() {
				//     expr.def
				//   }
				// }
				let resource_name = Symbol {
					name: "$Resource1".to_string(),
					span: expr.span.clone(),
				};
				let handle_name = Symbol {
					name: "handle".to_string(),
					span: expr.span.clone(),
				};

				let resource_type_annotation = TypeAnnotation::UserDefined(UserDefinedType {
					root: resource_name.clone(),
					fields: vec![],
				});

				let resource_def = Stmt {
					kind: StmtKind::Class(Class {
						name: resource_name.clone(),
						is_resource: true,
						constructor: Constructor {
							signature: FunctionSignature {
								parameters: vec![],
								return_type: Some(Box::new(resource_type_annotation.clone())),
								phase: Phase::Preflight,
							},
							statements: Scope::new(vec![], expr.span.clone()),
						},
						fields: vec![],
						implements: vec![],
						parent: None,
						methods: vec![(handle_name.clone(), def)],
					}),
					idx: 0,
					span: expr.span.clone(),
				};

				// return_resource_instance = return new $Resource1();
				let return_resource_instance = Stmt {
					kind: StmtKind::Return(Some(Expr::new(
						ExprKind::New {
							class: resource_type_annotation,
							arg_list: ArgList::new(),
							obj_id: None,
							obj_scope: None,
						},
						expr.span.clone(),
					))),
					idx: 1,
					span: expr.span.clone(),
				};

				// make_resource_body = {
				//   resource_def
				//   return_resource_instance
				// }
				let make_resource_body = Scope::new(vec![resource_def, return_resource_instance], expr.span.clone());

				// make_resource_closure = (): resource => { ...make_resource_body }
				let make_resource_closure = Expr::new(
					ExprKind::FunctionClosure(FunctionDefinition {
						signature: FunctionSignature {
							parameters: vec![],
							return_type: Some(Box::new(TypeAnnotation::Resource)),
							phase: Phase::Preflight,
						},
						body: crate::ast::FunctionBody::Statements(make_resource_body),
						captures: RefCell::new(None),
						is_static: false,
						span: expr.span.clone(),
					}),
					expr.span.clone(),
				);

				// call_expr = make_resource_closure()
				let call_expr = Expr::new(
					ExprKind::Call {
						arg_list: ArgList::new(),
						callee: Box::new(make_resource_closure),
					},
					expr.span,
				);
				call_expr
			}
			_ => fold::fold_expr(self, expr),
		}
	}
}
