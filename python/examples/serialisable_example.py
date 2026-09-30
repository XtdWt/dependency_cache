from dependency_cache import DependencyCacheBase, dependency_cached


class SerialisableCalculation(DependencyCacheBase):

    @dependency_cached(serialisable=True)
    def A(self):
        print("calculating A")
        return 1

    @dependency_cached(serialisable=True)
    def B(self):
        print("calculating B")
        return 1

    @dependency_cached(serialisable=True, dependencies=["A", "B"])
    def C(self):
        print("calculating C")
        return self.A() + self.B()


if __name__ == "__main__":
    obj1 = SerialisableCalculation()
    print(obj1.C())

    saved_state = obj1.dump_cache()
    print(saved_state)

    obj2 = SerialisableCalculation()
    obj2.load_cache(saved_state)
    print(obj2.C())
