import subprocess
import sys
import logging
import os
import shutil
import json
from enum import StrEnum


logging.basicConfig(level=logging.INFO)


METALOC_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LLVM_ROOT = ""
LLVM_OPT = ""


class Task(StrEnum):
    SETUP = "setup"
    INSTRUMENT = "instrument"
    RUN = "run"
    CLEAN = "clean"


def config_parse(task: str):
    config = json.load(open(os.path.join(METALOC_ROOT, "config.json"), "r"))

    # check if the llvm path is valid
    global LLVM_ROOT
    LLVM_ROOT = os.path.expanduser(config["llvm"])
    if (task == Task.SETUP or task == Task.INSTRUMENT) and not (os.path.exists(LLVM_ROOT) and os.path.isdir(LLVM_ROOT)):
        logging.error(f"LLVM path {LLVM_ROOT} does not exist")
        exit(1)

    # check if the opt path is valid
    global LLVM_OPT
    LLVM_OPT = os.path.expanduser(config["opt"])
    if task == Task.RUN and not os.path.exists(LLVM_OPT):
        logging.error(f"LLVM opt path {LLVM_OPT} does not exist")
        exit(1)


def setup():
    # copy file ../library/DLMonitor.h to LLVM_ROOT/include/llvm/Transforms/Utils/DLMonitor.h
    library_path = os.path.join(LLVM_ROOT, "include", "llvm", "Transforms", "Utils", "DLMonitor.h")
    if os.path.exists(library_path):
        logging.info(f"The library already exists, skipping")
        return

    logging.info("Creating the library")
    shutil.copy(
        os.path.join(METALOC_ROOT, "library", "DLMonitor.h"),
        library_path
    )


def instrument(pass_path: str):
    pass_dir = os.path.dirname(os.path.abspath(pass_path))

    # If the path.bak exists, do nothing
    if os.path.exists(pass_path + ".bak"):
        print(f"The file {os.path.basename(pass_path)} has already been instrumented")
        return
    
    # find any file with the suffix ".bak" and restore and remove it
    for file in os.listdir(pass_dir):
        if file.endswith(".bak"):
            backup_path = os.path.join(pass_dir, file)
            shutil.copy(backup_path, backup_path[:-4])
            os.remove(backup_path)

    # Back up the original file
    shutil.copy(pass_path, pass_path + ".bak")

    # Instrument the file
    logging.info(f"Instrumenting {pass_path}")
    command = f"cargo run -- {pass_path} --output {pass_dir}"
    os.system(command)


def synthesize_cmd(command: str, test_case: str) -> str:
    # extract the command between "RUN:" and "|"
    command = command.split("; RUN:")[1].split("|")[0].strip()
    # replace the "< %s" in command with the file name
    command = command.replace("< %s", test_case) \
        .replace("%s", test_case) \
        .replace("opt", LLVM_OPT + " --disable-output", count=1)
    # insert "debugify" after "-passes=" in command
    if "-passes=\"" in command:
        command = command.replace("-passes=\"", "-passes=\"debugify,")
    elif "-passes=\'" in command:
        command = command.replace("-passes=\'", "-passes=\'debugify,")
    else:
        print("here")
        command = command.replace("-passes=", "-passes=debugify,")
    
    return command


def execute(test_path: str):
    logging.info(f"Running tests under {test_path}")

    # iterate the test path and record all the files under the test path and the subdirectories
    workdir = []
    for root, dirs, files in os.walk(test_path):
        for file in files:
            workdir.append(os.path.join(root, file))

    for file in workdir:
        if file.endswith(".ll"):
            logging.info(f"Running test {file}")

            # read the file and print the line if it contains "RUN"
            with open(file, "r") as f:
                for line in f:
                    if line.startswith("; RUN:"):
                        command = synthesize_cmd(line, file)
                        # execute the command and record the output
                        try:
                            logging.info(f"Running command: {command}")
                            output = subprocess.check_output(command, shell=True).decode('ISO-8859-2')
                            print(output)
                            input("Continue?")
                        except subprocess.CalledProcessError as e:
                            logging.error(f"Command {command} failed with error {e}")


def clean():
    # if the file exists, remove it
    library_path = os.path.join(LLVM_ROOT, "include", "llvm", "Transforms", "Utils", "DLMonitor.h")
    if os.path.exists(library_path):
        logging.info(f"Removing the library")
        os.remove(library_path)

    pass_dir = os.path.join(LLVM_ROOT, "lib", "Transforms", "Scalar")
    for file in os.listdir(pass_dir):
        if file.endswith(".bak"):
            logging.info(f"Restoring the original file")
            backup_path = os.path.join(pass_dir, file)
            shutil.copy(backup_path, backup_path[:-4])
            os.remove(backup_path)


if __name__ == "__main__":
    task = sys.argv[1]
    config_parse(task)

    if task == Task.SETUP:
        setup()
    elif task == Task.INSTRUMENT:
        if len(sys.argv) < 3:
            logging.error("Please specify the path to the pass")
            exit(1)
        instrument(sys.argv[2])
    elif task == Task.RUN:
        if len(sys.argv) < 3:
            logging.error("Please specify the path to the tests")
            exit(1)
        execute(sys.argv[2])
    elif task == Task.CLEAN:
        clean()
    else:
        logging.error(f"Invalid task {task}. Please specify one of the following: {Task.SETUP}, {Task.INSTRUMENT}, {Task.RUN}, {Task.CLEAN}")
        exit(1)


                        
