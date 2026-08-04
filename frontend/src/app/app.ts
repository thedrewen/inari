import { Component, signal } from '@angular/core';
import { WebsocketService } from './services/websocket.service';

enum PageState {
    WAIT_FOR_CODE = 0
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

    public on_connection = signal(false);

    async login() {
        this.on_connection.set(true);

        await this.websocketService.init();
    }
}