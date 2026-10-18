import asyncio
import pyccp

def run():
    async def main():
        n = 4
        s = pyccp.Store([list(range(n + 1))] * n)
        ports = [s.var(f"p{i}", i) for i in range(n)]
        seen = []

        async def agent(i):
            v = await ports[i].read()
            seen.append(i)
            target = ports[(i + 1) % n]
            if not target.is_fixed():
                await target.commit(v + 1)

        d = asyncio.create_task(pyccp.dispatcher(s))
        await ports[0].commit(0)
        await pyccp.join(*[lambda i=i: agent(i) for i in range(n)])
        d.cancel()
        assert seen == [0, 1, 2, 3]
        print("agent_counter:", seen)
        return seen

    return asyncio.run(main())
