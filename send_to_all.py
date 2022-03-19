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
                    failure.append(payload['chat_id'])
                else:
                    print(payload['chat_id'])
                    sent.append(payload['chat_id']);
            except Exception as e:
                failure.append(payload['chat_id'])

            await asyncio.sleep(2)


URL = "https://api.telegram.org/bot{}/sendMessage"

TEXT = """
🔔 🔔 🔔

📌 Now you can copy the link of <b>any</b> twitt (not only those containing video) 
and convert it to a <b>telegram</b> message\n
Try it now 👉 copy and send <a href="https://twitter.com/telegram/status/1497210881557647365?s=20&t=UCHU5p2ZeKe-ZsZGO55DmQS">this</a> link
"""


async def main():
    tasks = []
    for peer_id in users:
        _payload = {
            "chat_id": peer_id,
            "text": TEXT,
            "parse_mode": "html",
            "disable_web_page_preview": True
        }
        tasks.append(send_request(URL.format(os.getenv("TELOXIDE_TOKEN2")), _payload))

    await asyncio.gather(*tasks)


if __name__ == '__main__':
    try:
        asyncio.get_event_loop().run_until_complete(main())
    except:
        print ("_________________________ ERROR _________________________")

    print("failure: \n", failure)
    print("\n\n")
    print("sent \n", sent)
