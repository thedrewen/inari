import { inject, Service, signal } from '@angular/core';
import { UrlService } from './url.service';
import { v4 as uuidv4 } from 'uuid';

enum PageState {
    WAIT_FOR_CODE = 0,
    LOGGING_IN = 1
}

@Service()
export class WebsocketService {

    // Injects
    private readonly urlService = inject(UrlService);

    // Internal
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;
    private ws!: WebSocket;

    async init(authCode: string) {
        return new Promise((res) => {
            document.body.style.animation = 'backgroundAnimationSpeed 1s linear infinite';
            this.pageState.set(this.PageState.LOGGING_IN);

            this.ws = new WebSocket(this.urlService.wsUrl);

            this.ws.onopen = async () => {
                console.log('Connected to inari ws !');
                await this.send('link', {code: authCode});
                res(null);
            }

            this.ws.onclose = () => {
                document.body.style.animation = 'backgroundAnimation 20s linear infinite';
                this.pageState.set(PageState.WAIT_FOR_CODE);
                res(null);
            }
        });
    }

    async send(kind: string, data: any) {
        let uuid = uuidv4()
        this.ws.send(JSON.stringify({kind, data, transaction_uuid: uuid}));
    }
}
