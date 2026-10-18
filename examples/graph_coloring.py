from pyccp import Store

def run():
    # K4: every pair of 4 nodes differs, needs 4 colours
    s = Store([list(range(4))] * 4)
    for i in range(4):
        for j in range(i + 1, 4):
            s.post(("neq", (i, j)))
    sol = s.solve()
    assert sol is not None and len(set(sol)) == 4
    print("graph_coloring:", sol)
    return sol
