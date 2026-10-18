import asyncio
import pyccp

def run():
    async def main():
        s = pyccp.Store([[1, 2], [2, 3]])
        x, y = s.var("x", 0), s.var("y", 1)
        s.post(("linear_eq", ([(1, 1), (0, -1)], 1)))   # y = x + 1
        seen = []

        async def producer():
            await x.commit(1)

        async def consumer():
            seen.append(await y.read())

        d = asyncio.create_task(pyccp.dispatcher(s))
        await pyccp.join(producer, consumer)
        d.cancel()
        assert seen == [2]
        print("producer_consumer:", seen)
        return seen

    return asyncio.run(main())
