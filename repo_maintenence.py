import os
import subprocess

def add_relative_path_comment(directory):
    for root, _, files in os.walk(directory):
        for file in files:
            if file.endswith(".rs"):
                filepath = os.path.join(root, file)
                relative_path = os.path.relpath(filepath, directory)
                with open(filepath, 'r') as f:
                    content = f.readlines()
                if not content or not content[0].startswith("//$"):
                    content.insert(0, f"//$ {relative_path}\n")
                    with open(filepath, 'w') as f:
                        f.writelines(content)

                        f.writelines(content)

def save_project_structure(directory):
    with open("project_structure.txt", "w") as outfile:
        subprocess.run(["lsd", "--tree", "--depth", "2"], cwd=directory, stdout=outfile, text=True)

project_dir = os.getcwd()
add_relative_path_comment(project_dir)
save_project_structure(project_dir)