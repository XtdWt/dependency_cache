from dependency_cache import DependencyCacheBase, dependency_cached


class FunctionCallObj(DependencyCacheBase):
    def __init__(self, x):
        super().__init__()
        self.x = x

    @dependency_cached(dependencies=["C"])
    def A(self, x):
        print("calculating A!")
        return x + self.x + self.C()

    @dependency_cached(dependencies=[("A", {"x": 0}), ("A", {"x": 1})])
    def B(self):
        print("calculating B!")
        total = 0
        for i in range(2):
            total += self.A(i)
        return total

    @dependency_cached()
    def C(self):
        print("calculating C!")
        return 3


if __name__ == "__main__":
    print("dependency graph can have each function call cached")
    c = FunctionCallObj(1)

    print(f"Result of B {c.B()}")  # calculates parent B and does calculates A(0), C, A(1)

    print(f"Result of B {c.B()}")  # cache works for B, no more prints
    print(f"Result of A {c.A(0)}")
    print(f"Result of A {c.A(1)}")  # cache works for A, with args, no more prints
    print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
