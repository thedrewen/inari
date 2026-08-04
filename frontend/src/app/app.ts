import { Component, OnInit, signal } from '@angular/core';
import { WebsocketService } from './services/websocket.service';


@Component({
    selector: 'app-root',
    imports: [],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App implements OnInit {

    constructor (
        public readonly websocketService: WebsocketService
    ) { }

    ngOnInit(): void {
        document.body.style.animationDuration = '20s';
    }

    public onConnection = signal(false);

    async login() {
        this.onConnection.set(true);
        await this.websocketService.init(this.authCode());
        this.onConnection.set(false);
    }

    public authCode = signal('');

    updateAuthCode(value: string) {
        this.authCode.set(value);
    }
}