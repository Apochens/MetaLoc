#![allow(dead_code)]

use colored::Colorize;
use tree_sitter::Node;

pub trait AstNode {
    fn row(&self) -> usize;

    fn to_raw(&self, code: &str) -> String;
    fn to_source(&self, code: &str) -> String;
    fn dump_ast(&self);
    fn dump_source(&self, code: &str);

    fn is_header_include(&self) -> bool;
    fn is_using_decl(&self) -> bool;
    fn is_fn_def(&self) -> bool;
}

impl<'tree> AstNode for Node<'tree> {
    fn row(&self) -> usize {
        self.start_position().row + 1
    }

    fn to_raw(&self, code: &str) -> String {
        code[self.start_byte()..self.end_byte()].to_string()
    }

    fn to_source(&self, code: &str) -> String {
        let source: Vec<&str> = (&code[self.start_byte()..self.end_byte()])
            .split("\n")
            .map(|s| s.trim())
            .collect();
        source.join(" ")
    }

    fn dump_ast(&self) {
        println!("{}", self.to_sexp());
    }

    fn dump_source(&self, code: &str) {
        println!(
            "{} ({}): {}",
            self.start_position().row.to_string().red().bold(),
            self.kind().green().bold(),
            self.to_raw(code),
        );
    }

    fn is_fn_def(&self) -> bool {
        self.kind() == "function_definition"
    }
    fn is_header_include(&self) -> bool {
        self.kind() == "preproc_include"
    }
    fn is_using_decl(&self) -> bool {
        self.kind() == "using_declaration"
    }
}

// pub enum ASTNodeKind {
//     HeaderInclude,
//     UsingDecl,
//     FnDef,
//     CallExpr,
//     NewExpr,
//     FieldExpr,
// }

// impl ASTNodeKind {
//     pub fn to_string(&self) -> &'static str {
//         match self {
//             ASTNodeKind::HeaderInclude => "preproc_include",
//             ASTNodeKind::UsingDecl => "using_declaration",
//             ASTNodeKind::FnDef => "function_definition",
//             ASTNodeKind::CallExpr => "call_expression",
//             ASTNodeKind::NewExpr => "new_expression",
//             ASTNodeKind::FieldExpr => "field_expression",
//         }
//     }
// }

// impl Into<&str> for ASTNodeKind {
//     fn into(self) -> &'static str {
//         self.to_string()
//     }
// }