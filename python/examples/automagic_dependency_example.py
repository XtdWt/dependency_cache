from dependency_cache import DependencyCacheBase, automagically_dependency_cached


class ExampleCalculationMagic(DependencyCacheBase):
    """  E
       //  \\
       D    \\
    //  \\   \\
    A    B    C
    """

    def __init__(self, x, y, z):
        super().__init__()
        self.x = x
        self.y = y
        self.z = z

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
        return self.z

    @automagically_dependency_cached()
    def D(self):
        print("calculating D")
        return self.A() + self.B()

    @automagically_dependency_cached()
    def E(self):
        print("calculating E")
        return self.D() + self.C()


if __name__ == "__main__":
    print("       E")
    print("     /  \\")
    print("    D    \\")
    print("  /   \\   \\")
    print(" A     B   C")
    c1 = ExampleCalculationMagic(1, 2, 3)
    print(f"Result of E = {c1.E()}")  # calculates all, prints 6
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(f"Result of E = {c1.E()}")  # no calculation, returns cached 6
    c1.update_cached_value("A", 2)  # invalidates D, E
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(f"Result of E = {c1.E()}")  # recalculates D, E returns 7
