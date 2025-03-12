# Artifact Evaluation

To eliminate the need for LLVM compilation and facilitate tool evaluation, we provide a pre-configured container.
The container includes:  
    (1) the tool's source code (`MetaLoc/`),  
    (2) the LLVM project (`llvm-trunk/`),  
    (3) three precompiled LLVM `opt` binaries along with the corresponding LLVM IR programs (`evaluation/`).

You can use the precompiled `opt` binaries to quickly evaluate our tool.
The naming format is "opt-\<fix commit id\>-\<instrumented pass\>-\<fix update operation\>".
These three binaries illustrate the three error examples presented in our paper:

- **evaluation/preserve**: This folder shows the update error in pass *LoopLoadElimination*, which is fixed by Preserve. The binary (built from commit 7c16e7d, the commit before the fix commit) includes the instrumented *LoopLoadElimination*. Running the binary with the LLVM IR program `looploadelim-preserve.ll` (which is the regression test added into LLVM by our work) yields the found update error and the constructed correct update.

```bash
$ ./opt-d0e2808-LoopLoadElim-Preserve looploadelim-preserve.ll -passes=loop-load-elim -S --disable-output

OUTPUT:
FAIL: LINE 452, PRESERVE(PHI, Cand.Load)
```

- **evaluation/merge**: This folder shows the update error in pass *GVNSink*, which is fixed by Merge. The binary (built from commit 6af4118, the commit before the fix commit) includes the instrumented *GVNSink*. Running the binary with the LLVM IR program `gvnsink-merge.ll` (which is the regression test added into LLVM by our work) yields the found update error and the constructed correct update.

```bash
$ ./opt-a487616-GVNSink-Merge gvnsink-merge.ll -passes=gvn-sink -S --disable-output

OUTPUT:
FAIL: LINE 914, MERGE(I0, I0, I)
```

- **evaluation/drop**: This folder shows the update error in pass *TailRecursionElimination*, which is fixed by Drop. The binary (built from commit b84323c, the commit before the fix commit) includes the instrumented *TailRecursionElimination*. Running the binary with the LLVM IR program `tailcallelim-drop.ll` (which is the regression test added into LLVM by our work) yields the found update error and the constructed correct update.


```bash
$ ./opt-ace069d-TailCallElim-Drop tailcallelim-drop.ll -passes=tailcallelim -S --disable-output

OUTPUT:
FAIL: LINE 777, DROP(AccRecInstrNew)
```