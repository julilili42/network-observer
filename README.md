# rustdrop

A small local-network tool for packet capture, ARP discovery, and peer-to-peer messages/files.

Rust backend. React/Vite frontend.

## Features

- live TCP/UDP and ARP capture
- ARP-based host discovery
- session stats by IP/port pair
- mDNS peer discovery
- peer messages
- file transfer between discovered peers
- HTTPS/WSS backend with local self-signed certs

## Run

Backend:

```sh
sudo apt install libpcap-dev
PORT=3000 DEVICE_NAME=my-device INTERFACE=eth0 cargo run
```

Packet capture usually needs elevated permissions.

Frontend:

```sh
cd gui
npm install
npm run dev
```

The UI connects to `https://localhost:3000` by default.

Since the backend uses a self-signed cert, your browser may ask you to trust it once.

## Docker

```sh
docker build -f dockerfile -t rustdrop .
docker run --rm --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  -e DEVICE_NAME=my-device \
  -e INTERFACE=eth0 \
  rustdrop
```

## API

Main routes:

```txt
POST   /capture
DELETE /capture
POST   /scan
DELETE /scan
GET    /packets
GET    /hosts
GET    /sessions
GET    /peers
GET    /peers/messages
GET    /ws
```

## Notes

Use this only on networks you own or are allowed to inspect.
