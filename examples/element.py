from pyccp import Store

def run():
    # y == array[idx], array = cells 0..2, idx = 3, y = 4
    s = Store([list(range(1, 4))] * 3 + [[0, 1, 2], list(range(1, 4))])
    s.post(("element", ([0, 1, 2], 3, 4)))
    s.post(("lt", (3, 4)))            # idx < y, so the answer is not trivial
    sol = s.solve()
    assert sol is not None and sol[4] == sol[sol[3]]
    print("element:", sol)
    return sol
