from dependency_cache import DependencyCacheBase, automagically_dependency_cached, custom_scenario


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
    print("example of using dependency_cache with custom scenarios, restoring state after")
    print("   C")
    print(" /   \\")
    print("A     B")
    c = ExampleCalculation(1, 2)

    calcuations = [c]
    print(f"Result of C = {c.C()}")  # # calculates all, prints 3
    with custom_scenario(calcuations):
        c.update_cached_value("A", 100)
        print(f"Result of C = {c.C()}")  # recalculates C returns 102

        with custom_scenario(calcuations):
            c.update_cached_value("B", 200)
            print(f"Result of C = {c.C()}")  # recalculates C returns 300

        print(f"Result of C = {c.C()}")  # recalculates C returns 102
    print(f"Result of C = {c.C()}")  # recalculates C returns 3
