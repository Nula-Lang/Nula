import ast
import sys
import subprocess
import os

class NulaVisitor(ast.NodeVisitor):
    def visit_Print(self, node):
        # Mapuj print do write exec
        print(f"Interpreting: {node.values[0].s if hasattr(node.values[0], 's') else 'write'}")
        return super().generic_visit(node)

    def visit_Call(self, node):
        if node.func.id == 'write':
            arg = ast.unparse(node.args[0]) if sys.version_info >= (3,9) else 'write arg'
            print(arg.strip('"'))
        return super().generic_visit(node)

def run_nula(file_path):
    with open(file_path, 'r') as f:
        code = f.read()

    # Prosty parse: jeśli .nula, mapuj do Python AST
    tree = ast.parse(code.replace('write', 'print'), filename=file_path)
    visitor = NulaVisitor()
    visitor.visit(tree)

    # Dla deps: sprawdź <(dep)> i import
    if '<' in code:
        deps = [d.strip('<>') for d in code.split() if d.startswith('<')]
        for dep in deps:
            lib_path = f"/usr/lib/.nula-lib/{dep}"
            if os.path.exists(lib_path):
                sys.path.append(lib_path)
                subprocess.run(['python', '-c', f'import {dep}'], check=True)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: nula-python <file.nula>")
        sys.exit(1)
    run_nula(sys.argv[1])
