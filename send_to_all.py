import asyncio

import aiohttp
import json

with open('users.json', 'r') as fd:
    json_data = json.load(fd)

sent = []

users = filter(lambda x: x, map(lambda x: x[0] if (x[0] not in sent) else None, json_data['values']))

failure = []


async def send_request(url, payload):
    async with aiohttp.ClientSession() as session:
        async with session.post(url=url, data=payload) as response:
            try:
                print(await response.json())
                # if response.status != 200:
                #     failure.append(payload['chat_id'])
                # else:
                #     print(payload['chat_id'])
            except Exception as e:
                print("fail: {}".format(payload['chat_id']))

            await asyncio.sleep(0.5)


URL = "https://api.telegram.org/{}/sendMessage"

TEXT = """
Hey 👋
Happy New Year 🎆🎉🥳🎄

We are 🆙 again

Sorry for 1week down time 👀

I have a good news too 🕊

✅ Twideo will enables you to download videos from <b>Instagram</b>. I'll inform you ASAP
"""


async def main():
    tasks = []
    for peer_id in users:
        _payload = {
            "chat_id": peer_id,
            "text": TEXT,
            "parse_mode": "html",
        }
        tasks.append(send_request(URL, _payload))

    await asyncio.gather(*tasks)


if __name__ == '__main__':
    asyncio.get_event_loop().run_until_complete(main())
    print(failure)
