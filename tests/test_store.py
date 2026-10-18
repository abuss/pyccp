import pytest
import pyccp

def test_send_more_money_through_ffi():
    domains = [list(range(10))] * 8
    s = pyccp.Store(domains)
    s.post(("all_diff", (list(range(8)),)))
    s.post(("nonzero", (0,)))
    s.post(("nonzero", (4,)))
    s.post(("linear_eq", ([(0, 1000), (1, 100), (2, 10), (3, 1),
                            (4, 1000), (5, 100), (6, 10), (1, 1),
                            (4, -10000), (5, -1000), (2, -100), (1, -10), (7, -1)], 0)))
    assert s.solve() == [9, 5, 6, 7, 1, 0, 8, 2]

def test_unknown_kind_raises():
    s = pyccp.Store([[0, 1]])
    with pytest.raises(ValueError):
        s.post(("bogus", (0,)))

def test_conflict_is_pyccp_exception():
    s = pyccp.Store([[1], [1]])
    with pytest.raises(pyccp.Conflict):
        s.post(("neq", (0, 1)))
