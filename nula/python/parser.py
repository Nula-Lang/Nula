import ast
import re
from typing import List, Union, Tuple

class NulaNode:
    """Represents a node in the Nula AST."""
    def __init__(self, node_type: str, value: Union[str, List['NulaNode']] = None):
        self.node_type = node_type  # e.g., "print", "assign", "comment"
        self.value = value  # String for literals, list for blocks

class NulaParser:
    """Parser for Nula code using Python AST for translation and native parsing."""
    
    def __init__(self):
        self.dependencies = []

    def parse(self, code: str) -> List[NulaNode]:
        """Parse Nula code into an AST."""
        # Handle translation blocks: # = lang = {code}
        translated_code = self._translate_code(code)
        lines = translated_code.split('\n')
        ast_nodes = []

        for line in lines:
            line = line.strip()
            if not line or line.startswith('@'):  # Handle comments
                if line.startswith('@'):
                    ast_nodes.append(NulaNode("comment", line[1:].strip()))
                continue
            # Match write("text");
            if write_match := re.match(r'write\s*\(\s*"([^"]*)"\s*\)\s*;', line):
                ast_nodes.append(NulaNode("print", write_match.group(1)))
            # Match var = value;
            elif assign_match := re.match(r'(\w+)\s*=\s*"?([^"]*)"?\s*;', line):
                var, val = assign_match.groups()
                ast_nodes.append(NulaNode("assign", (var, val)))
            # Handle dependency declarations: <(dep)>
            elif dep_match := re.match(r'<\((.*?)\)>\s*', line):
                self.dependencies.append(dep_match.group(1))

        return ast_nodes

    def _translate_code(self, code: str) -> str:
        """Translate code from other languages (e.g., # = python = {code})."""
        translated = []
        lines = code.split('\n')
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            if line.startswith('# ='):
                # Extract lang and code block
                if match := re.match(r'# = (\w+) = \{([^}]*)\}', line):
                    lang, inner_code = match.groups()
                    if lang == "python":
                        # Simple translation: print -> write
                        translated.append(inner_code.replace('print(', 'write('))
                    else:
                        translated.append(inner_code)  # Fallback: no translation
                i += 1
            else:
                translated.append(line)
                i += 1
        return '\n'.join(translated)

    def get_dependencies(self) -> List[str]:
        """Return list of dependencies found during parsing."""
        return self.dependencies
