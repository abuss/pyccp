from pyccp import Store

def run():
    # 3x3 magic square, cells 0..8, every line sums to 15
    lines = [(0,1,2), (3,4,5), (6,7,8), (0,3,6), (1,4,7), (2,5,8), (0,4,8), (2,4,6)]
    s = Store([list(range(1, 10))] * 9)
    s.post(("all_diff", (list(range(9)),)))
    for line in lines:
        s.post(("linear_eq", ([(c, 1) for c in line], 15)))
    sol = s.solve()
    assert sol is not None
    for line in lines:
        assert sum(sol[c] for c in line) == 15
    print("magic_square:", sol)
    return sol
