import os
import subprocess
import re
from gitignore_parser import parse_gitignore

def add_relative_path_comment(directory):
    gitignore_path = os.path.join(directory, '.gitignore')
    is_ignored = parse_gitignore(gitignore_path) if os.path.exists(gitignore_path) else lambda path: False

    for root, _, files in os.walk(directory):
        for file in files:
            filepath = os.path.join(root, file)
            if is_ignored(filepath):
                continue

            if file.endswith(".rs") or file.endswith(".toml"):
                relative_path = os.path.relpath(filepath, directory)

                # Determine the comment prefix based on file type
                comment_prefix = "//!" if file.endswith(".rs") else "#"

                # Read file content
                with open(filepath, 'r') as f:
                    content = f.readlines()

                # Replace or insert the relative path comment
                expected_comment = f"{comment_prefix} {relative_path}\n"
                if content and content[0].strip().startswith(comment_prefix):
                    content[0] = expected_comment  # Replace the first line
                else:
                    content.insert(0, expected_comment)  # Add as the first line

                # Write updated content back to the file
                with open(filepath, 'w') as f:
                    f.writelines(content)

def save_project_structure(directory):
    gitignore_path = os.path.join(directory, '.gitignore')
    is_ignored = parse_gitignore(gitignore_path) if os.path.exists(gitignore_path) else lambda path: False

    with open("project_structure.txt", "w") as outfile:
        for root, dirs, files in os.walk(directory):
            for dir_name in dirs[:]:
                dir_path = os.path.join(root, dir_name)
                if is_ignored(dir_path):
                    dirs.remove(dir_name)  # Skip ignored directories

            for file_name in files:
                file_path = os.path.join(root, file_name)
                if not is_ignored(file_path):
                    outfile.write(f"{os.path.relpath(file_path, directory)}\n")

if __name__ == "__main__":
    project_dir = os.getcwd()
    add_relative_path_comment(project_dir)
    save_project_structure(project_dir)
