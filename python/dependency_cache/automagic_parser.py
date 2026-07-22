import ast
import functools
import inspect
import textwrap

from .dependency_cache import dependency_cache


class SelfCallVisitor(ast.NodeVisitor):
    def __init__(self, dependencies: list[str]) -> None:
        super().__init__()
        self.seen: set[str] = set()
        self.dependencies: list[str] = dependencies

    def visit_Call(self, node: ast.Call) -> None:
        if (
            isinstance(node.func, ast.Attribute)
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "self"
        ):
            name = node.func.attr
            if name not in self.seen:
                self.seen.add(name)
                self.dependencies.append(name)
        self.generic_visit(node)


def calculate_dependency_list(code: str) -> list[str]:
    tree = ast.parse(textwrap.dedent(code).strip())
    dependencies = []

    visitor = SelfCallVisitor(dependencies)
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            for statement in node.body:
                visitor.visit(statement)

    return visitor.dependencies


def automagic_dependency_cache(**kwargs):
    def decorator(func):
        func_as_string = inspect.getsource(func)
        function_dependencies = kwargs.pop("dependencies", []) or calculate_dependency_list(func_as_string)

        @dependency_cache(dependencies=function_dependencies, **kwargs)
        @functools.wraps(func)
        def wrapper(*call_args, **call_kwargs):
            return func(*call_args, **call_kwargs)

        return wrapper

    return decorator
