const CREATE_FN: [&str; 38] = [
    "AllocaInst",
    "BinaryOperator::Create", /* BinaryOperator */
    "BranchInst::Create",     /* BranchInst */
    "CallBase::Create",       /* CallBase */
    "CallBase::addOperandBundle",
    "CallBase::removeOperandBundle",
    "CallBrInst::Create",         /* CallBrInst */
    "CallInst::Create",           /* CallInst */
    "CmpInst::Create",            /* CmpInst */
    "FCmpInst",                   /* FCmpInst */
    "ICmpInst",                   /* ICmpInst */
    "ExtractElementInst::Create", /* ExtractElementInst */
    "GetElementPtrInst::Create",  /* GetElementPtrInst */
    "InsertElementInst::Create",  /* InsertElementInst */
    "InsertValueInst::Create",    /* InsertValueInst */
    "PHINode::Create",            /* PHINode */
    "ReturnInst::Create",         /* ReturnInst */
    "SelectInst::Create",         /* SelectInst */
    "StoreInst",                  /* StoreInst */
    "SwitchInst::Create",         /* SwitchInst */
    "UnaryOperator::Create",
    "LoadInst",
    "FreezeInst",
    "ExtractValueInst::Create",
    "CastInst::Create", /* CastInst */
    "AddrSpaceCastInst",
    "BitCastInst",
    "FPExtInst",
    "FPToSIInst",
    "FPToUIInst",
    "FPTruncInst",
    "IntToPtrInst",
    "PtrToIntInst",
    "SExtInst",
    "SIToFPInst",
    "TruncInst",
    "UIToFPInst",
    "ZExtInst",
];

const CLONE_FN: [&str; 1] = ["clone"];

const MOVE_FN: [&str; 3] = ["moveBefore", "moveBeforePreserving", "moveAfter"];

const USE_REPLACE_FN: [&str; 2] = ["replaceAllUsesWith", "replaceUsesOfWith"];

const DL_UPDATE_FN: [&str; 4] = ["setDebugLoc", "applyMergedLocation", "dropLocation", "updateLocationAfterHoist"];

// const INSERT_FN: [&str; 3] = ["insertBefore", "insertAfter", "insertInto"];

const REMOVE_FN: [&str; 1] = ["eraseFromParent"];

#[derive(PartialEq)]
pub enum FnKind {
    Create,
    Clone,
    Move,
    UseReplace,
    DLUpdate,
    Remove,
}

pub trait FnMatch {
    fn get_fn_kind(&self) -> Option<FnKind>;
    fn is_pass_entry(&self) -> bool;
}

impl FnMatch for String {
    fn get_fn_kind(&self) -> Option<FnKind> {
        if CLONE_FN.contains(&self.as_str()) {
            return Some(FnKind::Clone);
        }
        if MOVE_FN.contains(&self.as_str()) {
            return Some(FnKind::Move);
        }
        for prefix in CREATE_FN {
            if (prefix.contains("::") && self.starts_with(prefix)) || self.as_str() == prefix {
                return Some(FnKind::Create);
            }
        }
        if USE_REPLACE_FN.contains(&self.as_str()) {
            return Some(FnKind::UseReplace);
        }
        if DL_UPDATE_FN.contains(&self.as_str()) {
            return Some(FnKind::DLUpdate);
        }
        if REMOVE_FN.contains(&self.as_str()) {
            return Some(FnKind::Remove);
        }
        None
    }

    fn is_pass_entry(&self) -> bool {
        self.ends_with("Pass::run")
    }
}
