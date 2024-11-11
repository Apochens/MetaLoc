import subprocess
import sys
import logging
import os

LLVM_OPT = "~/Code/build-llvm/bin/opt"  # replace with your own opt path

logging.basicConfig(level=logging.INFO)

if __name__ == "__main__":
    test_path = sys.argv[1]
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
                        # extract the command between "RUN:" and "|"
                        command = line.split("; RUN:")[1].split("|")[0].strip()
                        # replace the "< %s" in command with the file name
                        command = command.replace("< %s", file).replace("%s", file).replace("opt", LLVM_OPT + " --disable-output", count=1)
                        # insert "debugify" after "-passes=" in command
                        if "-passes=\"" in command:
                            command = command.replace("-passes=\"", "-passes=\"debugify,")
                        elif "-passes=\'" in command:
                            command = command.replace("-passes=\'", "-passes=\'debugify,")
                        else:
                            print("here")
                            command = command.replace("-passes=", "-passes=debugify,")
                        # execute the command and record the output
                        try:
                            logging.info(f"Running command: {command}")
                            output = subprocess.check_output(command, shell=True).decode('ISO-8859-2')
                            print(output)
                            input("Continue?")
                        except subprocess.CalledProcessError as e:
                            logging.error(f"Command {command} failed with error {e}")
                        
