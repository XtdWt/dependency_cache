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
    c = RuntimeGraphExample()
    print(c.A())  # calculates A, prints 1000
    print("graph only partially built!")

    print("repeatedly updating D!")
    c.update_cached_value("D", 1)
    c.update_cached_value("D", 2)
    c.update_cached_value("D", 3)

    print(c.A())  # no calculation, returns cached 1000
    print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
