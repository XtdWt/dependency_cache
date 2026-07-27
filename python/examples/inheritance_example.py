from dependency_cache import DependencyCacheBase, automagically_dependency_cached


class ChildClass(DependencyCacheBase):
    def __init__(self):
        super().__init__()

    @automagically_dependency_cached()
    def A(self):
        print("Calculating A from Child")
        return 1

    @automagically_dependency_cached()
    def B(self):
        print("Calculating B from Child")
        return 3


class ParentClass(ChildClass):
    def __init__(self):
        super().__init__()

    @automagically_dependency_cached()
    def A(self):
        print("Calculating A from Parent")
        return 2

    @automagically_dependency_cached()
    def B(self):
        print("Calculating B from Parent")
        return super().B()


if __name__ == "__main__":
    print("dependency graph obeys MRO (standard inheritance method order)")
    c = ParentClass()

    print(f"Result of A {c.A()}")  # calculates parent A and does not calculates child

    print(f"Result of B {c.B()}")  # calculates parent B and does calculates child

    print(f"Result of A {c.A()}")
    print(f"Result of B {c.B()}")  # cache works for both, no more prints
    print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
