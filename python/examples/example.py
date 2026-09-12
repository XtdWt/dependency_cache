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


if __name__ == "__main__":
    print("example of using dependency_cache for recomputation")
    print("   C")
    print(" /   \\")
    print("A     B")
    c = ExampleCalculation(3, 5)
    print(f"cache={c.cached_values} graph={c.dependency_graph} validation={c.validation_state}")
    print(f"Result of C = {c.C()}")  # calculates all, prints 8
    print(f"cache={c.cached_values} graph={c.dependency_graph} validation={c.validation_state}")
    print(f"Result of C = {c.C()}")  # hits cache, prints 8
    c.update_cached_value("A", 0)  # invalidates C
    print(f"Result of C = {c.C()}")  # recalculates C returns 5
    print(f"cache={c.cached_values} graph={c.dependency_graph} validation={c.validation_state}")
