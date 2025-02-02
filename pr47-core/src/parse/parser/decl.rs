#![allow(unused_variables)]
#![allow(dead_code)]

use super::Parser;

use crate::diag::diag_data;
use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::attr::Attribute;
use crate::syntax::decl::{
    ConcreteExportDecl,
    ConcreteFuncDecl,
    ConcreteImportDecl,
    ConcreteObjectDecl,
    ConcreteOpenImportDecl,
    FunctionParam
};
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::id::Identifier;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::ConcreteType;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_object_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteObjectDecl<'s>>
    {
        let kwd_range: SourceRange = kwd_token.range;
        let id: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None } )?;
        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let (obj_type, eq_range): (Option<ConcreteType>, SourceRange) =
            if self.current_token().token_inner == TokenInner::SymEq {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                (None, self.consume_token().range)
            } else {
                let ty: ConcreteType = self.parse_type(failsafe_set)?;
                let eq_range: SourceRange =
                    self.expect_n_consume(TokenInner::SymEq, failsafe_set)?.range;
                (Some(ty), eq_range)
            };
        let init_expr: ConcreteExpr = self.parse_expression_no_assign(failsafe_set)?;
        self.expect_n_consume(TokenInner::SymSemicolon, failsafe_set)?;
        Some(ConcreteObjectDecl {
            attr: None,
            name: id,
            obj_type,
            init_expr,
            kwd_range,
            eq_range
        })
    }

    pub fn parse_func_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteFuncDecl<'s>>
    {
        let func_kwd_range: SourceRange = kwd_token.range;
        let func_name: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None })?;
        let lparen_loc: SourceLoc =
            self.expect_n_consume(TokenInner::SymLParen, failsafe_set)?.range.left();
        let (func_param_list, rparen_range): (Vec<FunctionParam>, SourceRange) =
            self.parse_list_alike(
                Self::parse_func_param,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;

        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let func_return_types: Vec<ConcreteType<'s>> =
            if self.current_token().token_inner == TokenInner::SymLBrace ||
                self.current_token().token_inner == TokenInner::SymSemicolon
            {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                Vec::new()
            } else {
                self.parse_func_ret_type(failsafe_set)?
            };

        let func_body: Option<ConcreteCompoundStmt<'s>> =
            if self.current_token().token_inner == TokenInner::SymSemicolon {
                let _ = self.consume_token();
                None
            } else {
                let lbrace_token: Token<'s> = self.consume_token();
                self.parse_compound_stmt(lbrace_token, failsafe_set)
            };

        Some(ConcreteFuncDecl {
            attr: None,
            func_name,
            func_param_list,
            func_return_types,
            exception_spec: None,
            func_body,
            func_kwd_range,
            param_open_paren_loc: lparen_loc,
            param_close_paren_loc: rparen_range.left()
        })
    }

    pub fn parse_func_param(&mut self, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<FunctionParam<'s>>
    {
        let attr: Option<Attribute> = if self.current_token().token_inner == TokenInner::SymHash {
            let hash_token: Token<'s> = self.consume_token();
            Some(self.parse_attribute(hash_token, false, failsafe_set)?)
        } else {
            None
        };
        let param_name: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None })?;
        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let param_type: Option<ConcreteType> =
            if self.current_token().token_inner == TokenInner::SymComma ||
                self.current_token().token_inner == TokenInner::SymRParen
            {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                None
            } else {
                Some(self.parse_type(failsafe_set)?)
            };
        Some(FunctionParam {
            attr,
            param_name,
            param_type
        })
    }

    pub fn parse_func_ret_type(&mut self, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<Vec<ConcreteType<'s>>>
    {
        if self.current_token().token_inner == TokenInner::SymLParen {
            let _ = self.consume_token();
            let (types, _): (Vec<ConcreteType<'s>>, _) = self.parse_list_alike_nonnull(
                Self::parse_type,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;
            Some(types)
        } else {
            let ty: ConcreteType = self.parse_type(failsafe_set)?;
            Some(vec![ty])
        }
    }

    pub fn parse_export_decl(&mut self, kwd_token: Token<'s>, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteExportDecl<'s>>
    {
        todo!()
    }

    pub fn parse_import_decl(&mut self, kwd_token: Token<'s>, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteImportDecl<'s>>
    {
        todo!()
    }

    pub fn parse_open_import_decl(
        &mut self,
        kwd_token: Token<'s>,
        _failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteOpenImportDecl<'s>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;
    use crate::syntax::decl::{ConcreteFuncDecl, ConcreteObjectDecl};
    use crate::syntax::token::Token;

    #[test]
    fn test_parse_object_decl() {
        let source: &str = "const a = b::c::d.e().await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_object_decl2() {
        let source: &str = "const a vector<int> = b::c::d.e().await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_object_decl3() {
        let source: &str = "const a: vector<string> = b::c::d.e(f, g).await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_func() {
        let source: &str = "fn foo(bar int, #[reflect] baz: vector<string>) string;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let func: ConcreteFuncDecl = parser.parse_func_decl(kwd_token, &[]).unwrap();

        dbg!(func);
    }
}
