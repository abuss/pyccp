from pyccp import Store

def run():
    domain = [v for v in range(1, 101) if v % 4 != 0 and v % 7 != 0]
    s = Store([domain])
    s.post(("modulo", (0, 3, 0)))
    sols = s.solve_all()
    assert len(sols) == 22
    assert all(v[0] % 3 == 0 for v in sols)
    print("modulo_filter:", len(sols), "solutions")
    return sols
