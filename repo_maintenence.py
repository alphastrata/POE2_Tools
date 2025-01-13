import os

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

add_relative_path_comment('.')
