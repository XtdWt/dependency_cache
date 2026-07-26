from dependency_cache import DependencyCacheBase, dependency_cached


class ExampleCalculationManual(DependencyCacheBase):
    """D     E
        \\  //
          C
        //  \\
       A      B
    """

    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    @dependency_cached()
    def A(self):
        print("calculating A")
        return self.x

    @dependency_cached()
    def B(self):
        print("calculating B")
        return self.y

    @dependency_cached(dependencies=["A", "B"])
    def C(self):
        print("calculating C")
        return self.A() + self.B()

    @dependency_cached(dependencies=["C"])
    def D(self):
        print("calculating D")
        return self.C() / 2

    @dependency_cached(dependencies=["C"])
    def E(self):
        print("calculating E")
        return self.C() * 2


if __name__ == "__main__":
    print("D     E")
    print(" \\   /")
    print("   C")
    print(" /   \\")
    print("A     B")
    c2 = ExampleCalculationManual(6, 7)
    print(f"Result of E = {c2.E()}")  # calculates all, prints 26
    c2.update_cached_value("A", 0)  # invalidates C, E
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
    print(f"Result of E = {c2.E()}")  # recalculates C, E, returns 14
    print(f"Result of D = {c2.D()}")  # recalculates D, returns 3.5
