import json
from typing import Any, Dict


def load_data(filepath: str) -> Dict[str, Any]:
    """Loads and returns the JSON data from the given file path."""
    with open(filepath, 'r') as f:
        return json.load(f)
