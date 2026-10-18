import asyncio
import pyccp

def test_producer_propagator_consumer():
    async def main():
        # SEND+MORE=MONEY, agents feed digits, consumer observes fixes
        domains = [list(range(10))] * 8  # S E N D M O R Y
        store = pyccp.Store(domains)
        vars_ = [store.var(c, i) for i, c in enumerate("SENDMORY")]
        store.post(("all_diff", (list(range(8)),)))
        store.post(("nonzero", (0,)))
        store.post(("nonzero", (4,)))
        store.post(("linear_eq", ([(0, 1000), (1, 100), (2, 10), (3, 1),
                                   (4, 1000), (5, 100), (6, 10), (1, 1),
                                   (4, -10000), (5, -1000), (2, -100), (1, -10), (7, -1)], 0)))

        seen = []
        async def producer():
            await vars_[4].commit(1)          # M = 1
            await vars_[0].commit(9)          # S = 9
        async def consumer():
            seen.append(await vars_[4].read())
            seen.append(await vars_[0].read())

        d = asyncio.create_task(pyccp.dispatcher(store))
        await pyccp.join(producer, consumer)
        d.cancel()
        assert seen == [1, 9]

    asyncio.run(main())

def test_undo_clears_mirror():
    # v1: undo events clear mirrors; retroactive cancellation of an already-
    # woken reader is unreachable (fix is drained before undo) and deferred.
    async def main():
        store = pyccp.Store([[1, 2]])
        x = store.var("x", 0)
        store._apply(0, 1)
        assert x.is_fixed()
        store._apply(0, None)
        assert not x.is_fixed()

    asyncio.run(main())
