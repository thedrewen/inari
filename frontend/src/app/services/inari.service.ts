import { createPlatform, inject, Service, signal } from '@angular/core';
import { UrlService } from './url.service';
import { v4 as uuidv4 } from 'uuid';
import { Device } from 'mediasoup-client';
import { Producer, RtpCapabilities, Transport } from 'mediasoup-client/types';

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
    private producer!: Producer;

    private promises_ws: {uuid: string, res: any}[] = [];

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
                this.promises_ws.forEach((v) => v.res(false));
                this.promises_ws = [];
                res(null);
            }

            this.ws.onmessage = async (message) => {
                const data = JSON.parse(message.data);

                let promise = this.promises_ws.find((v) => v.uuid == data.transaction_uuid);
                
                if (promise) {
                    promise.res(data.data);
                    this.promises_ws = this.promises_ws.filter((v) => v.uuid != data.transaction_uuid);
                }

                if (data.kind == 'rtp_capabilities') {
                    this.routerRtpCapabilities = data.data.rtp_capabilities;
                    await this.device.load({ routerRtpCapabilities: this.routerRtpCapabilities });
                    const result: any = await this.send_and_receive('create_producer_transport', {});
                    const { id, dtls_parameters, ice_candidates, ice_parameters } = result; 
                    this.producer_transport = this.device.createSendTransport({ id, iceParameters: ice_parameters, dtlsParameters: dtls_parameters, iceCandidates: ice_candidates });
                    
                    this.producer_transport.on('connect', async ({ dtlsParameters }, callback, errback) => {
                        let result = await this.send_and_receive('connect_producer_transport', { dtls_parameters: dtlsParameters });
                        if (result) {
                            callback();
                        } else {
                            errback(Error());
                        }
                    });

                    this.producer_transport.on('produce', async ({ kind, rtpParameters }, callback, errback) => {
                        let result: any = await this.send_and_receive('produce', { kind, rtp_parameters: rtpParameters });
                        if (result) {
                            callback(result.id);
                        } else {
                            errback(Error());
                        }
                    }); 

                    const media = await navigator.mediaDevices.getUserMedia({audio: true}).catch(() => this.disconnect());
                    
                    if (!media) return;
                    
                    this.producer = await this.producer_transport.produce({ track: media.getTracks()[0] });
                } 
            } 
        });
    }

    async send(kind: string, data: any) {
        let uuid = uuidv4()
        this.ws.send(JSON.stringify({kind, data, transaction_uuid: uuid}));
    }

    async send_and_receive(kind: string, data: any) {
        let uuid = uuidv4()
        this.ws.send(JSON.stringify({kind, data, transaction_uuid: uuid}));

        return new Promise((res) => {
            this.promises_ws.push({uuid, res});
        });
    }

    async disconnect() {
        this.ws.close();
        this.pageState.set(PageState.WAIT_FOR_CODE);
    }
}
