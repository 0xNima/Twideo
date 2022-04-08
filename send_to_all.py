from dotenv import load_dotenv

import asyncio
import aiohttp
import json
import os

load_dotenv()

with open('users.json', 'r') as fd:
    json_data = json.load(fd)

with open('users2.json', 'r') as fd:
    other_json = json.load(fd)


all_users = set()

for data in [other_json, json_data]:
    for item in data['values']:
        all_users.add(item[0])

sent = []

failure = []

users = filter(lambda x: x not in sent, all_users)

async def send_request(url, payload):
    async with aiohttp.ClientSession() as session:
        async with session.post(url=url, data=payload) as response:
            try:
                if response.status != 200:
                    print(payload['chat_id'], ": ", await response.text())
                    failure.append(payload['chat_id'])
                else:
                    print(payload['chat_id'], ",")
                    sent.append(payload['chat_id']);
            except Exception as e:
                failure.append(payload['chat_id'])


URL = "https://api.telegram.org/bot{}/sendMessage"
VIDEO_MSG_URL = "https://api.telegram.org/bot{}/sendAnimation"


TEXT = """
🔔 🔔 🔔

📌

🔵
🔵
🔴
"""


async def main():
    tasks = []
    chunk = 100
    for peer_id in users:
        _payload = {
            "chat_id": peer_id,
            "caption": TEXT,
            "parse_mode": "html",
            "disable_web_page_preview": True,
            "animation": ""
        }
        tasks.append(send_request(VIDEO_MSG_URL.format(os.getenv("TELOXIDE_TOKEN2")), _payload))

    for i in range(0, len(tasks), chunk):
        chunked = tasks[i: i+chunk]
        await asyncio.gather(*chunked)
        await asyncio.sleep(1)

if __name__ == '__main__':
    try:
        asyncio.get_event_loop().run_until_complete(main())
    except:
        print ("_________________________ ERROR _________________________")

    print("failure: \n", failure)
    print("\n\n")
    print("sent \n", sent)
