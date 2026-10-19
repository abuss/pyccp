"""v1 agents observe the post-solve final state, not the main branch
incrementally: the dispatcher cannot run during a synchronous solve()."""
import asyncio
from queue import SimpleQueue
from typing import Optional
import pyccp_core
from pyccp.dispatcher import dispatcher


Conflict = pyccp_core.Conflict


class Var:
    def __init__(self, store, name, cell):
        self.store, self.name, self.cell = store, name, cell
        self._value = None
        self._event = asyncio.Event()

    def is_fixed(self) -> bool:
        return self._event.is_set()

    async def commit(self, value: int) -> None:
        self.store._core.commit(self.cell, value)

    async def read(self) -> int:
        self.store._readers[self.cell] = asyncio.current_task()
        await self._event.wait()
        return self._value


class Store:
    def __init__(self, domains: list[list[int]]):
        try:
            self._loop = asyncio.get_running_loop()
        except RuntimeError:
            self._loop = None
        self._queue = SimpleQueue()
        self._wake = asyncio.Event()
        self._core = pyccp_core.Store(domains, self._queue, self._loop, self._wake)
        self._vars: dict[int, Var] = {}
        self._readers: dict[int, asyncio.Task] = {}

    def var(self, name: str, cell: int) -> Var:
        v = Var(self, name, cell)
        self._vars[cell] = v
        return v

    def post(self, c: tuple) -> None:
        self._core.post(c)

    def solve(self) -> list[int] | None:
        return self._core.solve()

    def solve_all(self, limit: int = 1000) -> list[list[int]]:
        """All solutions in DFS order. Does not notify the bridge: enumeration
        is branch-local, so mirrors cannot reflect it."""
        return self._core.solve_all(limit)

    def _apply(self, cell: int, value) -> None:
        v = self._vars.get(cell)
        if v is None:
            return
        if value is None:  # undo
            v._value = None
            v._event.clear()
            t = self._readers.pop(cell, None)
            if t is not None and not t.done():
                t.cancel()
        else:
            v._value = value
            v._event.set()

    def drain(self) -> None:
        while not self._queue.empty():
            cell, value = self._queue.get()
            self._apply(cell, value)

    def set_objective(self, cell: int, minimize: bool = True) -> None:
        """Set the optimization objective: minimize (True) or maximize (False) the value in cell.
        
        Args:
            cell: Index of the cost cell
            minimize: True to minimize, False to maximize
        """
        self._core.set_objective(cell, minimize)

    def best_cost(self) -> Optional[int]:
        """Return the best cost found so far, or None if no feasible solution yet."""
        return self._core.best_cost()

    def is_bounds_domain(self, cell: int) -> bool:
        """Check if a domain is represented as Bounds (auto-converted for large domains)."""
        return self._core.is_bounds_domain(cell)




async def join(*agents):
    return await asyncio.gather(*[a() for a in agents])

async def choice(*agents):
    tasks = [asyncio.ensure_future(a()) for a in agents]
    done, pending = await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
    for t in pending:
        t.cancel()
    return done.pop().result()

async def at(var: Var) -> int:
    return await var.read()
