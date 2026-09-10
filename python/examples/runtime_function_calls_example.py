from dependency_cache import DependencyCacheBase, dependency_cached


class FibonacciExample(DependencyCacheBase):
    def __init__(self):
        super().__init__()

    @dependency_cached(track_runtime_dependencies=True)
    def Calculate(self, n):
        print(f"calculating the {n=} fibonacci number")
        if n <= 0:
            return 0
        if n == 1:
            return 1
        return self.Calculate(n - 1) + self.Calculate(n - 2)


if __name__ == "__main__":
    print("example of using runtime dependencies, with nested function calls")
    print("Fib(3)")
    print("|  \\")
    print("|  Fib(2)")
    print("|  |  \\")
    print("----> Fib(1)")
    print("   |    \\")
    print("   ----> Fib(0)")
    c = FibonacciExample()

    print(f"Result of Fib(3) {c.Calculate(3)}")  # calculates Calculate(3), Calculate(2), Calculate(1), Calculate(0)

    print(f"Result of Fib(2) {c.Calculate(2)}")  # cache works for Calculate(2), no more prints
    print(f"Result of Fib(1) {c.Calculate(1)}")  # cache works for Calculate(2), no more prints
    print(f"Result of Fib(4) {c.Calculate(4)}")  # calculates Calculate(4), rest cached
    print(c.get_cached_values(), c.get_dependency_graph(), c.get_validation_state())
