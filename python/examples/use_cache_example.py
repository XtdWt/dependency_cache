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
    if __name__ == "__main__":
        print("       B")
        print("       |")
        print("A (always recalcs)")
        c = UseCacheExample()
        print(f"Result of B = {c.B()}")  # calculates all
        print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
        print(f"Result of B = {c.B()}")  # still calculates all
        print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
