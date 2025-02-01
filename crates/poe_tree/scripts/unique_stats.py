import json
import re
import pandas as pd

# Load the POE2_Tree.json file
file_path = "data/POE2_Tree.json"
with open(file_path, "r") as f:
    poe_tree = json.load(f)

# Extract unique stat names and categorize them
stat_categories = {
    "addition (+)": set(),
    "subtraction (-)": set(),
    "multiplication (*)": set(),
    "division (/)": set(),
    "percentage (+%)": set(),
    "other": set(),
}

# Separate stats with and without numerical values
stats_with_values = set()
stats_without_values = set()

# Regular expression patterns for categorizing stats
percentage_pattern = re.compile(r"\+%")
addition_pattern = re.compile(r"\+$")
multiplication_pattern = re.compile(r"\*$")
division_pattern = re.compile(r"/")
subtraction_pattern = re.compile(r"-")

# Iterate through passive skills and categorize stats
for skill in poe_tree.get("passive_skills", {}).values():
    if "stats" in skill:
        for stat_name, value in skill["stats"].items():
            # Determine category based on the stat name
            if percentage_pattern.search(stat_name):
                stat_categories["percentage (+%)"].add(stat_name)
            elif addition_pattern.search(stat_name):
                stat_categories["addition (+)"].add(stat_name)
            elif multiplication_pattern.search(stat_name):
                stat_categories["multiplication (*)"].add(stat_name)
            elif division_pattern.search(stat_name):
                stat_categories["division (/)"].add(stat_name)
            elif subtraction_pattern.search(stat_name):
                stat_categories["subtraction (-)"].add(stat_name)
            else:
                stat_categories["other"].add(stat_name)

            # Check if the stat has a numerical value
            if isinstance(value, (int, float)):
                stats_with_values.add(stat_name)
            else:
                stats_without_values.add(stat_name)

# Create a DataFrame with categorized stats
stat_summary = []
for category, stats in stat_categories.items():
    for stat in stats:
        stat_summary.append(
            {
                "Stat Name": stat,
                "Category": category,
                "Has Numeric Value": stat in stats_with_values,
            }
        )

df = pd.DataFrame(stat_summary)

# Count occurrences of each stat name
stat_counts = df["Stat Name"].value_counts().reset_index()
stat_counts.columns = ["Stat Name", "Count"]

# Sort the counts in descending order
stat_counts = stat_counts.sort_values(by="Count", ascending=False)

# Print the results
print("Total number of unique stats:", len(stat_counts))

print("\nStats with numeric values:")
print(stat_counts[stat_counts["Stat Name"].str.contains(r"\d")])

print("\nStats without numeric values:")
print(stat_counts[~stat_counts["Stat Name"].str.contains(r"\d")])

print("\nTop 20 stats by frequency:")
print(stat_counts.head(20))

print("\nBottom 20 stats by frequency:")
print(stat_counts.tail(20))

# Optional: Save the results to a CSV file
# df.to_csv('unique_stats_results.csv', index=False)
