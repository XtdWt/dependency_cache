from dependency_cache import DependencyCacheBase, automagic_dependency_cache, dependency_cache


class ExampleCalculationMagic(DependencyCacheBase):
    """  E
       //  \\
       D    \\
    //  \\   \\
    A    B    C
    """

    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

    @automagic_dependency_cache()
    def A(self):
        print("calculating A")
        return self.x

    @automagic_dependency_cache()
    def B(self):
        print("calculating B")
        return self.y

    @automagic_dependency_cache()
    def C(self):
        print("calculating C")
        return self.z

    @automagic_dependency_cache()
    def D(self):
        print("calculating D")
        return self.A() + self.B()

    @automagic_dependency_cache(dependencies=["C", "D"])
    def E(self):
        print("calculating E")
        return self.D() + self.C()


class ExampleCalculationManual(DependencyCacheBase):
    """D     E
        \\  //
          C
        //  \\
       A      B
    """

    def __init__(self, x, y):
        self.x = x
        self.y = y

    @dependency_cache()
    def A(self):
        print("calculating A")
        return self.x

    @dependency_cache()
    def B(self):
        print("calculating B")
        return self.y

    @dependency_cache(dependencies=["A", "B"])
    def C(self):
        print("calculating C")
        return self.A() + self.B()

    @dependency_cache(dependencies=["C"])
    def D(self):
        print("calculating D")
        return self.C() / 2

    @dependency_cache(dependencies=["C"])
    def E(self):
        print("calculating E")
        return self.C() * 2


if __name__ == "__main__":
    print("testing calculation 1")
    c1 = ExampleCalculationMagic(1, 2, 3)
    print(f"Result of E = {c1.E()}")  # calculates all notes, prints 6
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(f"Result of E = {c1.E()}")  # no calculation, returns cached 6
    c1.update_cached_value("A", 2)  # invalidates D, E
    print(f"Result of E = {c1.E()}")  # recalculates D, E returns 7
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())

    print("testing calculation 2")
    c2 = ExampleCalculationManual(6, 7)
    print(f"Result of E = {c2.E()}")  # calculates all notes, prints 26
    c2.update_cached_value("A", 0)  # invalidates C, E
    print(f"Result of E = {c2.E()}")  # recalculates C, E, returns 14
    print(f"Result of D = {c2.D()}")  # recalculates D, returns 3.5
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
