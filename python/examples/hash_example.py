import inspect


def make_cache_key(func, *args, **kwargs):
    sig = inspect.signature(func)
    bound = sig.bind(*args, **kwargs)
    bound.apply_defaults()

    return tuple(bound.arguments.items())


if __name__ == "__main__":

    def f(a, b):
        return a + b

    k1 = make_cache_key(f, 1, 2)
    k2 = make_cache_key(f, a=1, b=2)
    print(f.__name__, k1, f.__name__, k2)
    print(k1 == k2)  # True
