import importlib
import pytest

EXAMPLES = [
    "nqueens", "send_more_money", "graph_coloring", "magic_square",
    "diophantine", "modulo_filter",
    "simple", "search", "scaling", "element", "cardinality",
    "boolean_logic", "cyclic",
]

@pytest.mark.parametrize("name", EXAMPLES)
def test_example(name):
    mod = importlib.import_module(f"examples.{name}")
    mod.run()
