pub const HEADER_INCLUDE: &str = "#include \"llvm/Transforms/Utils/DLMonitor.h\"\n";
pub const GLOBAL_VAR_DECL: &str = "namespace { DLMonitor *DLM = nullptr; }\n";

/// Hook for OnStart
pub fn on_start(pass_target: &str, pass_name: &str) -> String {
    format!("hook::OnStart(DLM, {}, \"{}\")", pass_target, pass_name)
}

/// Hook for OnFinish
pub fn on_finish() -> String {
    format!("hook::OnFinish(DLM)")
}

pub fn on_create(val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnCreate(DLM, {}, {}, \"{}\")", val, line, var_name)
}

pub fn on_move(val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnMove(DLM, {}, {}, \"{}\")", val, line, var_name)
}

pub fn on_clone(new_val: &str, old_val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnClone(DLM, {}, {}, {}, \"{}\")", new_val, old_val, line, var_name)
}

pub fn on_use_replace(from_val: &str, to_val: &str, line: usize, var_name: &str) -> String {
    format!("hook::OnUseReplace(DLM, {}, {}, {}, \"{}\")", from_val, to_val, line, var_name)
}

pub fn on_erase(val: &str, line: usize, var_name: &str) -> String {
    todo!()
}



