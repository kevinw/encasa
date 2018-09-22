@echo off

set PY=%LOCALAPPDATA%\Programs\Python\Python37-32\python.exe
%PY% -m pip uninstall apiclient
%PY% -m pip install --upgrade google-api-python-client oauth2-client
%PY% scrape_events.py
