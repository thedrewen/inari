import { WebSocket } from "ws";
import { v4 } from "uuid";
import { readFile } from 'fs/promises'
 
const ws = new WebSocket("wss://dev-inari.thedrewen.com/ws"); // Your address

function send(kind, data) {
    try {
        ws.send(JSON.stringify({kind, data, transaction_uuid: v4()}));
    } catch (error) {}
}

let interval;

ws.on('open', () => {
    console.log('Connected !');
    send('auth', {channel: 'default', auth_key: '1234', is_server: true});

    interval = setInterval(async () => {
        const data = await readFile('./positions.json', 'utf-8');
        send('clients', { clients: JSON.parse(data) });
    }, 1000);
});

ws.on('message', (message) => {
    console.log(message.toString());
});

ws.on('close', () => {
    console.log('Connection closed !');
    clearInterval(interval);
});