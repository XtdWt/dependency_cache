import re
import inspect
import functools

from dependency_cache import DependencyCacheBase, dependency_cached


def custom_decorator(func):
    code = inspect.getsource(func)
    # NOTE: this regex parsing is simply an example and NOT in any way recommended
    dependencies = re.findall(r'self\.([A-Za-z_]\w*)\(\)', code)
    @dependency_cached(dependencies=dependencies)
    @functools.wraps(func)
    def wrapper(*call_args, **call_kwargs):
        return func(*call_args, **call_kwargs)
    return wrapper


class MultipleDecorators(DependencyCacheBase):

    @dependency_cached()
    def A(self):
        print("Calculating A")
        return 1

    @dependency_cached()
    def B(self):
        print("Calculating B")
        return 2

    @custom_decorator
    def C(self):
        print("Calculating C")
        return self.A() + self.B()


if __name__ == '__main__':
    print("Wrap the base `dependency_cached` to create custom decorators")
    print("All decorators that are wrapped with dependency_cached will work together")
    obj = MultipleDecorators()
    print(f"Result of C = {obj.C()}")  # calculates all, prints 3
    obj.update_cached_value("B", 4)  # updates B, then invalidates C
    print(f"Result of C = {obj.C()}")  # calculates all, prints 5
