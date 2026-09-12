import pytest
from dependency_cache import DependencyCacheBase, automagically_dependency_cached, plot_dependency_graph


class IncorrectInheritance:
    def __init__(self):
        pass

    @automagically_dependency_cached()
    def A(self):
        return 1


class SimpleTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    @automagically_dependency_cached()
    def A(self):
        return self.x

    @automagically_dependency_cached()
    def B(self):
        return self.y

    @automagically_dependency_cached()
    def C(self):
        return self.A() + self.B()


class HardTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    @automagically_dependency_cached()
    def A(self):
        return self.x

    @automagically_dependency_cached()
    def B(self):
        return self.y

    @automagically_dependency_cached()
    def C(self):
        return self.A() + self.B()

    @automagically_dependency_cached()
    def D(self):
        return self.C() * 2

    @automagically_dependency_cached()
    def E(self):
        return self.C() * 3


class HarderTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        super().__init__()
        self.x = x
        self.y = y

    @automagically_dependency_cached()
    def A(self):
        return self.x

    @automagically_dependency_cached()
    def B(self):
        return self.y

    @automagically_dependency_cached()
    def C(self):
        return self.A() + self.B()

    @automagically_dependency_cached()
    def D(self):
        return self.C() * 2

    @automagically_dependency_cached()
    def E(self):
        return self.C() * 3

    @automagically_dependency_cached()
    def F(self):
        return self.E() + self.D()


class UseCacheObj(DependencyCacheBase):
    def __init__(self, x):
        super().__init__()
        self.x = x

    @automagically_dependency_cached(use_cache=False)
    def A(self):
        print("calculating A!")
        return self.x

    @automagically_dependency_cached()
    def B(self):
        return self.A() + 1


class FunctionCallObj(DependencyCacheBase):
    def __init__(self, x):
        super().__init__()
        self.x = x

    @automagically_dependency_cached()
    def A(self, x):
        print("calculating A!")
        return x + self.x

    @automagically_dependency_cached()
    def B(self):
        total = 0
        for i in range(2):
            total += self.A(i)
        return total


def test_raises_typeerror():
    c = IncorrectInheritance()
    with pytest.raises(TypeError):
        c.A()


def test_incorrect_method_raises_valueerror():
    with pytest.raises(TypeError):

        class IncorrectMethod(DependencyCacheBase):
            def __init__(self):
                super().__init__()

            @automagically_dependency_cached()
            def A():
                return 1

        c = IncorrectMethod()
        c.A()


@pytest.mark.parametrize(
    ("x_value", "y_value"),
    [
        (1, 2),
        (2_000, 3_000),
        ("1111", "2222"),
        (0, 0),
        (-1, 1),
        (3.14, -0.14),
    ],
)
def test_init(x_value, y_value):
    a = SimpleTestObj(x_value, y_value)

    assert isinstance(a, DependencyCacheBase)
    assert a is not None
    assert a.x == x_value
    assert a.y == y_value
    assert a.dependency_graph == {}
    assert a.validation_state == {}


@pytest.mark.parametrize(
    ("x_value", "y_value", "result"),
    [
        (1, 2, 3),
        (2_000, 3_000, 5_000),
        ("1111", "2222", "11112222"),
        (0, 0, 0),
        (-1, 1, 0.0),
        (3.14, -0.14, 3.0),
    ],
)
def test_calculation_simple(x_value, y_value, result):
    a = SimpleTestObj(x_value, y_value)

    assert a.C() == result

    assert a.dependency_graph == {"A": {"C"}, "B": {"C"}, "C": set()}
    assert a.validation_state == {"A": "valid", "B": "valid", "C": "valid"}

    a.update_cached_value("B", None)
    assert a.dependency_graph == {"A": {"C"}, "B": {"C"}, "C": set()}
    assert a.validation_state == {"A": "valid", "B": "valid", "C": "invalid"}


@pytest.mark.parametrize(
    ("x_value", "y_value", "result1", "result2"),
    [
        (1, 2, 6, 9),
        (2_000, 3_000, 10_000, 15_000),
        ("1111", "2222", "1111222211112222", "111122221111222211112222"),
        (0, 0, 0, 0),
        (-1, 1, 0.0, 0.0),
        (3.14, -0.14, 6.0, 9.0),
    ],
)
def test_calculation_hard(x_value, y_value, result1, result2):
    a = HardTestObj(x_value, y_value)

    assert a.D() == result1
    assert a.E() == result2

    assert a.dependency_graph == {"A": {"C"}, "B": {"C"}, "C": {"D", "E"}, "D": set(), "E": set()}
    assert a.validation_state == {"A": "valid", "B": "valid", "C": "valid", "D": "valid", "E": "valid"}

    a.update_cached_value("B", None)
    assert a.dependency_graph == {"A": {"C"}, "B": {"C"}, "C": {"D", "E"}, "D": set(), "E": set()}
    assert a.validation_state == {"A": "valid", "B": "valid", "C": "invalid", "D": "invalid", "E": "invalid"}


@pytest.mark.parametrize(
    ("x_value", "y_value", "result"),
    [
        (1, 2, 15),
        (2_000, 3_000, 25_000),
        ("1111", "2222", "1111222211112222111122221111222211112222"),
        (0, 0, 0),
        (-1, 1, 0.0),
        (3.14, -0.14, 15.0),
    ],
)
def test_calculation_harder(x_value, y_value, result):
    a = HarderTestObj(x_value, y_value)

    assert a.F() == result

    assert a.dependency_graph == {
        "A": {"C"},
        "B": {"C"},
        "C": {"D", "E"},
        "D": {"F"},
        "E": {"F"},
        "F": set(),
    }
    assert a.validation_state == {
        "A": "valid",
        "B": "valid",
        "C": "valid",
        "D": "valid",
        "E": "valid",
        "F": "valid",
    }

    a.update_cached_value("B", None)
    assert a.dependency_graph == {
        "A": {"C"},
        "B": {"C"},
        "C": {"D", "E"},
        "D": {"F"},
        "E": {"F"},
        "F": set(),
    }
    assert a.validation_state == {
        "A": "valid",
        "B": "valid",
        "C": "invalid",
        "D": "invalid",
        "E": "invalid",
        "F": "invalid",
    }


def test_permanently_invalid():
    c = UseCacheObj(2)
    assert c.dependency_graph == {}
    assert c.validation_state == {}
    c.B()
    assert c.dependency_graph == {"A": {"B"}, "B": set()}
    assert c.validation_state == {"A": "permanently invalid", "B": "permanently invalid"}


def test_plot_dependency_graph_raises():
    c = IncorrectInheritance()

    with pytest.raises(TypeError):
        plot_dependency_graph(c)


def test_function_calls():
    c = FunctionCallObj(3)

    assert c.dependency_graph == {}
    assert c.validation_state == {}

    result = c.B()
    assert result == 7

    assert c.dependency_graph == {"B": set(), ("A", (("x", 0),)): {"B"}, ("A", (("x", 1),)): {"B"}}
    assert c.validation_state == {"B": "valid", ("A", (("x", 0),)): "valid", ("A", (("x", 1),)): "valid"}


def test_incorrect_decorator_format():
    with pytest.raises(ValueError):

        class IncorrectDependencyType(DependencyCacheBase):
            def __init__(self):
                pass

            @automagically_dependency_cached(dependencies=[3, ("B", [1])])
            def A(self):
                return 1

            @automagically_dependency_cached()
            def B(self):
                return 2
