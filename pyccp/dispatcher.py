import asyncio

async def dispatcher(store) -> None:
    while True:
        await store._wake.wait()
        store._wake.clear()
        store.drain()
