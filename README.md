# photon
frontend to cnvmp3 backend api

useful for:
* aspiring dj's
* music heads
* anyone with a soul

but actually, this will follow the steps for downloading music from the internet after
being converted from a youtube video to an mp3 file

|step|host|endpoint|http method|description|
|:---|:--:|:------:|:---------:|----------:|
|1|cnvmp3.com|/check\_database.php|POST|confirm whether song file is not local to cdn|
|2|cnvmp3.com|/get\_video\_data.php|POST|tells server to get video details from youtube|
|3|cnvmp3.com|/download\_video.php|POST|tells server to download song file into cdn|
|4|cnvmp3.com|/insert\_to\_database.php|POST|insert metadata about song in server database for faster retrieval on future requests to same song|
|5|N/A (CDN-defined)|/download.php|GET|download the song file locally|

###### Ethan Stoneman 2024
