# METALOC: Robustifying Debug Location Updates in LLVM

## Main Components

1. AST-level pass instrumentation tool (Rust, under `src/`)
2. Build-in LLVM library (C++, under `library/`)

## Prerequisite

* rust compilation environment (cargo, rustc...)
* Provide your own LLVM project directory (modify the configurations in `config.json`). `"llvm"` gives the path to llvm root; `"opt"` specifies the path to `opt` binary (required for running instrumented optimizations).
```json
{
    "llvm": "your/path/to/llvm-project/llvm",
    "opt": "your/opt/binary",
}
```

## How to Use

### STEP 1: Copy the Library into LLVM

This step copies the implemented library to the directory of LLVM's include files, *i.e.*, `llvm/include/llvm/Transforms/Utils/`.
```bash
$ python3 script/metaloc.py setup
```

### STEP 2: Instrument the Target Optimization Pass

This step instruments a given LLVM optimization pass.

```bash
$ python3 script/metaloc.py instrument path/to/llvm/lib/Transforms/Scalar/TailRecursionElimination.cpp
```

After this step, the LLVM project (especially `opt`) should be rebuild.
```bash
$ ninja opt
```

### STEP 3: Run the Instrumented Optimization Pass

One can simply use `opt` to run the instrumented optimization pass with a specified test case.
```bash
$ opt -S -passes=tailcallelim dropping_debugloc_acc_rec_inst_rnew.ll --disable-output
```

Or use the script to run the instrumented pass with a bunch of test cases.
```bash
$ python3 script/metaloc.py run path/to/llvm/test/Transforms/TailCallElim/
```

In the output resutls, `PASS` indicates the written updates are consistent with the constructed updates, while `FAIL` indicates potential debug location update errors.