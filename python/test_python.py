import pytest
from dependency_cache import DependencyCacheBase, dependency_cache


class SimpleTestObj(DependencyCacheBase):
    def __init__(self, x, y):
        self.x = x
        self.y = y

    @dependency_cache()
    def A(self):
        return self.x

    @dependency_cache()
    def B(self):
        return self.y

    @dependency_cache(dependencies=["A", "B"])
    def C(self):
        return self.A() + self.B()


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
