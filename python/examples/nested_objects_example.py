from dependency_cache import DependencyCacheBase, automagically_dependency_cached


class InnerNestedObject(DependencyCacheBase):
    def __init__(self, inner_value):
        super().__init__()
        self.value = inner_value

    @automagically_dependency_cached()
    def A(self):
        print("calculating inner A")
        return self.value


class OuterNestedObject(DependencyCacheBase):
    def __init__(self, parent_value, child_value):
        super().__init__()
        self.value = parent_value
        self.inner = InnerNestedObject(child_value)

    @automagically_dependency_cached()
    def InnerObj(self):
        print("calculating inner object")
        return self.inner

    @automagically_dependency_cached()
    def A(self):
        print("calculating outer A")
        return self.value

    @automagically_dependency_cached()
    def B(self):
        print("calculating outer B")
        return self.A() + self.InnerObj().A()


if __name__ == "__main__":
    print("   Outer B")
    print(" /         \\")
    print("Outer A    InnerObj")
    print("            |")
    print("           Inner A")
    c = OuterNestedObject(1, 1.5)
    print(f"Result of B = {c.B()}")  # calculates all, prints 2.5

    print(f"Result of B = {c.B()}")  # prints 2.5
    print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
