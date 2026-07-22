from dependency_cache import DependencyCacheBase, automagic_dependency_cache, dependency_cache


class ExampleClass(DependencyCacheBase):
    def __init__(self, name):
        self.name = name

    @dependency_cache(use_cache=True)
    def expensive(self, x, y=0):
        print(f"computing expensive({x}, y={y}) for {self.name}...")
        return f"{self.name}:{x}:{y}"

    @dependency_cache(use_cache=False)
    def uncached(self, x):
        print(f"computing uncached({x}) for {self.name}...")
        return f"{self.name}:{x}"


class ExampleCalculation1(DependencyCacheBase):
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


class ExampleCalculation2(DependencyCacheBase):
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
    # a = ExampleClass("a")
    # b = ExampleClass("b")
    # print("First call (miss):", a.expensive(1, y=2))
    # print("Second call, same args (hit):", a.expensive(1, y=2))
    # print("Third call, DIFFERENT args -- still returns the first result:")
    # print(" ", a.expensive(999, y=999))
    # print("Different instance -- its own cache:", b.expensive(7, y=8))

    # print("\nuse_cache=False recomputes every time:")
    # a.uncached(5)
    # a.uncached(5)

    # print("\na.clear_cache() only clears a's cache, not b's:")
    # a.clear_cache()
    # print(" ", a.expensive(42, y=42))
    # print(" ", b.expensive(1, y=1), "(unchanged)")
    print("testing calculation 1")
    c1 = ExampleCalculation1(1, 2, 3)
    # print(c.current_cache(), c.current_graph(), c.current_cache_validation())
    # print(c.D())
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(c1.E())
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(c1.E())

    c1.update_cached_value("A", 2)
    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())
    print(c1.E())

    print(c1.current_cache(), c1.current_graph(), c1.current_cache_validation())

    print("testing calculation 2")
    c2 = ExampleCalculation2(6, 7)
    print(c2.E())
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
    c2.update_cached_value("A", 0)
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
    print(c2.E())
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
    print(c2.D())
    print(c2.current_cache(), c2.current_graph(), c2.current_cache_validation())
