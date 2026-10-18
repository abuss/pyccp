import asyncio
import pytest
import pyccp

def test_consumer_wakes_on_propagated_fix():
    async def main():
        # y - x == 1; x in {1,2}, y in {2,3}
        store = pyccp.Store([[1, 2], [2, 3]])
        x = store.var("x", 0)
        y = store.var("y", 1)
        store.post(("linear_eq", ([(1, 1), (0, -1)], 1)))
        seen = []

        async def producer():
            await x.commit(1)

        async def consumer():
            seen.append(await y.read())

        d = asyncio.create_task(pyccp.dispatcher(store))
        await asyncio.gather(producer(), consumer())
        d.cancel()
        assert seen == [2]

    asyncio.run(main())

def test_read_suspends_until_fix():
    async def main():
        store = pyccp.Store([[1, 2]])
        x = store.var("x", 0)
        async def reader():
            return await x.read()
        d = asyncio.create_task(pyccp.dispatcher(store))
        t = asyncio.create_task(reader())
        await asyncio.sleep(0.05)
        assert not t.done()
        await x.commit(2)
        assert await t == 2
        d.cancel()

    asyncio.run(main())
