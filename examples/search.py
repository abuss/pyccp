from pyccp import Store

def run():
    # asymmetric domains, x < y, enumerate
    s = Store([list(range(1, 4)), list(range(3, 6))])
    s.post(("lt", (0, 1)))
    sols = s.solve_all()
    assert len(sols) == 8            # (1,3)..(1,5), (2,3)..(2,5), (3,4), (3,5)
    assert all(a < b for a, b in sols)
    print("search:", sols)
    return sols
