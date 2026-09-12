from dependency_cache import DependencyCacheBase, dependency_cached


class RuntimeGraphExample(DependencyCacheBase):
    """    A
        // | \\
       //  |  \\
      C    B    D (detached until B changes)
    """

    @dependency_cached(track_runtime_dependencies=True)
    def A(self):
        print("calculating A")
        if self.B():
            return self.C()
        return self.D()

    @dependency_cached()
    def B(self):
        print("calculating B")
        return True

    @dependency_cached()
    def C(self):
        print("calculating C")
        return 1000

    @dependency_cached()
    def D(self):
        print("calculating D")
        return 1001


if __name__ == "__main__":
    print("example of using runtime dependencies")
    print("graph only partially built!")
    print("    A")
    print("  / |")
    print(" /  |")
    print("C   B")
    c = RuntimeGraphExample()
    print(c.A())  # calculates A, prints 1000

    print("updating D!")
    print(c.D())
    c.update_cached_value("D", 1)
    print(c.D())
    c.update_cached_value("D", 3)
    print(c.D())
    print(c.A())  # no calculation, returns cached 1000

    print("graph fully built!")
    print("    A")
    print("  / | \\")
    print(" /  |  \\")
    print("C   B   D")
    c.update_cached_value("B", False)
    print(c.A())  # calculates A, prints 3

    print(c.cached_values, c.dependency_graph, c.validation_state)
