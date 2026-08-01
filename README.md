# Dependency Cache
## :construction: Work In Progress :construction:

### Description
A library for managing the caching of an object's methods with cache invalidation handled by a dependency graph.
Instead of recomputing expensive, deeply nested calculations on every call, Dependency Cache stores the result and only invalidates (and recalculates) it when one of its dependencies actually changes.
This project is written in rust, with [pyo3](https://github.com/PyO3/pyo3), and compiled using [maturin](https://github.com/PyO3/maturin).

### API Overview

| Export | What it's for |
|---|---|
| `DependencyCacheBase` | Base class to inherit from. Gives your instance a cache and a dependency graph. |
| `dependency_cached(use_cache=True, dependencies=[...])` | A footgun-enabled :gun: decorator for a method where you **explicitly declare** direct dependencies. |
| `automagically_dependency_cached(use_cache=True, dependencies=[...])` | A decorator which **automagically infers** the direct dependencies with the option to override manually using `dependencies=`. |
| `plot_dependency_graph(obj, **kwargs)` | Visualizes an instance's dependency graph, for inspection/debugging. |

### Setup
This project uses (and requires) [uv](https://github.com/astral-sh/uv) as a package manager and maturin to compile.

Steps:
- install python dependencies
```bash
uv sync
```
- generate local development file for testing (and update pyi file for accurate typechecking)
```bash
maturin develop --generate-stubs
maturin develop -r --generate-stubs
```
- run python tests
```bash
uv run pytest
uv run pytest -vv
```
- run rust tests
```bash
cargo test
```
- temporary fix: generate stubs currently produces a file that is incorrectly formatted, use ruff to format the pyi file immediately
```bash
maturin develop --generate-stubs | uv run ruff format ./python/dependency_cache/dependency_cache.pyi
```
### Quick Example

```python
from dependency_cache import DependencyCacheBase, automagically_dependency_cached


class ExampleCalculation(DependencyCacheBase):
    """   C
        //  \\
       A      B
    """

    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    @automagically_dependency_cached()
    def A(self):
        print("calculating A")
        return self.x

    @automagically_dependency_cached()
    def B(self):
        print("calculating B")
        return self.y

    @automagically_dependency_cached()
    def C(self):
        print("calculating C")
        return self.B() + self.A()


c = ExampleCalculation(3, 5)
print(c.C())  # calculates A, B, C -> 8
print(c.C())  # hits cache -> 8

c.update_cached_value("A", 0)  # invalidates C (but not B)
print(c.C())  # recalculates C -> 5
```

This example can be found [here](/python/examples/example.py) with more runnable examples and use cases provided in the [python/examples](/python/examples) folder.

```bash
uv run ./python/examples/example.py
```

### TODOs
This project is still a work in progress, with everything from API to underlying design subject to change on my whim.

Current TODO list (in no particular order):
- load and dump cache methods for base class
- decide what to do with methods that take arguments for caching
- move plot_dependency_graph to rust side instead of python
- refactor automagic parser code
- add object thread safety for python 3.14+
