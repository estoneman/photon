# photon
frontend to cnvmp3 backend api

useful for:
* aspiring dj's
* music heads
* anyone with a soul

but actually, this will follow the steps for downloading music from the internet after
being converted from a youtube video to an mp3 file

╒══════╤═══════════════════╤═════════════════════════╤═════════════╤══════════════════════════════╤═════════════════╕
│ step │ url               │ endpoint                │ http method │ payload                      │ result          │
├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
│ 1    │ cnvmp3.com        │ /check\_database.php     │ POST        │ check\_database.json          │ success message │
├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
│ 2    │ cnvmp3.com        │ /get\_video\_data.php     │ POST        │ get\_video\_data.json          │ success message │
├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
│ 3    │ cnvmp3.com        │ /download\_video.php     │ POST        │ download\_video.json          │ mp3 url         │
├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
│ 4    │ cnvmp3.com        │ /insert\_to\_database.php │ POST        │ insert\_to\_database.json      │ success message │
├──────┼───────────────────┼─────────────────────────┼─────────────┼──────────────────────────────┼─────────────────┤
│ 5    │ N/A (CDN-defined) │ /download.php           │ GET         │ N/A (query string parameter) │ mp3 file data   │
╘══════╧═══════════════════╧═════════════════════════╧═════════════╧══════════════════════════════╧═════════════════╛

###### Ethan Stoneman 2024
