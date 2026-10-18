from pyccp import Store

def run():
    # shift type = day mod 3 for days 1..7
    s = Store([list(range(1, 8))])
    s.post(("modulo", (0, 3, 1)))
    sol = s.solve()
    assert sol is not None and sol[0] % 3 == 1
    print("cyclic:", sol)
    return sol
