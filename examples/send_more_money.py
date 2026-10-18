from pyccp import Store

def run():
    s = Store([list(range(10))] * 8)          # S E N D M O R Y
    s.post(("all_diff", (list(range(8)),)))
    s.post(("nonzero", (0,)))
    s.post(("nonzero", (4,)))
    s.post(("linear_eq", ([(0, 1000), (1, 100), (2, 10), (3, 1),
                            (4, 1000), (5, 100), (6, 10), (1, 1),
                            (4, -10000), (5, -1000), (2, -100), (1, -10), (7, -1)], 0)))
    sol = s.solve()
    assert sol == [9, 5, 6, 7, 1, 0, 8, 2]
    print("send_more_money:", sol)
    return sol
