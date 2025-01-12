import re
from collections import defaultdict

def process_log_file(input_file, output_file):
    log_dict = defaultdict(list)
    seen_contexts = set()
    
    with open(input_file, 'r') as infile, open(output_file, 'w') as outfile:
        for line in infile:
            match = re.match(r".*?\[(.*?)\] (.*)", line)
            if match:
                key = match.group(1)  # Extract content within the first brackets
                context = match.group(2)  # Extract the rest of the line
                if context not in seen_contexts:
                    seen_contexts.add(context)
                    log_dict[key].append(line.strip())
        
        for key, entries in log_dict.items():
            outfile.write(f"--- {key} ---\n")
            outfile.write("\n".join(entries))
            outfile.write("\n\n")

# Usage
input_log = 'Client.txt'  # Replace with your input file path
output_log = 'output.txt'  # Replace with your desired output file path
process_log_file(input_log, output_log)
