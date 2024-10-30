use tree_sitter::Node;

use crate::ast::AstNode;

/// Top level collection
pub fn get_children_of_kind<'tree>(node: &Node<'tree>, kind: &str) -> Vec<Node<'tree>> {
    let mut res = vec![];
    for cid in 0..node.child_count() {
        let child = node.child(cid).unwrap();
        if child.kind() == kind {
            res.push(child);
        }
        res.append(&mut get_children_of_kind(&child, kind));
    }
    res
}

/// Return the closest
pub fn get_parent_of_kind<'tree>(node: &Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut nullable_parent = node.parent();
    let mut the_parent = None;
    while nullable_parent.is_some() {
        let parent = nullable_parent.unwrap();
        if parent.kind() == kind {
            the_parent = Some(parent);
            break;
        }
        nullable_parent = parent.parent();
    }
    the_parent
}

pub fn get_var_name_from_decl<'tree>(decl: &Node<'tree>) -> Node<'tree> {
    let var_name = decl
        .child_by_field_name("declarator")
        .unwrap()
        .child_by_field_name("declarator")
        .unwrap();
    if var_name.kind() == "pointer_declarator" {
        var_name.child_by_field_name("declarator").unwrap()
    } else {
        var_name
    }
}

pub fn get_var_name_from_assign<'tree>(assign: &Node<'tree>) -> Node<'tree> {
    assert_eq!(assign.kind(), "assignment_expression");
    let var_name = assign.child_by_field_name("left").unwrap();
    if var_name.kind() == "pointer_declarator" {
        var_name.child_by_field_name("declarator").unwrap()
    } else {
        var_name
    }
}

pub fn get_fn_identifier<'tree>(fn_def: &Node<'tree>) -> Node<'tree> {
    // Only one function declarator in one function definition
    let declarator = get_children_of_kind(fn_def, "function_declarator")[0];
    let identifier = declarator.child_by_field_name("declarator").unwrap();
    let identifier = if identifier.kind() == "function_declarator" {
        identifier.child_by_field_name("declarator").unwrap()
    } else {
        identifier
    };
    identifier
}

pub fn get_ident_from_call<'tree>(
    fn_call: &Node<'tree>,
    fn_name_str: &str,
    code: &str,
) -> Option<Node<'tree>> {
    assert_eq!(fn_call.kind(), "call_expression");

    let function = fn_call.child_by_field_name("function").unwrap();
    if function.kind() == "field_expression" {
        let ident = function.child_by_field_name("argument").unwrap();
        let fn_name = function.child_by_field_name("field").unwrap();
        if fn_name.to_source(code) == fn_name_str {
            return Some(ident);
        }
    }
    None
}