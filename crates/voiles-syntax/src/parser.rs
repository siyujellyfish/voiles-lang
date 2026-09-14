use voiles_lexer::{Diagnostic, Keyword, Span, Token, TokenKind, lex};

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse {
	pub root: SyntaxNode,
	pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn parse(source: &str) -> Parse {
	let lexed = lex(source);
	let mut parser = Parser::new(&lexed.tokens);
	let root = parser.parse_module();
	let mut diagnostics = lexed.diagnostics;
	diagnostics.extend(parser.diagnostics);
	Parse { root, diagnostics }
}

struct Parser<'tokens> {
	tokens: &'tokens [Token],
	position: usize,
	diagnostics: Vec<Diagnostic>,
}

impl<'tokens> Parser<'tokens> {
	fn new(tokens: &'tokens [Token]) -> Self {
		Self {
			tokens,
			position: 0,
			diagnostics: Vec::new(),
		}
	}

	fn parse_module(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		while !self.at(TokenKind::Eof) {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Eof) {
				break;
			}
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut children);
				continue;
			}

			let start = self.position;
			children.push(self.parse_module_item().into());
			if self.position == start {
				children.push(self.recover_line("VPAR004", "unable to parse module item").into());
			}
		}

		if self.at(TokenKind::Eof) {
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Module, children)
	}

	fn parse_module_item(&mut self) -> SyntaxNode {
		match self.current_kind() {
			TokenKind::Keyword(Keyword::Import) => self.parse_import(false),
			TokenKind::Keyword(Keyword::Extern)
				if self.peek_significant_kind(1) == Some(TokenKind::Keyword(Keyword::Import)) =>
			{
				self.parse_import(true)
			}
			TokenKind::Keyword(Keyword::Export) => self.parse_export(),
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				self.parse_binding()
			}
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => self.parse_function(),
			_ => self.parse_statement(),
		}
	}

	fn parse_statement(&mut self) -> SyntaxNode {
		match self.current_kind() {
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				self.parse_binding()
			}
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => self.parse_function(),
			TokenKind::Keyword(Keyword::If) => self.parse_if(),
			TokenKind::Keyword(Keyword::For) => self.parse_for(),
			TokenKind::Keyword(Keyword::Return) => self.parse_return(),
			TokenKind::Keyword(Keyword::Init | Keyword::Mount | Keyword::Cleanup) => {
				self.parse_lifecycle_block()
			}
			TokenKind::Dedent | TokenKind::Eof => {
				self.error_here("VPAR004", "expected statement");
				SyntaxNode::new(SyntaxKind::Error, Vec::new())
			}
			_ => self.parse_expression_statement(),
		}
	}

	fn parse_import(&mut self, is_extern: bool) -> SyntaxNode {
		let mut children = Vec::new();
		let kind = if is_extern {
			self.expect_keyword(
				&mut children,
				Keyword::Extern,
				"expected `extern` before foreign import",
			);
			SyntaxKind::ExternImportDecl
		} else {
			SyntaxKind::ImportDecl
		};
		self.expect_keyword(&mut children, Keyword::Import, "expected `import`");
		self.eat_trivia(&mut children);

		if self.at(TokenKind::Star) {
			let mut namespace = Vec::new();
			self.bump_into(&mut namespace);
			self.expect_keyword(&mut namespace, Keyword::As, "expected `as` after `*`");
			self.expect(
				&mut namespace,
				TokenKind::Identifier,
				"expected namespace import name",
			);
			children.push(SyntaxNode::new(SyntaxKind::NamespaceImport, namespace).into());
		} else {
			loop {
				self.eat_trivia(&mut children);
				if !self.at(TokenKind::Identifier) {
					self.error_here("VPAR001", "expected imported symbol name");
					break;
				}
				children.push(self.parse_import_item().into());
				self.eat_trivia(&mut children);
				if self.at(TokenKind::Comma) {
					self.bump_into(&mut children);
					continue;
				}
				break;
			}
		}

		self.expect_keyword(&mut children, Keyword::From, "expected `from` in import");
		self.expect(
			&mut children,
			TokenKind::String,
			"expected quoted module specifier",
		);
		self.finish_simple_line(&mut children);
		SyntaxNode::new(kind, children)
	}

	fn parse_import_item(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected imported symbol name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::Keyword(Keyword::As)) {
			self.expect_keyword(&mut children, Keyword::As, "expected `as`");
			self.expect(
				&mut children,
				TokenKind::Identifier,
				"expected import alias name",
			);
		}
		SyntaxNode::new(SyntaxKind::ImportItem, children)
	}

	fn parse_export(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Export, "expected `export`");
		self.eat_trivia(&mut children);
		let declaration = match self.current_kind() {
			TokenKind::Keyword(Keyword::Const | Keyword::State | Keyword::Shared) => {
				Some(self.parse_binding())
			}
			TokenKind::Keyword(Keyword::Fn | Keyword::Async) => Some(self.parse_function()),
			_ => None,
		};

		if let Some(declaration) = declaration {
			children.push(declaration.into());
		} else {
			self.error_here(
				"VPAR005",
				"expected exportable declaration after `export`",
			);
			children.push(
				self.recover_line("VPAR005", "invalid export declaration")
					.into(),
			);
		}
		SyntaxNode::new(SyntaxKind::ExportDecl, children)
	}

	fn parse_binding(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.bump_into(&mut children);
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected binding name",
		);

		if self.peek_significant_kind(0) == Some(TokenKind::Colon) {
			self.expect(&mut children, TokenKind::Colon, "expected `:`");
			children.push(self.parse_type().into());
		}

		self.expect(
			&mut children,
			TokenKind::Equal,
			"expected `=` in binding declaration",
		);
		children.push(self.parse_expression().into());
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::BindingDecl, children)
	}

	fn parse_function(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		if self.at_keyword(Keyword::Async) {
			self.bump_into(&mut children);
			self.expect_keyword(&mut children, Keyword::Fn, "expected `fn` after `async`");
		} else {
			self.expect_keyword(&mut children, Keyword::Fn, "expected `fn`");
		}

		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected function name",
		);
		children.push(self.parse_parameter_list().into());

		if self.peek_significant_kind(0) == Some(TokenKind::Arrow) {
			let mut return_type = Vec::new();
			self.expect(&mut return_type, TokenKind::Arrow, "expected `->`");
			return_type.push(self.parse_type().into());
			children.push(SyntaxNode::new(SyntaxKind::ReturnType, return_type).into());
		}

		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::FunctionDecl, children)
	}

	fn parse_parameter_list(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::LParen,
			"expected `(` before parameters",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
				break;
			}
			children.push(self.parse_parameter().into());
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			if !self.at(TokenKind::RParen) {
				self.error_here("VPAR006", "expected `,` or `)` after parameter");
				children.push(self.recover_until(&[TokenKind::Comma, TokenKind::RParen]).into());
				if self.at(TokenKind::Comma) {
					self.bump_into(&mut children);
				}
			}
		}

		self.expect(
			&mut children,
			TokenKind::RParen,
			"expected `)` after parameters",
		);
		SyntaxNode::new(SyntaxKind::ParameterList, children)
	}

	fn parse_parameter(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected parameter name",
		);
		if self.peek_significant_kind(0) == Some(TokenKind::Colon) {
			self.expect(&mut children, TokenKind::Colon, "expected `:`");
			children.push(self.parse_type().into());
		}
		if self.peek_significant_kind(0) == Some(TokenKind::Equal) {
			self.expect(&mut children, TokenKind::Equal, "expected `=`");
			children.push(self.parse_expression().into());
		}
		SyntaxNode::new(SyntaxKind::Parameter, children)
	}

	fn parse_type(&mut self) -> SyntaxNode {
		let mut node = self.parse_type_primary();
		loop {
			match self.peek_significant_kind(0) {
				Some(TokenKind::Less) => {
					let arguments = self.parse_generic_arguments();
					node = SyntaxNode::new(
						SyntaxKind::TypeRef,
						vec![node.into(), arguments.into()],
					);
				}
				Some(TokenKind::Question) => {
					let mut children = vec![node.into()];
					self.expect(&mut children, TokenKind::Question, "expected `?`");
					node = SyntaxNode::new(SyntaxKind::OptionalType, children);
				}
				_ => break,
			}
		}
		node
	}

	fn parse_type_primary(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.eat_trivia(&mut children);
		match self.current_kind() {
			TokenKind::Identifier => {
				self.bump_into(&mut children);
				while self.peek_significant_kind(0) == Some(TokenKind::Dot) {
					self.expect(&mut children, TokenKind::Dot, "expected `.`");
					self.expect(
						&mut children,
						TokenKind::Identifier,
						"expected qualified type name",
					);
				}
				SyntaxNode::new(SyntaxKind::TypeRef, children)
			}
			TokenKind::Keyword(Keyword::Fn) => {
				self.bump_into(&mut children);
				self.expect(
					&mut children,
					TokenKind::LParen,
					"expected `(` in function type",
				);
				loop {
					self.eat_trivia(&mut children);
					if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
						break;
					}
					children.push(self.parse_type().into());
					self.eat_trivia(&mut children);
					if self.at(TokenKind::Comma) {
						self.bump_into(&mut children);
						continue;
					}
					break;
				}
				self.expect(
					&mut children,
					TokenKind::RParen,
					"expected `)` in function type",
				);
				self.expect(
					&mut children,
					TokenKind::Arrow,
					"expected `->` in function type",
				);
				children.push(self.parse_type().into());
				SyntaxNode::new(SyntaxKind::FunctionType, children)
			}
			TokenKind::LParen => {
				self.bump_into(&mut children);
				children.push(self.parse_type().into());
				self.expect(
					&mut children,
					TokenKind::RParen,
					"expected `)` after type",
				);
				SyntaxNode::new(SyntaxKind::ParenthesizedType, children)
			}
			_ => {
				self.error_here("VPAR001", "expected type");
				if !matches!(
					self.current_kind(),
					TokenKind::Newline
						| TokenKind::Dedent
						| TokenKind::Eof
						| TokenKind::Comma
						| TokenKind::RParen
				) {
					self.bump_into(&mut children);
				}
				SyntaxNode::new(SyntaxKind::Error, children)
			}
		}
	}

	fn parse_generic_arguments(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(&mut children, TokenKind::Less, "expected `<`");
		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Greater) || self.at(TokenKind::Eof) {
				break;
			}
			children.push(self.parse_type().into());
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			break;
		}
		self.expect(
			&mut children,
			TokenKind::Greater,
			"expected `>` after generic arguments",
		);
		SyntaxNode::new(SyntaxKind::GenericArgumentList, children)
	}

	fn parse_if(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::If, "expected `if`");
		children.push(self.parse_expression().into());
		children.push(self.parse_block().into());

		self.eat_trivia(&mut children);
		if self.at_keyword(Keyword::Else) {
			let mut else_children = Vec::new();
			self.bump_into(&mut else_children);
			else_children.push(self.parse_block().into());
			children.push(SyntaxNode::new(SyntaxKind::ElseClause, else_children).into());
		}
		SyntaxNode::new(SyntaxKind::IfStmt, children)
	}

	fn parse_for(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::For, "expected `for`");
		self.expect(
			&mut children,
			TokenKind::Identifier,
			"expected loop binding name",
		);
		self.expect_keyword(&mut children, Keyword::In, "expected `in` in loop");
		children.push(self.parse_expression().into());

		if self.peek_significant_kind(0) == Some(TokenKind::Keyword(Keyword::Key)) {
			let mut key_children = Vec::new();
			self.expect_keyword(&mut key_children, Keyword::Key, "expected `key`");
			key_children.push(self.parse_expression().into());
			children.push(SyntaxNode::new(SyntaxKind::KeyClause, key_children).into());
		}

		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::ForStmt, children)
	}

	fn parse_return(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect_keyword(&mut children, Keyword::Return, "expected `return`");
		if self.peek_significant_kind(0) != Some(TokenKind::Newline) {
			children.push(self.parse_expression().into());
		}
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::ReturnStmt, children)
	}

	fn parse_lifecycle_block(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.bump_into(&mut children);
		children.push(self.parse_block().into());
		SyntaxNode::new(SyntaxKind::LifecycleBlock, children)
	}

	fn parse_expression_statement(&mut self) -> SyntaxNode {
		let mut children = vec![self.parse_expression().into()];
		self.finish_simple_line(&mut children);
		SyntaxNode::new(SyntaxKind::ExprStmt, children)
	}

	fn parse_block(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.expect(&mut children, TokenKind::Colon, "expected `:` before block");
		self.expect(
			&mut children,
			TokenKind::Newline,
			"expected newline after block header",
		);
		self.expect(
			&mut children,
			TokenKind::Indent,
			"expected indented block body",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::Dedent) {
				self.bump_into(&mut children);
				break;
			}
			if self.at(TokenKind::Eof) {
				self.error_here("VPAR003", "unterminated block; expected dedent");
				break;
			}
			if self.at(TokenKind::Newline) {
				self.bump_into(&mut children);
				continue;
			}

			let start = self.position;
			children.push(self.parse_statement().into());
			if self.position == start {
				children.push(self.recover_line("VPAR004", "unable to parse statement").into());
			}
		}
		SyntaxNode::new(SyntaxKind::Block, children)
	}

	fn parse_expression(&mut self) -> SyntaxNode {
		self.parse_expression_bp(0)
	}

	fn parse_expression_bp(&mut self, minimum_bp: u8) -> SyntaxNode {
		let mut left = self.parse_prefix_expression();

		loop {
			match self.peek_significant_kind(0) {
				Some(TokenKind::LParen) => {
					let mut children = vec![left.into()];
					self.eat_trivia(&mut children);
					children.push(self.parse_argument_list().into());
					left = SyntaxNode::new(SyntaxKind::CallExpr, children);
					continue;
				}
				Some(TokenKind::Dot) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::Dot, "expected `.`");
					self.expect(
						&mut children,
						TokenKind::Identifier,
						"expected member name after `.`",
					);
					left = SyntaxNode::new(SyntaxKind::MemberExpr, children);
					continue;
				}
				Some(TokenKind::LBracket) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::LBracket, "expected `[` ");
					children.push(self.parse_expression().into());
					self.expect(
						&mut children,
						TokenKind::RBracket,
						"expected `]` after index expression",
					);
					left = SyntaxNode::new(SyntaxKind::IndexExpr, children);
					continue;
				}
				Some(TokenKind::Question) => {
					let mut children = vec![left.into()];
					self.expect(&mut children, TokenKind::Question, "expected `?`");
					left = SyntaxNode::new(SyntaxKind::TryExpr, children);
					continue;
				}
				_ => {}
			}

			let Some(operator) = self.peek_significant_kind(0) else {
				break;
			};
			let Some((left_bp, right_bp, kind)) = infix_binding_power(operator) else {
				break;
			};
			if left_bp < minimum_bp {
				break;
			}

			let mut children = vec![left.into()];
			self.eat_trivia(&mut children);
			self.bump_into(&mut children);
			children.push(self.parse_expression_bp(right_bp).into());
			left = SyntaxNode::new(kind, children);
		}

		left
	}

	fn parse_prefix_expression(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		self.eat_trivia(&mut children);
		match self.current_kind() {
			TokenKind::Identifier => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::NameExpr, children)
			}
			TokenKind::Integer | TokenKind::Float | TokenKind::String => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::LiteralExpr, children)
			}
			TokenKind::Keyword(Keyword::True | Keyword::False | Keyword::None) => {
				self.bump_into(&mut children);
				SyntaxNode::new(SyntaxKind::LiteralExpr, children)
			}
			TokenKind::Plus
			| TokenKind::Minus
			| TokenKind::Bang
			| TokenKind::Keyword(Keyword::Not | Keyword::Await) => {
				self.bump_into(&mut children);
				children.push(self.parse_expression_bp(15).into());
				SyntaxNode::new(SyntaxKind::UnaryExpr, children)
			}
			TokenKind::LParen => {
				self.bump_into(&mut children);
				children.push(self.parse_expression().into());
				self.expect(
					&mut children,
					TokenKind::RParen,
					"expected `)` after expression",
				);
				SyntaxNode::new(SyntaxKind::ParenthesizedExpr, children)
			}
			_ => {
				self.error_here("VPAR002", "expected expression");
				if !is_expression_boundary(self.current_kind()) {
					self.bump_into(&mut children);
				}
				SyntaxNode::new(SyntaxKind::Error, children)
			}
		}
	}

	fn parse_argument_list(&mut self) -> SyntaxNode {
		let mut children = Vec::new();
		let mut seen_named = false;
		self.expect(
			&mut children,
			TokenKind::LParen,
			"expected `(` before arguments",
		);

		loop {
			self.eat_trivia(&mut children);
			if self.at(TokenKind::RParen) || self.at(TokenKind::Eof) {
				break;
			}

			let is_named = self.at(TokenKind::Identifier)
				&& self.peek_significant_kind(1) == Some(TokenKind::Equal);
			let argument = if is_named {
				seen_named = true;
				let mut argument_children = Vec::new();
				self.bump_into(&mut argument_children);
				self.expect(
					&mut argument_children,
					TokenKind::Equal,
					"expected `=` in named argument",
				);
				argument_children.push(self.parse_expression().into());
				SyntaxNode::new(SyntaxKind::NamedArgument, argument_children)
			} else {
				if seen_named {
					self.error_here(
						"VPAR007",
						"positional argument cannot follow a named argument",
					);
				}
				SyntaxNode::new(
					SyntaxKind::PositionalArgument,
					vec![self.parse_expression().into()],
				)
			};
			children.push(argument.into());
			self.eat_trivia(&mut children);

			if self.at(TokenKind::Comma) {
				self.bump_into(&mut children);
				continue;
			}
			if !self.at(TokenKind::RParen) {
				self.error_here("VPAR006", "expected `,` or `)` after argument");
				children.push(self.recover_until(&[TokenKind::Comma, TokenKind::RParen]).into());
				if self.at(TokenKind::Comma) {
					self.bump_into(&mut children);
				}
			}
		}

		self.expect(
			&mut children,
			TokenKind::RParen,
			"expected `)` after arguments",
		);
		SyntaxNode::new(SyntaxKind::ArgumentList, children)
	}

	fn finish_simple_line(&mut self, children: &mut Vec<SyntaxElement>) {
		self.eat_trivia(children);
		if self.at(TokenKind::Newline) {
			self.bump_into(children);
		} else {
			self.error_here("VPAR001", "expected end of logical line");
			if !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
				children.push(self.recover_line("VPAR001", "unexpected tokens after statement").into());
			}
		}
	}

	fn recover_line(&mut self, code: &'static str, message: &'static str) -> SyntaxNode {
		let mut children = Vec::new();
		if children.is_empty() {
			self.diagnostics
				.push(Diagnostic::error(code, message, self.current_span()));
		}
		while !matches!(
			self.current_kind(),
			TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
		) {
			self.bump_into(&mut children);
		}
		if self.at(TokenKind::Newline) {
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Error, children)
	}

	fn recover_until(&mut self, stop: &[TokenKind]) -> SyntaxNode {
		let mut children = Vec::new();
		while !stop.contains(&self.current_kind())
			&& !matches!(self.current_kind(), TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof)
		{
			self.bump_into(&mut children);
		}
		SyntaxNode::new(SyntaxKind::Error, children)
	}

	fn expect(&mut self, children: &mut Vec<SyntaxElement>, kind: TokenKind, message: &'static str) {
		self.eat_trivia(children);
		if self.at(kind) {
			self.bump_into(children);
		} else {
			self.error_here("VPAR001", message);
		}
	}

	fn expect_keyword(
		&mut self,
		children: &mut Vec<SyntaxElement>,
		keyword: Keyword,
		message: &'static str,
	) {
		self.expect(children, TokenKind::Keyword(keyword), message);
	}

	fn eat_trivia(&mut self, children: &mut Vec<SyntaxElement>) {
		while self.current_kind().is_trivia() {
			self.bump_into(children);
		}
	}

	fn bump_into(&mut self, children: &mut Vec<SyntaxElement>) {
		if let Some(token) = self.tokens.get(self.position).copied() {
			children.push(token.into());
			self.position += 1;
		}
	}

	fn error_here(&mut self, code: &'static str, message: &'static str) {
		self.diagnostics
			.push(Diagnostic::error(code, message, self.current_span()));
	}

	fn at(&self, kind: TokenKind) -> bool {
		self.current_kind() == kind
	}

	fn at_keyword(&self, keyword: Keyword) -> bool {
		self.at(TokenKind::Keyword(keyword))
	}

	fn current_kind(&self) -> TokenKind {
		self.tokens
			.get(self.position)
			.map_or(TokenKind::Eof, |token| token.kind)
	}

	fn current_span(&self) -> Span {
		self.tokens
			.get(self.position)
			.map_or_else(|| Span::empty(0), |token| token.span)
	}

	fn peek_significant_kind(&self, significant_offset: usize) -> Option<TokenKind> {
		let mut seen = 0;
		for token in &self.tokens[self.position..] {
			if token.kind.is_trivia() {
				continue;
			}
			if seen == significant_offset {
				return Some(token.kind);
			}
			seen += 1;
		}
		None
	}
}

fn infix_binding_power(kind: TokenKind) -> Option<(u8, u8, SyntaxKind)> {
	let result = match kind {
		TokenKind::Equal
		| TokenKind::PlusEqual
		| TokenKind::MinusEqual
		| TokenKind::StarEqual
		| TokenKind::SlashEqual
		| TokenKind::PercentEqual => (1, 1, SyntaxKind::AssignmentExpr),
		TokenKind::Keyword(Keyword::Or) | TokenKind::OrOr => (3, 4, SyntaxKind::BinaryExpr),
		TokenKind::Keyword(Keyword::And) | TokenKind::AndAnd => (5, 6, SyntaxKind::BinaryExpr),
		TokenKind::EqualEqual | TokenKind::BangEqual => (7, 8, SyntaxKind::BinaryExpr),
		TokenKind::Less | TokenKind::LessEqual | TokenKind::Greater | TokenKind::GreaterEqual => {
			(9, 10, SyntaxKind::BinaryExpr)
		}
		TokenKind::Plus | TokenKind::Minus => (11, 12, SyntaxKind::BinaryExpr),
		TokenKind::Star | TokenKind::Slash | TokenKind::Percent => {
			(13, 14, SyntaxKind::BinaryExpr)
		}
		_ => return None,
	};
	Some(result)
}

fn is_expression_boundary(kind: TokenKind) -> bool {
	matches!(
		kind,
		TokenKind::Newline
			| TokenKind::Dedent
			| TokenKind::Eof
			| TokenKind::Comma
			| TokenKind::RParen
			| TokenKind::RBracket
			| TokenKind::Colon
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn has_diagnostic(parse: &Parse, code: &str) -> bool {
		parse
			.diagnostics
			.iter()
			.any(|diagnostic| diagnostic.code == code)
	}

	#[test]
	fn cst_round_trips_source_with_trivia() {
		let source = "# lead\nimport A as B from \"./a.voil\" # keep\n\nstate count: Int = 1\n";
		let parsed = parse(source);

		assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
		assert_eq!(parsed.root.source_text(source), source);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::ImportDecl), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::BindingDecl), 1);
	}

	#[test]
	fn parses_function_parameters_defaults_and_return_type() {
		let source = "fn add(a: Int, b: Int = 2) -> Int:\n\treturn a + b * 2\n";
		let parsed = parse(source);

		assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::FunctionDecl), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::Parameter), 2);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::BinaryExpr), 2);
		assert_eq!(parsed.root.source_text(source), source);
	}

	#[test]
	fn parses_namespace_and_extern_imports() {
		let source = "import * as store from \"./store.voil\"\nextern import * as lib from \"pkg\"\n";
		let parsed = parse(source);

		assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::ImportDecl), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::ExternImportDecl), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::NamespaceImport), 2);
	}

	#[test]
	fn parses_if_for_key_and_lifecycle_blocks() {
		let source = "init:\n\tstate ready = true\nif ready:\n\tfor user in users key user.id:\n\t\tshow(user)\nelse:\n\tshow_empty()\n";
		let parsed = parse(source);

		assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::LifecycleBlock), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::IfStmt), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::ForStmt), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::KeyClause), 1);
		assert_eq!(parsed.root.source_text(source), source);
	}

	#[test]
	fn rejects_positional_argument_after_named_argument() {
		let parsed = parse("call(a=1, 2)\n");

		assert!(has_diagnostic(&parsed, "VPAR007"));
		assert_eq!(parsed.root.descendant_count(SyntaxKind::NamedArgument), 1);
		assert_eq!(parsed.root.descendant_count(SyntaxKind::PositionalArgument), 1);
	}

	#[test]
	fn recovers_after_malformed_statement() {
		let source = "state = 1\nstate good = 2\n";
		let parsed = parse(source);

		assert!(!parsed.diagnostics.is_empty());
		assert_eq!(parsed.root.descendant_count(SyntaxKind::BindingDecl), 2);
		assert_eq!(parsed.root.source_text(source), source);
	}
}
