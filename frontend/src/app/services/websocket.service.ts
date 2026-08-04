import { inject, Service } from '@angular/core';
import { UrlService } from './url.service';

@Service()
export class WebsocketService {

    // Injects
    private readonly urlService = inject(UrlService);

    // Internal
    private ws!: WebSocket;

    async init(authCode: string) {
        this.ws = new WebSocket(this.urlService.wsUrl);
    }
}
