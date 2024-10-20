use std::fmt::{Display, Formatter};

use crate::parse::Expr;

use super::ParseTree;

enum BackingLink<'a> {
    Nil,
    Link(bool, &'a BackingLink<'a>),
}

fn show_backlink(bl: &BackingLink, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    match bl {
        BackingLink::Nil => Ok(()),
        BackingLink::Link(left, next) => {
            show_backlink(next, f)?;
            write!(f, "{}", if *left { "│ " } else { "  " })
        }
    }
}

fn display_inner(
    depth: usize,
    backing_needed: &BackingLink,
    left: bool,
    node: &Expr,
    buf: &[Expr],
    f: &mut Formatter,
) -> Result<(), std::fmt::Error> {
    let bl = match backing_needed {
        BackingLink::Nil => &BackingLink::Nil,
        BackingLink::Link(_, next) => next,
    };
    show_backlink(bl, f)?;
    if depth > 0 {
        write!(f, "{}", if left { "├─" } else { "└─" })?;
    }
    match node {
        Expr::Int(value) => {
            writeln!(f, "{value}")
        }
        Expr::String(value) => {
            writeln!(f, "{value}")
        }
        Expr::SExpr(exprs) => {
            writeln!(f, "List")?;
            for (i, node) in exprs.iter().enumerate() {
                let node = &buf[*node as usize];
                let non_right = i + 1 != exprs.len();
                display_inner(
                    depth + 1,
                    &BackingLink::Link(non_right, backing_needed),
                    non_right,
                    node,
                    buf,
                    f,
                )?;
            }
            Ok(())
        }
    }
}

impl<'a> Display for ParseTree<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        for idx in &self.top_level_exprs {
            let node = &self.buffer[*idx as usize];
            display_inner(0, &BackingLink::Nil, true, node, &self.buffer, f)?;
        }
        Ok(())
    }
}
