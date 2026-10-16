import threading
import time

from dependency_cache import DependencyCacheBase, automagically_dependency_cached, parallel_call


class TracingObj(DependencyCacheBase):
    """Subclass that records which thread called `a()` and how long it ran."""

    def __init__(self, value):
        self.calls = []  # (thread_id, start, end)
        self.value = value

    @automagically_dependency_cached()
    def A(self):
        tid = threading.get_ident()
        start = time.perf_counter()
        time.sleep(0.5)  # releases the GIL -> real concurrency across workers
        end = time.perf_counter()
        self.calls.append((tid, start, end))
        return self.B() ** 2

    @automagically_dependency_cached()
    def B(self):
        return self.value


def main():
    n = 8
    objs = [TracingObj(i) for i in range(n)]

    wall_start = time.perf_counter()
    results = parallel_call(objs, "A")
    wall_end = time.perf_counter()

    print("results:", results)
    print(f"wall time: {wall_end - wall_start:.3f}s")

    # --- Analysis ---
    all_calls = [c for obj in objs for c in obj.calls]
    thread_ids = {tid for tid, _, _ in all_calls}

    print(f"distinct Python thread ids used: {len(thread_ids)} -> {sorted(thread_ids)}")

    # Each call's start/end; check for overlap
    intervals = sorted((s, e) for _, s, e in all_calls)
    max_concurrent = 0
    for i, (s_i, e_i) in enumerate(intervals):
        # count how many intervals contain the midpoint of interval i
        mid = (s_i + e_i) / 2
        concurrent = sum(1 for s, e in intervals if s <= mid <= e)
        max_concurrent = max(max_concurrent, concurrent)

    print(f"max observed concurrency: {max_concurrent}")

    # --- Verdict ---
    if len(thread_ids) > 1 and max_concurrent > 1:
        print("✅ PARALLEL: multiple threads, overlapping execution observed")
    else:
        print("❌ SEQUENTIAL: no overlap detected")


if __name__ == "__main__":
    main()
