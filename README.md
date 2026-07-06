# Inari

![Version](https://img.shields.io/badge/version-0.0.1-informational)
[![Discord](https://img.shields.io/discord/aaNVx6Pucw?logo=discord&logoColor=white&label=Discord&color=5865f2)](https://discord.com/invite/aaNVx6Pucw)
![Angular](https://img.shields.io/badge/Angular-22-dd0031?logo=angular&logoColor=white)
![NestJS](https://img.shields.io/badge/NestJS-11-e0234e?logo=nestjs&logoColor=white)
![Node.js](https://img.shields.io/badge/Node.js-26-339933?logo=node.js&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-6-3178c6?logo=typescript&logoColor=white)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPL%20v3-blue)](https://www.gnu.org/licenses/agpl-3.0)

Inari is a SaaS platform for voice channels with user account management, a plugin system, and deep customization. The project is open source and hosted by [FoxDroidLabs](https://foxdroidlabs.com).

## Tech stack

**Frontend** - Angular 22 with server-side rendering (SSR)  
**API** - NestJS  
**Server** - Node.js WebSocket server

## Architecture

```
inari/
  api/        NestJS — business logic and REST endpoints
  frontend/   Angular SSR — user interface
  server/     Node.js — WebSocket server
  resources/  Static assets (images, SVG...)
```

## Environment variables

The project requires a `.env` file at the root of each service. Refer to the corresponding `.env.example` files.

## Running

> This project is designed to be hosted by FoxDroidLabs and is not intended for self-hosting. Contributions are still welcome.

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

Any modification must be released under the same license. Using this software as a network service (SaaS) also requires making the source code publicly available. Commercial use without complying with these terms is not permitted.
