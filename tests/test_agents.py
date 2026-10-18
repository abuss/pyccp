import asyncio
import pytest
import pyccp

def test_join_runs_agents_concurrently():
    async def main():
        store = pyccp.Store([[1, 2], [1, 2]])
        a, b = store.var("a", 0), store.var("b", 1)
        order = []
        async def pa():
            await a.commit(1); order.append("a")
        async def pb():
            await b.commit(2); order.append("b")
        d = asyncio.create_task(pyccp.dispatcher(store))
        await pyccp.join(pa, pb)
        d.cancel()
        assert sorted(order) == ["a", "b"]

    asyncio.run(main())

def test_choice_first_wins_and_cancels_pending():
    async def main():
        store = pyccp.Store([[1, 2]])
        x = store.var("x", 0)
        async def slow():
            await x.read()
            return "slow"
        async def fast():
            return "fast"
        d = asyncio.create_task(pyccp.dispatcher(store))
        assert await pyccp.choice(slow, fast) == "fast"
        d.cancel()

    asyncio.run(main())

def test_conflict_raises_in_agent_mode():
    async def main():
        # post prunes a to {2} without conflict; committing 1 then conflicts
        store = pyccp.Store([[1, 2], [1]])
        a, b = store.var("a", 0), store.var("b", 1)
        store.post(("neq", (0, 1)))
        d = asyncio.create_task(pyccp.dispatcher(store))
        with pytest.raises(pyccp.Conflict):
            await a.commit(1)
        d.cancel()

    asyncio.run(main())
