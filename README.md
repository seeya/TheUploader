# Introduction

The idea of this project is to automatically upload files in a folder to a telegram chat. One use case could be auto-upload torrent files once it is completed.

In order to upload larger file size, the Bots api cannot be used. Hence `tdlib` is used and a real account.

# Torrent Client

When a magnet link is received, the magnet link will be posted to `TORRENT_API` defined in the `.env` file. This is to automatically start a download and once complete run a script to copy the files to the `WATCH_PATH` folder to start the upload to the `SEND_TO` chat.

The torrent client tested with can be found here https://github.com/boypt/simple-torrent

### Torrent Client Config

Note that the torrent client reads its config from `cloud-torrent.yaml`.
Update the keys `donecmd`, `downloaddirectory`, `watchdirectory` as well as the `doneCMD.sh`'s `RESTAPI` and `JOBPATH` to what you've set.

```
allowruntimeconfigure: true
autostart: true
disableutp: false
donecmd: "/home/pi/Torrents/scripts/doneCMD.sh"
downloaddirectory: "/home/pi/Torrents/downloaded"
enableseeding: true
enableupload: true
incomingport: 50007
maxconcurrenttask: 0
nodefaultportforwarding: true
obfspreferred: true
obfsrequirepreferred: false
proxyurl: ""
seedratio: 0
seedtime: "0"
watchdirectory: "/home/pi/Torrents/pending"
```

# .env file

```
WATCH_PATH=/home/pi/jobs
UPLOADING_PATH=/home/pi/uploading
SEND_TO=telegram_chat_id
API_ID=
API_HASH=
TORRENT_API=http://localhost:3000
```
