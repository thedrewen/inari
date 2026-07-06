# Inari

Inari is a SaaS platform for voice channels with user account management, a plugin system, and deep customization. The project is open source and hosted by [FoxDroidLabs](https://foxdroidlabs.com).

## Tech stack

**Frontend** — Angular 22 with server-side rendering (SSR)  
**API** — NestJS  
**Server** — Node.js WebSocket server

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

Open source — see `LICENSE`.
