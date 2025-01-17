import os
import subprocess
import re

def add_relative_path_comment(directory):
    for root, _, files in os.walk(directory):
        for file in files:
            if file.endswith(".rs"):
                filepath = os.path.join(root, file)
                relative_path = os.path.relpath(filepath, directory)

                # Read file content
                with open(filepath, 'r') as f:
                    content = f.readlines()

                # Remove any existing `//$` comments
                content = [
                    line for line in content
                    if not re.match(r"^//\$ .*", line.strip())
                ]

                # Add the new relative path comment
                expected_comment = f"//$ {relative_path}\n"
                content.insert(0, expected_comment)

                # Write updated content back to the file
                with open(filepath, 'w') as f:
                    f.writelines(content)

def save_project_structure(directory):
    with open("project_structure.txt", "w") as outfile:
        subprocess.run(
            ["lsd", "--tree", "--depth", "2"],
            cwd=directory,
            stdout=outfile,
            text=True
        )

if __name__ == "__main__":
    project_dir = os.getcwd()
    add_relative_path_comment(project_dir)
    save_project_structure(project_dir)
