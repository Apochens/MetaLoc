#ifndef LLVM_TRANSFORM_UTILS_DL_MONITOR_H
#define LLVM_TRANSFORM_UTILS_DL_MONITOR_H

#include "llvm/IR/Module.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Analysis/LoopInfo.h"
#include "llvm/Analysis/LoopNestAnalysis.h"
#include "llvm/ADT/Hashing.h"
#include <iostream>

using namespace llvm;

using BBSet = DenseSet<BasicBlock *>;
using Inst = std::pair<hash_code, StringRef>;

/// @brief Debug location collector. 
///        Statically collect debug locations on simple paths in given CFG.
class DebugLocInfo {
public:
    DebugLocInfo(Function *F) {
        outs() << "|===============<" << F->getName() << ">===============|\n";
        collect(F);
    }

    DenseSet<int> &queryDebugLocSet(hash_code InstHash) {
        return BBToDebugLocs[InstToBB[InstHash]];
    }
private:
    DenseMap<hash_code, hash_code> InstToBB;
    DenseMap<hash_code, DenseSet<int>> BBToDebugLocs;

    BasicBlock *EntryBB;
    BBSet ExitBBs;
    DenseMap<BasicBlock *, unsigned> BBCount;


    void collect(Function *F) {
        collectEntryAndExitBBs(F);
        // compute all debug locations in each basic block
        for (BasicBlock &BB: *F) {
            // collectDebugLocationsOnSimplePathsOf(&BB);
            collectCompDLOnControlFlowPaths(&BB);
            outs() << "DLSet<" << BB.getName() << ">: ";
            for (int line: BBToDebugLocs[hash_value(&BB)]) {
                outs() << line << " ";
            }
            outs() << "\n";
            for (Instruction &I: BB) {
                InstToBB[hash_value(&I)] = hash_value(&BB);
            }
        }
    }

    /// Record the entry block and all exit blocks
    void collectEntryAndExitBBs(Function *F) {
        EntryBB = &F->getEntryBlock();
        for (BasicBlock &BB: *F) {
            BBCount[&BB] = 0;
            if (isa<ReturnInst>(BB.getTerminator())) {
                ExitBBs.insert(&BB);
            }
        }
    }

    void collectCompDLOnControlFlowPaths(BasicBlock *TargetBB) {
        // Collect the compatible debug locations
        BBToDebugLocs[hash_value(TargetBB)] = {};
        DFS4TargetBB(EntryBB, TargetBB);
    }

    void DFS4TargetBB(BasicBlock *CurrentBB, BasicBlock *TargetBB) {
        // Reach the exits
        if (ExitBBs.contains(CurrentBB)) {
            BBCount[CurrentBB] += 1;
            if (BBCount[CurrentBB]) {
                for (auto [BB, count] : BBCount) {
                    if (count == 0) continue;
                    for (Instruction &I : *BB) {
                        if (const DebugLoc &DL = I.getDebugLoc()) {
                            if (DL.getLine() != 0)
                                BBToDebugLocs[hash_value(TargetBB)].insert(DL.getLine());
                        }
                    }
                }
            }
            BBCount[CurrentBB] -= 1;
            return ;
        }

        // If VisitedBBs contains CurrentBB, then CurrentBB must be the loop header.
        // We choose the successor block not contained in VistedBBs to exit the loop.
        bool IsHeader = BBCount[CurrentBB] > 0;

        BBCount[CurrentBB] += 1;
        for (succ_iterator SI = succ_begin(CurrentBB), EI = succ_end(CurrentBB); SI != EI; ++SI) {
            BasicBlock *SuccBB = *SI;
            if (IsHeader && BBCount[SuccBB])
                continue;
            DFS4TargetBB(SuccBB, TargetBB);
        }

        BBCount[CurrentBB] -= 1;
    }

    void DFS(BasicBlock *FromBB, BasicBlock *ToBB, BBSet &VisitedBBs, bool IsBackward=false) {
        // If `BB` has been visited, there is a loop.
        if (VisitedBBs.count(FromBB)) return ;
        
        if (FromBB == ToBB) {
            // outs().changeColor(outs().BRIGHT_RED, true);
            // outs() << "Path<" << ToBB->getName() << ">: ";
            // Save all debug locations along the path
            for (BasicBlock *BB: VisitedBBs) {
                // outs() << BB->getName() << " ";
                for (Instruction &I: *BB) {
                    if (const DebugLoc &DL = I.getDebugLoc()) {
                        if (DL.getLine() != 0)
                            BBToDebugLocs[hash_value(ToBB)].insert(DL.getLine());
                    }
                }
            }
            // outs() << "\n";
            // outs().resetColor();
        } else {
            // Go to the next basic block
            VisitedBBs.insert(FromBB);
            if (IsBackward) {
                for (pred_iterator PI = pred_begin(FromBB), PE = pred_end(FromBB); PI != PE; ++PI) {
                    BasicBlock *PredBB = *PI;
                    DFS(PredBB, ToBB, VisitedBBs, IsBackward);
                }
            } else {
                for (succ_iterator SI = succ_begin(FromBB), SE = succ_end(FromBB); SI != SE; ++SI) {
                    BasicBlock *SuccBB = *SI;
                    DFS(SuccBB, ToBB, VisitedBBs, IsBackward);
                }
            }
        }

        VisitedBBs.erase(FromBB);
    }

    void collectDebugLocationsOnSimplePathsOf(BasicBlock *TargetBB) {
        BBToDebugLocs[hash_value(TargetBB)] = {};
        BBSet VisitedBBs;

        // Collect debug locations on simple paths [EntryBB, ..., TargetBB)
        // Do not collect debug locations in TargetBB to reduce the duplicate computation
        if (EntryBB != TargetBB)
            DFS(EntryBB, TargetBB, VisitedBBs);

        // Collect debug locations on simple paths (TargetBB, ..., ExitBB]
        // Do not collect debug locations in TargetBB to reduce the duplicate computation
        for (BasicBlock *ExitBB: ExitBBs) {
            if (ExitBB != TargetBB)
                DFS(ExitBB, TargetBB, VisitedBBs, true);
        }

        // Collect debug locations of TargetBB
        for (Instruction &I: *TargetBB) {
            if (const DebugLoc &DL = I.getDebugLoc()) {
                if (DL.getLine() != 0)
                    BBToDebugLocs[hash_value(TargetBB)].insert(DL.getLine());
            }
        }
    }
};

enum class UpdateKind {
    Preserve,
    Merge,
    Drop,
    None,
};

enum class InstKind {
    Create,
    Clone,
    Move,
    None,
};

enum class Event {
    Create,
    Clone,
    Move,
    UseReplace,
};

class DLStat {
public:
    DLStat(InstKind IK, unsigned SL, StringRef VN)
        : IK(IK), VarName(VN), SrcLine(SL), UK(UpdateKind::None) {}

    void addSrc(hash_code SrcHash) { Srcs.insert(SrcHash); }
    DenseSet<hash_code> &srcs() { return Srcs; }

    unsigned getLine() const { return SrcLine; }
    StringRef getName() const { return VarName; }

    InstKind getInstKind() const { return IK; }

    void addEvent(Event E, unsigned SrcLine) {
        Events.push_back({E, SrcLine});
    }

    void printEvents(raw_fd_ostream &outs) {
        for (auto E: Events) {
            if (E.first == Event::Create)
                outs << "(" << "Create" << ", " << E.second << ")";
            if (E.first == Event::Clone)
                outs << "(" << "Clone" << ", " << E.second << ")";
            if (E.first == Event::Move)
                outs << "(" << "Move" << ", " << E.second << ")";
            if (E.first == Event::UseReplace)
                outs << "(" << "UseReplace" << ", " << E.second << ")";
        }   
    }
private:
    InstKind IK;
    StringRef VarName;
    unsigned SrcLine;

    UpdateKind UK;
    DenseSet<hash_code> Srcs;
    SmallVector<std::pair<Event, unsigned>> Events;
};

/// @brief Debug Location Monitor
class DLMonitor {
public:
    DenseMap<hash_code, DLStat *> InstToStat;
    DenseMap<hash_code, Inst> BBToNewTerm; // Map a basic block to its terminator
    DenseMap<hash_code, Inst> BBToOldTerm; // Map a basic block to its terminator

    DLMonitor(Function &F, StringRef PN)
        : PassName(PN), TargetF(&F) 
    {
        DebugLocBeforeOpt = new DebugLocInfo(&F);

        // Open the log output stream
        StringRef LogDir = "./tmp";
        sys::fs::create_directories(LogDir);
        std::error_code ErrorCode;
        Twine FileName = LogDir + PassName;
        Logs = new raw_fd_ostream(FileName.str(), ErrorCode, sys::fs::OpenFlags::OF_Append);
    }

    ~DLMonitor() {
        delete DebugLocBeforeOpt;
        delete DebugLocAfterOpt;
        delete Logs;
    }

    void onOptFinished() {
        DebugLocAfterOpt = new DebugLocInfo(TargetF);

        for (auto [Dst, Stat]: InstToStat) {
            outs() << "Checking " << Stat->getName() << "...\n";
            DenseSet<int> &DebugLocsOfDst = DebugLocAfterOpt->queryDebugLocSet(Dst);
            DenseSet<int> DebugLocsOfSrc = {};

            DenseSet<hash_code> Srcs = Stat->srcs();
            int NumberOfSrc = Srcs.size();

            // If the instruction does not replace any other instructions
            if (NumberOfSrc == 0) {
                outs().changeColor(outs().YELLOW, true);
                outs() << "WARNING: ";
                outs().resetColor();
                if (Stat->getInstKind() == InstKind::Create) {
                    outs() << "No debug location for ";
                }
                if (Stat->getInstKind() == InstKind::Clone) {
                    outs() << "Retain original debug location for ";
                }
                outs() << Stat->getName() << " at line " << Stat->getLine() << "\n";
                continue;
            }

            // Debug location conflict detection
            bool HasConflict = false;
            for (hash_code Src: Srcs)
                for (int Line: DebugLocBeforeOpt->queryDebugLocSet(Src))
                    DebugLocsOfSrc.insert(Line);
            for (int Line: DebugLocsOfDst)
                if (!DebugLocsOfSrc.contains(Line)) {
                    HasConflict = true;
                    break;
                }

            if (HasConflict) {
                // Should drop the debug location
                outs().changeColor(outs().RED, true);
                outs() << "DROP: ";
                outs().resetColor();
            } else {
                if (NumberOfSrc == 1) {
                    outs().changeColor(outs().RED, true);
                    outs() << "PRESERVE: ";
                    outs().resetColor();
                } else {
                    outs().changeColor(outs().RED, true);
                    outs() << "MERGE: ";
                    outs().resetColor();
                }
            }
            outs() << Stat->getName() << " at line " << Stat->getLine() << " with events [";
            Stat->printEvents(outs());
            outs() << "]\n";
        }
    }
private:
    StringRef PassName;
    Function *TargetF;
    raw_fd_ostream *Logs;

    DebugLocInfo *DebugLocBeforeOpt;
    DebugLocInfo *DebugLocAfterOpt;

    raw_fd_ostream &logs() { return *Logs; }
};

namespace hook {
    DLMonitor *DLM = nullptr;
    /*
     * Analysis initialization and finalization
     */
    void OnStart(Function &F, StringRef PassName) {
        DLM = new DLMonitor(F, PassName);
    }

    void OnStart(Loop &L, StringRef PassName) {
        DLM = new DLMonitor(*L.getLoopPreheader()->getParent(), PassName);
    }

    void OnStart(LoopNest &LN, StringRef PassName) {
        DLM = new DLMonitor(*LN.getParent(), PassName);
    }

    void OnFinish() {
        DLM->onOptFinished();
        delete DLM;
    }

    /*
     * Track instruction manipulations
     */
    void OnCreate(Value *V, unsigned SrcLine, StringRef VarName) {
        Instruction *I = dyn_cast<Instruction>(V);
        if (I == nullptr)
            return ;

        hash_code HashOfInst = hash_value(I);
        DLM->InstToStat[HashOfInst] = new DLStat(InstKind::Create, SrcLine, VarName);
        DLM->InstToStat[HashOfInst]->addEvent(Event::Create, SrcLine);

        if (I->isTerminator()) {
            hash_code HashOfBB = hash_value(I->getParent());
            if (!DLM->BBToOldTerm.contains(HashOfBB)) {
                DLM->BBToNewTerm[HashOfBB] = {HashOfInst, VarName};
            } else {
                DLM->InstToStat[HashOfInst]->addSrc(DLM->BBToOldTerm[HashOfBB].first);
                DLM->InstToStat[HashOfInst]->addEvent(Event::UseReplace, 0);
                DLM->BBToOldTerm.erase(HashOfBB);
            }
        }
    }

    void OnMove(Value *V, unsigned SrcLine, StringRef VarName) {
        if (Instruction *I = dyn_cast<Instruction>(V)) {
            DLM->InstToStat[hash_value(I)] = new DLStat(InstKind::Move, SrcLine, VarName);
            DLM->InstToStat[hash_value(I)]->addSrc(hash_value(I));
            DLM->InstToStat[hash_value(I)]->addEvent(Event::Move, SrcLine);
        }
    }

    void OnClone(Value *NV, Value *OV, unsigned SrcLine, StringRef VarName) {
        Instruction *NI = dyn_cast<Instruction>(NV);
        Instruction *OI = dyn_cast<Instruction>(OV);

        if (!NI || !OI) return ;

        DLM->InstToStat[hash_value(NI)] = new DLStat(InstKind::Clone, SrcLine, VarName);
        DLM->InstToStat[hash_value(NI)]->addSrc(hash_value(OI));
        DLM->InstToStat[hash_value(NI)]->addEvent(Event::Clone, SrcLine);
    }

    void OnUseReplace(Value *From, Value *To, unsigned SrcLine, StringRef VarName) {
        Instruction *FromI = dyn_cast<Instruction>(From);
        Instruction *ToI = dyn_cast<Instruction>(To);

        if (!FromI || !ToI) return ;

        if (DLM->InstToStat.contains(hash_value(ToI))) {
            DLM->InstToStat[hash_value(ToI)]->addSrc(hash_value(FromI));
            DLM->InstToStat[hash_value(ToI)]->addEvent(Event::UseReplace, SrcLine);
        }
    }

    void OnRemove(Value *DV, unsigned SrcLine, StringRef VarName) {
        Instruction *DI = dyn_cast<Instruction>(DV);
        if (DI == nullptr)
            return ;
        hash_code HashOfInst = hash_value(DI);

        // Process terminators
        if (DI->isTerminator()) {
            hash_code HashOfBB = hash_value(DI->getParent());
            if (!DLM->BBToNewTerm.contains(HashOfBB)) {
                DLM->BBToOldTerm[HashOfBB] = { HashOfInst, VarName };
            } else {
                hash_code UDst = DLM->BBToNewTerm[HashOfBB].first;
                DLM->InstToStat[UDst]->addSrc(HashOfInst);
                DLM->InstToStat[UDst]->addEvent(Event::UseReplace, SrcLine);
                DLM->BBToNewTerm.erase(HashOfBB);
            }
        }
    }

    /*
     * Track debug location updates
     */
    void OnPreserve(DLMonitor *DLM);
    void OnMerge(DLMonitor *DLM);
    void OnDrop(DLMonitor *DLM);
}

#endif // LLVM_TRANSFORM_UTILS_DL_MONITOR_H