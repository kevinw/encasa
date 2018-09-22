"""
Shows basic usage of the Google Calendar API. Creates a Google Calendar API
service object and outputs a list of the next 10 events on the user's calendar.
"""

# TODO: finish new google.auth upgrade https://google-auth.readthedocs.io/en/latest/user-guide.html

from __future__ import print_function
from googleapiclient.discovery import build
from httplib2 import Http
import google.auth
from pprint import pformat
import datetime
import os.path

# Setup the Calendar API
SCOPES = 'https://www.googleapis.com/auth/calendar.readonly'
store = file.Storage('credentials.json')
creds = store.get()
if not creds or creds.invalid:
    flow = client.flow_from_clientsecrets('client_secret.json', SCOPES)
    creds = tools.run_flow(flow, store)
service = build('calendar', 'v3', http=creds.authorize(Http()))

# Call the Calendar API
now = datetime.datetime.utcnow().isoformat() + 'Z' # 'Z' indicates UTC time

calendar_ids = service.calendarList().list().execute()
print(pformat(calendar_ids))

events_result = service.events().list(calendarId='primary', timeMin=now,
                                      maxResults=50, singleEvents=True,
                                      orderBy='startTime').execute()

events = events_result.get('items', [])

if not events:
    print('No upcoming events found.')

output = dict(deadlines=[])

import dateutil.parser
def parse_date_or_datetime(t):
    return t

    if 'timeZone' in t:
        assert false, "TODO: implement timeZone handling"
    if 'date' in t:
        assert 'datetime' not in t
        return dateutil.parser.parse(t['date'])
    if 'dateTime' in t:
        assert 'date' not in t
        return dateutil.parser.parse(t['dateTime'])

count = 0
for event in events:
    if "#deadline" in event.get('description', ''):
        count += 1
        start = event['start'].get('dateTime', event['start'].get('date'))
        # print(start, event['summary'])
        print(start, pformat(event))


        time = event["start"]

        output["deadlines"].append(event)

print('---')

import json
json_output = json.dumps(output, indent=4, separators=(',', ': '))
print(json_output)

json_path = os.path.expanduser("~/deadlines.json")
with open(json_path, "w") as f:
    f.write(json_output)
print("saved", count, "events to", json_path)
