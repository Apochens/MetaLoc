pub const HEADER_INCLUDE: &str = "#include \"llvm/Transforms/Utils/DLMonitor.h\"\n";
pub const GLOBAL_VAR_DECL: &str = "namespace { DLMonitor *DLM = nullptr; }\n";

/// Hook for OnStart
pub fn on_start(pass_target: &str, pass_name: &str) -> String {
    format!("hook::OnStart({}, \"{}\")", pass_target, pass_name)
}

/// Hook for OnFinish
pub fn on_finish() -> String {
    format!("hook::OnFinish()")
}

pub fn on_create(val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnCreate({}, {}, \"{}\")", val, line, var_name)
}

pub fn on_move(val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnMove({}, {}, \"{}\")", val, line, var_name)
}

pub fn on_clone(new_val: &str, old_val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnClone({}, {}, {}, \"{}\")", new_val, old_val, line, var_name)
}

pub fn on_use_replace(from_val: &str, to_val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnUseReplace({}, {}, {}, \"{}\")", from_val, to_val, line, var_name)
}

pub fn on_remove(val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnRemove({}, {}, \"{}\")", val, line, var_name)
}
