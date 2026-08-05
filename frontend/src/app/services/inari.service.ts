import { inject, Service, signal } from '@angular/core';
import { UrlService } from './url.service';
import { v4 as uuidv4 } from 'uuid';
import { Device } from 'mediasoup-client';
import { RtpCapabilities, Transport } from 'mediasoup-client/types';

enum PageState {
    WAIT_FOR_CODE = 0,
    LOGGING_IN = 1
}

@Service()
export class InariService {

    // Injects
    private readonly urlService = inject(UrlService);

    // Internal
    public pageState = signal<PageState>(PageState.WAIT_FOR_CODE);
    public PageState = PageState;
    private ws!: WebSocket;

    private device!: Device;
    private routerRtpCapabilities!: RtpCapabilities;
    private producer_transport!: Transport;

    async init(authCode: string) {
        return new Promise((res) => {
            document.body.style.animation = 'backgroundAnimationSpeed 1s linear infinite';
            this.pageState.set(this.PageState.LOGGING_IN);

            this.ws = new WebSocket(this.urlService.wsUrl);

            this.ws.onopen = async () => {
                console.log('Connected to inari ws !');
                await this.send('link', {code: authCode});

                this.device = new Device();
                res(null);
            }

            this.ws.onclose = () => {
                document.body.style.animation = 'backgroundAnimation 20s linear infinite';
                this.pageState.set(PageState.WAIT_FOR_CODE);
                res(null);
            }

            this.ws.onmessage = async (message) => {
                const data = JSON.parse(message.data);

                console.log(data);

                if (data.kind == 'rtp_capabilities') {
                    this.routerRtpCapabilities = data.data.rtp_capabilities;
                    await this.device.load({ routerRtpCapabilities: this.routerRtpCapabilities });
                    this.send('create_producer_transport', {});
                } else if (data.kind == 'create_producer_transport') {
                    const { id, dtls_parameters, ice_candidates, ice_parameters } = data.data;
                    
                    this.producer_transport = this.device.createSendTransport({ id, iceParameters: ice_parameters, dtlsParameters: dtls_parameters, iceCandidates: ice_candidates });
                    this.producer_transport.on('connect', ({ dtlsParameters }) => {
                        this.send('connect_producer_transport', { dtls_parameters: dtlsParameters });
                    });
                }
            } 
        });
    }

    async send(kind: string, data: any) {
        let uuid = uuidv4()
        this.ws.send(JSON.stringify({kind, data, transaction_uuid: uuid}));
    }

    async disconnect() {
        this.ws.close();
        this.pageState.set(PageState.WAIT_FOR_CODE);
    }
}
