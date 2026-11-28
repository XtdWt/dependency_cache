import threading

from dependency_cache import CopyLocal

ctx = CopyLocal()
ctx.user = "alice"


def inner_worker():
    print(ctx.hello())
    assert ctx.user == "bob"
    ctx.user = "charlie"
    assert ctx.user == "charlie"


def outer_worker():
    assert ctx.user == "alice"
    ctx.user = "bob"
    assert ctx.user == "bob"
    t_inner_threads = []
    for i in range(1):
        t_inner = threading.Thread(target=inner_worker)
        t_inner_threads.append(t_inner)
    for t_inner in t_inner_threads:
        t_inner.start()
    for t_inner in t_inner_threads:
        t_inner.join()

    assert ctx.user == "bob"


threads = []
assert ctx.user == "alice"
for _ in range(1):
    t = threading.Thread(target=outer_worker)
    threads.append(t)
for t in threads:
    t.start()
for t in threads:
    t.join()

assert ctx.user == "alice"
