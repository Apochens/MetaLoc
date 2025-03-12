# METALOC: Robustifying Debug Location Updates in LLVM

For artifact evaluation, please check [EVALUATION.md](./EVALUATION.md).

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

## Compile LLVM

> If the LLVM has already been compiled, please skip.

1. Clone the LLVM repo and install build tools. For saving time, you can add the flag `--depth=1` when cloning.

```bash
$ git clone https://github.com/llvm/llvm-project.git
$ sudo apt install ninja-build cmake
```

2. Initialize the build directory.

```bash
$ cd llvm-project
$ cmake -S llvm -B ../build-llvm -G Ninja \
	-DCMAKE_BUILD_TYPE=Release -DLLVM_ENABLE_ASSERTIONS=On \
	-DCMAKE_INSTALL_PREFIX=../llvm
```

3. Build the `opt`

```bash
$ cd ../build-llvm && ninja opt
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

Or use the script to analyze the instrumented pass with a bunch of test cases.
```bash
$ python3 script/metaloc.py analyze path/to/llvm/test/Transforms/TailCallElim/
```

In the output, potential debug location update errors denoted by `FAIL` are printed along with the constructed proper updates.