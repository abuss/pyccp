from pyccp import Store

def run():
    s = Store([list(range(1, 5))] * 5)
    s.post(("cardinality", (list(range(5)), 3, 2)))
    sol = s.solve()
    assert sol is not None and sol.count(3) == 2
    print("cardinality:", sol)
    return sol
