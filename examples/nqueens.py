from pyccp import Store

def run():
    n = 8
    s = Store([list(range(n)) for _ in range(n)])
    for i in range(n):
        for j in range(i + 1, n):
            s.post(("neq", (i, j)))
            s.post(("neq_linear", ([(i, 1), (j, -1)], i - j)))
            s.post(("neq_linear", ([(i, 1), (j, -1)], j - i)))
    sol = s.solve()
    assert sol is not None
    for i in range(n):
        for j in range(i + 1, n):
            assert sol[i] != sol[j]
            assert sol[i] - i != sol[j] - j
            assert sol[i] + i != sol[j] + j
    print("nqueens:", sol)
    return sol
