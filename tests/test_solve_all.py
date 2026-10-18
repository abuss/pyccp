import pyccp

def test_solve_all_returns_all_pairs():
    s = pyccp.Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    sols = s.solve_all()
    assert len(sols) == 11
    assert all(a + b == 10 for a, b in sols)

def test_solve_all_limit():
    s = pyccp.Store([list(range(11)), list(range(11))])
    s.post(("linear_eq", ([(0, 1), (1, 1)], 10)))
    assert len(s.solve_all(limit=3)) == 3
