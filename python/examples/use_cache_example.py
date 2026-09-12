import time

from dependency_cache import DependencyCacheBase, dependency_cached


class UseCacheExample(DependencyCacheBase):
    def __init__(self):
        super().__init__()

    @dependency_cached(use_cache=False)
    def A(self):
        print("calculating A")
        return time.time()

    @dependency_cached(dependencies=["A"])
    def B(self):
        print("calculating B")
        return self.A() + 1


if __name__ == "__main__":
    print("example of use_cache flagging permanent invalidation of graph")
    print("       B")
    print("       |")
    print("A (always recalcs)")
    c = UseCacheExample()
    print(f"Result of B = {c.B()}")  # calculates all
    print(c.cached_values, c.dependency_graph, c.validation_state)
    print(f"Result of B = {c.B()}")  # still calculates all
    print(c.cached_values, c.dependency_graph, c.validation_state)
