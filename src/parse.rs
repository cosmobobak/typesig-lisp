mod tree_display;

use std::fmt::Display;

use smallvec::SmallVec;

use crate::tokenise::{Token, TokenStream};

pub fn parse(token_stream: TokenStream) -> ParseTree {
    ParseTree::new(token_stream)
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ParseTree<'a> {
    buffer: Vec<Expr<'a>>,
    top_level_exprs: Vec<u32>,
}

impl ParseTree<'_> {
    pub const fn stringify(&self) -> ParseTreeStringifier<'_, '_> {
        ParseTreeStringifier { pt: self }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum Expr<'a> {
    /// An integer.
    Int(i64),
    /// A string literal.
    String(&'a str),
    /// Subexpressions.
    SExpr(SmallVec<[u32; 8]>),
}

impl<'a> ParseTree<'a> {
    const EMPTY: Self = Self {
        buffer: Vec::new(),
        top_level_exprs: Vec::new(),
    };

    pub fn new(tokens: TokenStream<'a>) -> Self {
        let mut tree = Self::EMPTY;

        // bookkeeping for the current list
        // that we're injecting tokens into:
        let mut list_stack = Vec::with_capacity(16);

        for token in tokens {
            match token {
                Token::Literal(lit) => {
                    // add this new node as a list element
                    // in the surrounding sexpr:
                    let idx: u32 = tree.buffer.len().try_into().unwrap();
                    if let Some(&list) = list_stack.last() {
                        match &mut tree.buffer[list] {
                            Expr::Int(_) | Expr::String(_) => unreachable!(),
                            Expr::SExpr(sub_exprs) => sub_exprs.push(idx),
                        }
                    } else {
                        tree.top_level_exprs.push(idx);
                    }
                    match lit.parse::<i64>() {
                        Ok(int) => tree.buffer.push(Expr::Int(int)),
                        Err(_) => tree.buffer.push(Expr::String(lit)),
                    }
                }
                Token::LParen => {
                    let idx: u32 = tree.buffer.len().try_into().unwrap();
                    if let Some(&list) = list_stack.last() {
                        match &mut tree.buffer[list] {
                            Expr::Int(_) | Expr::String(_) => unreachable!(),
                            Expr::SExpr(sub_exprs) => sub_exprs.push(idx),
                        }
                    } else {
                        tree.top_level_exprs.push(idx);
                    }
                    list_stack.push(tree.buffer.len());
                    tree.buffer.push(Expr::SExpr(SmallVec::new()));
                }
                Token::RParen => {
                    list_stack.pop();
                }
            }
        }

        tree
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ParseTreeStringifier<'a, 'b> {
    pt: &'a ParseTree<'b>,
}

impl Display for ParseTreeStringifier<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_expr(expr: &Expr, buf: &[Expr], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match expr {
                Expr::Int(i) => write!(f, "{i}"),
                Expr::String(s) => write!(f, "{s}"),
                Expr::SExpr(sub_exprs) => {
                    write!(f, "(")?;
                    for (i, se_idx) in sub_exprs.iter().enumerate() {
                        if i != 0 {
                            write!(f, " ")?;
                        }
                        let expr = &buf[*se_idx as usize];
                        write_expr(expr, buf, f)?;
                    }
                    write!(f, ")")?;
                    Ok(())
                }
            }
        }

        for top_level_expr in &self.pt.top_level_exprs {
            write_expr(&self.pt.buffer[*top_level_expr as usize], &self.pt.buffer, f)?;
        }

        Ok(())
    }
}