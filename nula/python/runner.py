import os
import sys
from typing import List
from .parser import NulaNode, NulaParser

class NulaRunner:
    """Executes Nula AST nodes in interpreted mode."""
    
    def __init__(self):
        self.variables = {}  # Store variable assignments
        self.lib_dir = "/usr/lib/.nula-lib" if not sys.platform.startswith("win") else os.path.join(os.environ.get("SystemRoot", "C:\\Windows"), "System32", ".nula-lib")

    def run(self, nodes: List[NulaNode]) -> None:
        """Execute a list of Nula AST nodes."""
        for node in nodes:
            if node.node_type == "print":
                print(node.value)
            elif node.node_type == "assign":
                var, val = node.value
                self.variables[var] = val
                # Substitute variables in print statements
                if var in self.variables:
                    print(f"Assigned {var} = {val}")
            elif node.node_type == "comment":
                pass  # Comments are ignored during execution

    def load_dependencies(self, dependencies: List[str]) -> None:
        """Load dependencies from /usr/lib/.nula-lib/."""
        for dep in dependencies:
            dep_path = os.path.join(self.lib_dir, dep)
            if os.path.exists(dep_path):
                sys.path.append(dep_path)
                try:
                    __import__(dep)
                    print(f"Loaded dependency: {dep}")
                except ImportError as e:
                    print(f"Error loading {dep}: {e}", file=sys.stderr)
            else:
                print(f"Dependency {dep} not found in {self.lib_dir}", file=sys.stderr)

def main():
    if len(sys.argv) < 2:
        print("Usage: nula-python <file.nula>", file=sys.stderr)
        sys.exit(1)

    file_path = sys.argv[1]
    if not os.path.exists(file_path):
        print(f"File {file_path} not found", file=sys.stderr)
        sys.exit(1)

    with open(file_path, 'r') as f:
        code = f.read()

    parser = NulaParser()
    nodes = parser.parse(code)
    runner = NulaRunner()

    # Load dependencies before execution
    runner.load_dependencies(parser.get_dependencies())
    
    # Execute the parsed AST
    runner.run(nodes)

if __name__ == "__main__":
    main()
