# Network Observer

A local network observability tool written in Rust. It captures packets,
discovers hosts via ARP, detects peers via mDNS, and exposes live network
state through an HTTPS/WSS API and React dashboard.

![Rust Network Observer dashboard](docs/dashboard.png)

## Why this project exists

This project explores system-level Rust, packet capture, async services,
local-network discovery, and runtime visibility for edge-style systems.

## Features

- TCP/UDP and ARP packet capture via pcap
- ARP-based host discovery
- Session aggregation by IP/port pair
- mDNS peer discovery
- Peer-to-peer messages and file transfer
- HTTPS/WSS backend with local self-signed certificates
- React/Vite dashboard for hosts, packets, sessions, and peers

## Tech Stack

- Rust, Tokio, Axum
- pcap, pnet, etherparse
- mDNS, rustls
- React, TypeScript, Vite
- Docker
