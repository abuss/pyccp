from pyccp import Store

def run():
    s = Store([list(range(1, 13)), list(range(1, 13))])
    s.post(("mul", (0, 1, 12)))
    sol = s.solve()
    assert sol is not None and sol[0] * sol[1] == 12
    print("scaling:", sol)
    return sol
