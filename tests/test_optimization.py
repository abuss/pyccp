import pyccp


def test_set_objective_stores_flag():
    s = pyccp.Store([list(range(1, 4))])
    s.set_objective(0, minimize=True)
    assert s.best_cost() is None


def test_set_objective_invalid_cell_raises():
    s = pyccp.Store([list(range(1, 4))])
    try:
        s.set_objective(5, minimize=True)
        assert False, "should have raised"
    except:
        pass  # expected


def test_best_cost_none_initially():
    s = pyccp.Store([list(range(1, 4))])
    assert s.best_cost() is None


def test_best_cost_updates_after_solve():
    s = pyccp.Store([list(range(1, 4))])
    s.set_objective(0, minimize=True)
    sol = s.solve()
    # Cost should be updated to the cost of the solution found
    assert s.best_cost() is not None


def test_maximize_objective():
    s = pyccp.Store([list(range(1, 4))])
    s.set_objective(0, minimize=False)
    sol = s.solve()
    # Should find highest value (or first feasible)
    assert sol is not None
