# Dependency Cache
## 🚧 Work In Progress 🚧

### Description
Dependency Cache is a caching layer for object methods, with invalidation driven by a dependency graph.
Rather than re-running expensive, deeply nested calculations on every call, it stores each method's result and only recomputes (and marks downstream methods for recalculation) when one of its declared dependencies actually changes.
Under the hood, this project is written in Rust using [pyo3](https://github.com/PyO3/pyo3) and built with [maturin](https://github.com/PyO3/maturin).
### Installation

Install the latest stable version from [PyPI](https://pypi.org/project/dependency-cache/):

```bash
pip install dependency-cache
```

### API Overview

| Export | What it's for |
|---|---|
| `DependencyCacheBase` | Base class to inherit from. Gives your instance a cache and a dependency graph. |
| `dependency_cached(use_cache=True, dependencies=[...], track_runtime_dependencies=False, serialisable=False)` | A footgun-enabled decorator for a method where you **explicitly declare** all direct dependencies. |
| `automagically_dependency_cached(use_cache=True, dependencies=[...], track_runtime_dependencies=True, serialisable=False)` | A decorator which **automagically infers** the direct dependencies. Optionally override the defaults and inference using the `dependencies` parameter. |
| `plot_dependency_graph(obj, **kwargs)` | Visualizes an instance's dependency graph, for inspection/debugging. |

### Setup
This project uses (and requires) [uv](https://github.com/astral-sh/uv) as a package manager and maturin to compile.

To set up a local dev environment:

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
- scenario analysis capability (add temporary cache to base)
- add validation to static dependencies, ensure methods are of decorator class
- improve plot_dependency_graph (maybe to GUI?) with moveable nodes
- add object thread safety for python 3.14+
- work out how best to handle nested objects
