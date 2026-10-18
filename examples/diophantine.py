from pyccp import Store

def run():
    s = Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    sols = s.solve_all()
    assert len(sols) == 11
    assert all(a + b == 10 for a, b in sols)
    print("diophantine:", sols)
    return sols
