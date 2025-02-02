#![allow(unused_variables)]
#![allow(dead_code)]

//! # Concrete syntax tree of declarations
//!
//! Declaration syntax:
//! ```text
//! declaration ::= maybe-attributed-declaration
//!               | import-declaration
//!               | export-declaration
//!
//! maybe-attributed-declaration ::= maybe-attribute attr-able-declaration
//!
//! maybe-attribute ::= attribute
//!                   | NIL
//!
//! attr-able-declaration ::= const-declaration
//!                         | func-declaration
//!                         | var-declaration
//!
//! import-declaration ::= 'import' identifier ';'
//!
//! open-import-declaration ::= 'open' 'import' identifier 'using' '(' identifier-list ')';
//!
//! export-declaration ::= 'export' '(' non-empty-identifier-list ')' ';'
//!
//! identifier-list ::= non-empty-identifier-list
//!                   | NIL
//!
//! non-empty-identifier-list ::= non-empty-identifier-list ',' identifier
//!                             | identifier
//! ```

use smallvec::SmallVec;
use xjbutil::either::Either;

use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::attr::Attribute;
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::id::Identifier;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::ty::ConcreteType;

#[cfg_attr(test, derive(Debug))]
pub enum ConcreteDecl<'a> {
    ConstDecl(ConcreteObjectDecl<'a>),
    ExportDecl(ConcreteExportDecl<'a>),
    FuncDecl(ConcreteFuncDecl<'a>),
    ImportDecl(ConcreteImportDecl<'a>),
    OpenImportDecl(ConcreteOpenImportDecl<'a>),
    VarDecl(ConcreteObjectDecl<'a>)
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteObjectDecl<'a> {
    pub attr: Option<Attribute<'a>>,

    pub name: Identifier<'a>,
    pub obj_type: Option<ConcreteType<'a>>,
    pub init_expr: ConcreteExpr<'a>,

    pub kwd_range: SourceRange,
    pub eq_range: SourceRange
}

#[cfg_attr(test, derive(Debug))]
pub struct FunctionParam<'a> {
    pub attr: Option<Attribute<'a>>,

    pub param_name: Identifier<'a>,
    pub param_type: Option<ConcreteType<'a>>
}

#[cfg_attr(test, derive(Debug))]
pub struct FuncDeclExceptionSpec<'a> {
    pub exc_list: SmallVec<[ConcreteType<'a>; 4]>,
    pub throws_range: SourceRange,
    pub lparen_loc: SourceLoc,
    pub rparen_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteFuncDecl<'a> {
    pub attr: Option<Attribute<'a>>,

    pub func_name: Identifier<'a>,
    pub func_param_list: Vec<FunctionParam<'a>>,
    pub func_return_types: Vec<ConcreteType<'a>>,
    pub exception_spec: Option<FuncDeclExceptionSpec<'a>>,
    pub func_body: Option<ConcreteCompoundStmt<'a>>,

    pub func_kwd_range: SourceRange,
    pub param_open_paren_loc: SourceLoc,
    pub param_close_paren_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteImportDecl<'a> {
    pub import_path: Identifier<'a>,
    pub import_kwd_range: SourceRange
}

#[cfg_attr(test, derive(Debug))]
pub struct OpenImportUsingAny {
    aster_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct OpenImportUsingList<'a> {
    used_idents: Vec<Identifier<'a>>,
    left_paren_loc: SourceLoc,
    right_paren_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteOpenImportDecl<'a> {
    pub import_path: Identifier<'a>,
    pub open_kwd_range: SourceRange,
    pub import_kwd_range: SourceRange,
    pub using_kwd_range: SourceRange,
    pub used_content: Either<OpenImportUsingAny, OpenImportUsingList<'a>>
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteExportDecl<'a> {
    pub export_path: Identifier<'a>,
    pub export_kwd_range: SourceRange
}
