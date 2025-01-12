from dataclasses import dataclass
from typing import List, Dict, Any

@dataclass
class Node:
    id: str
    parent: int
    radius: int
    position: int
    skill_id: str | None
    connections: List[str]
    skill_data: Dict[str, Any] | None = None

@dataclass
class Edge:
    source: str
    target: str

@dataclass
class Tree:
    nodes: Dict[str, Node]
    edges: List[Edge]
    groups: Dict[str, Dict]