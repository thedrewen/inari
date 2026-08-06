<div align="center">
<img src="./assets/inari-inverted.png" width="250px">
<h1>
Inari
</h1>

![Version](https://img.shields.io/badge/version-0.0.2-informational)
[![Discord](https://img.shields.io/discord/1532277204585676810?label=Discord)](https://discord.gg/4vCG6vMNDJ)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL%20v3-blue)](https://www.gnu.org/licenses/agpl-3.0.html)

A web app that lets you chat with people nearby in games like Minecraft, etc.
</div>

> ⚠️ The `main` branch contains the stable production code. `develop` is the active development branch, potentially unstable.

## Tech stack

**Frontend** - Angular   
**Server** - Rust with WebSocket

## Issues

If you have any issues, please contact us via Discord. We've set up a dedicated channel for this!

## Architecture

```
inari/
  frontend/   Angular - Web app
  server/     Rust - WebSocket Server
  assets/  Static assets (images, SVG...)
```

## Environment variables

The project requires a `.env` file at the root of each service. Refer to the corresponding `.env.example` files.

## Running

Docker Hub : https://hub.docker.com/r/thedrewen/inari

### Release
```bash
docker run --name inari -e API_KEY=<GENERATE API KEY> -e SERVER_PUBLIC_IP=<YOUR PUBLIC IP> -p 3030:3030 -p 44444:44444/udp --restart unless-stopped thedrewen/inari:stable
```

### Nightly
```bash
docker run --name inari -e API_KEY=<GENERATE API KEY> -e SERVER_PUBLIC_IP=<YOUR PUBLIC IP> -p 3030:3030 -p 44444:44444/udp --restart unless-stopped thedrewen/inari:nightly
```

By default, the project runs at http://localhost:3030.

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

Any modification must be released under the same license. Using this software as a network service (SaaS) also requires making the source code publicly available. Commercial use without complying with these terms is not permitted.
