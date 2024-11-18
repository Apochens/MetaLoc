use std::collections::HashSet;

use colored::Colorize;
use tree_sitter::{Node, Parser};

use crate::{
    ast::AstNode,
    hook,
    r#match::{FnKind, FnMatch},
    visit::{
        get_children_of_kind, get_fn_identifier, get_parent_of_kind, get_var_name_from_assign,
        get_var_name_from_decl,
    },
};

enum EditKind {
    Insert,
    Replace(usize), // End Position
}

struct Edit {
    pub content: String,
    pub start_pos: usize,
    pub kind: EditKind,
}

impl Edit {
    pub fn new_insert(content: String, insert_pos: usize) -> Self {
        Self {
            content,
            start_pos: insert_pos,
            kind: EditKind::Insert,
        }
    }

    pub fn new_replace(content: String, start_pos: usize, end_pos: usize) -> Self {
        Self {
            content,
            start_pos,
            kind: EditKind::Replace(end_pos),
        }
    }
}

pub struct Instrumenter {
    parser: Parser,

    edits: Vec<Edit>,
    edit_record: HashSet<String>,

    target_file: String,
}

impl<'tree> Instrumenter {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let grammar = tree_sitter_cpp::language();
        parser
            .set_language(&grammar)
            .expect("Error loading grammar");

        Self {
            parser,
            edits: vec![],
            edit_record: HashSet::new(),
            target_file: String::new(),
        }
    }

    pub fn instrument(&mut self, filename: &str, code: &mut String) {
        self.target_file = filename.to_owned();

        self.collect_edits(code);

        self.edits.sort_by(|a, b| b.start_pos.cmp(&a.start_pos));
        for edit in &self.edits {
            match edit.kind {
                EditKind::Insert => {
                    code.insert_str(edit.start_pos, &edit.content);
                }
                EditKind::Replace(end_pos) => {
                    code.replace_range(edit.start_pos..end_pos, &edit.content);
                }
            }
        }
        self.reset();
    }
}

impl<'tree> Instrumenter {
    fn reset(&mut self) {
        self.edits = vec![];
        self.edit_record = HashSet::new();
        self.target_file = String::new();
    }

    fn add_insert_edit(&mut self, content: String, insert_pos: usize) {
        let edit_hash = insert_pos.to_string() + &content;
        if !self.edit_record.contains(&edit_hash) {
            self.edits.push(Edit::new_insert(content, insert_pos));
            self.edit_record.insert(edit_hash);
        }
    }

    fn add_replace_edit(&mut self, content: String, start_pos: usize, end_pos: usize) {
        let edit_hash = start_pos.to_string() + &content;
        if !self.edit_record.contains(&edit_hash) {
            self.edits
                .push(Edit::new_replace(content, start_pos, end_pos));
            self.edit_record.insert(edit_hash);
        }
    }

    fn collect_edits(&mut self, code: &str) {
        let tree = self
            .parser
            .parse(code, None)
            .expect("Failed to parse the code!");
        let root_node = tree.root_node();

        self.visit_header_includes(get_children_of_kind(&root_node, "preproc_include"));
        self.visit_fn_defs(
            get_children_of_kind(&root_node, "function_definition"),
            code,
        );
    }
}

impl<'tree> Instrumenter {
    /// Add the edit to include our header file
    fn visit_header_includes(&mut self, nodes: Vec<Node>) {
        assert_eq!(nodes.is_empty(), false, "No header files included!");
        self.add_insert_edit(hook::HEADER_INCLUDE.to_string(), nodes[0].start_byte());
    }

    fn visit_fn_defs(&mut self, nodes: Vec<Node>, code: &str) {
        for fn_def in nodes {
            if get_children_of_kind(&fn_def, "function_declarator").len() == 0 {
                println!(
                    "{} {} {}:\n\t{} {}",
                    "Warning".yellow().bold(),
                    "Encounter an function definition without declarator at line".bold(),
                    fn_def.row(),
                    "->".blue().bold(),
                    fn_def.to_source(code)
                );
                continue;
            }

            let fn_ident = get_fn_identifier(&fn_def);
            if fn_ident.to_source(code).is_pass_entry() {
                /* Add initialization and clean up */
                self.collect_init_and_clean_up_edit(&fn_def, code);
            } else {
                /* Process all function calls */
                self.visit_fn_calls(get_children_of_kind(&fn_def, "call_expression"), code);
                /* Process all object news */
                self.visit_new_exprs(get_children_of_kind(&fn_def, "new_expression"), code);
            }
        }
    }

    fn collect_init_and_clean_up_edit(&mut self, pass_entry: &Node, code: &str) {
        let param_list = pass_entry
            .child_by_field_name("declarator")
            .unwrap()
            .child_by_field_name("parameters")
            .unwrap();
        let params = get_children_of_kind(&param_list, "parameter_declaration");
        assert!(
            params.len() >= 1,
            "The pass entry should have the target parameters!"
        );

        let pass_target_type = params[0].child_by_field_name("type").unwrap();
        let pass_target_type_str = pass_target_type.to_source(code);

        let pass_target = match pass_target_type_str.as_str() {
            "Function" | "Loop" | "LoopNest" => {
                let target_ref = params[0]
                    .child_by_field_name("declarator")
                    .unwrap()
                    .child(1)
                    .unwrap();
                target_ref.to_source(code)
            }
            _ => unreachable!(),
        };

        let fn_body = pass_entry.child_by_field_name("body").unwrap();
        let init_str = format!("{};\n  ", hook::on_start(&pass_target, &self.target_file),);
        self.add_insert_edit(init_str, fn_body.child(1).unwrap().start_byte());

        let return_stmts = get_children_of_kind(&fn_body, "return_statement");
        for return_stmt in return_stmts {
            let insert_str = format!("{{ {}; ", hook::on_finish());
            self.add_insert_edit(insert_str, return_stmt.start_byte());
            let insert_str = format!(" }}");
            self.add_insert_edit(insert_str, return_stmt.end_byte());
        }
    }

    fn visit_fn_calls(&mut self, nodes: Vec<Node>, code: &str) {
        for call in nodes {
            let callee = call.child_by_field_name("function").unwrap();
            let arguments = call.child_by_field_name("arguments").unwrap();

            let mut callee_name = callee.to_source(code);
            match callee.kind() {
                "field_expression" => {
                    callee_name = callee.child_by_field_name("field").unwrap().to_source(code);
                }
                "qualified_identifier" => {}
                _ => continue,
            };

            match callee_name.get_fn_kind() {
                Some(FnKind::Create) => {
                    if let Some(parent_decl) = get_parent_of_kind(&call, "declaration") {
                        let var_name = get_var_name_from_decl(&parent_decl);
                        let insert_str = format!(
                            " {};",
                            hook::on_create(
                                &var_name.to_source(code),
                                parent_decl.row(),
                                &var_name.to_source(code)
                            )
                        );
                        self.add_insert_edit(insert_str, parent_decl.end_byte());
                        continue;
                    }

                    if let Some(parent_assign) = get_parent_of_kind(&call, "assignment_expression")
                    {
                        let var_name = get_var_name_from_assign(&parent_assign);

                        let insert_str = format!("{{ ");
                        self.add_insert_edit(insert_str, parent_assign.start_byte());

                        let insert_str = format!(
                            " {}; }}",
                            hook::on_create(
                                &var_name.to_source(code),
                                parent_assign.row(),
                                &var_name.to_source(code)
                            )
                        );
                        self.add_insert_edit(insert_str, parent_assign.end_byte() + 1);
                        continue;
                    }

                    if let Some(parent_return) = get_parent_of_kind(&call, "return_statement") {
                        let replace_str = format!(
                            "{{ auto *V = {}; {}; return V; }}",
                            call.to_source(code),
                            hook::on_create("V", parent_return.row(), ""),
                        );
                        self.add_replace_edit(
                            replace_str,
                            parent_return.start_byte(),
                            parent_return.end_byte(),
                        );
                        continue;
                    }

                    // Function calls like BranchInst::Create(...)
                    if let Some(parent) = call.parent() {
                        if parent.kind() == "expression_statement" {
                            let replace_str = format!(
                                "Instruction *I = {}; {};",
                                call.to_source(code),
                                hook::on_create("I", call.row(), "I"),
                            );
                            self.add_replace_edit(replace_str, call.start_byte(), call.end_byte());
                        }
                    }
                }
                /* auto *NI = OI->clone(); */
                Some(FnKind::Clone) => {
                    let original_inst = callee.child_by_field_name("argument").unwrap();
                    let addr_op = if callee.child(1).unwrap().to_source(code).as_str() == "->" {
                        ""
                    } else {
                        "&"
                    };
                    if let Some(parent_decl) = get_parent_of_kind(&call, "declaration") {
                        let var_name = get_var_name_from_decl(&parent_decl);

                        let insert_str = format!(
                            " {};",
                            hook::on_clone(
                                &var_name.to_source(code),
                                &format!("{}{}", addr_op, original_inst.to_source(code)),
                                parent_decl.row(),
                                &var_name.to_source(code),
                                &original_inst.to_source(code),
                            )
                        );
                        self.add_insert_edit(insert_str, parent_decl.end_byte());
                        continue;
                    }

                    if let Some(parent_assign) = get_parent_of_kind(&call, "assignment_expression")
                    {
                        let var_name = get_var_name_from_assign(&parent_assign);

                        let insert_str = format!("{{ ");
                        self.add_insert_edit(insert_str, parent_assign.start_byte());

                        let insert_str = format!(
                            " {}; }}",
                            hook::on_clone(
                                &var_name.to_source(code),
                                &format!("{}{}", addr_op, original_inst.to_source(code)),
                                parent_assign.row(),
                                &var_name.to_source(code),
                                &original_inst.to_source(code),
                            )
                        );
                        self.add_insert_edit(insert_str, parent_assign.end_byte() + 1);
                        continue;
                    }

                    call.dump_source(code);
                    // panic!("Failed to parse instruction clone!");
                }
                /* I->moveBefore(D, ..); */
                Some(FnKind::Move) => {
                    let move_target = callee.child_by_field_name("argument").unwrap();
                    let field_op = callee.child(1).unwrap().to_source(code);
                    let ref_op = if field_op.as_str() == "->" { "" } else { "&" };

                    self.add_insert_edit(String::from("{ "), call.start_byte());

                    let insert_str = format!(
                        " {}; }}",
                        hook::on_move(
                            &format!("{}{}", ref_op, move_target.to_source(code)),
                            call.row(),
                            &move_target.to_source(code),
                        )
                    );
                    self.add_insert_edit(insert_str, call.end_byte() + 1);
                }
                Some(FnKind::UseReplace) => {
                    /* The S-expr of `DLS->replaceAllUsesWith(DLD)` is shown as following:
                     *  (call_expression
                     *       function: (field_expression
                     *           argument: (identifier)
                     *           field: (field_identifier)
                     *       )
                     *       arguments: (argument_list
                     *           (identifier)
                     *       )
                     *  )
                     */
                    if callee_name.as_str() == "replaceAllUsesWith" {
                        let debugloc_src = callee.child_by_field_name("argument").unwrap();
                        let debugloc_dst = arguments.child(1).unwrap();

                        let field_operator = callee.child(1).unwrap().to_source(code);
                        let prepare_str = format!(
                            "Value *DebugLocSrc = {}{}; Value *DebugLocDst = {};",
                            if field_operator.as_str() == "." {
                                "&"
                            } else {
                                ""
                            },
                            debugloc_src.to_source(code),
                            debugloc_dst.to_source(code),
                        );
                        let replace_str = format!("DebugLocSrc->replaceAllUsesWith(DebugLocDst);");

                        let hook_str = format!(
                            "{};",
                            hook::on_use_replace(
                                "DebugLocSrc",
                                "DebugLocDst",
                                call.row(),
                                &debugloc_dst.to_source(code),
                                &debugloc_src.to_source(code),
                            )
                        );

                        let replace_str =
                            format!("{{ {} {} {} }}", prepare_str, replace_str, hook_str);
                        self.add_replace_edit(replace_str, call.start_byte(), call.end_byte() + 1);
                    }
                    if callee_name.as_str() == "replaceUsesOfWith" {
                        if call.parent().unwrap().kind() != "expression_statement" {
                            // panic!("{}\n  {} {}", "Non expression statement parent!".red().bold(), "-->".blue().bold(), call.parent().unwrap().to_source(code));
                            continue;
                        }

                        let called_obj = callee.child_by_field_name("argument").unwrap();
                        let old_inst = arguments.child(1).unwrap();
                        let new_inst = arguments.child(3).unwrap();

                        let field_operator = callee.child(1).unwrap().to_source(code);
                        let prepare_str = format!(
                            "Value *DebugLocSrc = {}; Value *DebugLocDst = {};",
                            old_inst.to_source(code),
                            new_inst.to_source(code)
                        );

                        let inst_repl_str = format!(
                            "{}{}replaceUsesOfWith(DebugLocSrc, DebugLocDst);",
                            called_obj.to_source(code),
                            field_operator
                        );
                        let hook_str = format!(
                            "{};",
                            hook::on_use_replace(
                                "DebugLocSrc",
                                "DebugLocDst",
                                call.row(),
                                &new_inst.to_source(code),
                                &old_inst.to_source(code),
                            )
                        );

                        let replace_str =
                            format!("{{ {} {} {} }}", prepare_str, inst_repl_str, hook_str);
                        self.add_replace_edit(replace_str, call.start_byte(), call.end_byte() + 1);
                    }
                }
                Some(FnKind::Remove) => {
                    let called_obj = callee.child_by_field_name("argument").unwrap();
                    let insert_str = format!(
                        "{{ {}; ",
                        hook::on_remove(
                            &called_obj.to_source(code),
                            call.start_position().row,
                            &called_obj.to_source(code)
                        )
                    );
                    self.add_insert_edit(insert_str, call.start_byte());
                    let insert_str = format!("}}");
                    self.add_insert_edit(insert_str, call.end_byte() + 1);
                }
                Some(FnKind::DLPreserve) => {
                    let dst_inst = callee.child_by_field_name("argument").unwrap();
                    let insert_str = format!("{{ ");
                    self.add_insert_edit(insert_str, call.start_byte());

                    let insert_str = format!(
                        " {}; }}",
                        hook::on_preserve(&dst_inst.to_source(code), call.row())
                    );
                    self.add_insert_edit(insert_str, call.end_byte() + 1);
                }
                Some(FnKind::DLMerge) => {
                    let dst_inst = callee.child_by_field_name("argument").unwrap();
                    let insert_str = format!("{{ ");
                    self.add_insert_edit(insert_str, call.start_byte());

                    let insert_str = format!(
                        " {}; }}",
                        hook::on_merge(&dst_inst.to_source(code), call.row())
                    );
                    self.add_insert_edit(insert_str, call.end_byte() + 1);
                }
                Some(FnKind::DLDrop) => {
                    let dst_inst = callee.child_by_field_name("argument").unwrap();
                    let insert_str = format!("{{ ");
                    self.add_insert_edit(insert_str, call.start_byte());

                    let insert_str = format!(
                        " {}; }}",
                        hook::on_drop(&dst_inst.to_source(code), call.row())
                    );
                    self.add_insert_edit(insert_str, call.end_byte() + 1);
                }
                _ => {}
            };
        }
    }

    fn visit_new_exprs(&mut self, nodes: Vec<Node>, code: &str) {
        for new in nodes {
            let new_type = new.child_by_field_name("type").unwrap();
            let new_type_str = new_type.to_source(code);
            if let Some(FnKind::Create) = new_type_str.get_fn_kind() {
                if let Some(parent_decl) = get_parent_of_kind(&new, "declaration") {
                    let var_name = get_var_name_from_decl(&parent_decl);
                    let insert_str = format!(
                        " {};",
                        hook::on_create(
                            &var_name.to_source(code),
                            new.row(),
                            &var_name.to_source(code),
                        )
                    );

                    self.add_insert_edit(insert_str, parent_decl.end_byte());
                    continue;
                }

                if let Some(parent_assign) = get_parent_of_kind(&new, "assignment_expression") {
                    let var_name = get_var_name_from_assign(&parent_assign);

                    let insert_str = format!("{{ ");
                    self.add_insert_edit(insert_str, parent_assign.start_byte());

                    let insert_str = format!(
                        " {}; }}",
                        hook::on_create(
                            &var_name.to_source(code),
                            new.row(),
                            &var_name.to_source(code),
                        )
                    );

                    self.add_insert_edit(insert_str, parent_assign.end_byte() + 1);
                    continue;
                }

                if let Some(parent_return) = get_parent_of_kind(&new, "return_statement") {
                    let insert_str = format!(
                        "{{ Value *V = {}; {}; return V; }}",
                        new.to_source(code),
                        hook::on_create("V", new.row(), "")
                    );
                    self.add_replace_edit(
                        insert_str,
                        parent_return.start_byte(),
                        parent_return.end_byte(),
                    );
                    continue;
                }

                println!(
                    "{} {} {}:\n\t{} {}",
                    "Warning".yellow().bold(),
                    "Encounter an unsupported new expression at line".bold(),
                    new.row(),
                    "->".blue().bold(),
                    new.to_source(code),
                );
            }
        }
    }
}
