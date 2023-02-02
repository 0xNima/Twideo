from dotenv import load_dotenv

import asyncio
import aiohttp
import os
import psycopg
import redis

load_dotenv()

cache = redis.Redis(db=10)
pipe = cache.pipeline()
uids = []

with psycopg.connect(os.environ.get('DATABASE_URL')) as conn:
    with conn.cursor() as cur:
        for record in cur.execute("SELECT chat_id FROM users").fetchall():
            uid = record[0]
            uids.append(uid)
            pipe.hset('uids', uid, 1)

pipe.execute()


URL = f'https://api.telegram.org/bot{os.environ.get("TELOXIDE_TOKEN")}/sendMessage'

TEXT = """
"""


async def send_request(peer_id):
    _payload = {
        "chat_id": peer_id,
        "text": TEXT,
        "parse_mode": "html",
        "disable_web_page_preview": True,
    }
    async with aiohttp.ClientSession() as session:
        async with session.post(url=URL, data=_payload) as response:
            try:
                if response.status == 200:
                    cache.hdel('uids', peer_id)
                    print(f'requests is sent to {peer_id}')
                    return None
                msg = f'requests failed {peer_id}. status: {response.status}'
            except Exception as e:
                msg = f'requests failed {peer_id}. reason: {e}'
            
            print(msg)
            return peer_id


async def main():
    CHUNK = 100
    
    tasks = asyncio.Queue(len(uids))
    
    for peer_id in uids:
        await tasks.put(send_request(peer_id))

    while not tasks.empty():
        print(f'running tasks again... - queue size: {tasks.qsize()}')

        _chunked = []

        for _ in range(CHUNK):
            try:
                _chunked.append(tasks.get_nowait())
            except asyncio.QueueEmpty:
                continue

        if _chunked:
            result = await asyncio.gather(*_chunked)
            for response in result:
                if isinstance(response, int):
                    await tasks.put(send_request(response))

        await asyncio.sleep(5)


if __name__ == '__main__':
    try:
        asyncio.get_event_loop().run_until_complete(main())
    except:
        print ("_________________________ ERROR _________________________")
