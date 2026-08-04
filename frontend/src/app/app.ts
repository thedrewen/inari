import { Component, signal } from '@angular/core';
import { WebsocketService } from './services/websocket.service';

enum PageState {
    WAIT_FOR_CODE = 0,
    LOGGING_IN = 1
}

@Component({
    selector: 'app-root',
    imports: [],
    templateUrl: './app.html',
    styleUrl: './app.scss'
})
export class App {
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;

    constructor (
        private readonly websocketService: WebsocketService
    ) { }

    public onConnection = signal(false);

    async login() {
        this.onConnection.set(true);

        await this.websocketService.init(this.authCode());

        this.pageState.set(PageState.LOGGING_IN);
    }

    public authCode = signal('');

    updateAuthCode(value: string) {
        this.authCode.set(value);
    }
}