# Building TDLIB

As telegram bot has a max file upload size of `50MB`, we can use `tdlib` as a normal user to upload files which increases the limit to `2GB` and `4GB` for premium users.

In order for this to work, we will have a normal telegram bot and a normal user account.
The user account is responsible for uploading the files and forwarding the file to the Bot.
Once the Bot receive the file, it can forward to the user who requested the file.

### OSX

```
xcode-select --install
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install gperf cmake openssl
git clone https://github.com/tdlib/td.git
cd td
rm -rf build
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=Release -DOPENSSL_ROOT_DIR=/opt/homebrew/opt/openssl/ -DCMAKE_INSTALL_PREFIX:PATH=/usr/local -DTD_ENABLE_LTO=ON ..
sudo cmake --build . --target install
cd ..
cd ..
ls -l /usr/local
```

### RPI4

```
sudo apt-get update
sudo apt-get upgrade
sudo apt-get install make git zlib1g-dev libssl-dev gperf php5-cli cmake3 clang-3.9 libc++-dev
git clone https://github.com/tdlib/td.git
cd td
rm -rf build
mkdir build
cd build
CXXFLAGS="-stdlib=libc++" CC=/usr/bin/clang-3.9 CXX=/usr/bin/clang++-3.9 cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=/usr/local ..
sudo cmake --build . --target install
cd ..
cd ..
ls -l /usr/local
```

# Introduction

The idea of this project is to automatically upload files in a folder to a telegram chat. One use case could be auto-upload torrent files once it is completed.

In order to upload larger file size, the Bots api cannot be used. Hence `tdlib` is used and a real account.

# Torrent Client

I'm using qBittorent on a RPI4 and it works well.
You can find the latest binary here. https://github.com/userdocs/qbittorrent-nox-static

```
wget https://github.com/userdocs/qbittorrent-nox-static/releases/download/release-4.5.4_v2.0.9/aarch64-qbittorrent-nox
mv aarch64-qbittorrent-nox /usr/bin/qbittorrent-nox
chmod +x /usr/bin/qbittorrent-nox
```

Set the qBittorrent as a service `/etc/systemd/system/qbittorrent.service`.

```
[Unit]
Description=qBittorrent
After=network.target

[Service]
Type=forking
User=pi
Group=pi
UMask=002
ExecStart=/usr/bin/qbittorrent-nox -d --webui-port=8080
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Then we start the service, you should be able to access the webui at the machine's port :8080.
The default login credentials are `admin:adminadmin`

```bash
sudo systemctl start qbittorrent
```

# MongoDB

I'm using MongoDB Atlas as a database to store all files metadata which has been uploaded.
This is to prevent reuploading the files and just forward it using the `file_id`.

# .env file

```
TELEGRAM_TOKEN=
TORRENT_JOB_FOLDER=/home/pi/torrent/job
TORRENT_DONE_PATH=/home/pi/torrent/upload
MONGO_URI=
ADMIN_ID=
API_ID=
API_HASH=
SEND_TO=
```
