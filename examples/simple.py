from pyccp import Store

def run():
    s = Store([list(range(1, 4)), list(range(1, 4))])
    s.post(("lt", (0, 1)))
    sol = s.solve()
    assert sol is not None and sol[0] < sol[1]
    print("simple:", sol)
    return sol
