import pytest
from dependency_cache import DependencyCacheBase, dependency_cached


class SimpleTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        self.x = x
        self.y = y

    @dependency_cached()
    def A(self):
        return self.x

    @dependency_cached()
    def B(self):
        return self.y

    @dependency_cached(dependencies=["A", "B"])
    def C(self):
        return self.A() + self.B()


class HardTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        self.x = x
        self.y = y

    @dependency_cached()
    def A(self):
        return self.x

    @dependency_cached()
    def B(self):
        return self.y

    @dependency_cached(dependencies=["A", "B"])
    def C(self):
        return self.A() + self.B()

    @dependency_cached(dependencies=["C"])
    def D(self):
        return self.C() * 2

    @dependency_cached(dependencies=["C"])
    def E(self):
        return self.C() * 3


class HarderTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        self.x = x
        self.y = y

    @dependency_cached()
    def A(self):
        return self.x

    @dependency_cached()
    def B(self):
        return self.y

    @dependency_cached(dependencies=["A", "B"])
    def C(self):
        return self.A() + self.B()

    @dependency_cached(dependencies=["C"])
    def D(self):
        return self.C() * 2

    @dependency_cached(dependencies=["C"])
    def E(self):
        return self.C() * 3

    @dependency_cached(dependencies=["D", "E"])
    def F(self):
        return self.E() + self.D()


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
    assert a.current_graph() == {}
    assert a.current_cache_validation() == {}


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

    assert a.current_graph() == {"A": ["C"], "B": ["C"]}
    assert a.current_cache_validation() == {"A": True, "B": True, "C": True}

    a.update_cached_value("B", None)
    assert a.current_graph() == {"A": ["C"], "B": ["C"]}
    assert a.current_cache_validation() == {"A": True, "B": True, "C": False}


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

    assert {k: sorted(v) for k, v in a.current_graph().items()} == {"A": ["C"], "B": ["C"], "C": ["D", "E"]}
    assert a.current_cache_validation() == {"A": True, "B": True, "C": True, "D": True, "E": True}

    a.update_cached_value("B", None)
    assert {k: sorted(v) for k, v in a.current_graph().items()} == {"A": ["C"], "B": ["C"], "C": ["D", "E"]}
    assert a.current_cache_validation() == {"A": True, "B": True, "C": False, "D": False, "E": False}


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

    assert {k: sorted(v) for k, v in a.current_graph().items()} == {
        "A": ["C"],
        "B": ["C"],
        "C": ["D", "E"],
        "D": ["F"],
        "E": ["F"],
    }
    assert a.current_cache_validation() == {"A": True, "B": True, "C": True, "D": True, "E": True, "F": True}

    a.update_cached_value("B", None)
    assert {k: sorted(v) for k, v in a.current_graph().items()} == {
        "A": ["C"],
        "B": ["C"],
        "C": ["D", "E"],
        "D": ["F"],
        "E": ["F"],
    }
    assert a.current_cache_validation() == {"A": True, "B": True, "C": False, "D": False, "E": False, "F": False}
